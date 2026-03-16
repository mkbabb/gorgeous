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

    /// Multiline LET with column-only ranges, nested LAMBDA/LET, and equality comparisons.
    #[test]
    fn test_multiline_let_with_column_ranges() {
        let input = r#"=LET(
  scale, DURATION,

  psus, FILTER(B3:B, B3:B <> ""),
  providers, H2:O2,
  recurring, FILTER(H3:O, B3:B <> ""),

  normalize, LAMBDA(x, LOWER(TRIM(TO_TEXT(x)))),

  sheet1Psus, ARRAYFORMULA(Sheet1!C2:C),
  sheet1Providers, ARRAYFORMULA(Sheet1!B2:B),
  oneTimeCosts, Sheet1!AD2:AD,

  values,
    MAKEARRAY(
      ROWS(psus),
      COLUMNS(providers),
      LAMBDA(r, c,
        LET(
          psu, INDEX(psus, r),
          provider, INDEX(providers, c),
          monthlyValue, N(INDEX(recurring, r, c)),
          oneTimeCost,
            IFERROR(
              INDEX(
                oneTimeCosts,
                MATCH(
                  1,
                  (sheet1Psus = psu) * (sheet1Providers = provider),
                  0
                )
              ),
              0
            ),
          IF(monthlyValue > 0, monthlyValue * scale + N(oneTimeCost), 0)
        )
      )
    ),

  VSTACK(providers, values)
)"#;
        let ast = parse_formula(input);
        assert!(ast.is_some(), "multiline LET with column ranges should parse (AOT)");

        let config = PrinterConfig::new(80, 2);
        let formatted = prettify_formula(input, &config).unwrap();
        eprintln!("AOT multiline:\n{}", formatted);
        assert!(formatted.contains("LET"), "AOT formatted should contain LET");
        assert!(formatted.contains('\n'), "AOT formatted should have line breaks");

        // Without leading =
        let no_eq = &input[1..];
        assert!(parse_formula(no_eq).is_some(), "formula without = should parse (AOT)");
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
