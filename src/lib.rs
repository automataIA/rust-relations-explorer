//! knowledge-rs — Rust Knowledge Graph System
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
//! use knowledge_rs::graph::KnowledgeGraph;
//! use knowledge_rs::utils::cache::CacheMode;
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
//! knowledge-rs build --path . --json graph.json
//! knowledge-rs query connected-files --path . --file src/lib.rs --format text
//! ```
//!
//! # Ignore Behavior
//! Pass `--no-ignore` in CLI to include ignored files. Env `KNOWLEDGE_RS_NO_IGNORE` remains supported for compatibility.
pub mod graph;
pub mod parser;
pub mod visualization;
pub mod cli;
pub mod utils;
pub mod errors;
pub mod query;
pub mod app;
