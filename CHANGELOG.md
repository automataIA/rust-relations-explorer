# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog (@web https://keepachangelog.com/en/1.1.0/),
and this project adheres to Semantic Versioning (@web https://semver.org/spec/v2.0.0.html).

## [Unreleased]

_No changes yet._

## [0.1.3] - 2025-08-22

### Added
- CLI pagination flags `--offset` and `--limit` applied uniformly across multi-result queries.
- ItemInfo: name-only lookup via `--name/-n <NAME>` with optional `--kind/-k <Kind>`.
  - Prefer current crate matches and provide disambiguation hints when ambiguous.
- Global output controls: `-q/--quiet` and `-v/--verbose`.
  - Implicit short output at `v=0` for tables (fewer columns) and for `item-info` text (relation counts only).
  - JSON trimming for `item-info` at `v=0` (omit code and relation contexts).
- Project root auto-detection: default `--path` to nearest ancestor containing `Cargo.toml` and `src/`.
- CLI ergonomics:
  - Standardized short flags (e.g., `-c/--config`, `-I/--no-ignore`, `-f/--format`).
  - Visible aliases for `--no-ignore` (e.g., `no-gitignore`, `all`, `ni`).
  - ValueEnum adoption for flags: `OutputFormat` (`text|json`), `Direction` (`callers|callees`), `CentralityMetric` (`in|out|total`).
  - DOT/SVG options as ValueEnums: theme, rankdir (with `LR`/`TB` aliases), splines, rounded, clusters, legend, svg_interactive.
- Connected-files: made target `file` a positional operand (`<file>`) with `--file` kept as alias.
- Env/config layering:
  - Environment variables `RRE_PATH`, `RRE_GRAPH`, `RRE_FORMAT` supported across commands.
  - Optional TOML config (`--config`) to backfill defaults with precedence: CLI > env > config > defaults.
- Shell completions: `completions <shell>` subcommand (bash, zsh, fish, powershell, elvish).

### Changed
- Printing utilities/tables now respect verbosity levels.
- ItemInfo: when using name-only lookup, candidates are ranked to prefer items under the current crate's `src/` and shallower module depth; ties list top matches with guidance.
- ItemInfo (text): renamed labels from `Inbound/Outbound` to `Callers/Callees`; at default verbosity, show concise comma-separated IDs for callers/callees; verbose retains detailed listings.

### Tests
- Integration tests updated for new CLI flags and a new test to ensure quiet mode suppresses non-essential output.
- New CLI tests for ItemInfo name lookup: success (by `--name`), with `--kind`, ambiguous, and not found scenarios.
- New CLI tests for: project root detection, positional `<file>` for connected-files, ValueEnums for format/direction/metric and DOT options, and verbosity-level outputs.
- README updated with short-flags cargo-run examples and env/defaults usage; docs reflect config/env precedence and completions.

## [0.1.2] - 2025-08-19

### Changed
- Crate docs: retain `README.md` inclusion and `doc_cfg`.

### Fixed
- CI fmt failure: ensure no trailing blank lines in `src/lib.rs`.

### Tooling
- `scripts/release_from_changelog.sh`: robust changelog parsing (supports `## [X.Y.Z] - YYYY-MM-DD`), no awk warnings.

## [0.1.1] - 2025-08-19

### Fixed
- Rustfmt failure in `src/lib.rs` due to trailing blank line (CI fmt check now passes).

### Documentation
- Docs.rs configuration: build with all features and enable `docsrs` cfg via `[package.metadata.docs.rs]` in `Cargo.toml`.
- Crate-level docs now include `README.md` via `#![doc = include_str!("../README.md")]` and enable `doc_cfg`.
- Module overviews added to `src/graph/mod.rs` and `src/query/mod.rs` for better docs navigation.
- README: wrap `@web` bare URLs in angle brackets to silence rustdoc warnings.

### Tooling
- Verified `cargo doc --all-features --no-deps` and `cargo test --doc` succeed locally.

## [0.1.0] - 2025-08-19

### Added
- Initial release of `rust-relations-explorer`.
- Graph builder from source (`KnowledgeGraph::build_from_directory`).
- Incremental build with cache and CLI flags (`--no-cache`, `--rebuild`).
- Relationship analysis (imports, heuristics for calls).
- JSON persistence (save/load).
- DOT generation with styling (rankdir, splines, rounded, theme, clusters, legend).
- SVG enhancements (interactive highlights, clickable nodes).
- Query commands: `path`, `hubs`, `module-centrality`, `trait-impls`.
- CLI powered by `clap`.

### Tooling
- CI workflow with fmt/clippy/test and coverage (tarpaulin).
- Examples and benches.

[Unreleased]: @web <https://github.com/automataIA/rust-relations-explorer/compare/v0.1.3...HEAD>
[0.1.3]: @web <https://github.com/automataIA/rust-relations-explorer/releases/tag/v0.1.3>
[0.1.2]: @web <https://github.com/automataIA/rust-relations-explorer/releases/tag/v0.1.2>
[0.1.1]: @web <https://github.com/automataIA/rust-relations-explorer/releases/tag/v0.1.1>
[0.1.0]: @web <https://github.com/automataIA/rust-relations-explorer/releases/tag/v0.1.0>
