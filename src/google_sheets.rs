use bbnf_derive::Parser;
use pprint::pprint as render;

use crate::PrinterConfig;

#[derive(Parser)]
#[parser(
    path = "grammar/lang/google-sheets.bbnf",
    prettify
)]
pub struct GoogleSheetsParser;

/// Parse a Google Sheets formula. Returns the AST or None on failure.
pub fn parse_formula(input: &str) -> Option<GoogleSheetsParserEnum<'_>> {
    GoogleSheetsParser::formula().parse(input)
}

/// Parse and pretty-print a Google Sheets formula.
pub fn prettify_formula(input: &str, config: &PrinterConfig) -> Option<String> {
    let ast = GoogleSheetsParser::formula().parse(input)?;
    Some(render(ast.to_doc(), config.to_printer()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        assert!(parse_formula("=SUM(A1:A10)").is_some());
    }

    #[test]
    fn test_parse_if() {
        assert!(parse_formula("=IF(A1>0, 1, 0)").is_some());
    }

    #[test]
    fn test_parse_let() {
        let input = r#"=LET(data, A1:Z100, filtered, FILTER(data, INDEX(data,,1)>0), count, ROWS(filtered), IF(count>0, MAKEARRAY(count, 3, LAMBDA(r, c, INDEX(filtered, r, c))), "No data"))"#;
        assert!(parse_formula(input).is_some(), "LET formula should parse");
    }

    #[test]
    fn test_parse_pathological() {
        let input = r#"=LET(raw, A2:E1000, filtered, FILTER(raw, (INDEX(raw,,3)>100)*(INDEX(raw,,5)="Active")), sorted, SORT(filtered, 3, FALSE), IF(ROWS(sorted)>0, MAP(SEQUENCE(MIN(10, ROWS(sorted))), LAMBDA(i, INDEX(sorted, i, 1)&" - "&TEXT(INDEX(sorted, i, 3), "$#,##0"))), "No results"))"#;
        assert!(parse_formula(input).is_some(), "pathological should parse");
    }

    #[test]
    fn test_trailing_space_formatting() {
        let config = PrinterConfig::new(80, 2);
        let without_space = r#"=LET(raw, A2:E1000, filtered, FILTER(raw, (INDEX(raw,,3)>100)*(INDEX(raw,,5)="Active")), sorted, SORT(filtered, 3, FALSE), IF(ROWS(sorted)>0, MAP(SEQUENCE(MIN(10, ROWS(sorted))), LAMBDA(i, INDEX(sorted, i, 1)&" - "&TEXT(INDEX(sorted, i, 3), "$#,##0"))), "No results"))"#;
        let with_space = r#"=LET(raw, A2:E1000, filtered, FILTER(raw, (INDEX(raw,,3)>100)*(INDEX(raw,,5)="Active")), sorted, SORT(filtered, 3, FALSE), IF(ROWS(sorted)>0, MAP(SEQUENCE(MIN(10, ROWS(sorted))), LAMBDA(i, INDEX(sorted, i, 1)&" - "&TEXT(INDEX(sorted, i, 3), "$#,##0"))), "No results") )"#;
        let fmt_without = prettify_formula(without_space, &config).unwrap();
        let fmt_with = prettify_formula(with_space, &config).unwrap();
        eprintln!("AOT without space:\n{}", fmt_without);
        eprintln!("AOT with space:\n{}", fmt_with);
        assert_eq!(fmt_without, fmt_with, "trailing space should not change AOT formatting");
    }
}
