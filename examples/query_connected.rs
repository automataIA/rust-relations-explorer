use knowledge_rs::graph::{ItemId, KnowledgeGraph};
use knowledge_rs::utils::cache::CacheMode;

fn main() {
    let root = std::path::Path::new(".");
    let graph = KnowledgeGraph::build_from_directory_with_cache_opts(root, CacheMode::Use, false)
        .expect("build graph");

    // Pick a file like src/lib.rs if it exists; otherwise, print the first file.
    let target = std::path::Path::new("src/lib.rs");
    let file = if graph.files.contains_key(target) {
        target
    } else if let Some(first) = graph.files.keys().next() {
        first.as_path()
    } else {
        eprintln!("No files in graph");
        return;
    };

    // Build an index from ItemId -> owning file
    let mut owner: std::collections::HashMap<ItemId, std::path::PathBuf> = std::collections::HashMap::new();
    for (path, node) in &graph.files {
        for it in &node.items {
            owner.insert(it.id.clone(), path.clone());
            }
        }

    // List files that have direct relationships with any item in the target file
    let mut related = std::collections::BTreeSet::new();
    for rel in &graph.relationships {
        let from_file = owner.get(&rel.from_item);
        let to_file = owner.get(&rel.to_item);
        if let (Some(f), Some(t)) = (from_file, to_file) {
            if f == file && t != file {
                related.insert(t.clone());
            } else if t == file && f != file {
                related.insert(f.clone());
            }
        }
    }
    println!("Connected files to {}:", file.display());
    for f in related {
        println!("- {}", f.display());
    }
}
