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

- crates.io: @web https://crates.io/crates/rust-relations-explorer
- docs.rs: @web https://docs.rs/rust-relations-explorer
- CI status: @web https://github.com/automataIA/rust-relations-explorer/actions/workflows/ci.yml
- Releases: @web https://github.com/automataIA/rust-relations-explorer/releases


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
# Connected files for a given file
rust-relations-explorer query connected-files --path path/to/project --file src/lib.rs --format text

# Show detailed info for an item by ItemId (text or JSON)
rust-relations-explorer query item-info --path path/to/project --item-id fn:createIcons:6 --format text
rust-relations-explorer query item-info --path path/to/project --item-id fn:createIcons:6 --format json

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

## 🧩 Mapping: CLI queries ↔ API

- `query connected-files` → `query::ConnectedFilesQuery`
- `query item-info --item-id ID` → `query::ItemInfoQuery`
- `query function-usage --direction callers|callees` → `query::FunctionUsageQuery::{callers,callees}`
- `query cycles` → `query::CycleDetectionQuery`
- `query path --from A --to B` → `query::ShortestPathQuery::new(A,B)`
- `query hubs --metric in|out|total` → `query::HubsQuery` with `CentralityMetric`
- `query module-centrality` → `query::ModuleCentralityQuery`
- `query trait-impls --trait NAME` → `query::TraitImplsQuery::new(NAME)`

## 🛠️ Troubleshooting

- __Graphviz `dot` not found__
  - Ensure Graphviz is installed and `dot` is on PATH. The `generate_svg` example will print a hint on failure.
- __Empty graph__
  - Check ignore rules. Try `--no-ignore` to include ignored files. See `Ignore Patterns` section.
- __Slow runs on large repos__
  - Use saved JSON and query with `--graph graph.json`. Ensure cache (default) is enabled.

## 🧱 Design Overview

- __Graph model__
  - Nodes: items parsed from Rust files (functions, structs, enums, traits, impls, etc.). See `src/graph/mod.rs`.
  - Edges: relationships between items (uses, implements, contains, extends, calls). Projectable to file-level.
- __File discovery & ignores__
  - File walking in `src/utils/mod.rs` honors `.gitignore`, global ignores, nested ignores. Explicit override via `--no-ignore`.
- __Visualization__
  - DOT via `visualization::DotGenerator`; SVG via `visualization::SvgGenerator` using Graphviz `dot`.

## ❓ FAQ

- __How do item-level relationships map to files?__
  - Each relationship stores `from_item`/`to_item` (item IDs). For file-level analyses (e.g., connected files), items are mapped to their defining file and deduplicated.
- __Does the CLI modify environment variables for ignores?__
  - No. Ignoring is controlled explicitly via flags. Legacy env is still read by the library for backwards compatibility but not set by the CLI.


## 🗺️ Roadmap

See `plan-tasks.md` for full project plan, phases, and task status.

- Core queries: ✅
- Centrality/hubs: ✅
- Table output: 🚧
- Config file and ignores: 🚧
- Performance (parallelism, caching, incremental): 🚧

## 🏁 Benchmarks

- __Run all benches__

  ```bash
  cargo bench
  ```

- __Reports__
  - Criterion stores results under `target/criterion/`.
  - With `html_reports` enabled, open `target/criterion/report/index.html` for an overview.

- __What’s measured__
  - Build performance by cache mode: `benches/build_graph.rs`.
  - Query performance (connected files, hubs, shortest path): `benches/queries.rs`.

- __Notes__
  - Benches use `criterion` 0.5. Setup occurs outside `iter` and results are `black_box`ed.

### 📊 Sample Results (local, 2025-08-18)

- __Build graph__ (lower is better)
  - rebuild: [6.86 ms, 6.98 ms, 7.11 ms]
  - use_cache: [4.42 ms, 4.46 ms, 4.52 ms]
  - ignore_cache: [7.06 ms, 7.20 ms, 7.35 ms]

- __Queries__ (lower is better)
  - connected_files: [55.65 µs, 57.48 µs, 59.75 µs]
  - hubs (top10 total): [43.09 µs, 44.77 µs, 47.06 µs]
  - shortest_path (a→b): [42.73 µs, 43.17 µs, 43.64 µs]

#### 📦 10k synthetic project (release)

- Dataset: generated via `examples/generate_synthetic.rs` (10,000 files)
- Default (use cache), warm run: 0.29s elapsed, ~103 MB max RSS
- No-cache: 0.85s elapsed, ~100 MB max RSS
- Notes:
  - First invocation via `cargo run` includes compilation overhead; use the built binary for timing.
  - Results are for a simple synthetic workload (flat modules, tiny files). Real projects will vary.

Notes:
- These are Criterion min/mean/max for 100 samples on the current machine and can vary across runs and environments.
- Please record your machine specs (CPU model, cores, RAM, OS) alongside results for context.

#### 🖥️ Machine specs (example, replace with yours)

- CPU: Intel(R) Core(TM) i7-8700 CPU @ 3.20GHz (6 cores / 12 threads)
- RAM: 32 GB
- OS: Ubuntu 22.04 x86_64
- Rust: nightly toolchain (see `rust-toolchain.toml` or `rustup show`)

### 🔬 Validate 10k-file Performance (Phase 4)

Use the synthetic generator example to create a large project and measure performance.

1) Generate N files (e.g., 10,000):

```sh
cargo run --example generate_synthetic -- /tmp/kr_synth 10000
```

2) Measure build time (Use cache by default):

```sh
# Build release binary once
cargo build --release

# Use the built binary to avoid compile overhead
/usr/bin/time -v target/release/rust-relations-explorer build --path /tmp/kr_synth --json /tmp/kr_graph.json
```

3) Compare cache modes:

```sh
/usr/bin/time -v target/release/rust-relations-explorer build --path /tmp/kr_synth --json /tmp/kr_graph.json --rebuild
/usr/bin/time -v target/release/rust-relations-explorer build --path /tmp/kr_synth --json /tmp/kr_graph.json --no-cache
```

4) Optional memory profiling:

```sh
/usr/bin/time -v target/release/rust-relations-explorer build --path /tmp/kr_synth --json /tmp/kr_graph.json
# or use heaptrack if available
```

Record the elapsed wall time and peak memory, and confirm the 10k-in-30s target.

## 🧪 CLI Quick Examples

Build graph with DOT and SVG outputs:

```sh
cargo run -- build --path path/to/project --json graph.json --dot graph.dot --svg graph.svg
```

Connected files for a target file (JSON):

```sh
cargo run -- query connected-files --path path/to/project --file src/lib.rs --format json
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
cargo run -- query hubs --path path/to/project --metric total --top 10 --format text
```

Top modules (directories) by in-degree (top 5):

```sh
cargo run -- query module-centrality --path path/to/project --metric in --top 5 --format text
```

Types implementing a trait (e.g., Display):

```sh
cargo run -- query trait-impls --path path/to/project --trait Display --format json
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

- **Precedence**:
  - CLI flags override binary defaults.
  - Config values are applied on top of CLI defaults to provide convenient project-wide settings (as implemented, config values for DOT/SVG and query format will be applied even if CLI uses defaulted values).

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


## 📎 References

- @web https://doc.rust-lang.org/
- @web https://graphviz.org/
- @web https://crates.io/crates/clap
- @web https://serde.rs/
- @web https://docs.rs/regex


## 🤝 Contributing

Issues and PRs are welcome! Please run `cargo fmt`, `cargo clippy`, and `cargo test` before submitting.


## 📄 License

Licensed under either of

- Apache License, Version 2.0, see `LICENSE-APACHE`
- MIT license, see `LICENSE-MIT`

at your option.
