use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::graph::{ItemId, KnowledgeGraph};

/// Query trait implemented by all query types.
///
/// Given an immutable reference to a `KnowledgeGraph`, returns a result of type `R`.
pub trait Query<R> {
    fn run(&self, graph: &KnowledgeGraph) -> R;
}

/// List types implementing a given trait name.
///
/// Returns rows as `(file_path, type_name)` sorted by file then type.
pub struct TraitImplsQuery {
    pub trait_name: String,
}

impl TraitImplsQuery {
    /// Create a new query for the provided trait name (e.g., "Display").
    #[must_use]
    pub fn new(trait_name: &str) -> Self {
        Self { trait_name: trait_name.to_string() }
    }
}

// Returns Vec of (file_path, type_name)
impl Query<Vec<(PathBuf, String)>> for TraitImplsQuery {
    fn run(&self, graph: &KnowledgeGraph) -> Vec<(PathBuf, String)> {
        let mut out: Vec<(PathBuf, String)> = Vec::new();
        for (path, file) in &graph.files {
            for it in &file.items {
                if let crate::graph::ItemType::Impl { trait_name: Some(tn), type_name } =
                    &it.item_type
                {
                    if tn.as_ref() == self.trait_name {
                        out.push((path.clone(), type_name.to_string()));
                    }
                }
            }
        }
        // Stable sort by file then type
        out.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
        out
    }
}

/// Return the set of files directly connected to a target file by any relationship
/// involving items defined in that file (edge endpoints at item-level are projected to file-level).
pub struct ConnectedFilesQuery {
    pub file: PathBuf,
}

impl ConnectedFilesQuery {
    /// Construct a query targeting the specified file path.
    pub fn new<P: AsRef<Path>>(file: P) -> Self {
        Self { file: file.as_ref().to_path_buf() }
    }
}

impl Query<Vec<PathBuf>> for ConnectedFilesQuery {
    fn run(&self, graph: &KnowledgeGraph) -> Vec<PathBuf> {
        // Build item -> file index and file -> items index
        let mut item_to_file: HashMap<ItemId, PathBuf> = HashMap::new();
        let mut file_to_items: HashMap<PathBuf, Vec<ItemId>> = HashMap::new();
        for (path, file) in &graph.files {
            let ids: Vec<ItemId> = file.items.iter().map(|i| i.id.clone()).collect();
            for id in &ids {
                item_to_file.insert(id.clone(), path.clone());
            }
            file_to_items.insert(path.clone(), ids);
        }

        // Resolve the canonical path in the graph map
        let Some((target_path, _node)) = graph.files.iter().find(|(p, _)| p == &&self.file) else {
            return Vec::new();
        };

        let Some(target_items) = file_to_items.get(target_path) else {
            return Vec::new();
        };
        let target_set: HashSet<ItemId> = target_items.iter().cloned().collect();

        let mut out: HashSet<PathBuf> = HashSet::new();
        for rel in &graph.relationships {
            // If edge touches any item in the target file, add the opposing file
            if target_set.contains(&rel.from_item) {
                if let Some(fp) = item_to_file.get(&rel.to_item) {
                    if fp != target_path {
                        out.insert(fp.clone());
                    }
                }
            }
            if target_set.contains(&rel.to_item) {
                if let Some(fp) = item_to_file.get(&rel.from_item) {
                    if fp != target_path {
                        out.insert(fp.clone());
                    }
                }
            }
        }

        let mut v: Vec<PathBuf> = out.into_iter().collect();
        v.sort();
        v
    }
}

/// Direction for `FunctionUsageQuery`.
pub enum UsageDirection {
    Callers,
    Callees,
}

/// Find callers or callees for a given function name, returning unique file paths.
pub struct FunctionUsageQuery {
    pub function: String,
    pub direction: UsageDirection,
}

impl FunctionUsageQuery {
    /// Query files containing callers of the specified function name.
    #[must_use]
    pub fn callers(function: &str) -> Self {
        Self { function: function.to_string(), direction: UsageDirection::Callers }
    }
    /// Query files containing callees called by the specified function name.
    #[must_use]
    pub fn callees(function: &str) -> Self {
        Self { function: function.to_string(), direction: UsageDirection::Callees }
    }
}

impl Query<Vec<PathBuf>> for FunctionUsageQuery {
    fn run(&self, graph: &KnowledgeGraph) -> Vec<PathBuf> {
        // Build indices
        let mut item_to_file: HashMap<ItemId, PathBuf> = HashMap::new();
        let mut func_name_to_ids: HashMap<String, Vec<ItemId>> = HashMap::new();
        for (path, file) in &graph.files {
            for item in &file.items {
                item_to_file.insert(item.id.clone(), path.clone());
                // crude: function ids contain prefix "fn:"; but better match by type and name
                if let crate::graph::ItemType::Function { .. } = item.item_type {
                    func_name_to_ids
                        .entry(item.name.to_string())
                        .or_default()
                        .push(item.id.clone());
                }
            }
        }

        let Some(target_ids) = func_name_to_ids.get(&self.function) else {
            return Vec::new();
        };
        let target_set: HashSet<ItemId> = target_ids.iter().cloned().collect();

        let mut out: HashSet<PathBuf> = HashSet::new();
        for rel in &graph.relationships {
            match self.direction {
                UsageDirection::Callers => {
                    if target_set.contains(&rel.to_item) {
                        if let Some(fp) = item_to_file.get(&rel.from_item) {
                            out.insert(fp.clone());
                        }
                    }
                }
                UsageDirection::Callees => {
                    if target_set.contains(&rel.from_item) {
                        if let Some(fp) = item_to_file.get(&rel.to_item) {
                            out.insert(fp.clone());
                        }
                    }
                }
            }
        }

        let mut v: Vec<PathBuf> = out.into_iter().collect();
        v.sort();
        v
    }
}

/// Detect cycles over the file-level projection of the graph.
pub struct CycleDetectionQuery;

impl CycleDetectionQuery {
    /// Construct a cycle detection query.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for CycleDetectionQuery {
    fn default() -> Self {
        Self
    }
}

// Helper for DFS used by `CycleDetectionQuery::run`
fn dfs(
    u: usize,
    adj: &Vec<Vec<usize>>,
    visited: &mut [bool],
    stack: &mut [bool],
    path: &mut Vec<usize>,
    out: &mut Vec<Vec<PathBuf>>,
    names: &Vec<PathBuf>,
) {
    visited[u] = true;
    stack[u] = true;
    path.push(u);
    for &v in &adj[u] {
        if !visited[v] {
            dfs(v, adj, visited, stack, path, out, names);
        } else if stack[v] {
            // Found a cycle; extract from v to end
            if let Some(pos) = path.iter().position(|&x| x == v) {
                let cyc: Vec<PathBuf> = path[pos..].iter().map(|&i| names[i].clone()).collect();
                out.push(cyc);
            }
        }
    }
    path.pop();
    stack[u] = false;
}

impl Query<Vec<Vec<PathBuf>>> for CycleDetectionQuery {
    fn run(&self, graph: &KnowledgeGraph) -> Vec<Vec<PathBuf>> {
        // Build file-level adjacency ignoring self loops
        let mut file_ids: Vec<PathBuf> = graph.files.keys().cloned().collect();
        file_ids.sort();
        let index: HashMap<PathBuf, usize> =
            file_ids.iter().cloned().enumerate().map(|(i, p)| (p, i)).collect();

        // Map each relationship to file->file edge
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); file_ids.len()];
        // Prepare item->file map
        let mut item_to_file: HashMap<ItemId, usize> = HashMap::new();
        for (path, file) in &graph.files {
            if let Some(&i) = index.get(path) {
                for it in &file.items {
                    item_to_file.insert(it.id.clone(), i);
                }
            }
        }
        for rel in &graph.relationships {
            // Only consider call edges for cycle detection call-graph
            if !matches!(rel.relationship_type, crate::graph::RelationshipType::Calls { .. }) {
                continue;
            }
            if let (Some(&u), Some(&v)) =
                (item_to_file.get(&rel.from_item), item_to_file.get(&rel.to_item))
            {
                if u != v {
                    adj[u].push(v);
                }
            }
        }

        // Deduplicate adjacency lists to avoid redundant parallel edges
        for neigh in &mut adj {
            neigh.sort_unstable();
            neigh.dedup();
        }

        // DFS to find simple cycles (not necessarily unique, limited dedup)
        let mut visited = vec![false; file_ids.len()];
        let mut stack = vec![false; file_ids.len()];
        let mut path: Vec<usize> = Vec::new();
        let mut cycles: Vec<Vec<PathBuf>> = Vec::new();

        for u in 0..file_ids.len() {
            if !visited[u] {
                dfs(u, &adj, &mut visited, &mut stack, &mut path, &mut cycles, &file_ids);
            }
        }

        cycles
    }
}

/// Compute shortest path between two files (directed edges) on the file-level projection.
pub struct ShortestPathQuery {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl ShortestPathQuery {
    /// Create a shortest path query from `from` to `to` (file paths).
    #[must_use]
    pub fn new<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> Self {
        Self { from: from.as_ref().to_path_buf(), to: to.as_ref().to_path_buf() }
    }
}

impl Query<Vec<PathBuf>> for ShortestPathQuery {
    fn run(&self, graph: &KnowledgeGraph) -> Vec<PathBuf> {
        // Map files to indices
        let mut files: Vec<PathBuf> = graph.files.keys().cloned().collect();
        files.sort();
        let idx: HashMap<PathBuf, usize> =
            files.iter().cloned().enumerate().map(|(i, p)| (p, i)).collect();

        let (Some(&src), Some(&dst)) = (idx.get(&self.from), idx.get(&self.to)) else {
            return Vec::new();
        };

        // Build item->file index and adjacency list
        let mut item_to_file: HashMap<ItemId, usize> = HashMap::new();
        for (path, file) in &graph.files {
            if let Some(&i) = idx.get(path) {
                for it in &file.items {
                    item_to_file.insert(it.id.clone(), i);
                }
            }
        }
        let mut adj: Vec<Vec<usize>> = vec![Vec::new(); files.len()];
        for rel in &graph.relationships {
            if let (Some(&u), Some(&v)) =
                (item_to_file.get(&rel.from_item), item_to_file.get(&rel.to_item))
            {
                if u != v {
                    adj[u].push(v);
                }
            }
        }

        // BFS
        let mut prev: Vec<Option<usize>> = vec![None; files.len()];
        let mut q: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
        let mut visited = vec![false; files.len()];
        visited[src] = true;
        q.push_back(src);
        while let Some(u) = q.pop_front() {
            if u == dst {
                break;
            }
            for &v in &adj[u] {
                if !visited[v] {
                    visited[v] = true;
                    prev[v] = Some(u);
                    q.push_back(v);
                }
            }
        }

        if !visited[dst] {
            return Vec::new();
        }

        // Reconstruct path
        let mut path_indices: Vec<usize> = Vec::new();
        let mut cur = dst;
        while let Some(p) = prev[cur] {
            path_indices.push(cur);
            cur = p;
        }
        path_indices.push(src);
        path_indices.reverse();

        path_indices.into_iter().map(|i| files[i].clone()).collect()
    }
}

/// Metric for degree centrality used by `HubsQuery` and `ModuleCentralityQuery`.
pub enum CentralityMetric {
    In,
    Out,
    Total,
}

/// Compute top-N files by degree centrality.
pub struct HubsQuery {
    pub metric: CentralityMetric,
    pub top: usize,
}

impl HubsQuery {
    /// Create a hubs query for the given metric and number of results.
    #[must_use]
    pub fn new(metric: CentralityMetric, top: usize) -> Self {
        Self { metric, top }
    }
}

impl Query<Vec<(PathBuf, usize, usize)>> for HubsQuery {
    fn run(&self, graph: &KnowledgeGraph) -> Vec<(PathBuf, usize, usize)> {
        // Map files to indices
        let mut files: Vec<PathBuf> = graph.files.keys().cloned().collect();
        files.sort();
        let idx: HashMap<PathBuf, usize> =
            files.iter().cloned().enumerate().map(|(i, p)| (p, i)).collect();

        // Build item->file index and degree counters
        let mut item_to_file: HashMap<ItemId, usize> = HashMap::new();
        for (path, file) in &graph.files {
            if let Some(&i) = idx.get(path) {
                for it in &file.items {
                    item_to_file.insert(it.id.clone(), i);
                }
            }
        }
        let n = files.len();
        let mut indeg = vec![0usize; n];
        let mut outdeg = vec![0usize; n];

        // Count edges at file level; ignore self-loops
        for rel in &graph.relationships {
            if let (Some(&u), Some(&v)) =
                (item_to_file.get(&rel.from_item), item_to_file.get(&rel.to_item))
            {
                if u != v {
                    outdeg[u] += 1;
                    indeg[v] += 1;
                }
            }
        }

        let mut rows: Vec<(PathBuf, usize, usize)> =
            (0..n).map(|i| (files[i].clone(), indeg[i], outdeg[i])).collect();

        // Sort by chosen metric desc, then by path asc for stability
        rows.sort_by(|a, b| {
            let (ai, ao) = (a.1, a.2);
            let (bi, bo) = (b.1, b.2);
            let ak = match self.metric {
                CentralityMetric::In => ai,
                CentralityMetric::Out => ao,
                CentralityMetric::Total => ai + ao,
            };
            let bk = match self.metric {
                CentralityMetric::In => bi,
                CentralityMetric::Out => bo,
                CentralityMetric::Total => bi + bo,
            };
            bk.cmp(&ak).then_with(|| a.0.cmp(&b.0))
        });

        rows.truncate(self.top);
        rows
    }
}

/// Find items without inbound Uses/Calls edges.
///
/// Skips the synthetic file-level module (first item in each file). By default,
/// public items are excluded (they may be used by downstream crates). Set `include_public`
/// to include them as well.
pub struct UnreferencedItemsQuery {
    pub include_public: bool,
    pub exclude: Option<Regex>,
}

impl UnreferencedItemsQuery {
    #[must_use]
    pub fn new(include_public: bool, exclude: Option<Regex>) -> Self {
        Self { include_public, exclude }
    }
}

impl Query<Vec<(PathBuf, String, String, String, String)>> for UnreferencedItemsQuery {
    fn run(&self, graph: &KnowledgeGraph) -> Vec<(PathBuf, String, String, String, String)> {
        use crate::graph::{ItemType, RelationshipType, Visibility};
        let mut used: HashSet<ItemId> = HashSet::new();
        for rel in &graph.relationships {
            match rel.relationship_type {
                RelationshipType::Uses { .. } | RelationshipType::Calls { .. } => {
                    used.insert(rel.to_item.clone());
                }
                _ => {}
            }
        }

        let mut out: Vec<(PathBuf, String, String, String, String)> = Vec::new();
        for (path, file) in &graph.files {
            if let Some(re) = &self.exclude {
                if re.is_match(&path.display().to_string()) {
                    continue;
                }
            }
            for (idx, item) in file.items.iter().enumerate() {
                if idx == 0 {
                    continue;
                }
                if !self.include_public {
                    if let Visibility::Public = item.visibility {
                        continue;
                    }
                }
                if used.contains(&item.id) {
                    continue;
                }

                let kind = match &item.item_type {
                    ItemType::Module { .. } => "Module",
                    ItemType::Function { .. } => "Function",
                    ItemType::Struct { .. } => "Struct",
                    ItemType::Enum { .. } => "Enum",
                    ItemType::Trait { .. } => "Trait",
                    ItemType::Impl { .. } => "Impl",
                    ItemType::Const => "Const",
                    ItemType::Static { .. } => "Static",
                    ItemType::Type => "Type",
                    ItemType::Macro => "Macro",
                };
                let vis = match item.visibility {
                    Visibility::Public => "public",
                    Visibility::Private => "private",
                    Visibility::PubCrate => "pub(crate)",
                    Visibility::PubSuper => "pub(super)",
                    Visibility::PubIn(_) => "pub(in)",
                };
                out.push((
                    path.clone(),
                    item.id.0.clone(),
                    item.name.to_string(),
                    kind.to_string(),
                    vis.to_string(),
                ));
            }
        }
        out
    }
}

// Detailed info for a single item id
#[derive(Debug, Serialize)]
pub struct ItemInfoRelationEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub relation: String,
    pub context: String,
}

#[derive(Debug, Serialize)]
pub struct ItemInfoResult {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub visibility: String,
    pub path: String,
    pub line_start: usize,
    pub line_end: usize,
    pub code: Option<String>,
    pub inbound: Vec<ItemInfoRelationEntry>,
    pub outbound: Vec<ItemInfoRelationEntry>,
}

pub struct ItemInfoQuery {
    pub item_id: crate::graph::ItemId,
    pub show_code: bool,
}

impl ItemInfoQuery {
    #[must_use]
    pub fn new(item_id: crate::graph::ItemId, show_code: bool) -> Self {
        Self { item_id, show_code }
    }
}

impl Query<Option<ItemInfoResult>> for ItemInfoQuery {
    fn run(&self, graph: &KnowledgeGraph) -> Option<ItemInfoResult> {
        use crate::graph::{ItemType, Visibility};
        // Build an index of ItemId -> (path, &Item)
        let mut idx: HashMap<&crate::graph::ItemId, (&PathBuf, &crate::graph::Item)> =
            HashMap::new();
        for (p, f) in &graph.files {
            for it in &f.items {
                idx.insert(&it.id, (p, it));
            }
        }
        let (path, item) = idx.get(&self.item_id)?.to_owned();

        let kind = match &item.item_type {
            ItemType::Module { .. } => "Module",
            ItemType::Function { .. } => "Function",
            ItemType::Struct { .. } => "Struct",
            ItemType::Enum { .. } => "Enum",
            ItemType::Trait { .. } => "Trait",
            ItemType::Impl { .. } => "Impl",
            ItemType::Const => "Const",
            ItemType::Static { .. } => "Static",
            ItemType::Type => "Type",
            ItemType::Macro => "Macro",
        }
        .to_string();
        let visibility = match &item.visibility {
            Visibility::Public => "public".to_string(),
            Visibility::Private => "private".to_string(),
            Visibility::PubCrate => "pub(crate)".to_string(),
            Visibility::PubSuper => "pub(super)".to_string(),
            Visibility::PubIn(p) => format!("pub(in {p})"),
        };

        // Gather relations
        let mut inbound: Vec<ItemInfoRelationEntry> = Vec::new();
        let mut outbound: Vec<ItemInfoRelationEntry> = Vec::new();
        let rel_to_string = |r: &crate::graph::RelationshipType| -> String {
            match r {
                crate::graph::RelationshipType::Uses { import_type } => {
                    format!("Uses:{import_type}")
                }
                crate::graph::RelationshipType::Implements { trait_name } => {
                    format!("Implements:{trait_name}")
                }
                crate::graph::RelationshipType::Contains { containment_type } => {
                    format!("Contains:{containment_type}")
                }
                crate::graph::RelationshipType::Extends { extension_type } => {
                    format!("Extends:{extension_type}")
                }
                crate::graph::RelationshipType::Calls { call_type } => format!("Calls:{call_type}"),
            }
        };
        for r in &graph.relationships {
            if r.to_item == item.id {
                if let Some((pp, it)) = idx.get(&r.from_item) {
                    inbound.push(ItemInfoRelationEntry {
                        id: r.from_item.0.clone(),
                        name: it.name.to_string(),
                        path: pp.display().to_string(),
                        relation: rel_to_string(&r.relationship_type),
                        context: r.context.clone(),
                    });
                }
            }
            if r.from_item == item.id {
                if let Some((pp, it)) = idx.get(&r.to_item) {
                    outbound.push(ItemInfoRelationEntry {
                        id: r.to_item.0.clone(),
                        name: it.name.to_string(),
                        path: pp.display().to_string(),
                        relation: rel_to_string(&r.relationship_type),
                        context: r.context.clone(),
                    });
                }
            }
        }

        // Optional code snippet
        let mut code: Option<String> = None;
        if self.show_code {
            if let Ok(content) = std::fs::read_to_string(path) {
                let lines: Vec<&str> = content.lines().collect();
                let s = item.location.line_start.saturating_sub(1);
                let e = item.location.line_end.min(lines.len());
                if s < e {
                    code = Some(lines[s..e].join("\n"));
                }
            }
        }

        Some(ItemInfoResult {
            id: item.id.0.clone(),
            name: item.name.to_string(),
            kind,
            visibility,
            path: path.display().to_string(),
            line_start: item.location.line_start,
            line_end: item.location.line_end,
            code,
            inbound,
            outbound,
        })
    }
}

/// Compute top-N modules (by directory) by degree centrality.
pub struct ModuleCentralityQuery {
    pub metric: CentralityMetric,
    pub top: usize,
}

impl ModuleCentralityQuery {
    /// Create a module centrality query for the given metric and number of results.
    #[must_use]
    pub fn new(metric: CentralityMetric, top: usize) -> Self {
        Self { metric, top }
    }
}

impl Query<Vec<(PathBuf, usize, usize)>> for ModuleCentralityQuery {
    fn run(&self, graph: &KnowledgeGraph) -> Vec<(PathBuf, usize, usize)> {
        // Build list of modules identified by parent directory of file
        let mut modules: HashSet<PathBuf> = HashSet::new();
        let mut file_to_module: HashMap<PathBuf, PathBuf> = HashMap::new();
        for p in graph.files.keys() {
            let m = p.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf);
            modules.insert(m.clone());
            file_to_module.insert(p.clone(), m);
        }

        let mut mods: Vec<PathBuf> = modules.into_iter().collect();
        mods.sort();
        let midx: HashMap<PathBuf, usize> =
            mods.iter().cloned().enumerate().map(|(i, p)| (p, i)).collect();

        // Map items to module index
        let mut item_to_mod: HashMap<ItemId, usize> = HashMap::new();
        for (path, file) in &graph.files {
            let Some(module_path) = file_to_module.get(path) else { continue };
            if let Some(&mi) = midx.get(module_path) {
                for it in &file.items {
                    item_to_mod.insert(it.id.clone(), mi);
                }
            }
        }

        let n = mods.len();
        let mut indeg = vec![0usize; n];
        let mut outdeg = vec![0usize; n];

        // Count inter-module edges
        for rel in &graph.relationships {
            if let (Some(&u), Some(&v)) =
                (item_to_mod.get(&rel.from_item), item_to_mod.get(&rel.to_item))
            {
                if u != v {
                    outdeg[u] += 1;
                    indeg[v] += 1;
                }
            }
        }

        let mut rows: Vec<(PathBuf, usize, usize)> =
            (0..n).map(|i| (mods[i].clone(), indeg[i], outdeg[i])).collect();

        rows.sort_by(|a, b| {
            let (ai, ao) = (a.1, a.2);
            let (bi, bo) = (b.1, b.2);
            let ak = match self.metric {
                CentralityMetric::In => ai,
                CentralityMetric::Out => ao,
                CentralityMetric::Total => ai + ao,
            };
            let bk = match self.metric {
                CentralityMetric::In => bi,
                CentralityMetric::Out => bo,
                CentralityMetric::Total => bi + bo,
            };
            bk.cmp(&ak).then_with(|| a.0.cmp(&b.0))
        });

        rows.truncate(self.top);
        rows
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{FileNode, Item, ItemType, Relationship, RelationshipType};
    use std::sync::Arc;

    fn make_fn(path: &Path, id_prefix: &str, name: &str) -> Item {
        Item {
            id: ItemId(format!("fn:{}:{}", name, id_prefix)),
            item_type: ItemType::Function { is_async: false, is_const: false },
            name: Arc::from(name),
            visibility: crate::graph::Visibility::Public,
            location: crate::graph::Location {
                file: path.to_path_buf(),
                line_start: 1,
                line_end: 1,
            },
            attributes: vec![],
        }
    }

    // Build a small graph:
    // src/a.rs (A) -> calls -> src/b.rs (B)
    // src/b.rs (B) -> calls -> src/c.rs (C)
    // Optional cycle: c -> a
    fn graph_fixture(with_cycle: bool) -> KnowledgeGraph {
        let mut g = KnowledgeGraph::default();
        let a_path = PathBuf::from("src/a.rs");
        let b_path = PathBuf::from("src/b.rs");
        let c_path = PathBuf::from("src/c.rs");

        let a_item = make_fn(&a_path, "1", "fa");
        let b_item = make_fn(&b_path, "2", "fb");
        let c_item = make_fn(&c_path, "3", "fc");

        g.files.insert(
            a_path.clone(),
            FileNode {
                path: a_path.clone(),
                items: vec![a_item.clone()],
                imports: vec![],
                metrics: Default::default(),
            },
        );
        g.files.insert(
            b_path.clone(),
            FileNode {
                path: b_path.clone(),
                items: vec![b_item.clone()],
                imports: vec![],
                metrics: Default::default(),
            },
        );
        g.files.insert(
            c_path.clone(),
            FileNode {
                path: c_path.clone(),
                items: vec![c_item.clone()],
                imports: vec![],
                metrics: Default::default(),
            },
        );

        g.relationships.push(Relationship {
            from_item: a_item.id.clone(),
            to_item: b_item.id.clone(),
            relationship_type: RelationshipType::Calls { call_type: "test".to_string() },
            strength: 1.0,
            context: String::new(),
        });
        g.relationships.push(Relationship {
            from_item: b_item.id.clone(),
            to_item: c_item.id.clone(),
            relationship_type: RelationshipType::Calls { call_type: "test".to_string() },
            strength: 1.0,
            context: String::new(),
        });
        if with_cycle {
            g.relationships.push(Relationship {
                from_item: c_item.id.clone(),
                to_item: a_item.id.clone(),
                relationship_type: RelationshipType::Calls { call_type: "test".to_string() },
                strength: 1.0,
                context: String::new(),
            });
        }
        g
    }

    #[test]
    fn connected_files_query_basic() {
        let g = graph_fixture(false);
        let q = ConnectedFilesQuery::new("src/a.rs");
        let res = q.run(&g);
        // a is connected to b
        assert!(res.contains(&PathBuf::from("src/b.rs")));
        // b is connected to a and c as well, but for a we expect only b because c is reached via b (no direct edge)
        assert!(!res.contains(&PathBuf::from("src/c.rs")));
    }

    #[test]
    fn function_usage_callers_and_callees() {
        let g = graph_fixture(false);
        // callees of fa should include file of fb
        let q_callees = FunctionUsageQuery::callees("fa");
        let callees = q_callees.run(&g);
        assert!(callees.contains(&PathBuf::from("src/b.rs")));

        // callers of fb should include file of fa
        let q_callers = FunctionUsageQuery::callers("fb");
        let callers = q_callers.run(&g);
        assert!(callers.contains(&PathBuf::from("src/a.rs")));
    }

    #[test]
    fn cycle_detection_detects_simple_cycle() {
        let g = graph_fixture(true);
        let q = CycleDetectionQuery::new();
        let cycles = q.run(&g);
        // Expect at least one cycle involving a -> b -> c -> a
        assert!(cycles.iter().any(|cyc| {
            // convert to set for containment check
            let names: std::collections::HashSet<_> = cyc.iter().cloned().collect();
            names.contains(&PathBuf::from("src/a.rs"))
                && names.contains(&PathBuf::from("src/b.rs"))
                && names.contains(&PathBuf::from("src/c.rs"))
        }));
    }

    #[test]
    fn trait_impls_basic() {
        let mut g = KnowledgeGraph::default();
        let p = PathBuf::from("src/x.rs");
        let impl_item = Item {
            id: ItemId("impl:X:Display".to_string()),
            item_type: ItemType::Impl {
                trait_name: Some(Arc::from("Display")),
                type_name: Arc::from("X"),
            },
            name: Arc::from("impl Display for X"),
            visibility: crate::graph::Visibility::PubCrate,
            location: crate::graph::Location { file: p.clone(), line_start: 1, line_end: 1 },
            attributes: vec![],
        };
        g.files.insert(
            p.clone(),
            FileNode {
                path: p.clone(),
                items: vec![impl_item],
                imports: vec![],
                metrics: Default::default(),
            },
        );

        let q = TraitImplsQuery::new("Display");
        let rows = q.run(&g);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, p);
        assert_eq!(rows[0].1, "X");
    }
}
