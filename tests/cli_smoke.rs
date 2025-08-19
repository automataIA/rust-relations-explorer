use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

// Bottom-up: simple CLI smoke test for build and a query
#[test]
fn cli_build_and_connected_files_smoke() {
    // Arrange: temp project with two files and a simple import
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    // Create lib.rs and a.rs
    write_file(&src.join("lib.rs"), r#"
        pub mod a;
        pub fn top() {}
    "#);
    write_file(&src.join("a.rs"), r#"
        use crate::top;
        pub fn child() { top(); }
    "#);

    // Act: run build command
    let mut cmd = Command::cargo_bin("rust-relations-explorer").unwrap();
    cmd.arg("build")
        .arg("--path").arg(root)
        .arg("--json").arg(root.join("graph.json"));
    cmd.assert().success();

    // Assert: graph file exists and contains the module
    let json_path = root.join("graph.json");
    assert!(json_path.exists());
    let content = fs::read_to_string(&json_path).unwrap();
    assert!(content.contains("lib.rs"));

    // Act: run query connected-files
    let mut cmd2 = Command::cargo_bin("rust-relations-explorer").unwrap();
    cmd2.arg("query").arg("connected-files")
        .arg("--path").arg(root)
        .arg("--file").arg(src.join("a.rs"))
        .arg("--format").arg("json");
    cmd2.assert().success().stdout(predicate::str::contains("lib.rs"));
}

fn write_file(path: &PathBuf, content: &str) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}
