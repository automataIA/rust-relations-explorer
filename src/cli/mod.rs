use crate::utils::project_root::effective_path_opt;
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "rust-relations-explorer",
    version,
    about = "Rust Knowledge Graph System",
    long_about = "Parse Rust projects into a knowledge graph and run queries. File discovery respects .gitignore and .ignore with parent traversal. Global git excludes are disabled for determinism. Use --no-ignore to bypass ignore rules.",
    arg_required_else_help = true,
    propagate_version = true
)]
pub struct Cli {
    /// Increase output verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub verbose: u8,
    /// Suppress non-essential output
    #[arg(short = 'q', long, default_value_t = false)]
    pub quiet: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Clone, Debug, Copy, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Debug, Copy, ValueEnum)]
pub enum Direction {
    Callers,
    Callees,
}

#[derive(Clone, Debug, Copy, ValueEnum)]
pub enum CentralityMetricArg {
    In,
    Out,
    Total,
}

#[derive(Clone, Debug, Copy, ValueEnum, PartialEq, Eq)]
pub enum OnOffArg {
    On,
    Off,
}

#[derive(Clone, Debug, Copy, ValueEnum, PartialEq, Eq)]
pub enum DotThemeArg {
    Light,
    Dark,
}

#[derive(Clone, Debug, Copy, ValueEnum, PartialEq, Eq)]
pub enum DotRankDirArg {
    #[value(alias = "LR")]
    LR,
    #[value(alias = "TB")]
    TB,
}

#[derive(Clone, Debug, Copy, ValueEnum, PartialEq, Eq)]
pub enum DotSplinesArg {
    Curved,
    Ortho,
    Polyline,
}

// --------------------
// Config (TOML) schema
// --------------------
#[derive(Debug, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    dot: DotConfig,
    #[serde(default)]
    svg: SvgConfig,
    #[serde(default)]
    query: QueryConfig,
}

#[derive(Debug, Default, Deserialize)]
struct DotConfig {
    clusters: Option<bool>,
    legend: Option<bool>,
    theme: Option<String>,   // "light" | "dark"
    rankdir: Option<String>, // "LR" | "TB"
    splines: Option<String>, // "curved" | "ortho" | "polyline"
    rounded: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct SvgConfig {
    interactive: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct QueryConfig {
    default_format: Option<String>, // "text" | "json"
}

fn load_config(path: &str) -> Option<ConfigFile> {
    let content = std::fs::read_to_string(path).ok()?;
    toml::from_str::<ConfigFile>(&content).ok()
}

fn on_off(b: bool) -> OnOffArg {
    if b {
        OnOffArg::On
    } else {
        OnOffArg::Off
    }
}
fn parse_theme(s: &str) -> Option<DotThemeArg> {
    match s.to_ascii_lowercase().as_str() {
        "light" => Some(DotThemeArg::Light),
        "dark" => Some(DotThemeArg::Dark),
        _ => None,
    }
}
fn parse_rankdir(s: &str) -> Option<DotRankDirArg> {
    match s.to_ascii_uppercase().as_str() {
        "LR" => Some(DotRankDirArg::LR),
        "TB" => Some(DotRankDirArg::TB),
        _ => None,
    }
}
fn parse_splines(s: &str) -> Option<DotSplinesArg> {
    match s.to_ascii_lowercase().as_str() {
        "curved" => Some(DotSplinesArg::Curved),
        "ortho" => Some(DotSplinesArg::Ortho),
        "polyline" => Some(DotSplinesArg::Polyline),
        _ => None,
    }
}
fn parse_format(s: &str) -> Option<OutputFormat> {
    match s.to_ascii_lowercase().as_str() {
        "text" => Some(OutputFormat::Text),
        "json" => Some(OutputFormat::Json),
        _ => None,
    }
}

#[derive(Clone, Debug, Copy, ValueEnum)]
pub enum ItemKindArg {
    Module,
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Const,
    Static,
    Type,
    Macro,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Build the knowledge graph from a source directory
    Build {
        /// Path to the Rust project root (directory containing src/)
        #[arg(short, long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(
            short='I', long,
            visible_aliases=["no-gitignore","all","ni"],
            default_value_t = false,
            help = "Include files even if matched by .gitignore/.ignore. Global git excludes are always disabled for determinism."
        )]
        no_ignore: bool,
        /// Ignore cache when building (do not reuse cached files)
        #[arg(long, default_value_t = false)]
        no_cache: bool,
        /// Rebuild cache from scratch (clears previous cache)
        #[arg(long, default_value_t = false)]
        rebuild: bool,
        /// Output JSON file path
        #[arg(long)]
        json: Option<String>,
        /// Output DOT file path
        #[arg(long)]
        dot: Option<String>,
        /// Output SVG file path
        #[arg(long)]
        svg: Option<String>,
        /// DOT: enable/disable hierarchical clusters (default: on)
        #[arg(long, value_enum, default_value_t = OnOffArg::On)]
        dot_clusters: OnOffArg,
        /// DOT: include legend (default: on)
        #[arg(long, value_enum, default_value_t = OnOffArg::On)]
        dot_legend: OnOffArg,
        /// DOT: theme (light or dark)
        #[arg(long, value_enum, default_value_t = DotThemeArg::Light)]
        dot_theme: DotThemeArg,
        /// DOT: rank direction (LR or TB)
        #[arg(long, value_enum, default_value_t = DotRankDirArg::LR)]
        dot_rankdir: DotRankDirArg,
        /// DOT: edge splines style (curved, ortho, polyline)
        #[arg(long, value_enum, default_value_t = DotSplinesArg::Curved)]
        dot_splines: DotSplinesArg,
        /// DOT: rounded node corners (on/off)
        #[arg(long, value_enum, default_value_t = OnOffArg::On)]
        dot_rounded: OnOffArg,
        /// SVG: add interactive enhancements (on/off)
        #[arg(long, value_enum, default_value_t = OnOffArg::On)]
        svg_interactive: OnOffArg,
        /// Save built graph to JSON file path
        #[arg(long)]
        save: Option<String>,
    },
    /// Run queries over the knowledge graph
    Query {
        #[command(subcommand)]
        query: QueryCommands,
    },
    /// Generate shell completion scripts
    Completions {
        /// Target shell (bash, zsh, fish, powershell, elvish)
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Debug, Subcommand)]
pub enum QueryCommands {
    /// List files connected to the given file via relationships
    ConnectedFiles {
        /// Path to project root (directory containing src/)
        #[arg(short, long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(
            short='I', long,
            visible_aliases=["no-gitignore","all","ni"],
            default_value_t = false,
            help = "Include files even if matched by .gitignore/.ignore. Global git excludes are always disabled for determinism."
        )]
        no_ignore: bool,
        /// Positional form of the file to analyze (absolute or relative)
        #[arg(value_name = "FILE")]
        file_pos: Option<String>,
        /// The file to analyze (absolute or relative)
        #[arg(long)]
        file: Option<String>,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long, env = "RRE_GRAPH")]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Text, env = "RRE_FORMAT")]
        format: OutputFormat,
        /// Pagination offset (number of cycles to skip)
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Pagination limit (max number of cycles to show)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Show a single item's definition and relations by ItemId
    ItemInfo {
        /// Path to project root (directory containing src/)
        #[arg(short, long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(short='I', long, visible_aliases=["no-gitignore","all","ni"], default_value_t = false)]
        no_ignore: bool,
        /// ItemId (e.g., fn:createIcons:6). Optional when --name is provided
        #[arg(long, value_name = "ID")]
        item_id: Option<String>,
        /// Lookup by item name (e.g., createIcons). Use with optional --kind to disambiguate
        #[arg(short = 'n', long, value_name = "NAME")]
        name: Option<String>,
        /// Optional kind to narrow name lookup (e.g., function, struct)
        #[arg(short = 'k', long, value_enum)]
        kind: Option<ItemKindArg>,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long, env = "RRE_GRAPH")]
        graph: Option<String>,
        /// Include code snippet of the item's definition
        #[arg(long, default_value_t = true)]
        show_code: bool,
        /// Output format: text or json
        #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Text, env = "RRE_FORMAT")]
        format: OutputFormat,
    },
    /// List files that call or are called by a given function name
    FunctionUsage {
        /// Path to project root (directory containing src/)
        #[arg(short, long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(short='I', long, visible_aliases=["no-gitignore","all","ni"], default_value_t = false)]
        no_ignore: bool,
        /// Function name to analyze
        #[arg(long)]
        function: String,
        /// Direction: callers or callees
        #[arg(long, value_enum, default_value_t = Direction::Callers)]
        direction: Direction,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long, env = "RRE_GRAPH")]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Text, env = "RRE_FORMAT")]
        format: OutputFormat,
        /// Pagination offset (number of items to skip)
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Pagination limit (max number of items to show)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Detect cycles between files
    Cycles {
        /// Path to project root (directory containing src/)
        #[arg(short, long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(short='I', long, visible_aliases=["no-gitignore","all","ni"], default_value_t = false)]
        no_ignore: bool,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long, env = "RRE_GRAPH")]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Text, env = "RRE_FORMAT")]
        format: OutputFormat,
        /// Pagination offset (number of cycles to skip)
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Pagination limit (max number of cycles to show)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Compute shortest path between two files
    Path {
        /// Path to project root (directory containing src/)
        #[arg(short = 'p', long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(short='I', long, visible_aliases=["no-gitignore","all","ni"], default_value_t = false)]
        no_ignore: bool,
        /// Source file path
        #[arg(long)]
        from: String,
        /// Destination file path
        #[arg(long)]
        to: String,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long, env = "RRE_GRAPH")]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Text, env = "RRE_FORMAT")]
        format: OutputFormat,
        /// Pagination offset (number of steps to skip)
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Pagination limit (max number of steps to show)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// List top-N hub files by degree centrality
    Hubs {
        /// Path to project root (directory containing src/)
        #[arg(short, long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(short='I', long, visible_aliases=["no-gitignore","all","ni"], default_value_t = false)]
        no_ignore: bool,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long, env = "RRE_GRAPH")]
        graph: Option<String>,
        /// Metric: in, out, total
        #[arg(long, value_enum, default_value_t = CentralityMetricArg::Total)]
        metric: CentralityMetricArg,
        /// Top N results
        #[arg(short = 't', long, default_value_t = 10)]
        top: usize,
        /// Output format: text or json
        #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Text, env = "RRE_FORMAT")]
        format: OutputFormat,
        /// Pagination offset (number of rows to skip)
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Pagination limit (max number of rows to show)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// List top-N modules (directories) by degree centrality
    ModuleCentrality {
        /// Path to project root (directory containing src/)
        #[arg(short, long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(short='I', long, visible_aliases=["no-gitignore","all","ni"], default_value_t = false)]
        no_ignore: bool,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long, env = "RRE_GRAPH")]
        graph: Option<String>,
        /// Metric: in, out, total
        #[arg(long, value_enum, default_value_t = CentralityMetricArg::Total)]
        metric: CentralityMetricArg,
        /// Top N results
        #[arg(short = 't', long, default_value_t = 10)]
        top: usize,
        /// Output format: text or json
        #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Text, env = "RRE_FORMAT")]
        format: OutputFormat,
        /// Pagination offset (number of rows to skip)
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Pagination limit (max number of rows to show)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// List types implementing a trait
    TraitImpls {
        /// Path to project root (directory containing src/)
        #[arg(short, long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(short='I', long, visible_aliases=["no-gitignore","all","ni"], default_value_t = false)]
        no_ignore: bool,
        /// Trait name (e.g., Display)
        #[arg(long, value_name = "NAME")]
        r#trait: String,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long, env = "RRE_GRAPH")]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Text, env = "RRE_FORMAT")]
        format: OutputFormat,
        /// Pagination offset (number of rows to skip)
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Pagination limit (max number of rows to show)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// List items with no inbound usage edges (potentially dead code)
    UnreferencedItems {
        /// Path to project root (directory containing src/)
        #[arg(short, long, env = "RRE_PATH")]
        path: Option<PathBuf>,
        /// Path to a TOML configuration file
        #[arg(short = 'c', long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(short='I', long, visible_aliases=["no-gitignore","all","ni"], default_value_t = false)]
        no_ignore: bool,
        /// Include public items as well (by default public items are excluded)
        #[arg(long, default_value_t = false)]
        include_public: bool,
        /// Regex to exclude paths (e.g., 'tests|benches|examples')
        #[arg(long)]
        exclude: Option<String>,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long, env = "RRE_GRAPH")]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Text, env = "RRE_FORMAT")]
        format: OutputFormat,
        /// Pagination offset (number of rows to skip)
        #[arg(long, default_value_t = 0)]
        offset: usize,
        /// Pagination limit (max number of rows to show)
        #[arg(long)]
        limit: Option<usize>,
    },
}

#[must_use]
pub fn parse() -> Cli {
    let mut cli = Cli::parse();
    match &mut cli.command {
        Commands::Build {
            path,
            config,
            no_ignore: _,
            no_cache: _,
            rebuild: _,
            json: _,
            dot: _,
            svg: _,
            dot_clusters,
            dot_legend,
            dot_theme,
            dot_rankdir,
            dot_splines,
            dot_rounded,
            svg_interactive,
            save: _,
        } => {
            let p = effective_path_opt(path.as_deref());
            *path = Some(p);
            // Apply config if provided (only to defaulted values)
            if let Some(cfg_path) = config.as_deref() {
                if let Some(cfg) = load_config(cfg_path) {
                    if let Some(b) = cfg.dot.clusters {
                        if *dot_clusters == OnOffArg::On {
                            *dot_clusters = on_off(b);
                        }
                    }
                    if let Some(b) = cfg.dot.legend {
                        if *dot_legend == OnOffArg::On {
                            *dot_legend = on_off(b);
                        }
                    }
                    if let Some(s) = cfg.dot.theme.as_deref().and_then(parse_theme) {
                        if *dot_theme == DotThemeArg::Light {
                            *dot_theme = s;
                        }
                    }
                    if let Some(s) = cfg.dot.rankdir.as_deref().and_then(parse_rankdir) {
                        if *dot_rankdir == DotRankDirArg::LR {
                            *dot_rankdir = s;
                        }
                    }
                    if let Some(s) = cfg.dot.splines.as_deref().and_then(parse_splines) {
                        if *dot_splines == DotSplinesArg::Curved {
                            *dot_splines = s;
                        }
                    }
                    if let Some(b) = cfg.dot.rounded {
                        if *dot_rounded == OnOffArg::On {
                            *dot_rounded = on_off(b);
                        }
                    }
                    if let Some(b) = cfg.svg.interactive {
                        if *svg_interactive == OnOffArg::On {
                            *svg_interactive = on_off(b);
                        }
                    }
                }
            }
            if cli.verbose > 0 && !cli.quiet {
                eprintln!("Using project root: {}", path.as_ref().unwrap().display());
            }
        }
        Commands::Query { query } => match query {
            QueryCommands::ConnectedFiles { path, file_pos, file, config, format, .. } => {
                let p = effective_path_opt(path.as_deref());
                *path = Some(p);
                if let Some(cfg_path) = config.as_deref() {
                    if let Some(cfg) = load_config(cfg_path) {
                        if let Some(f) = cfg.query.default_format.as_deref().and_then(parse_format)
                        {
                            if *format == OutputFormat::Text {
                                *format = f;
                            }
                        }
                    }
                }
                if cli.verbose > 0 && !cli.quiet {
                    eprintln!("Using project root: {}", path.as_ref().unwrap().display());
                }
                // Normalize positional <file> vs --file
                let merged = if file.is_none() { file_pos.clone() } else { file.clone() };
                if let Some(f) = merged {
                    *file = Some(f);
                }
            }
            QueryCommands::ItemInfo { path, config, format, .. } => {
                let p = effective_path_opt(path.as_deref());
                *path = Some(p);
                if let Some(cfg_path) = config.as_deref() {
                    if let Some(cfg) = load_config(cfg_path) {
                        if let Some(f) = cfg.query.default_format.as_deref().and_then(parse_format)
                        {
                            if *format == OutputFormat::Text {
                                *format = f;
                            }
                        }
                    }
                }
                if cli.verbose > 0 && !cli.quiet {
                    eprintln!("Using project root: {}", path.as_ref().unwrap().display());
                }
            }
            QueryCommands::FunctionUsage { path, config, format, .. } => {
                let p = effective_path_opt(path.as_deref());
                *path = Some(p);
                if let Some(cfg_path) = config.as_deref() {
                    if let Some(cfg) = load_config(cfg_path) {
                        if let Some(f) = cfg.query.default_format.as_deref().and_then(parse_format)
                        {
                            if *format == OutputFormat::Text {
                                *format = f;
                            }
                        }
                    }
                }
                if cli.verbose > 0 && !cli.quiet {
                    eprintln!("Using project root: {}", path.as_ref().unwrap().display());
                }
            }
            QueryCommands::Cycles { path, config, format, .. } => {
                let p = effective_path_opt(path.as_deref());
                *path = Some(p);
                if let Some(cfg_path) = config.as_deref() {
                    if let Some(cfg) = load_config(cfg_path) {
                        if let Some(f) = cfg.query.default_format.as_deref().and_then(parse_format)
                        {
                            if *format == OutputFormat::Text {
                                *format = f;
                            }
                        }
                    }
                }
                if cli.verbose > 0 && !cli.quiet {
                    eprintln!("Using project root: {}", path.as_ref().unwrap().display());
                }
            }
            QueryCommands::Path { path, config, format, .. } => {
                let p = effective_path_opt(path.as_deref());
                *path = Some(p);
                if let Some(cfg_path) = config.as_deref() {
                    if let Some(cfg) = load_config(cfg_path) {
                        if let Some(f) = cfg.query.default_format.as_deref().and_then(parse_format)
                        {
                            if *format == OutputFormat::Text {
                                *format = f;
                            }
                        }
                    }
                }
                if cli.verbose > 0 && !cli.quiet {
                    eprintln!("Using project root: {}", path.as_ref().unwrap().display());
                }
            }
            QueryCommands::Hubs { path, config, format, .. } => {
                let p = effective_path_opt(path.as_deref());
                *path = Some(p);
                if let Some(cfg_path) = config.as_deref() {
                    if let Some(cfg) = load_config(cfg_path) {
                        if let Some(f) = cfg.query.default_format.as_deref().and_then(parse_format)
                        {
                            if *format == OutputFormat::Text {
                                *format = f;
                            }
                        }
                    }
                }
                if cli.verbose > 0 && !cli.quiet {
                    eprintln!("Using project root: {}", path.as_ref().unwrap().display());
                }
            }
            QueryCommands::ModuleCentrality { path, config, format, .. } => {
                let p = effective_path_opt(path.as_deref());
                *path = Some(p);
                if let Some(cfg_path) = config.as_deref() {
                    if let Some(cfg) = load_config(cfg_path) {
                        if let Some(f) = cfg.query.default_format.as_deref().and_then(parse_format)
                        {
                            if *format == OutputFormat::Text {
                                *format = f;
                            }
                        }
                    }
                }
                if cli.verbose > 0 && !cli.quiet {
                    eprintln!("Using project root: {}", path.as_ref().unwrap().display());
                }
            }
            QueryCommands::TraitImpls { path, config, format, .. } => {
                let p = effective_path_opt(path.as_deref());
                *path = Some(p);
                if let Some(cfg_path) = config.as_deref() {
                    if let Some(cfg) = load_config(cfg_path) {
                        if let Some(f) = cfg.query.default_format.as_deref().and_then(parse_format)
                        {
                            if *format == OutputFormat::Text {
                                *format = f;
                            }
                        }
                    }
                }
                if cli.verbose > 0 && !cli.quiet {
                    eprintln!("Using project root: {}", path.as_ref().unwrap().display());
                }
            }
            QueryCommands::UnreferencedItems { path, config, format, .. } => {
                let p = effective_path_opt(path.as_deref());
                *path = Some(p);
                if let Some(cfg_path) = config.as_deref() {
                    if let Some(cfg) = load_config(cfg_path) {
                        if let Some(f) = cfg.query.default_format.as_deref().and_then(parse_format)
                        {
                            if *format == OutputFormat::Text {
                                *format = f;
                            }
                        }
                    }
                }
                if cli.verbose > 0 && !cli.quiet {
                    eprintln!("Using project root: {}", path.as_ref().unwrap().display());
                }
            }
        },
        Commands::Completions { .. } => {
            // No path normalization or config backfilling needed here
        }
    }
    cli
}
