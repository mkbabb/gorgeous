use bbnf_derive::Parser;
use pprint::pprint as render;

use crate::{PrinterConfig, SourceRange, ToDoc};

#[derive(Parser)]
#[parser(
    path = "grammar/lang/json.bbnf",
    prettify
)]
pub struct JsonParser;

// Implement the crate traits for the generated enum.
impl<'a> ToDoc<'a> for JsonParserEnum<'a> {
    fn to_doc(&self) -> pprint::Doc<'a> {
        JsonParserEnum::to_doc(self)
    }
}

impl<'a> SourceRange for JsonParserEnum<'a> {
    fn source_range(&self) -> Option<(usize, usize)> {
        JsonParserEnum::source_range(self)
    }
}

/// Pretty-print a JSON string.
pub fn prettify_json(input: &str, config: &PrinterConfig) -> Option<String> {
    let ast = JsonParser::value().parse(input)?;
    let doc = ast.to_doc();
    Some(render(doc, Some(config.to_printer())))
}

/// Pretty-print only AST nodes overlapping `[range.start, range.end)`.
pub fn prettify_json_range(
    input: &str,
    range: std::ops::Range<usize>,
    config: &PrinterConfig,
) -> Option<String> {
    let ast = JsonParser::value().parse(input)?;
    let doc = crate::range_to_doc(&ast, input, range);
    Some(render(doc, Some(config.to_printer())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prettify_null() {
        let config = PrinterConfig::default();
        let result = prettify_json("null", &config).unwrap();
        assert_eq!(result.trim(), "null");
    }

    #[test]
    fn test_prettify_string() {
        let config = PrinterConfig::default();
        let result = prettify_json(r#""hello""#, &config).unwrap();
        assert_eq!(result.trim(), r#""hello""#);
    }

    #[test]
    fn test_prettify_number() {
        let config = PrinterConfig::default();
        let result = prettify_json("42", &config).unwrap();
        assert_eq!(result.trim(), "42");
    }

    #[test]
    fn test_prettify_simple_array() {
        let config = PrinterConfig::default();
        let result = prettify_json("[1, 2, 3]", &config).unwrap();
        assert!(result.contains("1"));
        assert!(result.contains("2"));
        assert!(result.contains("3"));
    }

    #[test]
    fn test_prettify_simple_object() {
        let config = PrinterConfig::default();
        let result = prettify_json(r#"{"a": 1, "b": 2}"#, &config).unwrap();
        assert!(result.contains(r#""a""#));
        assert!(result.contains("1"));
    }

    #[test]
    fn test_idempotent() {
        let config = PrinterConfig::default();
        let input = r#"{"key": [1, 2, {"nested": true}]}"#;
        let first = prettify_json(input, &config).unwrap();
        let second = prettify_json(&first, &config).unwrap();
        assert_eq!(first, second, "prettify should be idempotent");
    }

    #[test]
    fn test_nested_json_output() {
        let config = PrinterConfig::default();
        let input = r#"{"users": [{"id": 1, "name": "Alice", "email": "alice@example.com", "active": true}, {"id": 2, "name": "Bob", "email": "bob@example.com", "active": false}], "total": 2, "page": 1}"#;
        let result = prettify_json(input, &config).unwrap();
        // Each object in the array should be on its own line
        assert!(result.contains("},\n"),
            "Objects in array should be on separate lines:\n{}", result);
        // Verify idempotency
        let second = prettify_json(&result, &config).unwrap();
        assert_eq!(result, second, "nested JSON prettify should be idempotent");
    }

    #[test]
    fn test_json_formatting_samples() {
        let config = PrinterConfig::default();
        // Short objects stay inline
        let simple = prettify_json(r#"{"a": 1, "b": 2}"#, &config).unwrap();
        assert_eq!(simple.trim(), r#"{"a": 1, "b": 2}"#);
        // Deeply nested small objects stay inline
        let deep = prettify_json(r#"{"a": {"b": {"c": {"d": 1}}}}"#, &config).unwrap();
        assert_eq!(deep.trim(), r#"{"a": {"b": {"c": {"d": 1}}}}"#);
    }

    #[test]
    fn test_minified_equals_pretty() {
        let config = PrinterConfig::default();
        let minified = r#"{"a":1,"b":[2,3],"c":{"d":"e"}}"#;
        let pretty = r#"{ "a": 1, "b": [2, 3], "c": { "d": "e" } }"#;
        let from_min = prettify_json(minified, &config).unwrap();
        let from_pretty = prettify_json(pretty, &config).unwrap();
        // Both should produce structurally equivalent output.
        assert_eq!(from_min.trim(), from_pretty.trim());
    }
}
