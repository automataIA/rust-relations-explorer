use rust_relations_explorer::app::run_cli;
use rust_relations_explorer::cli::{
    CentralityMetricArg, Cli, Commands, Direction, DotRankDirArg, DotSplinesArg, DotThemeArg,
    OnOffArg, OutputFormat, QueryCommands,
};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tempfile::tempdir;

fn write_file(path: &PathBuf, content: &str) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let mut f = fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

#[test]
fn app_build_generates_dot_and_json() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "pub fn top() {}\n");

    let dot_out = root.join("graph.dot");
    let json_out = root.join("graph.json");

    let cli = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Build {
            path: Some(root.to_path_buf()),
            config: None,
            no_ignore: false,
            no_cache: false,
            rebuild: false,
            json: Some(json_out.display().to_string()),
            dot: Some(dot_out.display().to_string()),
            svg: None,
            dot_clusters: OnOffArg::On,
            dot_legend: OnOffArg::On,
            dot_theme: DotThemeArg::Light,
            dot_rankdir: DotRankDirArg::LR,
            dot_splines: DotSplinesArg::Curved,
            dot_rounded: OnOffArg::On,
            svg_interactive: OnOffArg::On,
            save: None,
        },
    };

    let code = run_cli(cli);
    assert_eq!(code, 0);
    assert!(dot_out.exists());
    assert!(json_out.exists());

    let dot_str = fs::read_to_string(&dot_out).unwrap();
    assert!(dot_str.contains("rankdir=LR"));
}

#[test]
fn app_query_connected_files_json_branch() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "mod a; mod b; pub use a::foo;\n");
    write_file(&src.join("a.rs"), "pub fn foo() {}\n");
    write_file(&src.join("b.rs"), "pub fn bar() { crate::a::foo(); }\n");

    let cli = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::ConnectedFiles {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                file_pos: None,
                file: Some(src.join("a.rs").display().to_string()),
                graph: None,
                format: OutputFormat::Json,
                offset: 0,
                limit: None,
            },
        },
    };

    let code = run_cli(cli);
    assert_eq!(code, 0);
}

#[test]
fn app_query_function_usage_callers_and_callees() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "mod a; mod b; pub use a::foo;\n");
    write_file(&src.join("a.rs"), "pub fn foo() {}\n");
    write_file(&src.join("b.rs"), "pub fn bar() { crate::a::foo(); }\n");

    // callers branch (default)
    let cli_callers = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::FunctionUsage {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                function: "foo".into(),
                direction: Direction::Callers,
                graph: None,
                format: OutputFormat::Text,
                offset: 0,
                limit: None,
            },
        },
    };
    assert_eq!(run_cli(cli_callers), 0);

    // callees branch
    let cli_callees = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::FunctionUsage {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                function: "bar".into(),
                direction: Direction::Callees,
                graph: None,
                format: OutputFormat::Json,
                offset: 0,
                limit: None,
            },
        },
    };
    assert_eq!(run_cli(cli_callees), 0);
}

#[test]
fn app_query_cycles_json_branch() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    write_file(&src.join("lib.rs"), "mod a; mod b;\n");
    write_file(&src.join("a.rs"), "pub fn a() {}\n");
    write_file(&src.join("b.rs"), "pub fn b() {}\n");

    let cli = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::Cycles {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                graph: None,
                format: OutputFormat::Json,
                offset: 0,
                limit: None,
            },
        },
    };
    assert_eq!(run_cli(cli), 0);
}

#[test]
fn app_query_path_json_path_and_no_path() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    write_file(&src.join("lib.rs"), "mod a; mod b;\n");
    write_file(&src.join("a.rs"), "pub fn a() {}\n");
    // create a call from a -> b to have a path
    write_file(&src.join("b.rs"), "pub fn b() { super::a::a(); }\n");

    // json format
    let cli_json = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::Path {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                from: src.join("lib.rs").display().to_string(),
                to: src.join("a.rs").display().to_string(),
                graph: None,
                format: OutputFormat::Json,
                offset: 0,
                limit: None,
            },
        },
    };
    assert_eq!(run_cli(cli_json), 0);

    // no path text branch: a -> lib.rs likely no direct path
    let cli_no_path = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::Path {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                from: src.join("a.rs").display().to_string(),
                to: src.join("lib.rs").display().to_string(),
                graph: None,
                format: OutputFormat::Text,
                offset: 0,
                limit: None,
            },
        },
    };
    assert_eq!(run_cli(cli_no_path), 0);
}

#[test]
fn app_query_module_centrality_json() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    write_file(&src.join("lib.rs"), "mod a; mod b;\n");
    write_file(&src.join("a.rs"), "pub fn a() {}\n");
    write_file(&src.join("b.rs"), "pub fn b() {}\n");

    let cli = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::ModuleCentrality {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                graph: None,
                metric: CentralityMetricArg::Total,
                top: 3,
                format: OutputFormat::Json,
                offset: 0,
                limit: None,
            },
        },
    };
    assert_eq!(run_cli(cli), 0);
}

#[test]
fn app_query_trait_impls_text_and_json() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    // Minimal crate; analyzer may or may not find trait impls; still exercise branches
    write_file(
        &src.join("lib.rs"),
        "pub trait T { fn f(&self); } struct S; impl T for S { fn f(&self) {} }\n",
    );

    let cli_text = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::TraitImpls {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                r#trait: "T".into(),
                graph: None,
                format: OutputFormat::Text,
                offset: 0,
                limit: None,
            },
        },
    };
    assert_eq!(run_cli(cli_text), 0);

    let cli_json = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::TraitImpls {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                r#trait: "T".into(),
                graph: None,
                format: OutputFormat::Json,
                offset: 0,
                limit: None,
            },
        },
    };
    assert_eq!(run_cli(cli_json), 0);
}

#[test]
fn app_build_with_cache_flags_and_no_ignore() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();
    write_file(&src.join("lib.rs"), "pub fn top() {}\n");

    // First build with no-cache
    let cli_no_cache = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Build {
            path: Some(root.to_path_buf()),
            config: None,
            no_ignore: true,
            no_cache: true,
            rebuild: false,
            json: None,
            dot: None,
            svg: None,
            dot_clusters: OnOffArg::Off,
            dot_legend: OnOffArg::Off,
            dot_theme: DotThemeArg::Light,
            dot_rankdir: DotRankDirArg::LR,
            dot_splines: DotSplinesArg::Curved,
            dot_rounded: OnOffArg::Off,
            svg_interactive: OnOffArg::Off,
            save: None,
        },
    };
    assert_eq!(run_cli(cli_no_cache), 0);

    // Then rebuild to ensure rebuild branch executes
    let cli_rebuild = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Build {
            path: Some(root.to_path_buf()),
            config: None,
            no_ignore: false,
            no_cache: false,
            rebuild: true,
            json: None,
            dot: None,
            svg: None,
            dot_clusters: OnOffArg::Off,
            dot_legend: OnOffArg::Off,
            dot_theme: DotThemeArg::Light,
            dot_rankdir: DotRankDirArg::LR,
            dot_splines: DotSplinesArg::Curved,
            dot_rounded: OnOffArg::Off,
            svg_interactive: OnOffArg::Off,
            save: None,
        },
    };
    assert_eq!(run_cli(cli_rebuild), 0);
}

#[test]
fn app_query_hubs_text_branch() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    write_file(&src.join("lib.rs"), "mod a; mod b;\n");
    write_file(&src.join("a.rs"), "pub fn fa() {}\n");
    write_file(&src.join("b.rs"), "pub fn fb() { }\n");

    let cli = Cli {
        verbose: 0,
        quiet: false,
        command: Commands::Query {
            query: QueryCommands::Hubs {
                path: Some(root.to_path_buf()),
                config: None,
                no_ignore: false,
                graph: None,
                metric: CentralityMetricArg::In,
                top: 5,
                format: OutputFormat::Text,
                offset: 0,
                limit: None,
            },
        },
    };

    let code = run_cli(cli);
    assert_eq!(code, 0);
}
