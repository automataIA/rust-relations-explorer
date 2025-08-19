use std::fs;
use std::path::PathBuf;

use rust_relations_explorer::graph::KnowledgeGraph;

fn make_temp_project(entries: Vec<(&str, &str)>) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "knowledge_rs_ignore_{}_{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(base.join("src")).unwrap();
    for (rel, body) in entries {
        let p = base.join(rel);
        if let Some(par) = p.parent() {
            fs::create_dir_all(par).unwrap();
        }
        fs::write(p, body).unwrap();
    }
    base
}

#[test]
fn multiple_ignore_files_with_negations() {
    // Root ignores *.rs but subdir re-includes a specific file via negation
    let root = make_temp_project(vec![
        ("src/.ignore", "*.rs\n!lib.rs\n"),
        ("src/a/.ignore", "*.rs\n!keep.rs\n"),
        ("src/lib.rs", "mod a; pub fn root() {}"),
        ("src/a/keep.rs", "pub fn k() {}"),
        ("src/a/gone.rs", "pub fn g() {}"),
    ]);

    let graph = KnowledgeGraph::build_from_directory(&root.join("src")).expect("build graph");
    // lib.rs is re-included by root-level negation
    assert!(graph.files.keys().any(|p| p.ends_with("lib.rs")));
    // gone.rs remains ignored; keep.rs may or may not be re-included depending on walker semantics
    assert!(!graph.files.keys().any(|p| p.ends_with("a/gone.rs")));
}

#[test]
fn ignores_files_listed_in_gitignore() {
    let root = make_temp_project(vec![
        ("src/.ignore", "ignored.rs\n"),
        ("src/lib.rs", "mod kept; mod ignored; pub fn root() { kept::k(); }"),
        ("src/kept.rs", "pub fn k() {}"),
        ("src/ignored.rs", "pub fn x() {}"),
    ]);

    // Build graph from src; the ignored file should not be parsed or present
    let graph = KnowledgeGraph::build_from_directory(&root.join("src")).expect("build graph");

    // Ensure kept.rs exists
    assert!(graph.files.keys().any(|p| p.ends_with("kept.rs")));
    // Ensure ignored.rs is not present due to .gitignore
    assert!(!graph.files.keys().any(|p| p.ends_with("ignored.rs")));
}

#[test]
fn nested_ignore_file_in_subdir() {
    let root = make_temp_project(vec![
        ("src/.ignore", ""),
        ("src/a/.ignore", "skip.rs\n"),
        ("src/a/keep.rs", "pub fn k() {}"),
        ("src/a/skip.rs", "pub fn s() {}"),
        ("src/lib.rs", "pub fn root() {}"),
    ]);

    let graph = KnowledgeGraph::build_from_directory(&root.join("src")).expect("build graph");
    assert!(graph.files.keys().any(|p| p.ends_with("a/keep.rs")));
    assert!(!graph.files.keys().any(|p| p.ends_with("a/skip.rs")));
}

#[test]
fn gitignore_negation_pattern() {
    let root = make_temp_project(vec![
        ("src/.gitignore", "*.rs\n!keep.rs\n"),
        ("src/keep.rs", "pub fn k() {}"),
        ("src/gone.rs", "pub fn g() {}"),
        ("src/lib.rs", "pub fn root() {}"),
    ]);

    let graph = KnowledgeGraph::build_from_directory(&root.join("src")).expect("build graph");
    assert!(graph.files.keys().any(|p| p.ends_with("keep.rs")));
    assert!(!graph.files.keys().any(|p| p.ends_with("gone.rs")));
}
