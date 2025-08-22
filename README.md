## 🛠️ CI and Coverage

- CI workflow: `.github/workflows/ci.yml`
  - Runs format check, clippy (deny warnings), build, tests, docs.
- Coverage workflow: `.github/workflows/coverage.yml`
  - Installs `cargo-tarpaulin` and uploads `lcov.info`.

Run coverage locally:

```sh
cargo install cargo-tarpaulin --locked
cargo tarpaulin --engine llvm --out Lcov --timeout 600 --workspace --exclude-files benches/*,examples/*,target/*
```

# 📚 rust-relations-explorer — Rust Knowledge Graph System

![Rust Edition](https://img.shields.io/badge/edition-2021-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)
![Status](https://img.shields.io/badge/status-Alpha-yellow)
![Coverage](https://img.shields.io/badge/coverage-87.6%25-brightgreen)
![CLI](https://img.shields.io/badge/CLI-clap%204.5-9cf)
[![CI](https://github.com/automataIA/rust-relations-explorer/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/automataIA/rust-relations-explorer/actions/workflows/ci.yml)
![crates.io](https://img.shields.io/crates/v/rust-relations-explorer)
![docs.rs](https://img.shields.io/docsrs/rust-relations-explorer)

A fast, lightweight tool to parse Rust projects into a knowledge graph and run insightful queries over code structure. Generate DOT/SVG visualizations, persist graphs to JSON, and explore code relationships via a clean CLI. ✨


## ⚡ Quick install

```bash
cargo install rust-relations-explorer
```


## 🔗 Links

- crates.io: @web <https://crates.io/crates/rust-relations-explorer>
- docs.rs: @web <https://docs.rs/rust-relations-explorer>
- CI status: @web <https://github.com/automataIA/rust-relations-explorer/actions/workflows/ci.yml>
- Releases: @web <https://github.com/automataIA/rust-relations-explorer/releases>


## ✨ Features

- ✅ Graph builder from source (`KnowledgeGraph::build_from_directory`)
- ✅ Incremental builds with cache (reuse unchanged files; `--no-cache`, `--rebuild`)
- ✅ Relationship analysis (imports, heuristics for calls)
- ✅ JSON persistence (save/load)
- ✅ DOT generation with styling (rankdir, splines, rounded, theme, clusters, legend)
- ✅ SVG enhancement (interactive highlights, clickable nodes)
- ✅ CLI powered by `clap`
- ✅ Queries
  - `connected-files` — files related to a target file
  - `item-info` — show item metadata, code, and relations by ItemId
  - `function-usage` — callers/callees of a function
  - `cycles` — detect file-level cycles
  - `path` — shortest path between two files
  - `hubs` — top-N files by degree centrality (in/out/total)
  - `module-centrality` — top-N modules (directories) by degree centrality
  - `trait-impls` — list types and files implementing a given trait
- 🚧 Pretty table output for terminal
- 🚧 Advanced analyses and config system


## 🧰 Installation

Install from crates.io:

```bash
cargo install rust-relations-explorer
```

Build from source:

```bash
# from repo root
cargo build --release
```

Run checks and tests:

```bash
cargo check
cargo test
```


## 🚀 Usage

Build a graph from a Rust project and export artifacts:

### Defaults & name lookup (quick examples)

```bash
# Use env defaults (see Configuration section)
export RRE_PATH=/path/to/project
export RRE_FORMAT=json

# Name lookup without ItemId
rust-relations-explorer query item-info --path path/to/project --name createIcons --format text

# Short flags and defaults (local dev via cargo)
# Explicit short flags
cargo run -- query item-info -p src -n resolver -f text
# Using environment defaults (omit -p and -f)
export RRE_PATH=src
export RRE_FORMAT=text
cargo run -- query item-info -n resolver
# Increase verbosity (-v, -vv, -vvv)
rust-relations-explorer -vv query cycles

# Quiet mode suppresses non-essential logs
rust-relations-explorer -q query hubs --metric total --top 5 --format text
rust-relations-explorer -q query hubs --metric total --top 5
```

```bash
# Build and save artifacts (uses cache by default)
rust-relations-explorer build --path path/to/project \
  --json graph.json \
  --dot graph.dot \
  --svg graph.svg \
  --dot-rankdir LR --dot-splines curved --dot-rounded on \
  --dot-theme light --dot-clusters on --dot-legend on

# Force parsing all files without using cache
rust-relations-explorer build --path path/to/project --no-cache

# Rebuild cache from scratch (clears previous cache file)
rust-relations-explorer build --path path/to/project --rebuild

# Apply options from a configuration file
rust-relations-explorer build --path path/to/project --config rust-relations-explorer.toml --svg graph.svg

# Bypass ignore rules (include files even if ignored)
rust-relations-explorer build --path path/to/project --no-ignore
```

Run queries (builds the graph on-the-fly unless `--graph` is provided):

```bash
# Connected files for a given file (positional <file>)
rust-relations-explorer query connected-files --path path/to/project src/lib.rs --format text

# Show detailed info for an item by ItemId (text or JSON)
rust-relations-explorer query item-info --path path/to/project --item-id fn:createIcons:6 --format text
rust-relations-explorer query item-info --path path/to/project --item-id fn:createIcons:6 --format json

# Name-only lookup (no ItemId needed)
# Prefer current crate matches; if ambiguous, CLI lists candidates and hints how to disambiguate.
rust-relations-explorer query item-info --path path/to/project --name createIcons --format text

# Narrow by kind to avoid ambiguity (kinds: module|function|struct|enum|trait|impl|const|static|type|macro)
rust-relations-explorer query item-info --path path/to/project --name createIcons --kind function --format text

# Example disambiguation flow (pseudo):
# > Multiple items named 'createIcons' were found. Use --kind or --item-id to disambiguate.
# > Candidates:
# > - fn:createIcons:6  src/a.rs
# > - fn:createIcons:12 src/b.rs
# Then run with --kind function or the exact --item-id shown above.

# Function usage: who calls `foo` (callers) or who does `foo` call (callees)
rust-relations-explorer query function-usage --path path/to/project --function foo --direction callers --format json

# Detect cycles
rust-relations-explorer query cycles --path path/to/project --format text

# Shortest path between files
rust-relations-explorer query path --path path/to/project --from src/a.rs --to src/b.rs --format text

# Hubs: top-N by degree centrality
rust-relations-explorer query hubs --path path/to/project --metric total --top 10 --format text

# Module centrality: top-N modules by degree
rust-relations-explorer query module-centrality --path path/to/project --metric total --top 10 --format text

# Trait implementations for Display
rust-relations-explorer query trait-impls --path path/to/project --trait Display --format json

# Any query can also bypass ignore rules when building on-the-fly
rust-relations-explorer query cycles --path path/to/project --no-ignore --format text
```

### Output controls (quiet/verbose, pagination)

- Quiet mode suppresses non-essential stderr/info logs:

```bash
rust-relations-explorer -q query hubs --path path/to/project --format json
```

- Increase verbosity with repeated `-v` (e.g., `-v`, `-vv`, `-vvv`). Defaults to concise output.

- Paginate large result sets with `--offset` and `--limit` (supported by all queries except `item-info`):

```bash
# First page (10 rows)
rust-relations-explorer query hubs --path path/to/project --metric total --top 100 --offset 0 --limit 10 --format text

# Next page (rows 10..20)
rust-relations-explorer query hubs --path path/to/project --metric total --top 100 --offset 10 --limit 10 --format text

# JSON output is also paginated by the same flags
rust-relations-explorer query function-usage --path path/to/project --function foo --direction callers --offset 0 --limit 20 --format json
```

Use a prebuilt graph for faster queries:

```bash
rust-relations-explorer query hubs --graph graph.json --metric in --top 5 --format json
```

## 📎 Examples

Run the included examples to see the library API in action:

```bash
# Build all examples
cargo build --examples

# Basic graph build and stats
cargo run --example basic_build

# Print connected files to a target (defaults to src/lib.rs if present)
cargo run --example query_connected
```

Example sources:

- `examples/basic_build.rs`
- `examples/query_connected.rs`

## 🧭 Typical Workflows

- __Build and save graph to JSON__

  ```bash
  rust-relations-explorer build --path path/to/project --json graph.json
  rust-relations-explorer query hubs --graph graph.json --metric total --top 10 --format json
  ```

- __Programmatic DOT/SVG generation__

  ```bash
  cargo run --example generate_svg
  # Produces graph.dot and graph.svg
  ```

  - Requires Graphviz `dot` executable on PATH. Install via your OS package manager.
  - Options like clusters, theme, rankdir, splines are in `visualization::DotOptions` and `visualization::SvgOptions`.

- __Bypass ignore rules for one-off analyses__

  ```bash
  rust-relations-explorer build --path path/to/project --no-ignore --svg graph.svg
  ```

- __Save and load graph JSON__

  ```bash
  # Save
  rust-relations-explorer build --path path/to/project --json graph.json

  # Query using saved graph (faster)
  rust-relations-explorer query hubs --graph graph.json --metric in --top 5 --format json
  ```

## 🚢 Releasing

Create a GitHub release using notes from `CHANGELOG.md` via GitHub CLI:

```bash
# Mark as latest and use notes from CHANGELOG.md [0.1.0]
scripts/release_from_changelog.sh 0.1.0 --latest

# Without marking latest
scripts/release_from_changelog.sh 0.1.1
```

Requirements:

- GitHub CLI installed and authenticated:
  - gh auth login -w
- Existing tag `v<version>` pushed (e.g., `v0.1.0`).
- `CHANGELOG.md` includes a section like `## [0.1.0] - YYYY-MM-DD`.

- __Programmatic save/load JSON__

  ```rust
  use rust_relations_explorer::graph::KnowledgeGraph;
  use rust_relations_explorer::utils::cache::CacheMode;

  fn main() -> Result<(), Box<dyn std::error::Error>> {
      let root = std::path::Path::new(".");
      let graph = KnowledgeGraph::build_from_directory_with_cache_opts(root, CacheMode::Use, false)?;
      // Save
      let json = serde_json::to_string_pretty(&graph)?;
      std::fs::write("graph.json", json)?;
      // Load later
      let loaded: KnowledgeGraph = serde_json::from_str(&std::fs::read_to_string("graph.json")?)?;
      println!("Loaded files: {}", loaded.files.len());
      Ok(())
  }
  ```

  Note: `KnowledgeGraph` implements `serde::Serialize`/`Deserialize`.

## 🧪 CLI Quick Examples

Build graph with DOT and SVG outputs:

```sh
cargo run -- build --path path/to/project --json graph.json --dot graph.dot --svg graph.svg
```

Connected files for a target file (JSON):

```sh
cargo run -- query connected-files --path path/to/project src/lib.rs --format json
```

Function usage: callers of function `foo` (JSON):

```sh
cargo run -- query function-usage --path path/to/project --function foo --direction callers --format json
```

Shortest path between two files (JSON):

```sh
cargo run -- query path --path path/to/project --from src/a.rs --to src/b.rs --format json
```

Top hubs by total degree (top 10):

```sh
cargo run -- query hubs --path path/to/project --metric total --top 10 --offset 0 --limit 10 --format text
```

Top modules (directories) by in-degree (top 5):

```sh
cargo run -- query module-centrality --path path/to/project --metric in --top 5 --format text
```

Types implementing a trait (e.g., Display):

```sh
cargo run -- query trait-impls --path path/to/project --trait Display --offset 0 --limit 50 --format json
```

## 🧠 Caching Modes

- **Default (Use)** — reuse unchanged files from `.knowledge_cache.json` and only reparse changed/added files.
- **--no-cache (Ignore)** — do not reuse cache; parse all files, then write a fresh cache at end.
- **--rebuild (Rebuild)** — remove existing cache file first, then parse all files and write a new cache.

Cache file location: `.knowledge_cache.json` at the project root passed to `--path`.

## 🗂️ Ignore Patterns

- File discovery respects `.gitignore` and `.ignore` files. Parent directories are traversed, so nested ignore files apply.
- Global git excludes are intentionally disabled for determinism.
- This affects which `.rs` files are scanned and parsed by `build` and on-the-fly builds for `query`.

Determinism: The CLI now passes ignore behavior explicitly to the file walker. This removes reliance on ambient environment state during normal operation.

Examples:

```gitignore
# src/.ignore (or .gitignore)
ignored.rs
```

```gitignore
# Nested ignore (src/a/.ignore)
skip.rs
```

```gitignore
# Negation pattern (src/.gitignore)
*.rs
!keep.rs
```

Bypass ignores (use with care):

```bash
# Preferred: CLI flag (explicit)
rust-relations-explorer build --path path/to/project --no-ignore

# Optional: environment variable (legacy/compat)
# Still supported, but not required since the CLI now passes the flag directly.
KNOWLEDGE_RS_NO_IGNORE=1 rust-relations-explorer build --path path/to/project
```

## ⚙️ Configuration

You can provide a TOML config with `--config <path>` for both `build` and all `query` subcommands.

- **Precedence (highest to lowest)**:
  - CLI flags
  - Environment variables
  - Config file (`--config`)
  - Binary defaults

Notes:
- Config only backfills when a value is still at its default. It never overwrites values provided by CLI flags or env vars.
- Currently supported config keys: DOT/SVG options for `build`, and default query output format for all queries.

Example `rust-relations-explorer.toml`:

```toml
[dot]
clusters = true
legend = false
theme = "dark"      # "light" | "dark"
rankdir = "TB"      # "LR" | "TB"
splines = "ortho"   # "curved" | "ortho" | "polyline"
rounded = true

[svg]
interactive = true

[query]
default_format = "json" # "text" | "json"
```

### Environment variables

- `RRE_PATH` — default project root for all commands with `--path`
- `RRE_GRAPH` — default graph JSON for query subcommands
- `RRE_FORMAT` — default output format for queries (`text` or `json`)

Examples:

```bash
# Use a default project root and JSON output for queries
export RRE_PATH=/path/to/project
export RRE_FORMAT=json

# Point queries to a prebuilt graph by default
export RRE_GRAPH=/path/to/graph.json

# Now flags can be omitted
rust-relations-explorer query cycles --format json
rust-relations-explorer query hubs --metric total --top 10
```


## 🐚 Shell completions

Generate completion scripts with the built-in subcommand and install them in your shell's completion directory.

- __Bash__ (user-level):

  ```bash
  rust-relations-explorer completions bash > ~/.local/share/bash-completion/completions/rust-relations-explorer
  # If directory doesn't exist, create it and re-source bashrc
  mkdir -p ~/.local/share/bash-completion/completions
  source ~/.bashrc
  ```

  System-wide (requires sudo):

  ```bash
  sudo sh -c 'rust-relations-explorer completions bash > /etc/bash_completion.d/rust-relations-explorer'
  ```

- __Zsh__:

  ```bash
  rust-relations-explorer completions zsh > ~/.zsh/completions/_rust-relations-explorer
  mkdir -p ~/.zsh/completions
  fpath+=(~/.zsh/completions)
  autoload -U compinit && compinit
  ```

- __Fish__:

  ```bash
  rust-relations-explorer completions fish > ~/.config/fish/completions/rust-relations-explorer.fish
  ```

- __PowerShell__ (Windows, current session):

  ```powershell
  rust-relations-explorer completions powershell | Out-String | Invoke-Expression
  ```

  Persist for all sessions (PowerShell profile):

  ```powershell
  $path = "$HOME/.config/powershell/rust-relations-explorer.ps1"
  rust-relations-explorer completions powershell > $path
  Add-Content -Path $PROFILE -Value ". $path"
  ```

- __Elvish__:

  ```bash
  rust-relations-explorer completions elvish > ~/.elvish/lib/rust-relations-explorer.elv
  ```

To test without installing, you can print to stdout:

```bash
rust-relations-explorer completions zsh
```


## 📎 References

- @web <https://doc.rust-lang.org/>
- @web <https://graphviz.org/>
- @web <https://crates.io/crates/clap>
- @web <https://serde.rs/>
- @web <https://docs.rs/regex>


## 🤝 Contributing

Issues and PRs are welcome! Please run `cargo fmt`, `cargo clippy`, and `cargo test` before submitting.


## 📄 License

Licensed under either of

- Apache License, Version 2.0, see `LICENSE-APACHE`
- MIT license, see `LICENSE-MIT`

at your option.
