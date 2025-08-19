use rust_relations_explorer::graph::KnowledgeGraph;
use rust_relations_explorer::utils::cache::CacheMode;

fn main() {
    let root = std::path::Path::new(".");
    let no_ignore = false; // set true to include ignored files
    let graph =
        KnowledgeGraph::build_from_directory_with_cache_opts(root, CacheMode::Use, no_ignore)
            .expect("build graph");
    println!(
        "Built graph: files={}, relationships={}",
        graph.files.len(),
        graph.relationships.len()
    );
}
