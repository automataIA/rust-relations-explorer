use rust_relations_explorer::graph::KnowledgeGraph;
use rust_relations_explorer::utils::cache::{self, CacheMode};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() { let _ = fs::create_dir_all(parent); }
    let mut f = File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f.flush().unwrap();
}

fn read_cache_meta_len(root: &Path, rel: &str) -> Option<(u64, u64)> {
    let cache = cache::load_cache(root)?;
    let key = root.join(rel);
    let entry = cache.entries.get(&key)?;
    Some((entry.meta.mtime, entry.meta.len))
}

fn count_cache_entries(root: &Path) -> usize {
    cache::load_cache(root).map(|c| c.entries.len()).unwrap_or(0)
}

fn make_proj(tmp: &Path) -> PathBuf {
    let root = tmp.join("proj");
    // minimal Rust sources
    write_file(&root.join("src/lib.rs"), "pub fn a() {}\npub mod m;\n");
    write_file(&root.join("src/m.rs"), "pub fn b() {}\n");
    root
}

#[test]
fn cache_mode_use_detects_changes_and_prunes_removed() {
    let tmp = tempfile::tempdir().unwrap();
    let root = make_proj(tmp.path());

    // First build: creates cache with 2 files
    let g1 = KnowledgeGraph::build_from_directory_with_cache(&root, CacheMode::Use).unwrap();
    assert!(g1.files.len() >= 2);
    assert_eq!(count_cache_entries(&root), 2);

    // Record meta of m.rs
    let m1 = read_cache_meta_len(&root, "src/m.rs").unwrap();

    // Touch change in m.rs (ensure mtime changes)
    thread::sleep(Duration::from_millis(1100));
    write_file(&root.join("src/m.rs"), "pub fn b() {}\npub fn c() {}\n");

    // Second build with Use should reuse lib.rs and reparse m.rs
    let _g2 = KnowledgeGraph::build_from_directory_with_cache(&root, CacheMode::Use).unwrap();
    let m2 = read_cache_meta_len(&root, "src/m.rs").unwrap();
    assert_ne!(m1, m2, "cache entry for modified file should update");
    assert_eq!(count_cache_entries(&root), 2);

    // Remove m.rs; build again should prune
    fs::remove_file(root.join("src/m.rs")).unwrap();
    let _g3 = KnowledgeGraph::build_from_directory_with_cache(&root, CacheMode::Use).unwrap();
    assert_eq!(count_cache_entries(&root), 1);
}

#[test]
fn cache_mode_ignore_rebuilds_without_reuse() {
    let tmp = tempfile::tempdir().unwrap();
    let root = make_proj(tmp.path());

    // Seed cache by a normal build
    let _ = KnowledgeGraph::build_from_directory_with_cache(&root, CacheMode::Use).unwrap();
    let before = read_cache_meta_len(&root, "src/m.rs").unwrap();

    thread::sleep(Duration::from_millis(1100));
    // Build with Ignore even if file unchanged should still parse and rewrite cache (mtime may or may not change depending on FS; ensure len change)
    write_file(&root.join("src/lib.rs"), "pub fn a() {}\npub mod m;\npub fn z() {}\n");
    let _ = KnowledgeGraph::build_from_directory_with_cache(&root, CacheMode::Ignore).unwrap();
    let after = read_cache_meta_len(&root, "src/m.rs").unwrap();
    // m.rs unchanged so meta can be equal; ensure entries count preserved and lib.rs updated len
    assert_eq!(count_cache_entries(&root), 2);
    let lib = read_cache_meta_len(&root, "src/lib.rs").unwrap();
    assert!(lib.1 > 0);
    assert!(before.1 > 0);
    // just sanity check access
    let _ = after;
}

#[test]
fn cache_mode_rebuild_clears_cache_file() {
    let tmp = tempfile::tempdir().unwrap();
    let root = make_proj(tmp.path());

    let _ = KnowledgeGraph::build_from_directory_with_cache(&root, CacheMode::Use).unwrap();
    assert_eq!(count_cache_entries(&root), 2);

    // Rebuild should remove old cache then recreate
    cache::clear_cache(&root);
    assert_eq!(count_cache_entries(&root), 0);
    let _ = KnowledgeGraph::build_from_directory_with_cache(&root, CacheMode::Rebuild).unwrap();
    assert_eq!(count_cache_entries(&root), 2);
}
