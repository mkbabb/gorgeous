use bbnf_derive::Parser;
use pprint::pprint as render;

use crate::{PrinterConfig, SourceRange, ToDoc};

#[derive(Parser)]
#[parser(
    path = "grammar/css/css-stylesheet-pretty.bbnf",
    prettify,
    skip_recover
)]
pub struct CssParser;

impl<'a> ToDoc<'a> for CssParserEnum<'a> {
    fn to_doc(&self) -> pprint::Doc<'a> {
        CssParserEnum::to_doc(self)
    }
}

impl<'a> SourceRange for CssParserEnum<'a> {
    fn source_range(&self) -> Option<(usize, usize)> {
        CssParserEnum::source_range(self)
    }
}

/// Pretty-print a CSS stylesheet.
pub fn prettify_css(input: &str, config: &PrinterConfig) -> Option<String> {
    let ast = CssParser::stylesheet().parse(input)?;
    let doc = ast.to_doc();
    Some(render(doc, config.to_printer()))
}

/// Pretty-print only AST nodes overlapping `[range.start, range.end)`.
pub fn prettify_css_range(
    input: &str,
    range: std::ops::Range<usize>,
    config: &PrinterConfig,
) -> Option<String> {
    let ast = CssParser::stylesheet().parse(input)?;
    let doc = crate::range_to_doc(&ast, input, range);
    Some(render(doc, config.to_printer()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prettify_simple_rule() {
        let config = PrinterConfig::default();
        let input = "body { color: red; }";
        let result = prettify_css(input, &config);
        assert!(result.is_some(), "should parse simple CSS rule");
        let output = result.unwrap();
        assert!(output.contains("body"), "should contain selector");
        assert!(output.contains("color"), "should contain property");
        assert!(output.contains("red"), "should contain value");
    }

    #[test]
    fn test_prettify_multi_declaration() {
        let config = PrinterConfig::default();
        let input = "h1 { font-size: 24px; color: blue; margin: 0; }";
        let result = prettify_css(input, &config);
        assert!(result.is_some(), "should parse multi-declaration rule");
        let output = result.unwrap();
        assert!(output.contains("font-size"), "should contain font-size");
        assert!(output.contains("color"), "should contain color");
        assert!(output.contains("margin"), "should contain margin");
    }

    #[test]
    fn test_prettify_media_query() {
        let config = PrinterConfig::default();
        let input = "@media (max-width: 768px) { .sidebar { display: none; } }";
        let result = prettify_css(input, &config);
        assert!(result.is_some(), "should parse @media rule");
        let output = result.unwrap();
        assert!(output.contains("@media"), "should contain @media");
        assert!(output.contains("sidebar"), "should contain nested selector");
    }

    #[test]
    fn test_prettify_multi_rule() {
        let config = PrinterConfig::default();
        let input = "h1 { color: red; }\np { margin: 0; }";
        let result = prettify_css(input, &config);
        assert!(result.is_some(), "should parse multiple rules");
        let output = result.unwrap();
        assert!(output.contains("h1"), "should contain first selector");
        assert!(output.contains("margin"), "should contain second rule prop");
    }

    #[test]
    fn test_prettify_css_idempotent() {
        let config = PrinterConfig::default();
        let input = "body { color: red; font-size: 16px; }";
        let first = prettify_css(input, &config).unwrap();
        let second = prettify_css(&first, &config).unwrap();
        assert_eq!(first.trim(), second.trim(), "prettify should be idempotent");
    }

    #[test]
    fn test_prettify_minified_css() {
        let config = PrinterConfig::default();
        let input = "html{line-height:1.15;-webkit-text-size-adjust:100%}body{margin:0}";
        let result = prettify_css(input, &config);
        assert!(result.is_some(), "should parse minified CSS");
        let output = result.unwrap();
        assert!(output.contains("html"), "should contain html selector");
        assert!(output.contains("line-height"), "should contain line-height");
        assert!(output.contains("-webkit-text-size-adjust"), "should contain vendor prefix");
        assert!(output.contains("body"), "should contain body selector");
    }

    #[test]
    fn test_css_formatting_output() {
        let config = PrinterConfig::default();
        // Spaced CSS should produce properly formatted output
        let input = "html { line-height: 1.15; } body { margin: 0 }";
        let result = prettify_css(input, &config).unwrap();
        // Rules should be separated by blank lines
        assert!(result.contains("\n\n"), "top-level rules should have blank line separation");
        // Declarations should be indented
        assert!(result.contains("    line-height"), "declarations should be indented");
    }

    #[test]
    fn test_prettify_no_trailing_semicolon() {
        let config = PrinterConfig::default();
        let input = "body{color:red}";
        let result = prettify_css(input, &config);
        assert!(result.is_some(), "should parse CSS without trailing semicolon");
        let output = result.unwrap();
        assert!(output.contains("color"), "should contain property");
        assert!(output.contains("red"), "should contain value");
    }

    #[test]
    fn test_prettify_multi_selector() {
        let config = PrinterConfig::default();
        let input = "h1, h2, h3 { color: red; }";
        let result = prettify_css(input, &config).unwrap();
        // Multi-selector rules should contain all selectors
        assert!(result.contains("h1"), "should contain h1");
        assert!(result.contains("h2"), "should contain h2");
        assert!(result.contains("h3"), "should contain h3");
        // Should be idempotent
        let second = prettify_css(&result, &config).unwrap();
        assert_eq!(result.trim(), second.trim(), "multi-selector prettify should be idempotent");
    }

    #[test]
    fn test_prettify_selector_with_pseudo_class() {
        let config = PrinterConfig::default();
        let input = ":is(.a, .b), .c { color: red; }";
        let result = prettify_css(input, &config).unwrap();
        // Commas inside :is() should NOT split the selector
        assert!(result.contains(":is(.a, .b)"), "should preserve :is() pseudo-class");
        assert!(result.contains(".c"), "should contain .c selector");
        // Should be idempotent
        let second = prettify_css(&result, &config).unwrap();
        assert_eq!(result.trim(), second.trim(), "pseudo-class selector prettify should be idempotent");
    }
}
