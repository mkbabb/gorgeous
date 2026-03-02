use bbnf_derive::Parser;
use pprint::pprint as render;

use crate::{PrinterConfig, SourceRange, ToDoc};

#[derive(Parser)]
#[parser(
    path = "grammar/lang/bnf.bbnf",
    prettify
)]
pub struct BnfParser;

impl<'a> ToDoc<'a> for BnfParserEnum<'a> {
    fn to_doc(&self) -> pprint::Doc<'a> {
        BnfParserEnum::to_doc(self)
    }
}

impl<'a> SourceRange for BnfParserEnum<'a> {
    fn source_range(&self) -> Option<(usize, usize)> {
        BnfParserEnum::source_range(self)
    }
}

/// Pretty-print a BNF grammar string.
pub fn prettify_bnf(input: &str, config: &PrinterConfig) -> Option<String> {
    let ast = BnfParser::grammar().parse(input)?;
    let doc = ast.to_doc();
    Some(render(doc, Some(config.to_printer())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prettify_simple_bnf() {
        let config = PrinterConfig::default();
        let input = "<expr> ::= <term>\n";
        let result = prettify_bnf(input, &config);
        assert!(result.is_some(), "should parse simple BNF rule");
        let output = result.unwrap();
        assert!(output.contains("expr"), "should contain rule name: got '{}'", output);
        assert!(output.contains("::="), "should contain definition: got '{}'", output);
        assert!(output.contains("term"), "should contain rhs: got '{}'", output);
    }

    #[test]
    fn test_prettify_bnf_alternation() {
        let config = PrinterConfig::default();
        let input = "<expr> ::= <term> | <factor>\n";
        let result = prettify_bnf(input, &config);
        assert!(result.is_some(), "should parse BNF alternation");
        let output = result.unwrap();
        assert!(output.contains("term"), "should contain first alt");
        assert!(output.contains("factor"), "should contain second alt");
    }

    #[test]
    fn test_prettify_bnf_multi_rule() {
        let config = PrinterConfig::default();
        let input = "<expr> ::= <term>\n<term> ::= <factor>\n";
        let result = prettify_bnf(input, &config);
        assert!(result.is_some(), "should parse multiple BNF rules");
        let output = result.unwrap();
        assert!(output.contains("expr"), "should contain first rule");
        assert!(output.contains("factor"), "should contain second rule rhs");
    }

    #[test]
    fn test_prettify_bnf_terminal() {
        let config = PrinterConfig::default();
        let input = "<digit> ::= \"0\" | \"1\" | \"2\"\n";
        let result = prettify_bnf(input, &config);
        assert!(result.is_some(), "should parse BNF with terminals");
        let output = result.unwrap();
        assert!(output.contains("digit"), "should contain rule name");
    }

    #[test]
    fn test_prettify_bnf_idempotent() {
        let config = PrinterConfig::default();
        let input = "<expr> ::= <term> | <factor>\n";
        let first = prettify_bnf(input, &config).unwrap();
        let second = prettify_bnf(&first, &config).unwrap();
        assert_eq!(first, second, "prettify should be idempotent");
    }
}
