use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn write_file(path: &PathBuf, content: &str) {
    if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

#[test]
fn build_uses_config_overrides_and_save() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    // simple crate
    write_file(&src.join("lib.rs"), "pub fn top() {}\n");

    // config file overriding DOT and SVG options + query defaults
    let cfg_path = root.join("knowledge-rs.toml");
    write_file(&cfg_path, r#"
[dot]
clusters = true
legend = true
theme = "dark"
rankdir = "TB"
splines = "polyline"
rounded = false

[svg]
interactive = true

[query]
default_format = "json"
"#);

    let dot_out = root.join("graph.dot");
    let svg_out = root.join("graph.svg");
    let json_out = root.join("graph.json");

    // run build with config and outputs and save
    let mut cmd = Command::cargo_bin("knowledge-rs").unwrap();
    cmd.arg("build")
        .arg("--path").arg(&root)
        .arg("--config").arg(&cfg_path)
        .arg("--dot").arg(&dot_out)
        .arg("--json").arg(&json_out)
        .arg("--save").arg(root.join("saved.json"));
    // Append svg flag only if graphviz dot is present
    let dot_available = Command::new("dot").arg("-V").output().is_ok();
    if dot_available {
        cmd.arg("--svg").arg(&svg_out);
    }
    cmd.assert().success().stdout(predicate::str::contains("Build completed for path"));

    // dot reflects config overrides
    let dot_str = fs::read_to_string(&dot_out).unwrap();
    assert!(dot_str.contains("rankdir=TB"));
    assert!(dot_str.contains("splines=polyline"));

    // svg produced if dot is available
    if dot_available {
        assert!(svg_out.exists());
    }
    assert!(json_out.exists());
}

#[test]
fn queries_without_graph_build_from_path_and_text_formats() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "mod a; mod b; pub use a::foo;\n");
    write_file(&src.join("a.rs"), "pub fn foo() {}\n");
    write_file(&src.join("b.rs"), "pub fn bar() { crate::a::foo(); }\n");

    // connected-files text output via table (no --graph)
    let mut cf = Command::cargo_bin("knowledge-rs").unwrap();
    cf.arg("query").arg("connected-files")
        .arg("--path").arg(&root)
        .arg("--file").arg(src.join("a.rs"))
        .arg("--format").arg("text");
    cf.assert().success().stdout(predicate::str::contains("#").and(predicate::str::contains("Path")));

    // function-usage callees branch (no --graph)
    let mut fu = Command::cargo_bin("knowledge-rs").unwrap();
    fu.arg("query").arg("function-usage")
        .arg("--path").arg(&root)
        .arg("--function").arg("bar")
        .arg("--direction").arg("callees")
        .arg("--format").arg("text");
    fu.assert().success();

    // hubs with metric in and text output
    let mut hubs = Command::cargo_bin("knowledge-rs").unwrap();
    hubs.arg("query").arg("hubs")
        .arg("--path").arg(&root)
        .arg("--metric").arg("in")
        .arg("--top").arg("5")
        .arg("--format").arg("text");
    hubs.assert().success().stdout(predicate::str::contains("In").and(predicate::str::contains("Out")));

    // module-centrality with metric out and text output
    let mut mc = Command::cargo_bin("knowledge-rs").unwrap();
    mc.arg("query").arg("module-centrality")
        .arg("--path").arg(&root)
        .arg("--metric").arg("out")
        .arg("--top").arg("5")
        .arg("--format").arg("text");
    mc.assert().success().stdout(predicate::str::contains("Module").and(predicate::str::contains("Total")));

    // path query no path branch should print <no path>
    let mut pathq = Command::cargo_bin("knowledge-rs").unwrap();
    pathq.arg("query").arg("path")
        .arg("--path").arg(&root)
        .arg("--from").arg(src.join("a.rs"))
        .arg("--to").arg(src.join("b.rs"))
        .arg("--format").arg("text");
    pathq.assert().success().stdout(predicate::str::contains("<no path>"));
}

#[test]
fn cycles_text_output_branch() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    // small project likely without cycles; text output still goes through branch
    write_file(&src.join("lib.rs"), "mod a; mod b;\n");
    write_file(&src.join("a.rs"), "pub fn fa() {}\n");
    write_file(&src.join("b.rs"), "pub fn fb() { }\n");

    let mut cy = Command::cargo_bin("knowledge-rs").unwrap();
    cy.arg("query").arg("cycles")
        .arg("--path").arg(&root)
        .arg("--format").arg("text");
    cy.assert().success();
}
