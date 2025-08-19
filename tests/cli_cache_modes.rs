use assert_cmd::prelude::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn cli_build_cache_modes_succeed() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    // Minimal crate
    write_file(&src.join("lib.rs"), r#"
        pub fn f() {}
    "#);

    let json = root.join("graph.json");

    // Default (Use cache)
    let mut use_cache = Command::cargo_bin("rust-relations-explorer").unwrap();
    use_cache.arg("build")
        .arg("--path").arg(root)
        .arg("--json").arg(&json);
    use_cache.assert().success();
    assert!(json.exists());

    // Rebuild
    let mut rebuild = Command::cargo_bin("rust-relations-explorer").unwrap();
    rebuild.arg("build")
        .arg("--path").arg(root)
        .arg("--json").arg(&json)
        .arg("--rebuild");
    rebuild.assert().success();
    assert!(json.exists());

    // No-cache
    let mut no_cache = Command::cargo_bin("rust-relations-explorer").unwrap();
    no_cache.arg("build")
        .arg("--path").arg(root)
        .arg("--json").arg(&json)
        .arg("--no-cache");
    no_cache.assert().success();
    assert!(json.exists());
}

fn write_file(path: &PathBuf, content: &str) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}
