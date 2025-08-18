use crate::errors::KnowledgeGraphError;
use crate::graph::{ItemType, KnowledgeGraph, RelationshipType};
use std::fmt::Write as _;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub enum DotTheme { Light, Dark }

#[derive(Debug, Clone, Copy)]
pub enum RankDir { LR, TB }

#[derive(Debug, Clone, Copy)]
pub enum EdgeStyle { Curved, Ortho, Polyline }

#[derive(Debug, Clone, Copy)]
pub struct DotOptions {
    pub clusters: bool,
    pub legend: bool,
    pub theme: DotTheme,
    pub rankdir: RankDir,
    pub splines: EdgeStyle,
    pub rounded: bool,
}

impl Default for DotOptions {
    fn default() -> Self {
        Self { clusters: true, legend: true, theme: DotTheme::Light, rankdir: RankDir::LR, splines: EdgeStyle::Curved, rounded: true }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SvgOptions {
    pub dot: DotOptions,
    pub interactive: bool,
}

impl Default for SvgOptions {
    fn default() -> Self { Self { dot: DotOptions::default(), interactive: true } }
}

#[derive(Debug, Default)]
pub struct SvgGenerator;

impl SvgGenerator {
    #[must_use]
    pub fn new() -> Self { Self {} }

    /// Generate an SVG rendering using Graphviz.
    ///
    /// # Errors
    /// Returns `KnowledgeGraphError::Visualization` if invoking Graphviz fails,
    /// if the process exits with a non-success status, or if its output is not valid UTF-8.
    pub fn generate_svg_with_options(&self, graph: &KnowledgeGraph, opts: SvgOptions) -> Result<String, KnowledgeGraphError> {
        let dot = DotGenerator::new().generate_dot_with_options(graph, opts.dot)?;
        // Render with Graphviz `dot -Tsvg`
        let output = std::process::Command::new("dot")
            .arg("-Tsvg")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(dot.as_bytes())?;
                }
                child.wait_with_output()
            })
            .map_err(|e| KnowledgeGraphError::Visualization(format!("Failed to run graphviz 'dot': {e}")))?;
        if !output.status.success() {
            return Err(KnowledgeGraphError::Visualization(format!("Graphviz 'dot' failed with code {:?}", output.status.code())));
        }
        let mut svg = String::from_utf8(output.stdout).map_err(|e| KnowledgeGraphError::Visualization(format!("Invalid UTF-8 from dot: {e}")))?;
        if opts.interactive {
            svg = enhance_svg(&svg);
        }
        Ok(svg)
    }
}

fn enhance_svg(svg: &str) -> String {
    // Inject minimal CSS/JS for hover highlight and clickable nodes
    let injection = r"
<style>
svg .node:hover ellipse, svg .node:hover polygon, svg .node:hover path, svg .node:hover rect { filter: brightness(1.15); stroke-width: 2; }
svg .edge:hover path { stroke-width: 2.2; }
</style>
<script><![CDATA[
(function(){
  document.querySelectorAll('g.node a').forEach(function(a){
    a.addEventListener('click', function(ev){
      ev.preventDefault();
      const href = a.getAttribute('xlink:href') || a.getAttribute('href');
      if (href) { console.log('clicked', href); }
    });
  });
})();
]]></script>
";
    if let Some(pos) = svg.rfind("</svg>") {
        let mut out = String::with_capacity(svg.len() + injection.len());
        out.push_str(&svg[..pos]);
        out.push_str(injection);
        out.push_str(&svg[pos..]);
        out
    } else {
        let mut out = svg.to_string();
        out.push_str(injection);
        out
    }
}

#[derive(Debug, Default)]
pub struct DotGenerator;

impl DotGenerator {
    #[must_use]
    pub fn new() -> Self { Self {} }

    /// Generate DOT with default options.
    ///
    /// # Errors
    /// Returns a `KnowledgeGraphError` if DOT generation fails for any reason.
    pub fn generate_dot(&self, graph: &KnowledgeGraph) -> Result<String, KnowledgeGraphError> {
        self.generate_dot_with_options(graph, DotOptions::default())
    }

    /// Generate DOT with the given `opts`.
    ///
    /// # Errors
    /// Returns a `KnowledgeGraphError` if DOT generation fails for any reason.
    pub fn generate_dot_with_options(&self, graph: &KnowledgeGraph, opts: DotOptions) -> Result<String, KnowledgeGraphError> {
        let mut s = String::new();
        s.push_str("digraph KnowledgeRS\n{");
        s.push('\n');
        let rank = match opts.rankdir { RankDir::LR => "LR", RankDir::TB => "TB" };
        let splines = match opts.splines { EdgeStyle::Curved => "curved", EdgeStyle::Ortho => "ortho", EdgeStyle::Polyline => "polyline" };
        let node_style = if opts.rounded { "filled,rounded" } else { "filled" };
        let _ = write!(
            s,
            "  rankdir={rank};\n  graph [fontname=Helvetica, splines={splines}] ;\n  node [shape=box, fontsize=10, style={node_style}] ;\n  edge [fontname=Helvetica, fontsize=9];\n"
        );

        if opts.clusters {
            // Build hierarchical clusters from module roots
            let mut visited: HashSet<String> = HashSet::new();
            // Identify roots: files without a module parent
            let mut roots: Vec<_> = graph
                .files
                .keys()
                .filter(|p| graph.get_module_parent(p).is_none())
                .cloned()
                .collect();
            // Stable order for determinism
            roots.sort();
            for root in roots {
                self.write_module_cluster(graph, &root, &mut s, &mut visited, opts.theme);
            }
        } else {
            // No clusters: emit all nodes flat
            let mut paths: Vec<_> = graph.files.keys().cloned().collect();
            paths.sort();
            for path in paths {
                if let Some(file) = graph.files.get(&path) {
                    for item in &file.items {
                        let node_id = sanitize_id(&item.id.0);
                        let (fill, shape) = style_for_item_with_theme(&item.item_type, opts.theme);
                        let url = format!("item://{node_id}");
                        let tooltip = escape_label(&item.name);
                        let _ = writeln!(
                            s,
                            "  \"{node_id}\" [label=\"{}\", fillcolor=\"{fill}\", shape=\"{shape}\", URL=\"{url}\", tooltip=\"{tooltip}\"];",
                            escape_label(&item.name)
                        );
                    }
                }
            }
        }

        // Emit edges (relationships)
        for rel in &graph.relationships {
            let from = sanitize_id(&rel.from_item.0);
            let to = sanitize_id(&rel.to_item.0);
            let (label, color, style) = match &rel.relationship_type {
                RelationshipType::Uses { import_type } => (format!("uses:{import_type}"), "#1f77b4", "dashed"),
                RelationshipType::Implements { trait_name } => (format!("impl:{trait_name}"), "#2ca02c", "dotted"),
                RelationshipType::Contains { containment_type } => (format!("contains:{containment_type}"), "#7f7f7f", "solid"),
                RelationshipType::Extends { extension_type } => (format!("extends:{extension_type}"), "#9467bd", "dashed"),
                RelationshipType::Calls { call_type } => (format!("calls:{call_type}"), "#d62728", "solid"),
            };
            let penwidth = 0.8_f64.max(rel.strength).min(3.0);
            let _ = writeln!(
                s,
                "  \"{from}\" -> \"{to}\" [label=\"{}\", color=\"{color}\", style=\"{style}\", penwidth={penwidth}];",
                escape_label(&label)
            );
        }

        if opts.legend {
            // Legend cluster
            s.push_str("  subgraph cluster_legend {\n    label=\"Legend\";\n    color=grey;\n");
            let legend_items = [
                ("Module", ItemType::Module { is_inline: false }),
                ("Function", ItemType::Function { is_async: false, is_const: false }),
                ("Struct", ItemType::Struct { is_tuple: false }),
                ("Enum", ItemType::Enum { variant_count: 0 }),
                ("Trait", ItemType::Trait { is_object_safe: false }),
            ];
            for (name, t) in legend_items {
                let (fill, shape) = style_for_item_with_theme(&t, opts.theme);
                let id = sanitize_id(&format!("legend_{name}"));
                let _ = writeln!(s, "    \"{id}\" [label=\"{name}\", fillcolor=\"{fill}\", shape=\"{shape}\"]; ");
            }
            s.push_str("  }\n");
        }

        s.push_str("}\n");
        Ok(s)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn write_module_cluster(
        &self,
        graph: &KnowledgeGraph,
        path: &std::path::PathBuf,
        out: &mut String,
        visited: &mut HashSet<String>,
        theme: DotTheme,
    ) {
        let key = path.to_string_lossy().to_string();
        if !visited.insert(key.clone()) {
            return;
        }
        let cluster_id = format!("cluster_{}", sanitize_id(&key));
        let label = path.file_name().and_then(|p| p.to_str()).unwrap_or("");
        let _ = write!(out, "  subgraph \"{cluster_id}\" {{\n    label=\"{}\";\n    color=lightgrey;\n", escape_label(label));

        if let Some(file) = graph.files.get(path) {
            for item in &file.items {
                let node_id = sanitize_id(&item.id.0);
                let (fill, shape) = style_for_item_with_theme(&item.item_type, theme);
                let url = format!("item://{node_id}");
                let tooltip = escape_label(&item.name);
                let _ = writeln!(
                    out,
                    "    \"{node_id}\" [label=\"{}\", fillcolor=\"{fill}\", shape=\"{shape}\", URL=\"{url}\", tooltip=\"{tooltip}\"];",
                    escape_label(&item.name)
                );
            }
        }

        // Children
        for child in graph.get_module_children(path) {
            self.write_module_cluster(graph, child, out, visited, theme);
        }
        out.push_str("  }\n");
    }
}

fn sanitize_id(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => c,
            _ => '_',
        })
        .collect()
}

fn escape_label(s: &str) -> String {
    s.replace('"', "\\\"")
}

fn style_for_item_with_theme(t: &ItemType, theme: DotTheme) -> (&'static str, &'static str) {
    match (theme, t) {
        (DotTheme::Light, ItemType::Module { .. }) => ("#e0f3ff", "component"),
        (DotTheme::Light, ItemType::Function { .. }) => ("#e8ffe0", "oval"),
        (DotTheme::Light, ItemType::Struct { .. }) => ("#fff4e0", "box"),
        (DotTheme::Light, ItemType::Enum { .. }) => ("#ffe0f0", "hexagon"),
        (DotTheme::Light, ItemType::Trait { .. }) => ("#f0e0ff", "parallelogram"),
        (DotTheme::Light, ItemType::Impl { .. }) => ("#f0fff0", "box3d"),
        (DotTheme::Light, ItemType::Const) => ("#ffffe0", "note"),
        (DotTheme::Light, ItemType::Static { .. }) => ("#ffffe0", "folder"),
        (DotTheme::Light, ItemType::Type) => ("#f0ffff", "box"),
        (DotTheme::Light, ItemType::Macro) => ("#e0ffe8", "cds"),

        (DotTheme::Dark, ItemType::Module { .. }) => ("#124559", "component"),
        (DotTheme::Dark, ItemType::Function { .. }) => ("#0b6e4f", "oval"),
        (DotTheme::Dark, ItemType::Struct { .. }) => ("#7a4c00", "box"),
        (DotTheme::Dark, ItemType::Enum { .. }) => ("#6a1e44", "hexagon"),
        (DotTheme::Dark, ItemType::Trait { .. }) => ("#3c2a5a", "parallelogram"),
        (DotTheme::Dark, ItemType::Impl { .. }) => ("#1a5e1a", "box3d"),
        (DotTheme::Dark, ItemType::Const) => ("#6b6b00", "note"),
        (DotTheme::Dark, ItemType::Static { .. }) => ("#6b6b00", "folder"),
        (DotTheme::Dark, ItemType::Type) => ("#004f4f", "box"),
        (DotTheme::Dark, ItemType::Macro) => ("#0f5e3a", "cds"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::ItemType;

    #[test]
    fn test_sanitize_id_basic() {
        let s = "file:/path/to:thing.rs#1";
        let got = sanitize_id(s);
        // Allowed: letters, numbers, underscore; others -> underscore
        assert_eq!(got, "file__path_to_thing_rs_1");
        assert_eq!(sanitize_id("abc_DEF012"), "abc_DEF012");
    }

    #[test]
    fn test_escape_label_quotes() {
        let s = "a\"b\"c";
        let got = escape_label(s);
        assert_eq!(got, "a\\\"b\\\"c");
    }

    #[test]
    fn test_style_for_item_with_theme_all_variants() {
        // Light theme expectations
        let cases_light: Vec<(ItemType, (&str, &str))> = vec![
            (ItemType::Module { is_inline: false }, ("#e0f3ff", "component")),
            (ItemType::Function { is_async: false, is_const: false }, ("#e8ffe0", "oval")),
            (ItemType::Struct { is_tuple: false }, ("#fff4e0", "box")),
            (ItemType::Enum { variant_count: 0 }, ("#ffe0f0", "hexagon")),
            (ItemType::Trait { is_object_safe: false }, ("#f0e0ff", "parallelogram")),
            (ItemType::Impl { trait_name: None, type_name: "T".into() }, ("#f0fff0", "box3d")),
            (ItemType::Const, ("#ffffe0", "note")),
            (ItemType::Static { is_mut: false }, ("#ffffe0", "folder")),
            (ItemType::Type, ("#f0ffff", "box")),
            (ItemType::Macro, ("#e0ffe8", "cds")),
        ];
        for (t, expected) in cases_light {
            assert_eq!(style_for_item_with_theme(&t, DotTheme::Light), expected);
        }

        // Dark theme expectations
        let cases_dark: Vec<(ItemType, (&str, &str))> = vec![
            (ItemType::Module { is_inline: false }, ("#124559", "component")),
            (ItemType::Function { is_async: false, is_const: false }, ("#0b6e4f", "oval")),
            (ItemType::Struct { is_tuple: false }, ("#7a4c00", "box")),
            (ItemType::Enum { variant_count: 0 }, ("#6a1e44", "hexagon")),
            (ItemType::Trait { is_object_safe: false }, ("#3c2a5a", "parallelogram")),
            (ItemType::Impl { trait_name: None, type_name: "T".into() }, ("#1a5e1a", "box3d")),
            (ItemType::Const, ("#6b6b00", "note")),
            (ItemType::Static { is_mut: false }, ("#6b6b00", "folder")),
            (ItemType::Type, ("#004f4f", "box")),
            (ItemType::Macro, ("#0f5e3a", "cds")),
        ];
        for (t, expected) in cases_dark {
            assert_eq!(style_for_item_with_theme(&t, DotTheme::Dark), expected);
        }
    }

    #[test]
    fn test_enhance_svg_injection() {
        let minimal = "<svg></svg>";
        let out = enhance_svg(minimal);
        assert!(out.contains("<style>"));
        assert!(out.ends_with("</svg>"));

        // No closing tag case
        let no_close = "<svg>";
        let out2 = enhance_svg(no_close);
        assert!(out2.contains("<style>"));
    }
}
