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
fn invalid_enum_value_for_format_yields_error() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "pub fn top() {}\n");

    let mut cmd = Command::cargo_bin("rust-relations-explorer").unwrap();
    cmd.arg("query").arg("cycles").arg("--path").arg(root).arg("--format").arg("nope");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'nope' for '--format"))
        .stderr(predicate::str::contains("[possible values: text, json]"));
}

#[test]
fn valid_enum_values_are_accepted() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "pub fn top() {}\n");

    for fmt in ["text", "json"] {
        let mut cmd = Command::cargo_bin("rust-relations-explorer").unwrap();
        cmd.arg("query").arg("cycles").arg("--path").arg(root).arg("--format").arg(fmt);
        cmd.assert().success();
    }
}
