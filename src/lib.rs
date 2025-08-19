//! rust-relations-explorer — Rust Knowledge Graph System
//!
//! Build a knowledge graph from a Rust codebase and query relationships.
//!
//! # Features
//! - File discovery with `.gitignore`/`.ignore` support (deterministic; global excludes off)
//! - Incremental builds with on-disk cache
//! - Queries: connected files, function usage, cycles, path, hubs, module centrality, trait impls
//! - DOT and SVG visualization
//!
//! # Quickstart (Library)
//! ```no_run
//! use rust_relations_explorer::graph::KnowledgeGraph;
//! use rust_relations_explorer::utils::cache::CacheMode;
//!
//! let root = std::path::Path::new(".");
//! // Build with cache and respect ignore rules
//! let graph = KnowledgeGraph::build_from_directory_with_cache_opts(root, CacheMode::Use, /* no_ignore = */ false)
//!     .expect("build graph");
//! println!("files: {} relationships: {}", graph.files.len(), graph.relationships.len());
//! ```
//!
//! # Quickstart (CLI)
//! ```text
//! rust-relations-explorer build --path . --json graph.json
//! rust-relations-explorer query connected-files --path . --file src/lib.rs --format text
//! ```
//!
//! # Ignore Behavior
//! Pass `--no-ignore` in CLI to include ignored files. Env `KNOWLEDGE_RS_NO_IGNORE` remains supported for compatibility.
pub mod app;
pub mod cli;
pub mod errors;
pub mod graph;
pub mod parser;
pub mod query;
pub mod utils;
pub mod visualization;
