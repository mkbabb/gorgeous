use bbnf_derive::Parser;

use crate::{PrinterConfig, SourceRange, ToDoc};

#[derive(Parser)]
#[parser(path = "grammar/css/css-fast.bbnf", prettify, skip_recover)]
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

/// Parse-only CSS using the fast (opaque-span) grammar.
pub fn prettify_css_fast(input: &str, config: &PrinterConfig) -> Option<String> {
    let ast = CssFastParser::stylesheet().parse(input)?;
    let doc = ast.to_doc();
    Some(pprint::pprint(doc, config.to_printer()))
}
