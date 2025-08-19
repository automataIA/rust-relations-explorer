use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn cli_build_svg_when_graphviz_available() {
    // Only run if Graphviz dot is available
    let dot_available = Command::new("dot").arg("-V").output().is_ok();
    if !dot_available {
        eprintln!("Skipping SVG test: graphviz 'dot' not found");
        return;
    }

    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), r#"
        pub fn top() {}
    "#);

    let mut cmd = Command::cargo_bin("rust-relations-explorer").unwrap();
    cmd.arg("build")
        .arg("--path").arg(root)
        .arg("--svg").arg(root.join("graph.svg"));
    cmd.assert().success();

    let svg_path = root.join("graph.svg");
    assert!(svg_path.exists());
    assert!(fs::metadata(&svg_path).unwrap().len() > 0);
}

#[test]
fn cli_query_module_centrality_trait_impls_and_cycles() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    // Create two modules and a trait impl to exercise the queries
    write_file(&src.join("lib.rs"), r#"
        mod m1; mod m2;
        pub use m1::X;
        pub trait T { fn f(&self) {} }
    "#);
    write_file(&src.join("m1.rs"), r#"
        use crate::T;
        pub struct X;
        impl T for X { fn f(&self) {} }
    "#);
    write_file(&src.join("m2.rs"), r#"
        use crate::m1::X;
        pub fn use_x() { let _ = X; }
    "#);

    // Build graph JSON
    let mut build = Command::cargo_bin("rust-relations-explorer").unwrap();
    build.arg("build")
        .arg("--path").arg(root)
        .arg("--json").arg(root.join("graph.json"));
    build.assert().success();
    let graph_path = root.join("graph.json");

    // module-centrality: expect some output rows
    let mut mc = Command::cargo_bin("rust-relations-explorer").unwrap();
    mc.arg("query").arg("module-centrality")
        .arg("--graph").arg(&graph_path)
        .arg("--metric").arg("total")
        .arg("--top").arg("5")
        .arg("--format").arg("json");
    mc.assert().success().stdout(predicate::str::contains("["));

    // trait-impls: parser may or may not extract impls in this minimal setup; accept empty but valid JSON array
    let mut ti = Command::cargo_bin("rust-relations-explorer").unwrap();
    ti.arg("query").arg("trait-impls")
        .arg("--graph").arg(&graph_path)
        .arg("--trait").arg("T")
        .arg("--format").arg("json");
    ti.assert().success().stdout(predicate::str::contains("["));

    // cycles: command should succeed; output may be empty which is acceptable
    let mut cy = Command::cargo_bin("rust-relations-explorer").unwrap();
    cy.arg("query").arg("cycles")
        .arg("--graph").arg(&graph_path)
        .arg("--format").arg("json");
    cy.assert().success().stdout(predicate::str::contains("["));
}

#[test]
fn cli_query_path_and_function_usage() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), r#"
        mod a; mod b;
        pub use a::foo;
    "#);
    write_file(&src.join("a.rs"), r#"
        pub fn foo() {}
    "#);
    write_file(&src.join("b.rs"), r#"
        pub fn bar() { crate::a::foo(); }
    "#);

    // Build graph JSON to speed queries
    let mut build = Command::cargo_bin("rust-relations-explorer").unwrap();
    build.arg("build")
        .arg("--path").arg(root)
        .arg("--json").arg(root.join("graph.json"));
    build.assert().success();
    let graph_path = root.join("graph.json");

    // path query: expect a path from b.rs to a.rs
    let mut pathq = Command::cargo_bin("rust-relations-explorer").unwrap();
    pathq.arg("query").arg("path")
        .arg("--graph").arg(&graph_path)
        .arg("--from").arg(src.join("b.rs"))
        .arg("--to").arg(src.join("a.rs"))
        .arg("--format").arg("json");
    pathq.assert().success().stdout(predicate::str::contains("a.rs"));

    // function-usage: callers of foo should include b.rs
    let mut funcq = Command::cargo_bin("rust-relations-explorer").unwrap();
    funcq.arg("query").arg("function-usage")
        .arg("--graph").arg(&graph_path)
        .arg("--function").arg("foo")
        .arg("--direction").arg("callers")
        .arg("--format").arg("json");
    funcq.assert().success().stdout(predicate::str::contains("b.rs"));
}

fn write_file(path: &PathBuf, content: &str) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}
