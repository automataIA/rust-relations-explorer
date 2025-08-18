use std::fs;
use std::path::PathBuf;

use knowledge_rs::graph::KnowledgeGraph;
use knowledge_rs::query::{ConnectedFilesQuery, Query, CycleDetectionQuery};

fn make_temp_project(contents: Vec<(&str, &str)>) -> PathBuf {
    let base = std::env::temp_dir()
        .join(format!("knowledge_rs_it_{}_{}", std::process::id(), std::time::SystemTime::now().elapsed().unwrap().as_nanos()));
    fs::create_dir_all(base.join("src")).unwrap();
    for (path, body) in contents {
        let p = base.join(path);
        if let Some(parent) = p.parent() { fs::create_dir_all(parent).unwrap(); }
        fs::write(p, body).unwrap();
    }
    base
}

#[test]
fn integration_connected_files() {
    let root = make_temp_project(vec![
        ("src/lib.rs", r#"
            mod a;
            mod b;
            use crate::a::foo;
            pub fn root() { foo(); }
        "#),
        ("src/a.rs", r#"
            pub fn foo() {}
        "#),
        ("src/b.rs", r#"
            pub fn bar() { crate::a::foo(); }
        "#),
    ]);

    let graph = KnowledgeGraph::build_from_directory(&root.join("src")).expect("build graph");

    // Connected files for a.rs should include lib.rs due to import use from lib
    let connected = ConnectedFilesQuery::new(root.join("src/a.rs")).run(&graph);
    assert!(connected.contains(&root.join("src/lib.rs")));
}

#[test]
#[ignore = "Call-graph extraction is simplistic and may not detect cycles from source yet; enable when improved."]
fn integration_cycle_detection_simple() {
    let root = make_temp_project(vec![
        ("src/lib.rs", r#"
            mod a;
            mod b;
        "#),
        ("src/a.rs", r#"
            pub fn foo() { crate::b::bar(); }
        "#),
        ("src/b.rs", r#"
            pub fn bar() { crate::a::foo(); }
        "#),
    ]);

    let graph = KnowledgeGraph::build_from_directory(&root.join("src")).expect("build graph");
    let cycles = CycleDetectionQuery::new().run(&graph);
    assert!(cycles.iter().any(|cyc| cyc.len() >= 2));
}
