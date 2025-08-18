use crate::cli::{Cli, Commands, QueryCommands};
use crate::graph::KnowledgeGraph;
use crate::query::Query;
use crate::visualization::{DotGenerator, DotOptions, DotTheme, EdgeStyle, RankDir, SvgGenerator, SvgOptions};
use std::fs;

/// Run the CLI logic in-process.
///
/// Returns an exit code (0 = success).
///
/// # Panics
/// May panic during JSON serialization if graph serialization fails when producing
/// `--json` output in the build command.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn run_cli(cli: Cli) -> i32 {
    match cli.command {
        Commands::Build { path, config, no_ignore, no_cache, rebuild, json, dot, svg, dot_clusters, dot_legend, dot_theme, dot_rankdir, dot_splines, dot_rounded, svg_interactive, save } => {
            // Determine cache mode
            let mode = if rebuild {
                crate::utils::cache::CacheMode::Rebuild
            } else if no_cache {
                crate::utils::cache::CacheMode::Ignore
            } else {
                crate::utils::cache::CacheMode::Use
            };

            let build_path = std::path::Path::new(&path);
            if matches!(mode, crate::utils::cache::CacheMode::Rebuild) {
                crate::utils::cache::clear_cache(build_path);
            }

            let graph = match KnowledgeGraph::build_from_directory_with_cache_opts(build_path, mode, no_ignore) {
                Ok(g) => g,
                Err(e) => { eprintln!("Build failed: {e}"); return 1; }
            };

            // Optionally write JSON output
            if let Some(json_path) = json {
                let serialized = serde_json::to_string_pretty(&graph).expect("serialize graph to JSON");
                if let Err(e) = fs::write(&json_path, serialized) { eprintln!("Failed to write JSON output {json_path}: {e}"); }
            }

            // DOT options from flags and optional config overrides
            let mut clusters = matches!(dot_clusters.as_str(), "on");
            let mut legend = matches!(dot_legend.as_str(), "on");
            let mut theme = match dot_theme.as_str() { "dark" => DotTheme::Dark, _ => DotTheme::Light };
            let mut rankdir = match dot_rankdir.as_str() { "TB" => RankDir::TB, _ => RankDir::LR };
            let mut splines = match dot_splines.as_str() { "ortho" => EdgeStyle::Ortho, "polyline" => EdgeStyle::Polyline, _ => EdgeStyle::Curved };
            let mut rounded = matches!(dot_rounded.as_str(), "on");
            if let Some(cfg_path) = config.as_ref() {
                if let Some(cfg) = crate::utils::config::load_config_at(std::path::Path::new(cfg_path)) {
                    if let Some(dot) = cfg.dot {
                        if let Some(v) = dot.clusters { clusters = v; }
                        if let Some(v) = dot.legend { legend = v; }
                        if let Some(v) = dot.theme { theme = if v == "dark" { DotTheme::Dark } else { DotTheme::Light }; }
                        if let Some(v) = dot.rankdir { rankdir = if v == "TB" { RankDir::TB } else { RankDir::LR }; }
                        if let Some(v) = dot.splines { splines = match v.as_str() { "ortho" => EdgeStyle::Ortho, "polyline" => EdgeStyle::Polyline, _ => EdgeStyle::Curved }; }
                        if let Some(v) = dot.rounded { rounded = v; }
                    }
                }
            }
            let dot_opts = DotOptions { clusters, legend, theme, rankdir, splines, rounded };

            if let Some(dot_path) = dot {
                match DotGenerator::new().generate_dot_with_options(&graph, dot_opts) {
                    Ok(content) => { if let Err(e) = fs::write(&dot_path, content) { eprintln!("Failed to write DOT output {dot_path}: {e}"); } }
                    Err(e) => eprintln!("Visualization error: {e}"),
                }
            }

            if let Some(svg_path) = svg {
                let mut interactive = matches!(svg_interactive.as_str(), "on");
                if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = crate::utils::config::load_config_at(std::path::Path::new(cfg_path)) {
                        if let Some(svg) = cfg.svg { if let Some(v) = svg.interactive { interactive = v; } }
                    }
                }
                let svg_opts = SvgOptions { dot: dot_opts, interactive };
                match SvgGenerator::new().generate_svg_with_options(&graph, svg_opts) {
                    Ok(content) => { if let Err(e) = fs::write(&svg_path, content) { eprintln!("Failed to write SVG output {svg_path}: {e}"); } }
                    Err(e) => eprintln!("Visualization error: {e}"),
                }
            }

            if let Some(save_path) = save {
                if let Err(e) = KnowledgeGraph::save_json(&graph, std::path::Path::new(&save_path)) {
                    eprintln!("Failed to save graph JSON {save_path}: {e}");
                }
            }

            println!("Build completed for path: {path}");
            0
        }
        Commands::Query { query } => match query {
            QueryCommands::ConnectedFiles { path, config, no_ignore, file, graph: graph_path, format } => {
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) { Ok(g) => g, Err(e) => { eprintln!("Load graph failed: {e}"); return 1; } }
                } else {
                    let res = match KnowledgeGraph::build_from_directory_opts(std::path::Path::new(&path), no_ignore) { Ok(g) => g, Err(e) => { eprintln!("Build failed: {e}"); return 1; } };
                    res
                };
                let q = crate::query::ConnectedFilesQuery::new(&file);
                let results = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = crate::utils::config::load_config_at(std::path::Path::new(cfg_path)) { cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone()) } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    let out: Vec<String> = results.into_iter().map(|p| p.display().to_string()).collect();
                    match serde_json::to_string_pretty(&out) { Ok(s) => println!("{s}"), Err(e) => { eprintln!("JSON encode error: {e}"); return 1; } }
                } else {
                    let rows: Vec<Vec<String>> = results.into_iter().enumerate().map(|(i,p)| vec![format!("{}", i+1), p.display().to_string()]).collect();
                    let table = crate::utils::table::render(&["#", "Path"], &rows);
                    println!("{table}");
                }
                0
            }
            QueryCommands::FunctionUsage { path, config, no_ignore, function, direction, graph: graph_path, format } => {
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) { Ok(g) => g, Err(e) => { eprintln!("Load graph failed: {e}"); return 1; } }
                } else {
                    if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                    let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) { Ok(g) => g, Err(e) => { eprintln!("Build failed: {e}"); if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); } return 1; } };
                    if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                    res
                };
                let dir = match direction.as_str() { "callees" => crate::query::UsageDirection::Callees, _ => crate::query::UsageDirection::Callers };
                let q = crate::query::FunctionUsageQuery { function, direction: dir };
                let results = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = crate::utils::config::load_config_at(std::path::Path::new(cfg_path)) { cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone()) } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    let out: Vec<String> = results.into_iter().map(|p| p.display().to_string()).collect();
                    match serde_json::to_string_pretty(&out) { Ok(s) => println!("{s}"), Err(e) => { eprintln!("JSON encode error: {e}"); return 1; } }
                } else {
                    for p in results { println!("{}", p.display()); }
                }
                0
            }
            QueryCommands::Cycles { path, config, no_ignore, graph: graph_path, format } => {
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) { Ok(g) => g, Err(e) => { eprintln!("Load graph failed: {e}"); return 1; } }
                } else {
                    if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                    let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) { Ok(g) => g, Err(e) => { eprintln!("Build failed: {e}"); if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); } return 1; } };
                    if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                    res
                };
                let q = crate::query::CycleDetectionQuery::new();
                let cycles = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = crate::utils::config::load_config_at(std::path::Path::new(cfg_path)) { cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone()) } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    let out: Vec<Vec<String>> = cycles.into_iter().map(|cyc| cyc.into_iter().map(|p| p.display().to_string()).collect()).collect();
                    match serde_json::to_string_pretty(&out) { Ok(s) => println!("{s}"), Err(e) => { eprintln!("JSON encode error: {e}"); return 1; } }
                } else {
                    for cyc in cycles {
                        let parts: Vec<String> = cyc.iter().map(|p| p.display().to_string()).collect();
                        println!("{}", parts.join(" -> "));
                    }
                }
                0
            }
            QueryCommands::Path { path, config, no_ignore, from, to, graph: graph_path, format } => {
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) { Ok(g) => g, Err(e) => { eprintln!("Load graph failed: {e}"); return 1; } }
                } else {
                    if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                    let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) { Ok(g) => g, Err(e) => { eprintln!("Build failed: {e}"); if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); } return 1; } };
                    if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                    res
                };
                let q = crate::query::ShortestPathQuery::new(&from, &to);
                let results = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = crate::utils::config::load_config_at(std::path::Path::new(cfg_path)) { cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone()) } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    let out: Vec<String> = results.into_iter().map(|p| p.display().to_string()).collect();
                    match serde_json::to_string_pretty(&out) { Ok(s) => println!("{s}"), Err(e) => { eprintln!("JSON encode error: {e}"); return 1; } }
                } else if results.is_empty() { println!("<no path>"); } else {
                    let rows: Vec<Vec<String>> = results.into_iter().enumerate().map(|(i,p)| vec![format!("{}", i+1), p.display().to_string()]).collect();
                    let table = crate::utils::table::render(&["Step", "Path"], &rows);
                    println!("{table}");
                }
                0
            }
            QueryCommands::Hubs { path, config, no_ignore, graph: graph_path, metric, top, format } => {
                use crate::query::{CentralityMetric, HubsQuery};
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) { Ok(g) => g, Err(e) => { eprintln!("Load graph failed: {e}"); return 1; } }
                } else {
                    if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                    let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) { Ok(g) => g, Err(e) => { eprintln!("Build failed: {e}"); if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); } return 1; } };
                    if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                    res
                };
                let m = match metric.as_str() { "in" => CentralityMetric::In, "out" => CentralityMetric::Out, _ => CentralityMetric::Total };
                let q = HubsQuery::new(m, top);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = crate::utils::config::load_config_at(std::path::Path::new(cfg_path)) { cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone()) } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    #[derive(serde::Serialize)]
                    struct HubRow { path: String, indegree: usize, outdegree: usize }
                    let out: Vec<HubRow> = rows.into_iter().map(|(p,i,o)| HubRow { path: p.display().to_string(), indegree: i, outdegree: o }).collect();
                    match serde_json::to_string_pretty(&out) { Ok(s) => println!("{s}"), Err(e) => { eprintln!("JSON encode error: {e}"); return 1; } }
                } else {
                    let body: Vec<Vec<String>> = rows.into_iter().map(|(p,i,o)| vec![p.display().to_string(), i.to_string(), o.to_string(), (i+o).to_string()]).collect();
                    let table = crate::utils::table::render(&["Path", "In", "Out", "Total"], &body);
                    println!("{table}");
                }
                0
            }
            QueryCommands::ModuleCentrality { path, config, no_ignore, graph: graph_path, metric, top, format } => {
                use crate::query::{CentralityMetric, ModuleCentralityQuery};
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) { Ok(g) => g, Err(e) => { eprintln!("Load graph failed: {e}"); return 1; } }
                } else {
                    if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                    let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) { Ok(g) => g, Err(e) => { eprintln!("Build failed: {e}"); if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); } return 1; } };
                    if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                    res
                };
                let m = match metric.as_str() { "in" => CentralityMetric::In, "out" => CentralityMetric::Out, _ => CentralityMetric::Total };
                let q = ModuleCentralityQuery::new(m, top);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = crate::utils::config::load_config_at(std::path::Path::new(cfg_path)) { cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone()) } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    #[derive(serde::Serialize)]
                    struct Row { module: String, indegree: usize, outdegree: usize }
                    let out: Vec<Row> = rows.into_iter().map(|(p,i,o)| Row { module: p.display().to_string(), indegree: i, outdegree: o }).collect();
                    match serde_json::to_string_pretty(&out) { Ok(s) => println!("{s}"), Err(e) => { eprintln!("JSON encode error: {e}"); return 1; } }
                } else {
                    let body: Vec<Vec<String>> = rows.into_iter().map(|(p,i,o)| vec![p.display().to_string(), i.to_string(), o.to_string(), (i+o).to_string()]).collect();
                    let table = crate::utils::table::render(&["Module", "In", "Out", "Total"], &body);
                    println!("{table}");
                }
                0
            }
            QueryCommands::TraitImpls { path, config, no_ignore, r#trait, graph: graph_path, format } => {
                use crate::query::TraitImplsQuery;
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) { Ok(g) => g, Err(e) => { eprintln!("Load graph failed: {e}"); return 1; } }
                } else {
                    if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                    let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) { Ok(g) => g, Err(e) => { eprintln!("Build failed: {e}"); if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); } return 1; } };
                    if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                    res
                };
                let q = TraitImplsQuery::new(&r#trait);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = crate::utils::config::load_config_at(std::path::Path::new(cfg_path)) { cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone()) } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    #[derive(serde::Serialize)]
                    struct Row { path: String, r#type: String }
                    let out: Vec<Row> = rows.into_iter().map(|(p,t)| Row { path: p.display().to_string(), r#type: t }).collect();
                    match serde_json::to_string_pretty(&out) { Ok(s) => println!("{s}"), Err(e) => { eprintln!("JSON encode error: {e}"); return 1; } }
                } else if rows.is_empty() { println!("<no implementations found>"); } else {
                    let body: Vec<Vec<String>> = rows.into_iter().map(|(p,t)| vec![p.display().to_string(), t]).collect();
                    let table = crate::utils::table::render(&["Path", "Type"], &body);
                    println!("{table}");
                }
                0
            }
        },
    }
}
