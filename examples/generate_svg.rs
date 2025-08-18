use knowledge_rs::graph::KnowledgeGraph;
use knowledge_rs::utils::cache::CacheMode;
use knowledge_rs::visualization::{DotGenerator, DotOptions, EdgeStyle, RankDir, SvgGenerator, SvgOptions, DotTheme};

fn main() {
    let root = std::path::Path::new(".");
    let graph = KnowledgeGraph::build_from_directory_with_cache_opts(root, CacheMode::Use, false)
        .expect("build graph");

    // Generate DOT with options
    let dot_opts = DotOptions { clusters: true, legend: true, theme: DotTheme::Light, rankdir: RankDir::LR, splines: EdgeStyle::Curved, rounded: true };
    let dot = DotGenerator::new().generate_dot_with_options(&graph, dot_opts).expect("dot");
    std::fs::write("graph.dot", dot).expect("write dot");

    // Generate SVG (requires `dot` from Graphviz on PATH)
    let svg_opts = SvgOptions { dot: dot_opts, interactive: true };
    match SvgGenerator::new().generate_svg_with_options(&graph, svg_opts) {
        Ok(svg) => {
            std::fs::write("graph.svg", svg).expect("write svg");
            println!("Wrote graph.svg");
        }
        Err(e) => {
            eprintln!("SVG generation failed: {}\nHint: ensure Graphviz 'dot' is installed and on PATH.", e);
        }
    }
}
