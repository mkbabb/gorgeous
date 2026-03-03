use bbnf_derive::Parser;
use pprint::pprint as render;

use crate::{PrinterConfig, SourceRange, ToDoc};

#[derive(Parser)]
#[parser(
    path = "grammar/css/css-fast.bbnf",
    prettify
)]
pub struct CssFastParser;

impl<'a> ToDoc<'a> for CssFastParserEnum<'a> {
    fn to_doc(&self) -> pprint::Doc<'a> {
        CssFastParserEnum::to_doc(self)
    }
}

impl<'a> SourceRange for CssFastParserEnum<'a> {
    fn source_range(&self) -> Option<(usize, usize)> {
        CssFastParserEnum::source_range(self)
    }
}

/// Pretty-print a CSS stylesheet using the fast grammar.
pub fn prettify_css_fast(input: &str, config: &PrinterConfig) -> Option<String> {
    let ast = CssFastParser::stylesheet().parse(input)?;
    let doc = ast.to_doc();
    Some(render(doc, Some(config.to_printer())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fast_simple_rule() {
        let config = PrinterConfig::default();
        let input = "body { color: red; }";
        let result = prettify_css_fast(input, &config);
        assert!(result.is_some(), "should parse simple CSS rule");
        let output = result.unwrap();
        assert!(output.contains("body"), "should contain selector");
        assert!(output.contains("color"), "should contain property");
        assert!(output.contains("red"), "should contain value");
    }

    #[test]
    fn test_fast_multi_declaration() {
        let config = PrinterConfig::default();
        let input = "h1 { font-size: 24px; color: blue; margin: 0; }";
        let result = prettify_css_fast(input, &config);
        assert!(result.is_some(), "should parse multi-declaration rule");
        let output = result.unwrap();
        assert!(output.contains("font-size"), "should contain font-size");
        assert!(output.contains("color"), "should contain color");
        assert!(output.contains("margin"), "should contain margin");
    }

    #[test]
    fn test_fast_media_query() {
        let config = PrinterConfig::default();
        let input = "@media (max-width: 768px) { .sidebar { display: none; } }";
        let result = prettify_css_fast(input, &config);
        assert!(result.is_some(), "should parse @media rule");
        let output = result.unwrap();
        assert!(output.contains("@media"), "should contain @media");
        assert!(output.contains("sidebar"), "should contain nested selector");
    }

    #[test]
    fn test_fast_minified_css() {
        let config = PrinterConfig::default();
        let input = "html{line-height:1.15;-webkit-text-size-adjust:100%}body{margin:0}";
        let result = prettify_css_fast(input, &config);
        assert!(result.is_some(), "should parse minified CSS");
        let output = result.unwrap();
        assert!(output.contains("html"), "should contain html selector");
        assert!(output.contains("body"), "should contain body selector");
    }

    #[test]
    fn test_fast_idempotent() {
        let config = PrinterConfig::default();
        let input = "body { color: red; font-size: 16px; }";
        let first = prettify_css_fast(input, &config).unwrap();
        let second = prettify_css_fast(&first, &config).unwrap();
        assert_eq!(first.trim(), second.trim(), "prettify should be idempotent");
    }

    #[test]
    fn test_fast_no_trailing_semicolon() {
        let config = PrinterConfig::default();
        let input = "body{color:red}";
        let result = prettify_css_fast(input, &config);
        assert!(result.is_some(), "should parse CSS without trailing semicolon");
        let output = result.unwrap();
        assert!(output.contains("color"), "should contain property");
        assert!(output.contains("red"), "should contain value");
    }
}
