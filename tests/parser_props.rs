use proptest::prelude::*;
use rust_relations_explorer::parser::RustParser;

// Bottom-up property-based tests: parser robustness on arbitrary inputs
proptest! {
    // The parser should never panic on arbitrary UTF-8 input
    #[test]
    fn parser_never_panics_on_arbitrary_input(s in ".*") {
        let parser = RustParser::new();
        let _ = parser.parse_file(&s, std::path::Path::new("/prop.rs"));
        // No assertion needed: the test passes if it doesn't panic
    }

    // Basic invariant: item/import counts are finite and consistent
    #[test]
    fn parser_produces_reasonable_counts(s in ".*") {
        let parser = RustParser::new();
        if let Ok(node) = parser.parse_file(&s, std::path::Path::new("/prop.rs")) {
            // Items and imports should be non-negative (Vec len) and not overflow typical bounds
            prop_assert!(node.items.len() <= s.len() + 1);
            prop_assert!(node.imports.len() <= s.len() + 1);
        }
    }
}
