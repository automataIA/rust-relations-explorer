use crate::errors::ParseError;
use crate::graph::{FileMetrics, FileNode, Import, Item, ItemId, ItemType, Location, Visibility};
use regex::Regex;
use std::path::Path;

#[derive(Debug, Default)]
pub struct RustParser {
    patterns: RegexPatterns,
}

#[derive(Debug)]
pub struct RegexPatterns {
    pub fn_sig: Regex,
    pub struct_def: Regex,
    pub enum_def: Regex,
    pub vis_pub_in: Regex,
    pub import_stmt: Regex,
}

impl RegexPatterns {
    pub fn compile() -> Self {
        // Simple, conservative regexes to avoid catastrophic backtracking
        let fn_sig = Regex::new(r"(?m)^\s*(?P<vis>pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:const\s+)?fn\s+(?P<name>[a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
        let struct_def = Regex::new(r"(?m)^\s*(?P<vis>pub(?:\([^)]*\))?\s+)?struct\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)").unwrap();
        let enum_def = Regex::new(r"(?m)^\s*(?P<vis>pub(?:\([^)]*\))?\s+)?enum\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)").unwrap();
        let vis_pub_in = Regex::new(r"^pub\((?P<sc>[^)]+)\)$").unwrap();
        let import_stmt = Regex::new(r"(?m)^\s*(?:pub\s+)?use\s+([^;{]+?)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;\s*$").unwrap();
        Self { fn_sig, struct_def, enum_def, vis_pub_in, import_stmt }
    }
}

impl Default for RegexPatterns {
    fn default() -> Self { Self::compile() }
}

impl RustParser {
    pub fn new() -> Self {
        Self { patterns: RegexPatterns::compile() }
    }

    pub fn parse_file(&self, content: &str, path: &Path) -> Result<FileNode, ParseError> {
        let items = self.extract_items(content, path);
        let imports = self.extract_imports(content);
        let metrics = FileMetrics { item_count: items.len(), import_count: imports.len() };
        Ok(FileNode { path: path.to_path_buf(), items, imports, metrics })
    }

    fn extract_items(&self, content: &str, path: &Path) -> Vec<Item> {
        // Pre-size output using rough counts to reduce reallocations
        let fn_count = self.patterns.fn_sig.captures_iter(content).count();
        let struct_count = self.patterns.struct_def.captures_iter(content).count();
        let enum_count = self.patterns.enum_def.captures_iter(content).count();
        let mut out = Vec::with_capacity(fn_count + struct_count + enum_count);

        for cap in self.patterns.fn_sig.captures_iter(content) {
            let name = cap.name("name").map(|m| m.as_str()).unwrap_or("").to_string();
            let vis = cap.name("vis").map(|m| m.as_str().trim()).unwrap_or("");
            let visibility = parse_visibility(&self.patterns.vis_pub_in, vis);
            let m0 = cap.get(0).unwrap();
            let line = line_number_for(content, m0.start());
            let span = m0.as_str();
            out.push(Item {
                id: ItemId(format!("fn:{}:{}", name, line)),
                item_type: ItemType::Function { is_async: span.contains("async "), is_const: span.contains("const ") },
                name,
                visibility,
                location: Location { file: path.to_path_buf(), line_start: line, line_end: line },
                attributes: vec![],
            });
        }

        for cap in self.patterns.struct_def.captures_iter(content) {
            let name = cap.name("name").map(|m| m.as_str()).unwrap_or("").to_string();
            let vis = cap.name("vis").map(|m| m.as_str().trim()).unwrap_or("");
            let visibility = parse_visibility(&self.patterns.vis_pub_in, vis);
            let line = line_number_for(content, cap.get(0).map(|m| m.start()).unwrap_or(0));
            out.push(Item {
                id: ItemId(format!("struct:{}:{}", name, line)),
                item_type: ItemType::Struct { is_tuple: false },
                name,
                visibility,
                location: Location { file: path.to_path_buf(), line_start: line, line_end: line },
                attributes: vec![],
            });
        }

        for cap in self.patterns.enum_def.captures_iter(content) {
            let name = cap.name("name").map(|m| m.as_str()).unwrap_or("").to_string();
            let vis = cap.name("vis").map(|m| m.as_str().trim()).unwrap_or("");
            let visibility = parse_visibility(&self.patterns.vis_pub_in, vis);
            let line = line_number_for(content, cap.get(0).map(|m| m.start()).unwrap_or(0));
            out.push(Item {
                id: ItemId(format!("enum:{}:{}", name, line)),
                item_type: ItemType::Enum { variant_count: 0 },
                name,
                visibility,
                location: Location { file: path.to_path_buf(), line_start: line, line_end: line },
                attributes: vec![],
            });
        }

        out
    }

    fn extract_imports(&self, content: &str) -> Vec<Import> {
        let import_count = self.patterns.import_stmt.captures_iter(content).count();
        let mut out = Vec::with_capacity(import_count);
        for cap in self.patterns.import_stmt.captures_iter(content) {
            let path = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("").to_string();
            let alias = cap.get(2).map(|m| m.as_str().to_string());
            out.push(Import { path, alias });
        }
        out
    }
}

fn parse_visibility(vis_pub_in: &Regex, vis: &str) -> Visibility {
    let v = vis.trim();
    if v.is_empty() { return Visibility::Private; }
    if v == "pub" { return Visibility::Public; }
    if v == "pub(crate)" { return Visibility::PubCrate; }
    if v == "pub(super)" { return Visibility::PubSuper; }
    if let Some(c) = vis_pub_in.captures(v) {
        return Visibility::PubIn(c.name("sc").map(|m| m.as_str().to_string()).unwrap_or_default());
    }
    Visibility::Private
}

fn line_number_for(content: &str, byte_idx: usize) -> usize {
    // 1-based line number
    content[..byte_idx].bytes().filter(|&b| b == b'\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_fn_struct_enum_and_visibility() {
        let src = r#"
        pub fn top() {}
        fn hidden() {}
        pub(crate) struct S;
        pub(super) enum E { A, B }
        "#;
        let parser = RustParser::new();
        let file = std::path::Path::new("/tmp/test.rs");
        let node = parser.parse_file(src, file).expect("parse");
        // items: 2 fn + 1 struct + 1 enum
        assert_eq!(node.items.len(), 4);
        // check visibility parsing
        let mut names: Vec<(String, Visibility)> = node.items.iter().map(|i| (i.name.clone(), i.visibility.clone())).collect();
        names.sort_by(|a,b| a.0.cmp(&b.0));
        assert!(names.iter().any(|(n,v)| n == "top" && matches!(v, Visibility::Public)));
        assert!(names.iter().any(|(n,v)| n == "hidden" && matches!(v, Visibility::Private)));
        assert!(names.iter().any(|(n,v)| n == "S" && matches!(v, Visibility::PubCrate)));
        assert!(names.iter().any(|(n,v)| n == "E" && matches!(v, Visibility::PubSuper)));
    }

    #[test]
    fn test_extract_imports_with_alias() {
        let src = r#"
        use std::collections::HashMap;
        pub use crate::module::Thing as Alias;
        "#;
        let parser = RustParser::new();
        let node = parser.parse_file(src, std::path::Path::new("/x.rs")).unwrap();
        assert_eq!(node.imports.len(), 2);
        assert!(node.imports.iter().any(|im| im.path.contains("std::collections::HashMap") && im.alias.is_none()));
        assert!(node.imports.iter().any(|im| im.path.contains("crate::module::Thing") && im.alias.as_deref() == Some("Alias")));
    }

    #[test]
    fn test_async_const_functions_and_tuple_struct() {
        let src = r#"
        pub async fn af() {}
        pub const fn cf() -> i32 { 0 }
        pub struct TS(u32, u32);
        pub(self) fn scoped() {}
        "#;
        let parser = RustParser::new();
        let node = parser.parse_file(src, std::path::Path::new("/y.rs")).unwrap();
        let names: Vec<_> = node.items.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"af"));
        assert!(names.contains(&"cf"));
        assert!(names.contains(&"TS"));
        // Ensure vis pub(in ..) patterns are accepted and mapped
        let scoped = node.items.iter().find(|i| i.name == "scoped").expect("scoped present");
        match scoped.visibility {
            Visibility::PubIn(ref s) => assert_eq!(s, "self"),
            _ => panic!("expected Visibility::PubIn('self') for scoped"),
        }
        // Sanity: counts align (2 fns + 1 tuple struct + 1 scoped fn)
        assert_eq!(node.items.len(), 4);
    }
}
