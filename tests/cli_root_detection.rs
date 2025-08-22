use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn write_file(path: &PathBuf, content: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

#[test]
fn root_detection_without_explicit_path() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "pub fn top() {}\n");

    // Run from the temp dir as CWD without --path; command should detect src/ and succeed
    let mut cmd = Command::cargo_bin("rust-relations-explorer").unwrap();
    cmd.current_dir(root).arg("build");
    cmd.assert().success();
}

#[test]
fn verbose_prints_project_root_hint() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "pub fn top() {}\n");

    let mut cmd = Command::cargo_bin("rust-relations-explorer").unwrap();
    cmd.current_dir(root).arg("-v").arg("build");
    cmd.assert().success().stderr(predicate::str::contains("Using project root:"));
}
