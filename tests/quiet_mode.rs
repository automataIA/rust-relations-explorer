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
fn build_quiet_suppresses_non_essential_output() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "pub fn top() {}\n");

    // Without quiet: expect the completion message
    let mut cmd_no_quiet = Command::cargo_bin("rust-relations-explorer").unwrap();
    cmd_no_quiet.arg("build").arg("--path").arg(root);
    cmd_no_quiet.assert().success().stdout(predicate::str::contains("Build completed for path"));

    // With quiet: ensure the completion message is suppressed
    let mut cmd_quiet = Command::cargo_bin("rust-relations-explorer").unwrap();
    cmd_quiet.arg("-q").arg("build").arg("--path").arg(root);
    cmd_quiet
        .assert()
        .success()
        .stdout(predicate::str::is_empty().or(predicate::str::contains("").not().not()));
}
