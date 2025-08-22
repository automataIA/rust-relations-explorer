// Utilities module placeholder per design
pub mod table {
    // Helper to render a separator line
    fn sep(widths: &[usize]) -> String {
        let mut s = String::from("+");
        for w in widths {
            s.push_str(&"-".repeat(w + 2));
            s.push('+');
        }
        s
    }

    // Helper to render a row line
    fn line(cells: &[String], widths: &[usize]) -> String {
        let mut s = String::from("|");
        for (i, cell) in cells.iter().enumerate() {
            let w = widths[i];
            s.push(' ');
            s.push_str(cell);
            if cell.len() < w {
                s.push_str(&" ".repeat(w - cell.len()));
            }
            s.push(' ');
            s.push('|');
        }
        s
    }

    // Render a simple ASCII table given headers and rows
    pub fn render(headers: &[&str], rows: &[Vec<String>]) -> String {
        let cols = headers.len();
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for row in rows {
            for (c, w) in widths.iter_mut().enumerate().take(cols) {
                *w = (*w).max(row.get(c).map_or(0, String::len));
            }
        }

        let mut out = String::new();
        out.push_str(&sep(&widths));
        out.push('\n');
        let header_cells: Vec<String> = headers.iter().map(|s| (*s).to_string()).collect();
        out.push_str(&line(&header_cells, &widths));
        out.push('\n');
        out.push_str(&sep(&widths));
        out.push('\n');
        for row in rows {
            let mut cells = Vec::with_capacity(cols);
            for i in 0..cols {
                cells.push(row.get(i).cloned().unwrap_or_default());
            }
            out.push_str(&line(&cells, &widths));
            out.push('\n');
        }
        out.push_str(&sep(&widths));
        out
    }
}

pub mod config {
    use serde::Deserialize;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Clone, Deserialize, Default)]
    pub struct DotConfig {
        pub clusters: Option<bool>,
        pub legend: Option<bool>,
        pub theme: Option<String>,   // "light" | "dark"
        pub rankdir: Option<String>, // "LR" | "TB"
        pub splines: Option<String>, // "curved" | "ortho" | "polyline"
        pub rounded: Option<bool>,
    }

    #[derive(Debug, Clone, Deserialize, Default)]
    pub struct SvgConfig {
        pub interactive: Option<bool>,
    }

    #[derive(Debug, Clone, Deserialize, Default)]
    pub struct QueryConfig {
        pub default_format: Option<String>, // "text" | "json"
    }

    #[derive(Debug, Clone, Deserialize, Default)]
    pub struct Config {
        pub root: Option<String>,
        pub dot: Option<DotConfig>,
        pub svg: Option<SvgConfig>,
        pub query: Option<QueryConfig>,
    }

    fn default_config_path(root: &Path) -> PathBuf {
        // Prefer new name, keep backward compatibility with old filename
        root.join("rust-relations-explorer.toml")
    }

    #[must_use]
    pub fn load_config_at(path: &Path) -> Option<Config> {
        let data = fs::read_to_string(path).ok()?;
        toml::from_str::<Config>(&data).ok()
    }

    #[must_use]
    pub fn load_config_near(root: &Path) -> Option<Config> {
        let p_new = default_config_path(root);
        if p_new.exists() {
            return load_config_at(&p_new);
        }
        let p_old = root.join("knowledge-rs.toml");
        if p_old.exists() {
            load_config_at(&p_old)
        } else {
            None
        }
    }
}

pub mod cache {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use crate::graph::FileNode;

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
    pub struct CacheEntryMeta {
        pub mtime: u64,
        pub len: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct CacheEntry {
        pub meta: CacheEntryMeta,
        pub node: FileNode,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct Cache {
        pub entries: HashMap<PathBuf, CacheEntry>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CacheMode {
        Use,
        Ignore,
        Rebuild,
    }

    fn cache_path(root: &Path) -> PathBuf {
        root.join(".knowledge_cache.json")
    }

    #[must_use]
    pub fn load_cache(root: &Path) -> Option<Cache> {
        let path = cache_path(root);
        let data = std::fs::read_to_string(path).ok()?;
        serde_json::from_str::<Cache>(&data).ok()
    }

    pub fn save_cache(root: &Path, cache: &Cache) {
        let path = cache_path(root);
        if let Ok(data) = serde_json::to_string_pretty(cache) {
            let _ = std::fs::write(path, data);
        }
    }

    pub fn clear_cache(root: &Path) {
        let path = cache_path(root);
        let _ = std::fs::remove_file(path);
    }
}

pub mod file_walker {
    /// Discover Rust source files under `root`, with an option to bypass ignore rules.
    #[must_use]
    pub fn rust_files_with_options(root: &str, no_ignore: bool) -> Vec<String> {
        let mut out = Vec::new();
        let mut walker = ignore::WalkBuilder::new(root);
        // Explicitly enable .gitignore/.ignore support and parent traversal (unless bypassed)
        walker
            .follow_links(false)
            .git_ignore(!no_ignore)
            .git_global(false)
            .git_exclude(false)
            .ignore(!no_ignore)
            .parents(true);
        // Build a Gitignore matcher from root-level ignore files for explicit checks (unless bypassed)
        let root_path = std::path::Path::new(root);
        let matcher = if no_ignore {
            None
        } else {
            let mut gi_builder = ignore::gitignore::GitignoreBuilder::new(root_path);
            let gi = root_path.join(".gitignore");
            if gi.exists() {
                let _ = gi_builder.add(gi);
            }
            let ign = root_path.join(".ignore");
            if ign.exists() {
                let _ = gi_builder.add(ign);
            }
            gi_builder.build().ok()
        };
        for entry in walker.build().flatten() {
            if entry.file_type().is_some_and(|t| t.is_file()) {
                // Explicit filter using matcher (in addition to WalkBuilder's own filtering)
                if let Some(m) = &matcher {
                    if m.matched(entry.path(), false).is_ignore() {
                        continue;
                    }
                }
                if entry.path().extension() == Some(std::ffi::OsStr::new("rs")) {
                    if let Some(s) = entry.path().to_str() {
                        out.push(s.to_string());
                    }
                }
            }
        }
        out
    }

    /// Backward-compatible helper that reads env var and delegates to `rust_files_with_options`.
    #[must_use]
    pub fn rust_files(root: &str) -> Vec<String> {
        let no_ignore = std::env::var("KNOWLEDGE_RS_NO_IGNORE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        rust_files_with_options(root, no_ignore)
    }
}

pub mod project_root {
    use std::env;
    use std::path::{Path, PathBuf};

    /// Detect the Cargo project root by walking ancestors looking for both `Cargo.toml` and `src/`.
    #[must_use]
    pub fn detect(start: Option<&Path>) -> PathBuf {
        let mut cur = start
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        loop {
            let cargo = cur.join("Cargo.toml");
            let src = cur.join("src");
            if cargo.exists() && src.is_dir() {
                return cur;
            }
            if let Some(parent) = cur.parent() {
                cur = parent.to_path_buf();
            } else {
                // Fallback to current_dir when nothing found
                return env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            }
        }
    }

    /// If `p` is "." (implicit default), replace with detected project root. Otherwise, return as-is.
    #[must_use]
    pub fn effective_path_str(p: &str) -> String {
        if p == "." {
            detect(None).to_string_lossy().to_string()
        } else {
            p.to_string()
        }
    }

    /// Convert an optional Path into an effective absolute project root path.
    /// None or "." resolve to the detected project root; any other path is returned as owned PathBuf.
    #[must_use]
    pub fn effective_path_opt(p: Option<&Path>) -> PathBuf {
        match p {
            None => detect(None),
            Some(path) if path == Path::new(".") => detect(None),
            Some(path) => path.to_path_buf(),
        }
    }
}
