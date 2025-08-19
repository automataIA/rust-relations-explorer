use rust_relations_explorer::utils::config::{self};
use std::fs;
use std::path::Path;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, content).unwrap();
}

#[test]
fn parses_full_config_file() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("knowledge-rs.toml");
    let data = r#"
[dot]
clusters = true
legend = false
theme = "dark"
rankdir = "TB"
splines = "ortho"
rounded = true

[svg]
interactive = true

[query]
default_format = "json"
"#;
    write(&cfg_path, data);

    let cfg = config::load_config_at(&cfg_path).expect("config parsed");
    assert_eq!(cfg.dot.as_ref().and_then(|d| d.clusters), Some(true));
    assert_eq!(cfg.dot.as_ref().and_then(|d| d.legend), Some(false));
    assert_eq!(cfg.dot.as_ref().and_then(|d| d.theme.as_ref()).map(|s| s.as_str()), Some("dark"));
    assert_eq!(cfg.dot.as_ref().and_then(|d| d.rankdir.as_ref()).map(|s| s.as_str()), Some("TB"));
    assert_eq!(
        cfg.dot.as_ref().and_then(|d| d.splines.as_ref()).map(|s| s.as_str()),
        Some("ortho")
    );
    assert_eq!(cfg.dot.as_ref().and_then(|d| d.rounded), Some(true));

    assert_eq!(cfg.svg.as_ref().and_then(|s| s.interactive), Some(true));
    assert_eq!(
        cfg.query.as_ref().and_then(|q| q.default_format.as_ref()).map(|s| s.as_str()),
        Some("json")
    );
}

#[test]
fn load_config_near_looks_for_default_name() {
    let tmp = tempfile::tempdir().unwrap();
    // create default location
    let default_path = tmp.path().join("knowledge-rs.toml");
    write(&default_path, "[query]\ndefault_format = 'text'\n");

    let cfg = config::load_config_near(tmp.path()).expect("found default config");
    assert_eq!(cfg.query.and_then(|q| q.default_format), Some("text".to_string()));
}
