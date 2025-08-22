use crate::cli::{Cli, Commands, ItemKindArg, OutputFormat, QueryCommands};
use crate::graph::KnowledgeGraph;
use crate::query::Query;
use crate::visualization::{
    DotGenerator, DotOptions, DotTheme, EdgeStyle, RankDir, SvgGenerator, SvgOptions,
};
use clap::CommandFactory;
use clap_complete::generate;
use std::fs;
use std::io;

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
        Commands::Completions { shell } => {
            let mut cmd = crate::cli::Cli::command();
            let bin_name = env!("CARGO_PKG_NAME");
            let mut out = io::stdout();
            generate(shell, &mut cmd, bin_name, &mut out);
            0
        }
        Commands::Build {
            path,
            config,
            no_ignore,
            no_cache,
            rebuild,
            json,
            dot,
            svg,
            dot_clusters,
            dot_legend,
            dot_theme,
            dot_rankdir,
            dot_splines,
            dot_rounded,
            svg_interactive,
            save,
        } => {
            // Determine cache mode
            let mode = if rebuild {
                crate::utils::cache::CacheMode::Rebuild
            } else if no_cache {
                crate::utils::cache::CacheMode::Ignore
            } else {
                crate::utils::cache::CacheMode::Use
            };

            let build_path = path.as_ref().unwrap().as_path();
            if matches!(mode, crate::utils::cache::CacheMode::Rebuild) {
                crate::utils::cache::clear_cache(build_path);
            }

            let graph = match KnowledgeGraph::build_from_directory_with_cache_opts(
                build_path, mode, no_ignore,
            ) {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("Build failed: {e}");
                    return 1;
                }
            };

            // Optionally write JSON output
            if let Some(json_path) = json {
                let serialized =
                    serde_json::to_string_pretty(&graph).expect("serialize graph to JSON");
                if let Err(e) = fs::write(&json_path, serialized) {
                    eprintln!("Failed to write JSON output {json_path}: {e}");
                }
            }

            // DOT options from flags and optional config overrides
            let mut clusters = matches!(dot_clusters, crate::cli::OnOffArg::On);
            let mut legend = matches!(dot_legend, crate::cli::OnOffArg::On);
            let mut theme = match dot_theme {
                crate::cli::DotThemeArg::Dark => DotTheme::Dark,
                crate::cli::DotThemeArg::Light => DotTheme::Light,
            };
            let mut rankdir = match dot_rankdir {
                crate::cli::DotRankDirArg::TB => RankDir::TB,
                crate::cli::DotRankDirArg::LR => RankDir::LR,
            };
            let mut splines = match dot_splines {
                crate::cli::DotSplinesArg::Ortho => EdgeStyle::Ortho,
                crate::cli::DotSplinesArg::Polyline => EdgeStyle::Polyline,
                crate::cli::DotSplinesArg::Curved => EdgeStyle::Curved,
            };
            let mut rounded = matches!(dot_rounded, crate::cli::OnOffArg::On);
            if let Some(cfg_path) = config.as_ref() {
                if let Some(cfg) =
                    crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                {
                    if let Some(dot) = cfg.dot {
                        if let Some(v) = dot.clusters {
                            clusters = v;
                        }
                        if let Some(v) = dot.legend {
                            legend = v;
                        }
                        if let Some(v) = dot.theme {
                            theme = if v == "dark" { DotTheme::Dark } else { DotTheme::Light };
                        }
                        if let Some(v) = dot.rankdir {
                            rankdir = if v == "TB" { RankDir::TB } else { RankDir::LR };
                        }
                        if let Some(v) = dot.splines {
                            splines = match v.as_str() {
                                "ortho" => EdgeStyle::Ortho,
                                "polyline" => EdgeStyle::Polyline,
                                _ => EdgeStyle::Curved,
                            };
                        }
                        if let Some(v) = dot.rounded {
                            rounded = v;
                        }
                    }
                }
            }
            let dot_opts = DotOptions { clusters, legend, theme, rankdir, splines, rounded };

            if let Some(dot_path) = dot {
                match DotGenerator::new().generate_dot_with_options(&graph, dot_opts) {
                    Ok(content) => {
                        if let Err(e) = fs::write(&dot_path, content) {
                            eprintln!("Failed to write DOT output {dot_path}: {e}");
                        }
                    }
                    Err(e) => eprintln!("Visualization error: {e}"),
                }
            }

            if let Some(svg_path) = svg {
                let mut interactive = matches!(svg_interactive, crate::cli::OnOffArg::On);
                if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        if let Some(svg) = cfg.svg {
                            if let Some(v) = svg.interactive {
                                interactive = v;
                            }
                        }
                    }
                }
                let svg_opts = SvgOptions { dot: dot_opts, interactive };
                match SvgGenerator::new().generate_svg_with_options(&graph, svg_opts) {
                    Ok(content) => {
                        if let Err(e) = fs::write(&svg_path, content) {
                            eprintln!("Failed to write SVG output {svg_path}: {e}");
                        }
                    }
                    Err(e) => eprintln!("Visualization error: {e}"),
                }
            }

            if let Some(save_path) = save {
                if let Err(e) = KnowledgeGraph::save_json(&graph, std::path::Path::new(&save_path))
                {
                    eprintln!("Failed to save graph JSON {save_path}: {e}");
                }
            }

            if !cli.quiet {
                println!("Build completed for path: {}", build_path.display());
            }
            0
        }
        Commands::Query { query } => match query {
            QueryCommands::ConnectedFiles {
                path,
                config,
                no_ignore,
                file,
                graph: graph_path,
                format,
                offset,
                limit,
                file_pos: _,
            } => {
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Load graph failed: {e}");
                            return 1;
                        }
                    }
                } else {
                    let res = match KnowledgeGraph::build_from_directory_opts(
                        path.as_ref().unwrap().as_path(),
                        no_ignore,
                    ) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Build failed: {e}");
                            return 1;
                        }
                    };
                    res
                };
                let file = match file.as_ref() {
                    Some(f) => f,
                    None => {
                        eprintln!("Missing file argument. Provide <file> or --file <path>.");
                        return 2;
                    }
                };
                let q = crate::query::ConnectedFilesQuery::new(file);
                let results = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        match cfg.query.and_then(|q| q.default_format).as_deref() {
                            Some("json") => OutputFormat::Json,
                            Some("text") => OutputFormat::Text,
                            _ => format,
                        }
                    } else {
                        format
                    }
                } else {
                    format
                };
                let start = offset.min(results.len());
                let end = match limit {
                    Some(l) => (start + l).min(results.len()),
                    None => results.len(),
                };
                let page = &results[start..end];
                if matches!(fmt, OutputFormat::Json) {
                    let out: Vec<String> = page.iter().map(|p| p.display().to_string()).collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{s}"),
                        Err(e) => {
                            eprintln!("JSON encode error: {e}");
                            return 1;
                        }
                    }
                } else {
                    let rows: Vec<Vec<String>> = page
                        .iter()
                        .enumerate()
                        .map(|(i, p)| vec![format!("{}", start + i + 1), p.display().to_string()])
                        .collect();
                    let table = crate::utils::table::render(&["#", "Path"], &rows);
                    println!("{table}");
                }
                0
            }
            QueryCommands::FunctionUsage {
                path,
                config,
                no_ignore,
                function,
                direction,
                graph: graph_path,
                format,
                offset,
                limit,
            } => {
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Load graph failed: {e}");
                            return 1;
                        }
                    }
                } else {
                    if no_ignore {
                        std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1");
                    }
                    let res = match KnowledgeGraph::build_from_directory(
                        path.as_ref().unwrap().as_path(),
                    ) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Build failed: {e}");
                            if no_ignore {
                                std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                            }
                            return 1;
                        }
                    };
                    if no_ignore {
                        std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                    }
                    res
                };
                let dir = match direction {
                    crate::cli::Direction::Callees => crate::query::UsageDirection::Callees,
                    crate::cli::Direction::Callers => crate::query::UsageDirection::Callers,
                };
                let q = crate::query::FunctionUsageQuery { function, direction: dir };
                let results = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        match cfg.query.and_then(|q| q.default_format).as_deref() {
                            Some("json") => OutputFormat::Json,
                            Some("text") => OutputFormat::Text,
                            _ => format,
                        }
                    } else {
                        format
                    }
                } else {
                    format
                };
                let start = offset.min(results.len());
                let end = match limit {
                    Some(l) => (start + l).min(results.len()),
                    None => results.len(),
                };
                let page = &results[start..end];
                if matches!(fmt, OutputFormat::Json) {
                    let out: Vec<String> = page.iter().map(|p| p.display().to_string()).collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{s}"),
                        Err(e) => {
                            eprintln!("JSON encode error: {e}");
                            return 1;
                        }
                    }
                } else {
                    for p in page {
                        println!("{}", p.display());
                    }
                }
                0
            }
            QueryCommands::Cycles {
                path,
                config,
                no_ignore,
                graph: graph_path,
                format,
                offset,
                limit,
            } => {
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Load graph failed: {e}");
                            return 1;
                        }
                    }
                } else {
                    if no_ignore {
                        std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1");
                    }
                    let res = match KnowledgeGraph::build_from_directory(
                        path.as_ref().unwrap().as_path(),
                    ) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Build failed: {e}");
                            if no_ignore {
                                std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                            }
                            return 1;
                        }
                    };
                    if no_ignore {
                        std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                    }
                    res
                };
                let q = crate::query::CycleDetectionQuery::new();
                let cycles = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        match cfg.query.and_then(|q| q.default_format).as_deref() {
                            Some("json") => OutputFormat::Json,
                            Some("text") => OutputFormat::Text,
                            _ => format,
                        }
                    } else {
                        format
                    }
                } else {
                    format
                };
                let start = offset.min(cycles.len());
                let end = match limit {
                    Some(l) => (start + l).min(cycles.len()),
                    None => cycles.len(),
                };
                let page = &cycles[start..end];
                if matches!(fmt, OutputFormat::Json) {
                    let out: Vec<Vec<String>> = page
                        .iter()
                        .map(|cyc| cyc.iter().map(|p| p.display().to_string()).collect())
                        .collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{s}"),
                        Err(e) => {
                            eprintln!("JSON encode error: {e}");
                            return 1;
                        }
                    }
                } else {
                    for cyc in page {
                        let parts: Vec<String> =
                            cyc.iter().map(|p| p.display().to_string()).collect();
                        println!("{}", parts.join(" -> "));
                    }
                }
                0
            }
            QueryCommands::Path {
                path,
                config,
                no_ignore,
                from,
                to,
                graph: graph_path,
                format,
                offset,
                limit,
            } => {
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Load graph failed: {e}");
                            return 1;
                        }
                    }
                } else {
                    if no_ignore {
                        std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1");
                    }
                    let res = match KnowledgeGraph::build_from_directory(
                        path.as_ref().unwrap().as_path(),
                    ) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Build failed: {e}");
                            if no_ignore {
                                std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                            }
                            return 1;
                        }
                    };
                    if no_ignore {
                        std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                    }
                    res
                };
                let q = crate::query::ShortestPathQuery::new(&from, &to);
                let results = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        match cfg.query.and_then(|q| q.default_format).as_deref() {
                            Some("json") => OutputFormat::Json,
                            Some("text") => OutputFormat::Text,
                            _ => format,
                        }
                    } else {
                        format
                    }
                } else {
                    format
                };
                if matches!(fmt, OutputFormat::Json) {
                    let start = offset.min(results.len());
                    let end = match limit {
                        Some(l) => (start + l).min(results.len()),
                        None => results.len(),
                    };
                    let page = &results[start..end];
                    let out: Vec<String> = page.iter().map(|p| p.display().to_string()).collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{s}"),
                        Err(e) => {
                            eprintln!("JSON encode error: {e}");
                            return 1;
                        }
                    }
                } else if results.is_empty() {
                    println!("<no path>");
                } else {
                    let start = offset.min(results.len());
                    let end = match limit {
                        Some(l) => (start + l).min(results.len()),
                        None => results.len(),
                    };
                    let page = &results[start..end];
                    let rows: Vec<Vec<String>> = page
                        .iter()
                        .enumerate()
                        .map(|(i, p)| vec![format!("{}", start + i + 1), p.display().to_string()])
                        .collect();
                    let table = crate::utils::table::render(&["Step", "Path"], &rows);
                    println!("{table}");
                }
                0
            }
            QueryCommands::Hubs {
                path,
                config,
                no_ignore,
                graph: graph_path,
                metric,
                top,
                format,
                offset,
                limit,
            } => {
                use crate::query::{CentralityMetric, HubsQuery};
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Load graph failed: {e}");
                            return 1;
                        }
                    }
                } else {
                    if no_ignore {
                        std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1");
                    }
                    let res = match KnowledgeGraph::build_from_directory(
                        path.as_ref().unwrap().as_path(),
                    ) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Build failed: {e}");
                            if no_ignore {
                                std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                            }
                            return 1;
                        }
                    };
                    if no_ignore {
                        std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                    }
                    res
                };
                let m = match metric {
                    crate::cli::CentralityMetricArg::In => CentralityMetric::In,
                    crate::cli::CentralityMetricArg::Out => CentralityMetric::Out,
                    crate::cli::CentralityMetricArg::Total => CentralityMetric::Total,
                };
                let q = HubsQuery::new(m, top);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        match cfg.query.and_then(|q| q.default_format).as_deref() {
                            Some("json") => OutputFormat::Json,
                            Some("text") => OutputFormat::Text,
                            _ => format,
                        }
                    } else {
                        format
                    }
                } else {
                    format
                };
                let start = offset.min(rows.len());
                let end = match limit {
                    Some(l) => (start + l).min(rows.len()),
                    None => rows.len(),
                };
                let page = &rows[start..end];
                if matches!(fmt, OutputFormat::Json) {
                    #[derive(serde::Serialize)]
                    struct HubRow {
                        path: String,
                        indegree: usize,
                        outdegree: usize,
                    }
                    let out: Vec<HubRow> = page
                        .iter()
                        .map(|(p, i, o)| HubRow {
                            path: p.display().to_string(),
                            indegree: *i,
                            outdegree: *o,
                        })
                        .collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{s}"),
                        Err(e) => {
                            eprintln!("JSON encode error: {e}");
                            return 1;
                        }
                    }
                } else {
                    let body: Vec<Vec<String>> = if cli.verbose == 0 {
                        page.iter()
                            .map(|(p, i, o)| vec![p.display().to_string(), (i + o).to_string()])
                            .collect()
                    } else {
                        page.iter()
                            .map(|(p, i, o)| {
                                vec![
                                    p.display().to_string(),
                                    i.to_string(),
                                    o.to_string(),
                                    (i + o).to_string(),
                                ]
                            })
                            .collect()
                    };
                    let headers: &[&str] = if cli.verbose == 0 {
                        &["Path", "Total"]
                    } else {
                        &["Path", "In", "Out", "Total"]
                    };
                    let table = crate::utils::table::render(headers, &body);
                    println!("{table}");
                }
                0
            }
            QueryCommands::ModuleCentrality {
                path,
                config,
                no_ignore,
                graph: graph_path,
                metric,
                top,
                format,
                offset,
                limit,
            } => {
                use crate::query::{CentralityMetric, ModuleCentralityQuery};
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Load graph failed: {e}");
                            return 1;
                        }
                    }
                } else {
                    if no_ignore {
                        std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1");
                    }
                    let res = match KnowledgeGraph::build_from_directory(
                        path.as_ref().unwrap().as_path(),
                    ) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Build failed: {e}");
                            if no_ignore {
                                std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                            }
                            return 1;
                        }
                    };
                    if no_ignore {
                        std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                    }
                    res
                };
                let m = match metric {
                    crate::cli::CentralityMetricArg::In => CentralityMetric::In,
                    crate::cli::CentralityMetricArg::Out => CentralityMetric::Out,
                    crate::cli::CentralityMetricArg::Total => CentralityMetric::Total,
                };
                let q = ModuleCentralityQuery::new(m, top);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        match cfg.query.and_then(|q| q.default_format).as_deref() {
                            Some("json") => OutputFormat::Json,
                            Some("text") => OutputFormat::Text,
                            _ => format,
                        }
                    } else {
                        format
                    }
                } else {
                    format
                };
                let start = offset.min(rows.len());
                let end = match limit {
                    Some(l) => (start + l).min(rows.len()),
                    None => rows.len(),
                };
                let page = &rows[start..end];
                if matches!(fmt, OutputFormat::Json) {
                    #[derive(serde::Serialize)]
                    struct Row {
                        module: String,
                        indegree: usize,
                        outdegree: usize,
                    }
                    let out: Vec<Row> = page
                        .iter()
                        .map(|(p, i, o)| Row {
                            module: p.display().to_string(),
                            indegree: *i,
                            outdegree: *o,
                        })
                        .collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{s}"),
                        Err(e) => {
                            eprintln!("JSON encode error: {e}");
                            return 1;
                        }
                    }
                } else {
                    let body: Vec<Vec<String>> = if cli.verbose == 0 {
                        page.iter()
                            .map(|(p, i, o)| vec![p.display().to_string(), (i + o).to_string()])
                            .collect()
                    } else {
                        page.iter()
                            .map(|(p, i, o)| {
                                vec![
                                    p.display().to_string(),
                                    i.to_string(),
                                    o.to_string(),
                                    (i + o).to_string(),
                                ]
                            })
                            .collect()
                    };
                    let headers: &[&str] = if cli.verbose == 0 {
                        &["Module", "Total"]
                    } else {
                        &["Module", "In", "Out", "Total"]
                    };
                    let table = crate::utils::table::render(headers, &body);
                    println!("{table}");
                }
                0
            }
            QueryCommands::TraitImpls {
                path,
                config,
                no_ignore,
                r#trait,
                graph: graph_path,
                format,
                offset,
                limit,
            } => {
                use crate::query::TraitImplsQuery;
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Load graph failed: {e}");
                            return 1;
                        }
                    }
                } else {
                    if no_ignore {
                        std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1");
                    }
                    let res = match KnowledgeGraph::build_from_directory(
                        path.as_ref().unwrap().as_path(),
                    ) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Build failed: {e}");
                            if no_ignore {
                                std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                            }
                            return 1;
                        }
                    };
                    if no_ignore {
                        std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                    }
                    res
                };
                let q = TraitImplsQuery::new(&r#trait);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        match cfg.query.and_then(|q| q.default_format).as_deref() {
                            Some("json") => OutputFormat::Json,
                            Some("text") => OutputFormat::Text,
                            _ => format,
                        }
                    } else {
                        format
                    }
                } else {
                    format
                };
                let start = offset.min(rows.len());
                let end = match limit {
                    Some(l) => (start + l).min(rows.len()),
                    None => rows.len(),
                };
                let page = &rows[start..end];
                if matches!(fmt, OutputFormat::Json) {
                    #[derive(serde::Serialize)]
                    struct Row {
                        path: String,
                        r#type: String,
                    }
                    let out: Vec<Row> = page
                        .iter()
                        .map(|(p, t)| Row { path: p.display().to_string(), r#type: t.to_string() })
                        .collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{s}"),
                        Err(e) => {
                            eprintln!("JSON encode error: {e}");
                            return 1;
                        }
                    }
                } else if rows.is_empty() {
                    println!("<no implementations found>");
                } else {
                    let body: Vec<Vec<String>> = if cli.verbose == 0 {
                        page.iter().map(|(p, _t)| vec![p.display().to_string()]).collect()
                    } else {
                        page.iter().map(|(p, t)| vec![p.display().to_string(), t.clone()]).collect()
                    };
                    let headers: &[&str] =
                        if cli.verbose == 0 { &["Path"] } else { &["Path", "Type"] };
                    let table = crate::utils::table::render(headers, &body);
                    println!("{table}");
                }
                0
            }
            QueryCommands::UnreferencedItems {
                path,
                config,
                no_ignore,
                include_public,
                exclude,
                graph: graph_path,
                format,
                offset,
                limit,
            } => {
                use crate::query::UnreferencedItemsQuery;
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Load graph failed: {e}");
                            return 1;
                        }
                    }
                } else {
                    if no_ignore {
                        std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1");
                    }
                    let res = match KnowledgeGraph::build_from_directory(
                        path.as_ref().unwrap().as_path(),
                    ) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Build failed: {e}");
                            if no_ignore {
                                std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                            }
                            return 1;
                        }
                    };
                    if no_ignore {
                        std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                    }
                    res
                };
                let exclude_re = if let Some(pat) = exclude.as_ref() {
                    match regex::Regex::new(pat) {
                        Ok(r) => Some(r),
                        Err(e) => {
                            eprintln!("Invalid --exclude regex: {e}");
                            return 1;
                        }
                    }
                } else {
                    None
                };
                let q = UnreferencedItemsQuery::new(include_public, exclude_re);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        match cfg.query.and_then(|q| q.default_format).as_deref() {
                            Some("json") => OutputFormat::Json,
                            Some("text") => OutputFormat::Text,
                            _ => format,
                        }
                    } else {
                        format
                    }
                } else {
                    format
                };
                let start = offset.min(rows.len());
                let end = match limit {
                    Some(l) => (start + l).min(rows.len()),
                    None => rows.len(),
                };
                let page = &rows[start..end];
                if matches!(fmt, OutputFormat::Json) {
                    #[derive(serde::Serialize)]
                    struct Row {
                        path: String,
                        id: String,
                        name: String,
                        kind: String,
                        visibility: String,
                    }
                    let out: Vec<Row> = page
                        .iter()
                        .map(|(p, id, name, kind, vis)| Row {
                            path: p.display().to_string(),
                            id: id.clone(),
                            name: name.clone(),
                            kind: kind.clone(),
                            visibility: vis.clone(),
                        })
                        .collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{s}"),
                        Err(e) => {
                            eprintln!("JSON encode error: {e}");
                            return 1;
                        }
                    }
                } else if rows.is_empty() {
                    println!("<no unreferenced items>");
                } else {
                    let body: Vec<Vec<String>> = if cli.verbose == 0 {
                        page.iter()
                            .map(|(p, _id, name, _kind, _vis)| {
                                vec![p.display().to_string(), name.clone()]
                            })
                            .collect()
                    } else {
                        page.iter()
                            .map(|(p, id, name, kind, vis)| {
                                vec![
                                    p.display().to_string(),
                                    id.clone(),
                                    name.clone(),
                                    kind.clone(),
                                    vis.clone(),
                                ]
                            })
                            .collect()
                    };
                    let headers: &[&str] = if cli.verbose == 0 {
                        &["Path", "Name"]
                    } else {
                        &["Path", "ItemId", "Name", "Kind", "Vis"]
                    };
                    let table = crate::utils::table::render(headers, &body);
                    println!("{table}");
                }
                0
            }
            QueryCommands::ItemInfo {
                path,
                config,
                no_ignore,
                item_id,
                name,
                kind,
                graph: graph_path,
                show_code,
                format,
            } => {
                use crate::query::ItemInfoQuery;
                let graph = if let Some(p) = graph_path {
                    match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Load graph failed: {e}");
                            return 1;
                        }
                    }
                } else {
                    if no_ignore {
                        std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1");
                    }
                    let res = match KnowledgeGraph::build_from_directory(
                        path.as_ref().unwrap().as_path(),
                    ) {
                        Ok(g) => g,
                        Err(e) => {
                            eprintln!("Build failed: {e}");
                            if no_ignore {
                                std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                            }
                            return 1;
                        }
                    };
                    if no_ignore {
                        std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE");
                    }
                    res
                };
                // Determine target ItemId: prefer explicit --item-id, else resolve by --name/--kind
                let id = if let Some(raw_id) = item_id {
                    crate::graph::ItemId(raw_id)
                } else if let Some(nm) = name {
                    use crate::graph::{resolver::Resolver, ItemId, ItemType};
                    use std::path::Path;

                    let resolver = Resolver::new(&graph);
                    let mut ids: Vec<ItemId> = resolver.find_by_name(&nm);
                    if let Some(k) = kind {
                        ids.retain(|id| {
                            matches!(
                                (k, resolver.item_kind(id)),
                                (ItemKindArg::Module, Some(ItemType::Module { .. }))
                                    | (ItemKindArg::Function, Some(ItemType::Function { .. }))
                                    | (ItemKindArg::Struct, Some(ItemType::Struct { .. }))
                                    | (ItemKindArg::Enum, Some(ItemType::Enum { .. }))
                                    | (ItemKindArg::Trait, Some(ItemType::Trait { .. }))
                                    | (ItemKindArg::Impl, Some(ItemType::Impl { .. }))
                                    | (ItemKindArg::Const, Some(ItemType::Const))
                                    | (ItemKindArg::Static, Some(ItemType::Static { .. }))
                                    | (ItemKindArg::Type, Some(ItemType::Type))
                                    | (ItemKindArg::Macro, Some(ItemType::Macro))
                            )
                        });
                    }
                    if ids.is_empty() {
                        eprintln!("No item found with name '{nm}'.");
                        if let Some(k) = kind {
                            eprintln!("Hint: try a different --kind (current: {:?})", k);
                        }
                        return 1;
                    }

                    // Build candidate tuples (id, kind_str, path)
                    let mut candidates: Vec<(ItemId, String, std::path::PathBuf)> =
                        Vec::with_capacity(ids.len());
                    for id in ids.into_iter() {
                        let kind_s = match resolver.item_kind(&id) {
                            Some(ItemType::Module { .. }) => "module",
                            Some(ItemType::Function { .. }) => "function",
                            Some(ItemType::Struct { .. }) => "struct",
                            Some(ItemType::Enum { .. }) => "enum",
                            Some(ItemType::Trait { .. }) => "trait",
                            Some(ItemType::Impl { .. }) => "impl",
                            Some(ItemType::Const) => "const",
                            Some(ItemType::Static { .. }) => "static",
                            Some(ItemType::Type) => "type",
                            Some(ItemType::Macro) => "macro",
                            None => "?",
                        };
                        if let Some(p) = resolver.item_path(&id) {
                            candidates.push((id, kind_s.to_string(), p.clone()));
                        }
                    }
                    if candidates.is_empty() {
                        eprintln!("No item found with name '{nm}'.");
                        return 1;
                    }

                    // Prefer matches in current crate src/ if path is known
                    let root_src: Option<std::path::PathBuf> = path
                        .as_ref()
                        .map(|pb| pb.join("src"))
                        .or_else(|| std::env::current_dir().ok().map(|d| d.join("src")));

                    // Rank: (in_root_src desc, shallower depth asc, path lex asc)
                    let mut ranked = candidates;
                    if let Some(root_src_path) = root_src.as_ref() {
                        let root_src_canon = root_src_path;
                        ranked.sort_by(|a, b| {
                            let a_in = a.2.starts_with(root_src_canon);
                            let b_in = b.2.starts_with(root_src_canon);
                            let in_cmp = b_in.cmp(&a_in); // true first
                            if in_cmp != std::cmp::Ordering::Equal {
                                return in_cmp;
                            }
                            let depth = |p: &Path| -> usize {
                                let comps: Vec<_> = p.components().collect();
                                let mut seen_src = false;
                                let mut c = 0usize;
                                for comp in comps {
                                    if let std::path::Component::Normal(os) = comp {
                                        if os.to_str() == Some("src") {
                                            seen_src = true;
                                            continue;
                                        }
                                        if seen_src {
                                            c += 1;
                                        }
                                    }
                                }
                                c
                            };
                            let a_d = depth(&a.2);
                            let b_d = depth(&b.2);
                            let d_cmp = a_d.cmp(&b_d);
                            if d_cmp != std::cmp::Ordering::Equal {
                                return d_cmp;
                            }
                            a.2.cmp(&b.2)
                        });
                    } else {
                        ranked.sort_by(|a, b| a.2.cmp(&b.2));
                    }

                    // If still multiple and top two tie in rank dimensions, present ambiguity
                    let top = &ranked[0];
                    let same_rank = ranked
                        .iter()
                        .take_while(|cand| {
                            let a = cand;
                            let b = top;
                            let a_in = if let Some(r) = root_src.as_ref() {
                                a.2.starts_with(r)
                            } else {
                                false
                            };
                            let b_in = if let Some(r) = root_src.as_ref() {
                                b.2.starts_with(r)
                            } else {
                                false
                            };
                            let depth = |p: &Path| -> usize {
                                let comps: Vec<_> = p.components().collect();
                                let mut seen_src = false;
                                let mut c = 0usize;
                                for comp in comps {
                                    if let std::path::Component::Normal(os) = comp {
                                        if os.to_str() == Some("src") {
                                            seen_src = true;
                                            continue;
                                        }
                                        if seen_src {
                                            c += 1;
                                        }
                                    }
                                }
                                c
                            };
                            a_in == b_in && depth(&a.2) == depth(&b.2)
                        })
                        .count();
                    if ranked.len() > 1 && same_rank > 1 {
                        eprintln!("Ambiguous name '{nm}'. Top matches:");
                        for (cid, ck, cp) in ranked.iter().take(10) {
                            eprintln!("- id={}  kind={}  path={}", cid.0, ck, cp.display());
                        }
                        eprintln!("Disambiguate by providing --item-id or add --kind.");
                        return 1;
                    }
                    top.0.clone()
                } else {
                    eprintln!("Missing --item-id or --name for item-info.");
                    return 1;
                };
                let q = ItemInfoQuery::new(id, show_code);
                let result = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) =
                        crate::utils::config::load_config_at(std::path::Path::new(cfg_path))
                    {
                        match cfg.query.and_then(|q| q.default_format).as_deref() {
                            Some("json") => OutputFormat::Json,
                            Some("text") => OutputFormat::Text,
                            _ => format,
                        }
                    } else {
                        format
                    }
                } else {
                    format
                };
                if matches!(fmt, OutputFormat::Json) {
                    // Trim heavy fields when not verbose: drop code and relation contexts
                    let result = if cli.verbose == 0 {
                        result.map(|mut info| {
                            info.code = None;
                            for r in &mut info.inbound {
                                r.context.clear();
                            }
                            for r in &mut info.outbound {
                                r.context.clear();
                            }
                            info
                        })
                    } else {
                        result
                    };
                    match serde_json::to_string_pretty(&result) {
                        Ok(s) => println!("{s}"),
                        Err(e) => {
                            eprintln!("JSON encode error: {e}");
                            return 1;
                        }
                    }
                } else {
                    match result {
                        None => println!("<item not found>"),
                        Some(info) => {
                            println!("Item: {}", info.name);
                            println!("Id: {}", info.id);
                            println!("Kind: {}", info.kind);
                            println!("Vis: {}", info.visibility);
                            println!(
                                "Location: {}:{}-{}",
                                info.path, info.line_start, info.line_end
                            );
                            if cli.verbose == 0 {
                                let callers: String = if info.inbound.is_empty() {
                                    "<none>".to_string()
                                } else {
                                    info.inbound
                                        .iter()
                                        .map(|r| r.id.clone())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                };
                                let callees: String = if info.outbound.is_empty() {
                                    "<none>".to_string()
                                } else {
                                    info.outbound
                                        .iter()
                                        .map(|r| r.id.clone())
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                };
                                println!("\nCallers: {}", callers);
                                println!("Callees: {}", callees);
                            } else {
                                if show_code {
                                    if let Some(code) = info.code.as_deref() {
                                        println!("\n--- code ---\n{}\n--- end code ---", code);
                                    }
                                }
                                if info.inbound.is_empty() {
                                    println!("\nCallers: <none>");
                                } else {
                                    println!("\nCallers:");
                                    for r in info.inbound {
                                        println!(
                                            "- [{}] {} ({}) @ {} :: {}",
                                            r.relation, r.name, r.id, r.path, r.context
                                        );
                                    }
                                }
                                if info.outbound.is_empty() {
                                    println!("\nCallees: <none>");
                                } else {
                                    println!("\nCallees:");
                                    for r in info.outbound {
                                        println!(
                                            "- [{}] {} ({}) @ {} :: {}",
                                            r.relation, r.name, r.id, r.path, r.context
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                0
            }
        },
    }
}
