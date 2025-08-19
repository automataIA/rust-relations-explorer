use rust_relations_explorer::utils::{file_walker, table};
use std::fs;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn table_renderer_produces_expected_grid() {
    let headers = ["A", "B"];
    let rows = vec![vec!["x".into(), "y".into()], vec!["long".into(), "z".into()]];
    let out = table::render(&headers, &rows);
    assert!(out.starts_with("+"));
    assert!(out.contains("| A"));
    assert!(out.contains("long"));
}

#[test]
fn file_walker_respects_ignore_and_no_ignore() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    // Create files
    write(&src.join("lib.rs"), "pub mod hidden;\n");
    write(&src.join("hidden.rs"), "pub fn f() {}\n");

    // Add .gitignore to hide hidden.rs
    write(&root.join(".gitignore"), "src/hidden.rs\n");

    // Default: should not see hidden.rs
    let files = file_walker::rust_files(root.to_str().unwrap());
    let listed: Vec<_> =
        files.iter().map(|s| s.ends_with("lib.rs") || s.ends_with("hidden.rs")).collect();
    assert!(listed.iter().any(|&b| b));
    assert!(!files.iter().any(|s| s.ends_with("hidden.rs")));

    // no_ignore=true: should include hidden.rs
    let files_all = file_walker::rust_files_with_options(root.to_str().unwrap(), true);
    assert!(files_all.iter().any(|s| s.ends_with("hidden.rs")));
}

fn write(path: &std::path::Path, s: &str) {
    let mut f = fs::File::create(path).unwrap();
    f.write_all(s.as_bytes()).unwrap();
}
