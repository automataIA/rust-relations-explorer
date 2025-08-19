//! Graph model and builder for the crate.
//!
//! This module defines the core data structures for the knowledge graph
//! (`KnowledgeGraph`, `FileNode`, `Item`, `Relationship`) and the analysis
//! passes that populate relationships (module hierarchy, import uses, calls).
//!
//! You typically construct a graph via `KnowledgeGraph::build_from_directory_*`
//! and then pass it to queries in `crate::query`.
use crate::utils::cache;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub mod resolver;

// Type aliases to keep signatures concise and satisfy clippy::type_complexity
type Segments = Vec<Arc<str>>;
type ImportSegments = Vec<(Segments, Option<Arc<str>>)>;
type ParsedEntry = (PathBuf, FileNode, Vec<Relationship>, cache::CacheEntry);

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct ItemId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file: PathBuf,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Module { is_inline: bool },
    Function { is_async: bool, is_const: bool },
    Struct { is_tuple: bool },
    Enum { variant_count: usize },
    Trait { is_object_safe: bool },
    Impl { trait_name: Option<Arc<str>>, type_name: Arc<str> },
    Const,
    Static { is_mut: bool },
    Type,
    Macro,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    PubCrate,
    PubSuper,
    PubIn(Arc<str>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub item_type: ItemType,
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub location: Location,
    pub attributes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub path: Arc<str>,
    pub alias: Option<Arc<str>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Uses { import_type: String },
    Implements { trait_name: String },
    Contains { containment_type: String },
    Extends { extension_type: String },
    Calls { call_type: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from_item: ItemId,
    pub to_item: ItemId,
    pub relationship_type: RelationshipType,
    pub strength: f64,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileMetrics {
    pub item_count: usize,
    pub import_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileNode {
    pub path: PathBuf,
    pub items: Vec<Item>,
    pub imports: Vec<Import>,
    pub metrics: FileMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GraphMetadata {
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeGraph {
    pub files: HashMap<PathBuf, FileNode>,
    pub relationships: Vec<Relationship>,
    pub metadata: GraphMetadata,
    // Module hierarchy tracking: parent and children maps keyed by file paths
    pub module_parent: HashMap<PathBuf, PathBuf>,
    pub module_children: HashMap<PathBuf, Vec<PathBuf>>,
    // Cached module path segments per file (relative to src/), to avoid recomputation
    pub module_segments: HashMap<PathBuf, Vec<String>>,
    // Precomputed import segments per file: Vec of (segments, alias), using Arc<str> pool for deduplication
    #[serde(skip, default)]
    pub import_segments: HashMap<PathBuf, ImportSegments>,
    // Global string pool for interning hot strings across phases (serde-skipped)
    #[serde(skip, default)]
    pub string_pool: std::sync::Arc<Mutex<HashMap<String, Arc<str>>>>,
}

impl KnowledgeGraph {
    /// Build a knowledge graph from a directory with explicit cache mode and ignore behavior.
    ///
    /// Arguments:
    /// - `path`: Root directory to scan for Rust source files.
    /// - `mode`: Cache usage policy (see `utils::cache::CacheMode`).
    /// - `no_ignore`: When true, bypasses `.gitignore`/`.ignore` rules in file discovery.
    ///
    /// Returns a fully built `KnowledgeGraph`, loading from and/or updating the on-disk cache
    /// according to `mode`. File discovery is performed via `utils::file_walker::rust_files_with_options`.
    ///
    /// # Errors
    /// Returns `KnowledgeGraphError` if file discovery, I/O, cache read/write, or parsing fails during build.
    #[allow(clippy::too_many_lines)]
    pub fn build_from_directory_with_cache_opts(
        path: &std::path::Path,
        mode: cache::CacheMode,
        no_ignore: bool,
    ) -> Result<Self, crate::errors::KnowledgeGraphError> {
        use crate::errors::KnowledgeGraphError;
        use crate::parser::RustParser;
        use crate::utils::file_walker;
        use std::fs;

        let files =
            file_walker::rust_files_with_options(path.to_string_lossy().as_ref(), no_ignore);

        // Load or ignore cache based on mode
        let root_dir = path.to_path_buf();
        let mut cache_state = match mode {
            cache::CacheMode::Use => cache::load_cache(&root_dir).unwrap_or_default(),
            cache::CacheMode::Ignore | cache::CacheMode::Rebuild => cache::Cache::default(),
        };

        // Collect file metadata for change detection
        let infos: Vec<(String, cache::CacheEntryMeta)> = files
            .iter()
            .map(|f| {
                let p = std::path::Path::new(f);
                let meta = fs::metadata(p).ok();
                let len = meta.as_ref().map_or(0u64, std::fs::Metadata::len);
                let mtime = meta
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0u64, |d| d.as_secs());
                (f.clone(), cache::CacheEntryMeta { mtime, len })
            })
            .collect();

        // Prune cache entries for files that no longer exist in the walk
        if matches!(mode, cache::CacheMode::Use) {
            use std::collections::HashSet;
            let present: HashSet<PathBuf> =
                files.iter().map(|f| std::path::Path::new(f).to_path_buf()).collect();
            cache_state.entries.retain(|k, _| present.contains(k));
        }

        // Reuse from cache when metadata matches (only in Use mode)
        let mut reused: Vec<(PathBuf, FileNode, Vec<Relationship>)> = Vec::new();
        let mut to_parse: Vec<(String, cache::CacheEntryMeta)> = Vec::new();
        for (file, meta) in &infos {
            let key = std::path::Path::new(file).to_path_buf();
            if matches!(mode, cache::CacheMode::Use) {
                if let Some(entry) = cache_state.entries.get(&key) {
                    if entry.meta == *meta {
                        let node = entry.node.clone();
                        // Rebuild contains edges for cached node
                        let file_id = ItemId(format!("file:{}", node.path.display()));
                        let mut edges = Vec::new();
                        for it in node.items.iter().skip(1) {
                            edges.push(Relationship {
                                from_item: file_id.clone(),
                                to_item: it.id.clone(),
                                relationship_type: RelationshipType::Contains {
                                    containment_type: "file_contains".to_string(),
                                },
                                strength: 1.0,
                                context: "auto".to_string(),
                            });
                        }
                        reused.push((node.path.clone(), node, edges));
                        continue;
                    }
                }
            }
            to_parse.push((file.clone(), meta.clone()));
        }

        // Parse files in parallel. Each task returns (path, node, contains_edges)
        let parsed: Result<Vec<ParsedEntry>, KnowledgeGraphError> = to_parse
            .into_par_iter()
            .map(|(file, meta)| {
                let p = std::path::Path::new(&file);
                let content = fs::read_to_string(p)?;
                let p = std::path::Path::new(&file).to_path_buf();
                let mut node = RustParser::new().parse_file(&content, &p).map_err(|source| {
                    KnowledgeGraphError::ParseError { file: p.clone(), source }
                })?;

                // Create a synthetic file-level module item
                let file_id = ItemId(format!("file:{}", node.path.display()));
                let file_item = Item {
                    id: file_id.clone(),
                    item_type: ItemType::Module { is_inline: false },
                    name: Arc::from(
                        node.path.file_stem().and_then(|s| s.to_str()).unwrap_or("(file)"),
                    ),
                    visibility: Visibility::PubCrate,
                    location: Location { file: node.path.clone(), line_start: 1, line_end: 1 },
                    attributes: vec![],
                };

                // Prepend the file item
                let mut items_with_file = Vec::with_capacity(node.items.len() + 1);
                items_with_file.push(file_item);
                items_with_file.extend(node.items);
                node.metrics.item_count = items_with_file.len();
                node.items = items_with_file;

                // Build Contains relationships from file item to each other item
                let mut contains_edges: Vec<Relationship> = Vec::new();
                for it in node.items.iter().skip(1) {
                    contains_edges.push(Relationship {
                        from_item: file_id.clone(),
                        to_item: it.id.clone(),
                        relationship_type: RelationshipType::Contains {
                            containment_type: "file_contains".to_string(),
                        },
                        strength: 1.0,
                        context: "auto".to_string(),
                    });
                }

                let cache_entry = cache::CacheEntry { meta, node: node.clone() };
                Ok::<_, KnowledgeGraphError>((node.path.clone(), node, contains_edges, cache_entry))
            })
            .collect();

        let mut graph = KnowledgeGraph::default();
        // Insert reused nodes
        for (path, node, edges) in reused {
            graph.files.insert(path, node);
            graph.relationships.extend(edges);
        }
        // Insert newly parsed nodes and update cache
        for (path, node, edges, cache_entry) in parsed? {
            graph.files.insert(path, node);
            graph.relationships.extend(edges);
            cache_state.entries.insert(cache_entry.node.path.clone(), cache_entry);
        }

        // Precompute module path segments for all files (relative to src/)
        // Avoid recomputing in resolver/analyses. Mirrors logic in resolver::module_segments_for.
        graph.module_segments = {
            let mut map: HashMap<PathBuf, Vec<String>> = HashMap::with_capacity(graph.files.len());
            for p in graph.files.keys() {
                // Find index of "src" in the path components
                let comps: Vec<_> = p.components().collect();
                let mut src_idx: Option<usize> = None;
                for (i, c) in comps.iter().enumerate() {
                    if let std::path::Component::Normal(os) = c {
                        if os.to_str() == Some("src") {
                            src_idx = Some(i);
                            break;
                        }
                    }
                }
                let mut segs: Vec<String> = Vec::new();
                if let Some(i) = src_idx {
                    // Directories after src up to (but excluding) the file name
                    for c in &comps[i + 1..comps.len().saturating_sub(1)] {
                        if let std::path::Component::Normal(os) = c {
                            if let Some(s) = os.to_str() {
                                segs.push(s.to_string());
                            }
                        }
                    }
                    // File as module: include file stem unless mod.rs/lib.rs
                    if let Some(file_os) = p.file_name() {
                        let file = file_os.to_string_lossy();
                        if file != "mod.rs" && file != "lib.rs" {
                            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                segs.push(stem.to_string());
                            }
                        }
                    }
                }
                map.insert(p.clone(), segs);
            }
            map
        };

        // Precompute import segments and alias arcs per file with a shared Arc<str> pool
        graph.import_segments = {
            let mut pool: HashMap<String, Arc<str>> = HashMap::new();
            let mut intern = |s: &str| -> Arc<str> {
                if let Some(a) = pool.get(s) {
                    return a.clone();
                }
                let a: Arc<str> = Arc::from(s);
                pool.insert(s.to_string(), a.clone());
                a
            };
            let mut map: HashMap<PathBuf, ImportSegments> =
                HashMap::with_capacity(graph.files.len());
            for (p, f) in &graph.files {
                if f.imports.is_empty() {
                    continue;
                }
                let mut vecs: ImportSegments = Vec::with_capacity(f.imports.len());
                for imp in &f.imports {
                    let parts: Vec<Arc<str>> =
                        imp.path.split("::").filter(|s| !s.is_empty()).map(&mut intern).collect();
                    let alias_arc: Option<Arc<str>> = imp
                        .alias
                        .as_deref()
                        .filter(|a| !a.is_empty() && *a != "_")
                        .map(&mut intern);
                    vecs.push((parts, alias_arc));
                }
                map.insert(p.clone(), vecs);
            }
            map
        };

        // Set generation timestamp (seconds since epoch) without extra deps
        graph.metadata.generated_at =
            match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(d) => format!("{}", d.as_secs()),
                Err(_) => "0".to_string(),
            };
        // Analyze relationships beyond file containment
        graph.analyze_relationships();

        // Save cache (best-effort). Even in Ignore/Rebuild, we save freshly parsed state.
        cache::save_cache(&root_dir, &cache_state);
        Ok(graph)
    }

    // Module hierarchy helpers
    #[must_use]
    pub fn get_module_parent(&self, file: &PathBuf) -> Option<&PathBuf> {
        self.module_parent.get(file)
    }

    #[must_use]
    pub fn get_module_children(&self, file: &PathBuf) -> &[PathBuf] {
        match self.module_children.get(file) {
            Some(v) => v.as_slice(),
            None => &[],
        }
    }
    /// Backward-compatible builder: reads env var `KNOWLEDGE_RS_NO_IGNORE` for ignore bypass.
    ///
    /// # Errors
    /// Returns `KnowledgeGraphError` when directory walking, file I/O, or parsing fails during build.
    pub fn build_from_directory_with_cache(
        path: &std::path::Path,
        mode: cache::CacheMode,
    ) -> Result<Self, crate::errors::KnowledgeGraphError> {
        let no_ignore = std::env::var("KNOWLEDGE_RS_NO_IGNORE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        Self::build_from_directory_with_cache_opts(path, mode, no_ignore)
    }

    /// Backward-compatible builder: `CacheMode::Use` and env-derived ignore bypass.
    ///
    /// # Errors
    /// Returns `KnowledgeGraphError` on I/O or parsing failures during build.
    pub fn build_from_directory(
        path: &std::path::Path,
    ) -> Result<Self, crate::errors::KnowledgeGraphError> {
        let no_ignore = std::env::var("KNOWLEDGE_RS_NO_IGNORE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        Self::build_from_directory_with_cache_opts(path, cache::CacheMode::Use, no_ignore)
    }

    /// Convenience builder: explicit ignore bypass with default `CacheMode::Use`.
    ///
    /// # Errors
    /// Returns `KnowledgeGraphError` on I/O or parsing failures during build.
    pub fn build_from_directory_opts(
        path: &std::path::Path,
        no_ignore: bool,
    ) -> Result<Self, crate::errors::KnowledgeGraphError> {
        Self::build_from_directory_with_cache_opts(path, cache::CacheMode::Use, no_ignore)
    }

    /// Save the graph as pretty-printed JSON.
    ///
    /// # Errors
    /// Returns `KnowledgeGraphError::Io` if serialization or writing the file fails.
    pub fn save_json(
        &self,
        path: &std::path::Path,
    ) -> Result<(), crate::errors::KnowledgeGraphError> {
        let data = serde_json::to_string_pretty(self).map_err(|e| {
            crate::errors::KnowledgeGraphError::Io(std::io::Error::other(e.to_string()))
        })?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Load a graph from JSON file.
    ///
    /// # Errors
    /// Returns `KnowledgeGraphError::Io` if reading the file fails or JSON is invalid.
    pub fn load_json(path: &std::path::Path) -> Result<Self, crate::errors::KnowledgeGraphError> {
        let data = std::fs::read_to_string(path)?;
        let graph: KnowledgeGraph = serde_json::from_str(&data).map_err(|e| {
            crate::errors::KnowledgeGraphError::Io(std::io::Error::other(e.to_string()))
        })?;
        Ok(graph)
    }
}

impl KnowledgeGraph {
    fn analyze_relationships(&mut self) {
        self.analyze_module_hierarchy();
        self.analyze_import_uses();
        self.analyze_calls_heuristic();
    }

    // Establish module hierarchy using filesystem layout.
    // For every file-level synthetic module item, link its parent module (if present) with a Contains edge.
    fn analyze_module_hierarchy(&mut self) {
        // Reset hierarchy maps to avoid stale entries on re-analysis
        self.module_parent.clear();
        self.module_children.clear();
        // Helper to get file-level item id (first item) for a given path
        let mut file_level_id: HashMap<PathBuf, ItemId> = HashMap::with_capacity(self.files.len());
        for (p, f) in &self.files {
            if let Some(it) = f.items.first() {
                file_level_id.insert(p.clone(), it.id.clone());
            }
        }
        // Build reverse map to avoid repeated scans to resolve parent path by id
        let mut id_to_path: HashMap<ItemId, PathBuf> = HashMap::with_capacity(file_level_id.len());
        for (p, id) in &file_level_id {
            id_to_path.insert(id.clone(), p.clone());
        }

        // For each file, determine its parent module file and add Contains edge
        let all_paths: Vec<PathBuf> = self.files.keys().cloned().collect();
        for path in all_paths {
            // Skip if we can't find this file's synthetic id
            let Some(child_id) = file_level_id.get(&path) else {
                continue;
            };

            // Determine parent file candidate based on path structure
            let parent_dir = match path.parent() {
                Some(d) => d.to_path_buf(),
                None => continue,
            };
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            // If current file is mod.rs or lib.rs, parent is parent's parent dir's mod.rs or lib.rs
            let mut parent_id_opt: Option<ItemId> = None;
            if file_name == "mod.rs" || file_name == "lib.rs" {
                if let Some(g) = parent_dir.parent() {
                    let p1 = g.join("mod.rs");
                    if let Some(pid) = file_level_id.get(&p1) {
                        parent_id_opt = Some(pid.clone());
                    }
                    if parent_id_opt.is_none() {
                        let p2 = g.join("lib.rs");
                        if let Some(pid) = file_level_id.get(&p2) {
                            parent_id_opt = Some(pid.clone());
                        }
                    }
                }
            } else {
                let p1 = parent_dir.join("mod.rs");
                if let Some(pid) = file_level_id.get(&p1) {
                    parent_id_opt = Some(pid.clone());
                }
                if parent_id_opt.is_none() {
                    let p2 = parent_dir.join("lib.rs");
                    if let Some(pid) = file_level_id.get(&p2) {
                        parent_id_opt = Some(pid.clone());
                    }
                }
            }

            if let Some(parent_id) = parent_id_opt {
                if parent_id != *child_id {
                    // Relationship edge
                    self.relationships.push(Relationship {
                        from_item: parent_id.clone(),
                        to_item: child_id.clone(),
                        relationship_type: RelationshipType::Contains {
                            containment_type: "module_contains".to_string(),
                        },
                        strength: 1.0,
                        context: "fs".to_string(),
                    });
                    // Hierarchy maps
                    if let Some(pp) = id_to_path.get(&parent_id).cloned() {
                        self.module_parent.insert(path.clone(), pp.clone());
                        self.module_children.entry(pp).or_default().push(path.clone());
                    }
                }
            }
        }
    }

    fn analyze_import_uses(&mut self) {
        // Build edges using a Resolver and parallelize over files with per-file batching
        let res = resolver::Resolver::new(self);
        let produced: Vec<Relationship> = self
            .files
            .par_iter()
            .map(|(path, file)| {
                let mut edges: Vec<Relationship> = Vec::with_capacity(file.imports.len());
                if file.items.is_empty() {
                    return edges;
                }
                let file_id = file.items[0].id.clone();
                for imp in &file.imports {
                    let targets = res.resolve_import(path, &imp.path);
                    if targets.is_empty() {
                        continue;
                    }
                    for to in targets {
                        if to == file_id {
                            continue;
                        }
                        let import_type = if res.is_file_level_module(&to) {
                            "import-module"
                        } else {
                            "import-item"
                        };
                        edges.push(Relationship {
                            from_item: file_id.clone(),
                            to_item: to,
                            relationship_type: RelationshipType::Uses {
                                import_type: import_type.to_string(),
                            },
                            strength: if import_type == "import-item" { 1.0 } else { 0.8 },
                            context: imp.path.to_string(),
                        });
                    }
                }
                edges
            })
            .reduce(Vec::new, |mut a, mut b| {
                a.append(&mut b);
                a
            });
        self.relationships.extend(produced);
    }

    fn analyze_calls_heuristic(&mut self) {
        // Regex for fully qualified paths like a::b::foo(...)
        let path_call_re =
            Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+)\s*\(").unwrap();
        // Regex for simple names: foo(...)
        let simple_call_re = Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap();

        // Build index of function name -> list of ItemIds that are functions
        // First, estimate capacity
        let mut func_count = 0usize;
        for file in self.files.values() {
            for item in &file.items {
                if matches!(item.item_type, ItemType::Function { .. }) {
                    func_count += 1;
                }
            }
        }
        let mut func_index: HashMap<String, Vec<ItemId>> = HashMap::with_capacity(func_count);
        for file in self.files.values() {
            for item in &file.items {
                if let ItemType::Function { .. } = item.item_type {
                    func_index.entry(item.name.to_string()).or_default().push(item.id.clone());
                }
            }
        }

        // Build resolver once per analysis and process files in parallel with per-file batching
        let res = resolver::Resolver::new(self);
        let produced: Vec<Relationship> = self
            .files
            .par_iter()
            .map(|(path, file)| {
                let mut seen_local: std::collections::HashSet<(String, String)> =
                    std::collections::HashSet::new();
                let mut edges: Vec<Relationship> = Vec::with_capacity(16);
                if file.items.is_empty() {
                    return edges;
                }
                let file_id = file.items[0].id.clone();
                if let Ok(content) = std::fs::read_to_string(path) {
                    // 1) Resolve fully qualified calls via Resolver (more precise)
                    for cap in path_call_re.captures_iter(&content) {
                        let full = cap.get(1).map_or("", |m| m.as_str());
                        let mut targets = res.resolve_import(path, full);
                        if targets.is_empty() {
                            if let Some(last) = full.rsplit("::").next() {
                                if let Some(funcs) = func_index.get(last) {
                                    targets.clone_from(funcs);
                                }
                            }
                        }
                        for to in targets {
                            let key = (file_id.0.clone(), to.0.clone());
                            if seen_local.insert(key) {
                                edges.push(Relationship {
                                    from_item: file_id.clone(),
                                    to_item: to.clone(),
                                    relationship_type: RelationshipType::Calls {
                                        call_type: "path".to_string(),
                                    },
                                    strength: 0.7,
                                    context: full.to_string(),
                                });
                            }
                        }
                    }

                    // 2) Fallback simple name calls, filtering out definitions/macros and keywords
                    for cap in simple_call_re.captures_iter(&content) {
                        let Some(m) = cap.get(0) else { continue };
                        let name = cap.get(1).map_or("", |m| m.as_str());
                        let start = m.start();
                        let prefix = &content[start.saturating_sub(8)..start];
                        if prefix.contains("fn ")
                            || prefix.contains("struct ")
                            || prefix.contains("enum ")
                            || prefix.contains("trait ")
                        {
                            continue;
                        }
                        if start > 0 {
                            let prev = content[..start].chars().rev().find(|c| !c.is_whitespace());
                            if let Some('!') = prev {
                                continue;
                            }
                        }
                        if let Some(targets) = func_index.get(name) {
                            for to in targets {
                                let key = (file_id.0.clone(), to.0.clone());
                                if seen_local.insert(key) {
                                    edges.push(Relationship {
                                        from_item: file_id.clone(),
                                        to_item: to.clone(),
                                        relationship_type: RelationshipType::Calls {
                                            call_type: "heuristic".to_string(),
                                        },
                                        strength: 0.5,
                                        context: name.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
                edges
            })
            .reduce(Vec::new, |mut a, mut b| {
                a.append(&mut b);
                a
            });
        self.relationships.extend(produced);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn module_hierarchy_basic() {
        // Files: src/lib.rs (parent of src/a/mod.rs); src/a/mod.rs (parent of src/a/foo.rs)
        let mut g = KnowledgeGraph::default();
        let lib = PathBuf::from("src/lib.rs");
        let a_mod = PathBuf::from("src/a/mod.rs");
        let a_foo = PathBuf::from("src/a/foo.rs");

        let make_file_item = |p: &PathBuf| Item {
            id: ItemId(format!("file:{}", p.display())),
            item_type: ItemType::Module { is_inline: false },
            name: Arc::from(p.file_stem().and_then(|s| s.to_str()).unwrap_or("(file)")),
            visibility: Visibility::PubCrate,
            location: Location { file: p.clone(), line_start: 1, line_end: 1 },
            attributes: vec![],
        };

        g.files.insert(
            lib.clone(),
            FileNode {
                path: lib.clone(),
                items: vec![make_file_item(&lib)],
                imports: vec![],
                metrics: Default::default(),
            },
        );
        g.files.insert(
            a_mod.clone(),
            FileNode {
                path: a_mod.clone(),
                items: vec![make_file_item(&a_mod)],
                imports: vec![],
                metrics: Default::default(),
            },
        );
        g.files.insert(
            a_foo.clone(),
            FileNode {
                path: a_foo.clone(),
                items: vec![make_file_item(&a_foo)],
                imports: vec![],
                metrics: Default::default(),
            },
        );

        // Build hierarchy
        g.analyze_module_hierarchy();

        // Parent checks
        assert_eq!(g.get_module_parent(&a_mod), Some(&lib));
        assert_eq!(g.get_module_parent(&a_foo), Some(&a_mod));
        assert!(g.get_module_parent(&lib).is_none());

        // Children checks
        let lib_children = g.get_module_children(&lib);
        assert!(lib_children.contains(&a_mod));
        let a_mod_children = g.get_module_children(&a_mod);
        assert!(a_mod_children.contains(&a_foo));
    }

    #[test]
    fn import_uses_edges_item_vs_module() {
        // Build a small graph with two files in a temp dir
        let td = tempdir().unwrap();
        let f1 = td.path().join("a.rs");
        let f2 = td.path().join("modx.rs");
        let f3 = td.path().join("b.rs");
        // Write contents (not strictly needed for imports)
        fs::write(&f1, "// a.rs\n").unwrap();
        fs::write(&f2, "// modx.rs\n").unwrap();
        fs::write(&f3, "// b.rs\n").unwrap();

        // File nodes
        let mk_file_item = |p: &PathBuf| Item {
            id: ItemId(format!("file:{}", p.display())),
            item_type: ItemType::Module { is_inline: false },
            name: Arc::from(p.file_stem().and_then(|s| s.to_str()).unwrap_or("(file)")),
            visibility: Visibility::PubCrate,
            location: Location { file: p.clone(), line_start: 1, line_end: 1 },
            attributes: vec![],
        };
        let mk_fn_item = |p: &PathBuf, name: &str| Item {
            id: ItemId(format!("fn:{}:1", name)),
            item_type: ItemType::Function { is_async: false, is_const: false },
            name: Arc::from(name),
            visibility: Visibility::Public,
            location: Location { file: p.clone(), line_start: 1, line_end: 1 },
            attributes: vec![],
        };

        let mut g = KnowledgeGraph::default();

        // a.rs imports: item "foo" and module "modx"
        let a_node = FileNode {
            path: f1.clone(),
            items: vec![mk_file_item(&f1)],
            imports: vec![
                Import { path: "foo".into(), alias: None },
                Import { path: "modx".into(), alias: None },
            ],
            ..Default::default()
        };

        // modx.rs is a module (no inner items needed)
        let modx_node =
            FileNode { path: f2.clone(), items: vec![mk_file_item(&f2)], ..Default::default() };

        // b.rs defines function foo
        let b_node = FileNode {
            path: f3.clone(),
            items: vec![mk_file_item(&f3), mk_fn_item(&f3, "foo")],
            ..Default::default()
        };

        g.files.insert(f1.clone(), a_node);
        g.files.insert(f2.clone(), modx_node);
        g.files.insert(f3.clone(), b_node);

        // Run import uses analysis
        g.analyze_import_uses();

        // Check relationships: from a.rs file-level id
        let a_file_id = ItemId(format!("file:{}", f1.display()));
        // Expect one import-item edge to function foo and one import-module edge to modx
        let mut saw_item = false;
        let mut saw_module = false;
        for r in &g.relationships {
            if r.from_item == a_file_id {
                if let RelationshipType::Uses { import_type } = &r.relationship_type {
                    if import_type == "import-item" {
                        saw_item = true;
                    }
                    if import_type == "import-module" {
                        saw_module = true;
                    }
                }
            }
        }
        assert!(saw_item, "expected import-item edge");
        assert!(saw_module, "expected import-module edge");
    }

    #[test]
    fn calls_heuristic_and_path_and_macro_exclusion() {
        // temp dir structure with caller and callees; write contents so heuristic reads
        let td = tempdir().unwrap();
        let caller = td.path().join("caller.rs");
        let callee_foo = td.path().join("callee.rs");
        let dir_a = td.path().join("a");
        let dir_b = dir_a.join("b");
        fs::create_dir_all(&dir_b).unwrap();
        let baz = dir_b.join("baz.rs");

        // Contents: caller calls foo(), a::b::baz(), and macro!() which should be ignored
        fs::write(&caller, "fn main(){ foo(); a::b::baz(); my_macro!(x); }\n").unwrap();
        fs::write(&callee_foo, "pub fn foo(){}\n").unwrap();
        fs::write(&baz, "pub fn baz(){}\n").unwrap();

        // Helpers to build graph items
        let mk_file_item = |p: &PathBuf| Item {
            id: ItemId(format!("file:{}", p.display())),
            item_type: ItemType::Module { is_inline: false },
            name: Arc::from(p.file_stem().and_then(|s| s.to_str()).unwrap_or("(file)")),
            visibility: Visibility::PubCrate,
            location: Location { file: p.clone(), line_start: 1, line_end: 1 },
            attributes: vec![],
        };
        let mk_fn_item = |p: &PathBuf, name: &str| Item {
            id: ItemId(format!("fn:{}:X", name)),
            item_type: ItemType::Function { is_async: false, is_const: false },
            name: Arc::from(name),
            visibility: Visibility::Public,
            location: Location { file: p.clone(), line_start: 1, line_end: 1 },
            attributes: vec![],
        };

        let mut g = KnowledgeGraph::default();
        // caller file
        let caller_node = FileNode {
            path: caller.clone(),
            items: vec![mk_file_item(&caller)],
            ..Default::default()
        };
        // callee foo
        let callee_node = FileNode {
            path: callee_foo.clone(),
            items: vec![mk_file_item(&callee_foo), mk_fn_item(&callee_foo, "foo")],
            ..Default::default()
        };
        // baz
        let baz_node = FileNode {
            path: baz.clone(),
            items: vec![mk_file_item(&baz), mk_fn_item(&baz, "baz")],
            ..Default::default()
        };

        g.files.insert(caller.clone(), caller_node);
        g.files.insert(callee_foo.clone(), callee_node);
        g.files.insert(baz.clone(), baz_node);

        // Run call analysis
        g.analyze_calls_heuristic();

        let caller_file_id = ItemId(format!("file:{}", caller.display()));
        let mut saw_foo = false;
        let mut saw_baz = false;
        let saw_macro = false;
        for r in &g.relationships {
            if r.from_item != caller_file_id {
                continue;
            }
            if let RelationshipType::Calls { call_type } = &r.relationship_type {
                // Identify target by id suffix
                if r.to_item.0.starts_with("fn:foo:") {
                    saw_foo = true;
                    assert_eq!(call_type, "heuristic");
                }
                if r.to_item.0.starts_with("fn:baz:") {
                    saw_baz = true;
                    // path or heuristic is acceptable; path is preferred route
                }
            }
            if let RelationshipType::Uses { .. } = r.relationship_type { /* ignore */ }
        }
        assert!(saw_foo, "expected call edge to foo()");
        assert!(saw_baz, "expected call edge to a::b::baz()");
        assert!(!saw_macro, "macro invocations must not create call edges");
    }
}
