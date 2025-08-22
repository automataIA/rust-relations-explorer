use assert_cmd::prelude::*;
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
fn connected_files_accepts_positional_file_argument() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "mod a; mod b;\n");
    write_file(&src.join("a.rs"), "pub fn a() {}\n");
    write_file(&src.join("b.rs"), "pub fn b() { super::a::a(); }\n");

    // Run: rust-relations-explorer query connected-files <file> --path <root> --format json
    let mut cmd = Command::cargo_bin("rust-relations-explorer").unwrap();
    cmd.arg("query")
        .arg("connected-files")
        .arg(src.join("a.rs"))
        .arg("--path")
        .arg(root)
        .arg("--format")
        .arg("json");

    cmd.assert().success();
}
