use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn cli_build_produces_dot_and_json_and_hubs_query_works() {
    // Arrange: temp project with two files and a simple call relationship
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), r#"
        pub mod a;
        pub fn top() {}
    "#);
    write_file(&src.join("a.rs"), r#"
        use crate::top;
        pub fn child() { top(); }
    "#);

    // Act: run build with dot and json outputs
    let mut cmd = Command::cargo_bin("knowledge-rs").unwrap();
    cmd.arg("build")
        .arg("--path").arg(root)
        .arg("--json").arg(root.join("graph.json"))
        .arg("--dot").arg(root.join("graph.dot"));
    cmd.assert().success();

    // Assert: outputs exist and are non-empty
    let json_path = root.join("graph.json");
    let dot_path = root.join("graph.dot");
    assert!(json_path.exists());
    assert!(dot_path.exists());
    assert!(fs::metadata(&json_path).unwrap().len() > 0);
    assert!(fs::metadata(&dot_path).unwrap().len() > 0);

    // Act: run hubs query on saved graph
    let mut q = Command::cargo_bin("knowledge-rs").unwrap();
    q.arg("query").arg("hubs")
        .arg("--graph").arg(&json_path)
        .arg("--metric").arg("total")
        .arg("--top").arg("5")
        .arg("--format").arg("json");
    q.assert().success().stdout(predicate::str::contains("["));
}

fn write_file(path: &PathBuf, content: &str) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}
