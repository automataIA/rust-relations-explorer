use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "knowledge-rs",
    version,
    about = "Rust Knowledge Graph System",
    long_about = "Parse Rust projects into a knowledge graph and run queries. File discovery respects .gitignore and .ignore with parent traversal. Global git excludes are disabled for determinism. Use --no-ignore to bypass ignore rules."
)] 
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Build the knowledge graph from a source directory
    Build {
        /// Path to the Rust project root (directory containing src/)
        #[arg(short, long, default_value = ".")] 
        path: String,
        /// Path to a TOML configuration file
        #[arg(long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(long, default_value_t = false, help = "Include files even if matched by .gitignore/.ignore. Global git excludes are always disabled for determinism.")]
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
        #[arg(long, value_parser = ["on", "off"], default_value = "on")]
        dot_clusters: String,
        /// DOT: include legend (default: on)
        #[arg(long, value_parser = ["on", "off"], default_value = "on")]
        dot_legend: String,
        /// DOT: theme (light or dark)
        #[arg(long, value_parser = ["light", "dark"], default_value = "light")]
        dot_theme: String,
        /// DOT: rank direction (LR or TB)
        #[arg(long, value_parser = ["LR", "TB"], default_value = "LR")]
        dot_rankdir: String,
        /// DOT: edge splines style (curved, ortho, polyline)
        #[arg(long, value_parser = ["curved", "ortho", "polyline"], default_value = "curved")]
        dot_splines: String,
        /// DOT: rounded node corners (on/off)
        #[arg(long, value_parser = ["on", "off"], default_value = "on")]
        dot_rounded: String,
        /// SVG: add interactive enhancements (on/off)
        #[arg(long, value_parser = ["on", "off"], default_value = "on")]
        svg_interactive: String,
        /// Save built graph to JSON file path
        #[arg(long)]
        save: Option<String>,
    },
    /// Run queries over the knowledge graph
    Query {
        #[command(subcommand)]
        query: QueryCommands,
    },
    
}

#[derive(Debug, Subcommand)]
pub enum QueryCommands {
    /// List files connected to the given file via relationships
    ConnectedFiles {
        /// Path to project root (directory containing src/)
        #[arg(short, long, default_value = ".")] 
        path: String,
        /// Path to a TOML configuration file
        #[arg(long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(long, default_value_t = false, help = "Include files even if matched by .gitignore/.ignore. Global git excludes are always disabled for determinism.")]
        no_ignore: bool,
        /// The file to analyze (absolute or relative)
        #[arg(long)]
        file: String,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long)]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(long, value_parser = ["text", "json"], default_value = "text")]
        format: String,
    },
    /// List files that call or are called by a given function name
    FunctionUsage {
        /// Path to project root (directory containing src/)
        #[arg(short, long, default_value = ".")] 
        path: String,
        /// Path to a TOML configuration file
        #[arg(long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(long, default_value_t = false)]
        no_ignore: bool,
        /// Function name to analyze
        #[arg(long)]
        function: String,
        /// Direction: callers or callees
        #[arg(long, value_parser = ["callers", "callees"], default_value = "callers")]
        direction: String,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long)]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(long, value_parser = ["text", "json"], default_value = "text")]
        format: String,
    },
    /// Detect cycles between files
    Cycles {
        /// Path to project root (directory containing src/)
        #[arg(short, long, default_value = ".")] 
        path: String,
        /// Path to a TOML configuration file
        #[arg(long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(long, default_value_t = false)]
        no_ignore: bool,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long)]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(long, value_parser = ["text", "json"], default_value = "text")]
        format: String,
    },
    /// Compute shortest path between two files
    Path {
        /// Path to project root (directory containing src/)
        #[arg(short, long, default_value = ".")] 
        path: String,
        /// Path to a TOML configuration file
        #[arg(long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(long, default_value_t = false)]
        no_ignore: bool,
        /// Source file path
        #[arg(long)]
        from: String,
        /// Destination file path
        #[arg(long)]
        to: String,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long)]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(long, value_parser = ["text", "json"], default_value = "text")]
        format: String,
    },
    /// List top-N hub files by degree centrality
    Hubs {
        /// Path to project root (directory containing src/)
        #[arg(short, long, default_value = ".")] 
        path: String,
        /// Path to a TOML configuration file
        #[arg(long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(long, default_value_t = false)]
        no_ignore: bool,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long)]
        graph: Option<String>,
        /// Metric: in, out, total
        #[arg(long, value_parser = ["in", "out", "total"], default_value = "total")]
        metric: String,
        /// Top N results
        #[arg(long, default_value_t = 10)]
        top: usize,
        /// Output format: text or json
        #[arg(long, value_parser = ["text", "json"], default_value = "text")]
        format: String,
    },
    /// List top-N modules (directories) by degree centrality
    ModuleCentrality {
        /// Path to project root (directory containing src/)
        #[arg(short, long, default_value = ".")] 
        path: String,
        /// Path to a TOML configuration file
        #[arg(long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(long, default_value_t = false)]
        no_ignore: bool,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long)]
        graph: Option<String>,
        /// Metric: in, out, total
        #[arg(long, value_parser = ["in", "out", "total"], default_value = "total")]
        metric: String,
        /// Top N results
        #[arg(long, default_value_t = 10)]
        top: usize,
        /// Output format: text or json
        #[arg(long, value_parser = ["text", "json"], default_value = "text")]
        format: String,
    },
    /// List types implementing a trait
    TraitImpls {
        /// Path to project root (directory containing src/)
        #[arg(short, long, default_value = ".")] 
        path: String,
        /// Path to a TOML configuration file
        #[arg(long)]
        config: Option<String>,
        /// Bypass ignore rules (.gitignore/.ignore) when discovering files
        #[arg(long, default_value_t = false)]
        no_ignore: bool,
        /// Trait name (e.g., Display)
        #[arg(long, value_name = "NAME")]
        r#trait: String,
        /// Optional path to a prebuilt graph JSON (skips rebuild)
        #[arg(long)]
        graph: Option<String>,
        /// Output format: text or json
        #[arg(long, value_parser = ["text", "json"], default_value = "text")]
        format: String,
    },
}

#[must_use]
pub fn parse() -> Cli {
    Cli::parse()
}
