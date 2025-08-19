# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog (@web https://keepachangelog.com/en/1.1.0/),
and this project adheres to Semantic Versioning (@web https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Planned improvements and enhancements.

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

[Unreleased]: @web https://github.com/automataIA/rust-relations-explorer/compare/v0.1.0...HEAD
[0.1.0]: @web https://github.com/automataIA/rust-relations-explorer/releases/tag/v0.1.0
