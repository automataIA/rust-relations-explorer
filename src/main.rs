fn main() {
    use knowledge_rs::cli::parse;
    let cli = parse();
    let code = knowledge_rs::app::run_cli(cli);
    if code != 0 { std::process::exit(code); }
}
/*
    
                        let res = match KnowledgeGraph::build_from_directory_opts(std::path::Path::new(&path), no_ignore) {
                            Ok(g) => g,
                            Err(e) => { eprintln!("Build failed: {}", e); std::process::exit(1); }
                        };
                        res
                    }
                };
                let q = knowledge_rs::query::ConnectedFilesQuery::new(&file);
                let results = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = knowledge_rs::utils::config::load_config_at(std::path::Path::new(cfg_path)) {
                        cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone())
                    } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    let out: Vec<String> = results.into_iter().map(|p| p.display().to_string()).collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{}", s),
                        Err(e) => { eprintln!("JSON encode error: {}", e); std::process::exit(1); }
                    }
                } else {
                    let rows: Vec<Vec<String>> = results
                        .into_iter()
                        .enumerate()
                        .map(|(i,p)| vec![format!("{}", i+1), p.display().to_string()])
                        .collect();
                    let table = knowledge_rs::utils::table::render(&["#", "Path"], &rows);
                    println!("{}", table);
                }
            }
            QueryCommands::FunctionUsage { path, config, no_ignore, function, direction, graph: graph_path, format } => {
                let graph = match graph_path {
                    Some(p) => match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => { eprintln!("Load graph failed: {}", e); std::process::exit(1); }
                    },
                    None => {
                        if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                        let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) {
                            Ok(g) => g,
                            Err(e) => { eprintln!("Build failed: {}", e); std::process::exit(1); }
                        };
                        if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                        res
                    }
                };
                let dir = match direction.as_str() {
                    "callees" => knowledge_rs::query::UsageDirection::Callees,
                    _ => knowledge_rs::query::UsageDirection::Callers,
                };
                let q = knowledge_rs::query::FunctionUsageQuery { function, direction: dir };
                let results = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = knowledge_rs::utils::config::load_config_at(std::path::Path::new(cfg_path)) {
                        cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone())
                    } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    let out: Vec<String> = results.into_iter().map(|p| p.display().to_string()).collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{}", s),
                        Err(e) => { eprintln!("JSON encode error: {}", e); std::process::exit(1); }
                    }
                } else {
                    for p in results { println!("{}", p.display()); }
                }
            }
            QueryCommands::Cycles { path, config, no_ignore, graph: graph_path, format } => {
                let graph = match graph_path {
                    Some(p) => match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => { eprintln!("Load graph failed: {}", e); std::process::exit(1); }
                    },
                    None => {
                        if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                        let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) {
                            Ok(g) => g,
                            Err(e) => { eprintln!("Build failed: {}", e); std::process::exit(1); }
                        };
                        if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                        res
                    }
                };
                let q = knowledge_rs::query::CycleDetectionQuery::new();
                let cycles = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = knowledge_rs::utils::config::load_config_at(std::path::Path::new(cfg_path)) {
                        cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone())
                    } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    let out: Vec<Vec<String>> = cycles
                        .into_iter()
                        .map(|cyc| cyc.into_iter().map(|p| p.display().to_string()).collect())
                        .collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{}", s),
                        Err(e) => { eprintln!("JSON encode error: {}", e); std::process::exit(1); }
                    }
                } else {
                    for cyc in cycles {
                        let parts: Vec<String> = cyc.iter().map(|p| p.display().to_string()).collect();
                        println!("{}", parts.join(" -> "));
                    }
                }
            }
            QueryCommands::Path { path, config, no_ignore, from, to, graph: graph_path, format } => {
                let graph = match graph_path {
                    Some(p) => match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => { eprintln!("Load graph failed: {}", e); std::process::exit(1); }
                    },
                    None => {
                        if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                        let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) {
                            Ok(g) => g,
                            Err(e) => { eprintln!("Build failed: {}", e); std::process::exit(1); }
                        };
                        if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                        res
                    }
                };
                let q = knowledge_rs::query::ShortestPathQuery::new(&from, &to);
                let results = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = knowledge_rs::utils::config::load_config_at(std::path::Path::new(cfg_path)) {
                        cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone())
                    } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    let out: Vec<String> = results.into_iter().map(|p| p.display().to_string()).collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{}", s),
                        Err(e) => { eprintln!("JSON encode error: {}", e); std::process::exit(1); }
                    }
                } else {
                    if results.is_empty() {
                        println!("<no path>");
                    } else {
                        let rows: Vec<Vec<String>> = results
                            .into_iter()
                            .enumerate()
                            .map(|(i,p)| vec![format!("{}", i+1), p.display().to_string()])
                            .collect();
                        let table = knowledge_rs::utils::table::render(&["Step", "Path"], &rows);
                        println!("{}", table);
                    }
                }
            }
            QueryCommands::Hubs { path, config, no_ignore, graph: graph_path, metric, top, format } => {
                use knowledge_rs::query::{HubsQuery, CentralityMetric};
                let graph = match graph_path {
                    Some(p) => match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => { eprintln!("Load graph failed: {}", e); std::process::exit(1); }
                    },
                    None => {
                        if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                        let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) {
                            Ok(g) => g,
                            Err(e) => { eprintln!("Build failed: {}", e); std::process::exit(1); }
                        };
                        if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                        res
                    }
                };
                let m = match metric.as_str() {
                    "in" => CentralityMetric::In,
                    "out" => CentralityMetric::Out,
                    _ => CentralityMetric::Total,
                };
                let q = HubsQuery::new(m, top);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = knowledge_rs::utils::config::load_config_at(std::path::Path::new(cfg_path)) {
                        cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone())
                    } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    #[derive(serde::Serialize)]
                    struct HubRow { path: String, indegree: usize, outdegree: usize }
                    let out: Vec<HubRow> = rows.into_iter().map(|(p,i,o)| HubRow { path: p.display().to_string(), indegree: i, outdegree: o }).collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{}", s),
                        Err(e) => { eprintln!("JSON encode error: {}", e); std::process::exit(1); }
                    }
                } else {
                    let body: Vec<Vec<String>> = rows
                        .into_iter()
                        .map(|(p,i,o)| vec![p.display().to_string(), i.to_string(), o.to_string(), (i+o).to_string()])
                        .collect();
                    let table = knowledge_rs::utils::table::render(&["Path", "In", "Out", "Total"], &body);
                    println!("{}", table);
                }
            }
            QueryCommands::ModuleCentrality { path, config, no_ignore, graph: graph_path, metric, top, format } => {
                use knowledge_rs::query::{ModuleCentralityQuery, CentralityMetric, Query};
                let graph = match graph_path {
                    Some(p) => match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => { eprintln!("Load graph failed: {}", e); std::process::exit(1); }
                    },
                    None => {
                        if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                        let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) {
                            Ok(g) => g,
                            Err(e) => { eprintln!("Build failed: {}", e); std::process::exit(1); }
                        };
                        if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                        res
                    }
                };
                let m = match metric.as_str() {
                    "in" => CentralityMetric::In,
                    "out" => CentralityMetric::Out,
                    _ => CentralityMetric::Total,
                };
                let q = ModuleCentralityQuery::new(m, top);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = knowledge_rs::utils::config::load_config_at(std::path::Path::new(cfg_path)) {
                        cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone())
                    } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    #[derive(serde::Serialize)]
                    struct Row { module: String, indegree: usize, outdegree: usize }
                    let out: Vec<Row> = rows.into_iter().map(|(p,i,o)| Row { module: p.display().to_string(), indegree: i, outdegree: o }).collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{}", s),
                        Err(e) => { eprintln!("JSON encode error: {}", e); std::process::exit(1); }
                    }
                } else {
                    let body: Vec<Vec<String>> = rows
                        .into_iter()
                        .map(|(p,i,o)| vec![p.display().to_string(), i.to_string(), o.to_string(), (i+o).to_string()])
                        .collect();
                    let table = knowledge_rs::utils::table::render(&["Module", "In", "Out", "Total"], &body);
                    println!("{}", table);
                }
            }
            QueryCommands::TraitImpls { path, config, no_ignore, r#trait, graph: graph_path, format } => {
                use knowledge_rs::query::{TraitImplsQuery, Query};
                let graph = match graph_path {
                    Some(p) => match KnowledgeGraph::load_json(std::path::Path::new(&p)) {
                        Ok(g) => g,
                        Err(e) => { eprintln!("Load graph failed: {}", e); std::process::exit(1); }
                    },
                    None => {
                        if no_ignore { std::env::set_var("KNOWLEDGE_RS_NO_IGNORE", "1"); }
                        let res = match KnowledgeGraph::build_from_directory(std::path::Path::new(&path)) {
                            Ok(g) => g,
                            Err(e) => { eprintln!("Build failed: {}", e); std::process::exit(1); }
                        };
                        if no_ignore { std::env::remove_var("KNOWLEDGE_RS_NO_IGNORE"); }
                        res
                    }
                };
                let q = TraitImplsQuery::new(&r#trait);
                let rows = q.run(&graph);
                let fmt = if let Some(cfg_path) = config.as_ref() {
                    if let Some(cfg) = knowledge_rs::utils::config::load_config_at(std::path::Path::new(cfg_path)) {
                        cfg.query.and_then(|q| q.default_format).unwrap_or(format.clone())
                    } else { format.clone() }
                } else { format.clone() };
                if fmt == "json" {
                    #[derive(serde::Serialize)]
                    struct Row { path: String, r#type: String }
                    let out: Vec<Row> = rows.into_iter().map(|(p,t)| Row { path: p.display().to_string(), r#type: t }).collect();
                    match serde_json::to_string_pretty(&out) {
                        Ok(s) => println!("{}", s),
                        Err(e) => { eprintln!("JSON encode error: {}", e); std::process::exit(1); }
                    }
                } else {
                    if rows.is_empty() {
                        println!("<no implementations found>");
                    } else {
                        let body: Vec<Vec<String>> = rows
                            .into_iter()
                            .map(|(p,t)| vec![p.display().to_string(), t])
                            .collect();
                        let table = knowledge_rs::utils::table::render(&["Path", "Type"], &body);
                        println!("{}", table);
                    }
                }
            }
        },
    }
}
*/
