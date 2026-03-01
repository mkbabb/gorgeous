use bbnf_derive::Parser;
use pprint::pprint as render;

use crate::{PrinterConfig, SourceRange, ToDoc};

#[derive(Parser)]
#[parser(
    path = "../bbnf-lang/grammar/lang/ebnf.bbnf",
    prettify
)]
pub struct EbnfParser;

impl<'a> ToDoc<'a> for EbnfParserEnum<'a> {
    fn to_doc(&self) -> pprint::Doc<'a> {
        EbnfParserEnum::to_doc(self)
    }
}

impl<'a> SourceRange for EbnfParserEnum<'a> {
    fn source_range(&self) -> Option<(usize, usize)> {
        EbnfParserEnum::source_range(self)
    }
}

/// Pretty-print an EBNF grammar string.
pub fn prettify_ebnf(input: &str, config: &PrinterConfig) -> Option<String> {
    let ast = EbnfParser::grammar().parse(input)?;
    let doc = ast.to_doc();
    Some(render(doc, Some(config.to_printer())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prettify_simple_ebnf() {
        let config = PrinterConfig::default();
        let input = r#"letter = "A" | "B" | "C" ;"#;
        let result = prettify_ebnf(input, &config);
        assert!(result.is_some(), "should parse simple EBNF rule");
        let output = result.unwrap();
        assert!(output.contains("letter"), "should contain rule name: got '{}'", output);
        assert!(output.contains("="), "should contain assignment: got '{}'", output);
    }

    #[test]
    fn test_prettify_repetition() {
        let config = PrinterConfig::default();
        let input = r#"identifier = letter , { letter | digit | "_" } ;"#;
        let result = prettify_ebnf(input, &config);
        assert!(result.is_some(), "should parse repetition rule");
    }

    #[test]
    fn test_prettify_multi_rule() {
        let config = PrinterConfig::default();
        let input = r#"digit = "0" | "1" | "2" ;
number = digit , { digit } ;"#;
        let result = prettify_ebnf(input, &config);
        assert!(result.is_some(), "should parse multiple rules");
        let output = result.unwrap();
        assert!(output.contains("digit"), "should contain first rule");
        assert!(output.contains("number"), "should contain second rule");
    }

    #[test]
    fn test_prettify_idempotent() {
        let config = PrinterConfig::default();
        let input = r#"letter = "A" | "B" | "C" ;"#;
        let first = prettify_ebnf(input, &config).unwrap();
        let second = prettify_ebnf(&first, &config).unwrap();
        assert_eq!(first, second, "prettify should be idempotent");
    }
}
