use assert_cmd::prelude::*;
use std::fs;
use std::io::Write;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn dot_generator_clusters_and_flat_themes() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    // Minimal multi-file crate to trigger clusters/children
    write_file(
        &src.join("lib.rs"),
        r#"
        mod m;
        pub struct S; pub enum E { A }
        pub trait T { fn f(&self); }
    "#,
    );
    write_file(
        &src.join("m.rs"),
        r#"
        pub fn foo() {}
    "#,
    );

    // Build a graph JSON then load it via CLI to ensure consistency
    let graph_json = root.join("graph.json");
    let mut build = Command::cargo_bin("rust-relations-explorer").unwrap();
    build.arg("build").arg("--path").arg(root).arg("--json").arg(&graph_json);
    build.assert().success();

    // Use the CLI to emit DOT with clusters + legend + dark theme
    let dot_dark = graph_json.parent().unwrap().join("dark.dot");
    let mut build_dot_dark = Command::cargo_bin("rust-relations-explorer").unwrap();
    build_dot_dark
        .arg("build")
        .arg("--path")
        .arg(graph_json.parent().unwrap())
        .arg("--dot")
        .arg(&dot_dark)
        .arg("--dot-clusters")
        .arg("on")
        .arg("--dot-legend")
        .arg("on")
        .arg("--dot-theme")
        .arg("dark")
        .arg("--dot-rankdir")
        .arg("TB")
        .arg("--dot-splines")
        .arg("polyline")
        .arg("--dot-rounded")
        .arg("off");
    build_dot_dark.assert().success();

    let dot_dark_str = fs::read_to_string(&dot_dark).unwrap();
    assert!(dot_dark_str.contains("digraph KnowledgeRS"));
    assert!(dot_dark_str.contains("rankdir=TB"));
    assert!(dot_dark_str.contains("splines=polyline"));
    assert!(dot_dark_str.contains("subgraph \"cluster_")); // clusters enabled
    assert!(dot_dark_str.contains("label=\"Legend\"")); // legend enabled

    // Emit DOT without clusters, light theme defaults
    let dot_light = graph_json.parent().unwrap().join("light.dot");
    let mut build_dot_light = Command::cargo_bin("rust-relations-explorer").unwrap();
    build_dot_light
        .arg("build")
        .arg("--path")
        .arg(graph_json.parent().unwrap())
        .arg("--dot")
        .arg(&dot_light)
        .arg("--dot-clusters")
        .arg("off")
        .arg("--dot-legend")
        .arg("off");
    build_dot_light.assert().success();

    let dot_light_str = fs::read_to_string(&dot_light).unwrap();
    assert!(dot_light_str.contains("rankdir=LR")); // default
    assert!(!dot_light_str.contains("cluster_")); // no clusters
}

fn write_file(path: &std::path::Path, content: &str) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}
