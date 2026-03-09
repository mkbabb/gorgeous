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

/// Split a CSS selector string on commas, respecting parentheses and brackets.
/// Handles `:is(.a, .b)`, `:not([href])`, `[attr="x,y"]`, and nested parens.
fn split_selectors(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0u32;
    let mut start = 0;
    for (i, b) in text.bytes().enumerate() {
        match b {
            b'(' | b'[' => depth += 1,
            b')' | b']' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                result.push(text[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(text[start..].trim());
    result
}

impl<'a> ToDoc<'a> for CssParserEnum<'a> {
    fn to_doc(&self) -> pprint::Doc<'a> {
        match self {
            // Override: selector lists use ",\n" when breaking, ", " when inline.
            CssParserEnum::qualifiedRule((selector, block)) => {
                let selector_doc = if let CssParserEnum::selectorSpan(span) = selector.as_ref() {
                    let text = span.as_str();
                    let selectors = split_selectors(text);
                    if selectors.len() > 1 {
                        let break_sep = pprint::Doc::IfBreak(
                            Box::new(
                                pprint::Doc::Char(b',')
                                    + pprint::Doc::Hardline,
                            ),
                            Box::new(pprint::Doc::String(std::borrow::Cow::Borrowed(", "))),
                        );
                        let mut body = pprint::Doc::String(
                            std::borrow::Cow::Borrowed(selectors[0]),
                        );
                        for sel in &selectors[1..] {
                            body = body
                                + break_sep.clone()
                                + pprint::Doc::String(std::borrow::Cow::Borrowed(sel));
                        }
                        pprint::Doc::Group(Box::new(body))
                    } else {
                        selector.to_doc()
                    }
                } else {
                    selector.to_doc()
                };
                selector_doc
                    + pprint::Doc::String(std::borrow::Cow::Borrowed(" "))
                    + block.to_doc()
            }
            _ => CssParserEnum::to_doc(self),
        }
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
}
