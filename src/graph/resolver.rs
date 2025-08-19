use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::graph::{Item, ItemId, ItemType, KnowledgeGraph};

pub struct Resolver<'a> {
    graph: &'a KnowledgeGraph,
    // name -> items (functions, types, etc.)
    name_index: HashMap<Arc<str>, Vec<ItemId>>,
    // module (file stem) -> file-level module item id
    module_index: HashMap<Arc<str>, ItemId>,
    // item -> file mapping
    item_to_file: HashMap<ItemId, PathBuf>,
    // alias (from pub use ... as Alias) -> fully-qualified target segments
    alias_map: HashMap<Arc<str>, Vec<Arc<str>>>,
    // per-file exposure of names via non-aliased re-exports: exposed name -> fully-qualified target segments
    exposure_map: HashMap<PathBuf, HashMap<Arc<str>, Vec<Arc<str>>>>,
}

impl Resolver<'_> {
    // Compute module segments relative to src/ for a given file path.
    fn module_segments_for(&self, path: &Path) -> Vec<String> {
        // Use cached precomputed segments when available
        if let Some(segs) = self.graph.module_segments.get(path) {
            return segs.clone();
        }
        // Fallback to on-the-fly computation (should be rare)
        let mut src_idx: Option<usize> = None;
        let comps: Vec<_> = path.components().collect();
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
            for c in &comps[i + 1..comps.len().saturating_sub(1)] {
                if let std::path::Component::Normal(os) = c {
                    if let Some(s) = os.to_str() {
                        segs.push(s.to_string());
                    }
                }
            }
            if let Some(file_os) = path.file_name() {
                let file = file_os.to_string_lossy();
                if file != "mod.rs" && file != "lib.rs" {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        segs.push(stem.to_string());
                    }
                }
            }
        }
        segs
    }
}

impl<'a> Resolver<'a> {
    #[must_use]
    pub fn new(graph: &'a KnowledgeGraph) -> Self {
        // Pre-size maps based on graph characteristics to reduce rehashing/allocations
        let files_len = graph.files.len();
        let mut approx_items = 0usize;
        let mut approx_imports = 0usize;
        for f in graph.files.values() {
            approx_items += f.items.len();
            approx_imports += f.imports.len();
        }

        // Global string interner via graph.string_pool to deduplicate hot strings
        let intern_str = |s: &str| -> Arc<str> {
            let mut pool =
                graph.string_pool.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(a) = pool.get(s) {
                return a.clone();
            }
            let a: Arc<str> = Arc::from(s);
            pool.insert(s.to_string(), a.clone());
            a
        };

        let mut name_index: HashMap<Arc<str>, Vec<ItemId>> =
            HashMap::with_capacity(approx_items * 2);
        let mut module_index: HashMap<Arc<str>, ItemId> =
            HashMap::with_capacity(files_len.saturating_mul(2));
        let mut item_to_file: HashMap<ItemId, PathBuf> = HashMap::with_capacity(approx_items);
        let mut alias_map: HashMap<Arc<str>, Vec<Arc<str>>> =
            HashMap::with_capacity(approx_imports);
        let mut exposure_map: HashMap<PathBuf, HashMap<Arc<str>, Vec<Arc<str>>>> =
            HashMap::with_capacity(files_len);

        for (path, file) in &graph.files {
            // Ensure an entry exists for this file in exposure_map to avoid repeated reallocation of inner map
            if !file.imports.is_empty() {
                exposure_map
                    .entry(path.clone())
                    .or_insert_with(|| HashMap::with_capacity(file.imports.len()));
            }
            for (idx, it) in file.items.iter().enumerate() {
                item_to_file.insert(it.id.clone(), path.clone());
                let nm = intern_str(it.name.as_ref());
                name_index.entry(nm).or_default().push(it.id.clone());
                if idx == 0 {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let st = intern_str(stem);
                        module_index.insert(st, it.id.clone());
                    }
                }
            }
            // Prefer precomputed import segments if available
            if let Some(pre) = graph.import_segments.get(path) {
                for (segments, alias_arc) in pre {
                    if segments.is_empty() {
                        continue;
                    }
                    if let Some(k) = alias_arc.clone() {
                        alias_map.insert(k, segments.clone());
                    } else if let Some(last) = segments.last().cloned() {
                        exposure_map
                            .entry(path.clone())
                            .or_default()
                            .insert(last, segments.clone());
                    }
                }
            } else {
                for imp in &file.imports {
                    let segments: Vec<Arc<str>> =
                        imp.path.split("::").filter(|s| !s.is_empty()).map(intern_str).collect();
                    if let Some(alias) = &imp.alias {
                        // Ignore underscore imports: `use path as _;` doesn't bind a name
                        if alias.as_ref() == "_" {
                            continue;
                        }
                        if !alias.is_empty() && !segments.is_empty() {
                            let k = intern_str(alias.as_ref());
                            alias_map.insert(k, segments);
                        }
                    } else if let Some(last) = segments.last().cloned() {
                        // Non-aliased re-export exposes the last segment under the same name within this file/module
                        exposure_map.entry(path.clone()).or_default().insert(last, segments);
                    }
                }
            }
        }
        Self { graph, name_index, module_index, item_to_file, alias_map, exposure_map }
    }

    // Resolve an import path relative to a given file.
    // Returns a list because globs or ambiguous names can map to multiple targets.
    pub fn resolve_import(&self, from_file: &Path, raw_path: &str) -> Vec<ItemId> {
        // Strip aliasing `as X`
        let path = raw_path.split(" as ").next().unwrap_or(raw_path).trim();
        let mut parts: Vec<Arc<str>> =
            path.split("::").filter(|s| !s.is_empty()).map(Arc::<str>::from).collect();
        if parts.is_empty() {
            return Vec::new();
        }

        // Best-effort normalization of crate/self/super using filesystem layout under src/
        let mut scope: Vec<String> = self.module_segments_for(from_file);
        loop {
            match parts.first().map(std::convert::AsRef::as_ref) {
                Some("crate") => {
                    parts.remove(0);
                    scope.clear();
                }
                Some("self") => {
                    parts.remove(0); /* stay in same scope */
                }
                Some("super") => {
                    parts.remove(0);
                    if !scope.is_empty() {
                        scope.pop();
                    }
                }
                _ => break,
            }
        }
        if parts.is_empty() {
            return Vec::new();
        }

        // Apply alias mapping on the first segment, if any
        if let Some(first) = parts.first().cloned() {
            if let Some(mapped) = self.alias_map.get(&first) {
                parts.remove(0);
                let mut new_parts = mapped.clone();
                new_parts.extend(parts);
                parts = new_parts;
            }
        }

        // Apply per-file exposure mapping (re-exports without alias)
        if let Some(first) = parts.first().cloned() {
            if let Some(map) = self.exposure_map.get(from_file) {
                if let Some(mapped) = map.get(&first) {
                    parts.remove(0);
                    let mut new_parts = mapped.clone();
                    new_parts.extend(parts);
                    parts = new_parts;
                }
            }
        }

        // Try to resolve using scoped module chain based on filesystem under src/
        // Prepare a borrowable slice of &str for scoped chain
        let parts_str: Vec<&str> = parts.iter().map(Arc::<str>::as_ref).collect();
        if let Some(ids) = self.resolve_scoped_chain(from_file, &scope, &parts_str) {
            return ids;
        }

        // Fallback: Try exact item name match on the last segment
        let Some(last) = parts.last() else {
            return Vec::new();
        };
        if let Some(ids) = self.name_index.get(last) {
            return ids.clone();
        }

        // Fallback: map segment to a module (file-level) item
        if let Some(mid) = self.module_index.get(last) {
            return vec![mid.clone()];
        }

        // If there are multiple segments, try mapping first to a module and last to a symbol
        if parts.len() >= 2 {
            let first = parts[0].as_ref();
            if let Some(_m0) = self.module_index.get(first) {
                if let Some(ids) = self.name_index.get(last) {
                    return ids.clone();
                }
            }
            // Try combining scope head with parts
            if let Some(scope_head) = scope.first() {
                if let Some(_m) = self.module_index.get(scope_head.as_str()) {
                    if let Some(ids) = self.name_index.get(last) {
                        return ids.clone();
                    }
                }
            }
        }

        Vec::new()
    }

    #[must_use]
    pub fn is_item_function(&self, id: &ItemId) -> bool {
        if let Some(file) = self.item_to_file.get(id).and_then(|p| self.graph.files.get(p)) {
            if let Some(Item { item_type, .. }) = file.items.iter().find(|it| &it.id == id) {
                return matches!(item_type, ItemType::Function { .. });
            }
        }
        false
    }

    #[must_use]
    pub fn is_file_level_module(&self, id: &ItemId) -> bool {
        if let Some(file_path) = self.item_to_file.get(id) {
            if let Some(file) = self.graph.files.get(file_path) {
                if let Some(first) = file.items.first() {
                    return &first.id == id;
                }
            }
        }
        false
    }

    // Attempt to walk modules using the scope and parts to find the target file/module and then resolve the final item.
    // Returns Some(vec) on success; None if chain cannot be mapped.
    fn resolve_scoped_chain(
        &self,
        from_file: &Path,
        scope: &[String],
        parts: &[&str],
    ) -> Option<Vec<ItemId>> {
        if parts.is_empty() {
            return None;
        }
        let (base_src, _src_idx) = Self::base_src_dir(from_file)?;
        // Build starting module path from scope
        let mut dir = base_src.clone();
        let mut scope_dirs: Vec<&str> = scope.iter().map(std::string::String::as_str).collect();
        // If from_file is a leaf file (not mod.rs/lib.rs), drop last scope segment (file stem)
        let is_leaf = from_file
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|f| f != "mod.rs" && f != "lib.rs");
        if is_leaf && !scope_dirs.is_empty() {
            scope_dirs.pop();
        }
        for seg in scope_dirs {
            dir.push(seg);
        }
        // Walk all segments except the last as module directories/files
        for seg in &parts[..parts.len().saturating_sub(1)] {
            // Try directory seg
            dir.push(seg);
            // Accept if there is either dir/mod.rs or dir/lib.rs in graph
            let has_mod = self.graph.files.contains_key(&dir.join("mod.rs"));
            let has_lib = !has_mod && self.graph.files.contains_key(&dir.join("lib.rs"));
            let found_dir = has_mod || has_lib;
            if !found_dir {
                // Try sibling file: parent/<seg>.rs
                dir.pop();
                let file_rs = dir.join(format!("{seg}.rs"));
                if self.graph.files.contains_key(&file_rs) {
                    // Now move into that file's dir scope for next segments
                    dir.push(seg);
                } else {
                    return None;
                }
            }
        }
        // Now resolve the last segment inside current dir/module
        let last = parts[parts.len() - 1];
        // First, try a file in this dir named last.rs
        let file_rs = dir.join(format!("{last}.rs"));
        if let Some(fnode) = self.graph.files.get(&file_rs) {
            // Prefer concrete items named `last` inside that file
            let mut ids: Vec<ItemId> = Vec::with_capacity(fnode.items.len());
            for it in &fnode.items {
                if it.name.as_ref() == last {
                    ids.push(it.id.clone());
                }
            }
            if !ids.is_empty() {
                return Some(ids);
            }
            // Else return the file-level module id if known
            if let Some(mid) = self.module_index.get(last) {
                return Some(vec![mid.clone()]);
            }
        }
        // Next, try dir/mod.rs or dir/lib.rs containing an item named `last`
        let mod_path = dir.join("mod.rs");
        let lib_path = dir.join("lib.rs");
        for cand in [mod_path, lib_path] {
            if let Some(fnode) = self.graph.files.get(&cand) {
                let mut ids: Vec<ItemId> = Vec::with_capacity(fnode.items.len());
                for it in &fnode.items {
                    if it.name.as_ref() == last {
                        ids.push(it.id.clone());
                    }
                }
                if !ids.is_empty() {
                    return Some(ids);
                }
            }
        }
        None
    }

    // Returns (base_src_dir, index_of_src_component) if src is found in the path
    fn base_src_dir(path: &Path) -> Option<(PathBuf, usize)> {
        let comps: Vec<_> = path.components().collect();
        let mut src_idx: Option<usize> = None;
        for (i, c) in comps.iter().enumerate() {
            if let std::path::Component::Normal(os) = c {
                if os.to_str() == Some("src") {
                    src_idx = Some(i);
                    break;
                }
            }
        }
        let i = src_idx?;
        let mut base = PathBuf::new();
        for c in &comps[..=i] {
            base.push(c.as_os_str());
        }
        Some((base, i))
    }
}
