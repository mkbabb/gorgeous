#![feature(cold_path)]

use bencher::{benchmark_group, benchmark_main, Bencher};
use pprint::{pprint as render, pprint_ref};
use gorgeous::json::prettify_json;
use gorgeous::css::{prettify_css, CssParser};
use gorgeous::css_fast::CssFastParser;
use gorgeous::google_sheets::{prettify_formula, GoogleSheetsParser};
use gorgeous::{PrinterConfig, ToDoc};

// ── Data loaders ─────────────────────────────────────────────────────────────

fn load_json_data() -> String {
    std::fs::read_to_string("data/json/data.json").expect("data.json not found")
}

fn load_json_canada() -> String {
    std::fs::read_to_string("data/json/canada.json").expect("canada.json not found")
}

fn load_css_normalize() -> String {
    std::fs::read_to_string("data/css/normalize.css").expect("normalize.css not found")
}

fn load_css_app() -> String {
    std::fs::read_to_string("data/css/app.css").expect("app.css not found")
}

fn load_css_bootstrap() -> String {
    std::fs::read_to_string("data/css/bootstrap.css").expect("bootstrap.css not found")
}

fn load_css_tailwind() -> String {
    std::fs::read_to_string("data/css/tailwind-output.css").expect("tailwind-output.css not found")
}

// ── JSON benchmarks ──────────────────────────────────────────────────────────

fn bench_json_small_object(b: &mut Bencher) {
    let input = r#"{"name": "Alice", "age": 30, "active": true}"#;
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_json(input, &config).unwrap();
    });
}

fn bench_json_data_end_to_end(b: &mut Bencher) {
    let input = load_json_data();
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_json(&input, &config).unwrap();
    });
}

fn bench_json_canada_end_to_end(b: &mut Bencher) {
    let input = load_json_canada();
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_json(&input, &config).unwrap();
    });
}

// ── CSS benchmarks ───────────────────────────────────────────────────────────

fn bench_css_small_rule(b: &mut Bencher) {
    let input = "body { color: red; font-size: 16px; margin: 0; padding: 0; }";
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_css(input, &config).unwrap();
    });
}

fn bench_css_normalize(b: &mut Bencher) {
    let input = load_css_normalize();
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_css(&input, &config).unwrap();
    });
}

fn bench_css_app(b: &mut Bencher) {
    let input = load_css_app();
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_css(&input, &config).unwrap();
    });
}

fn bench_css_bootstrap(b: &mut Bencher) {
    let input = load_css_bootstrap();
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_css(&input, &config).unwrap();
    });
}

// ── CSS cached benchmarks (parser built once, reused) ───────────────────────

fn bench_css_small_rule_cached(b: &mut Bencher) {
    let input = "body { color: red; font-size: 16px; margin: 0; padding: 0; }";
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

fn bench_css_normalize_cached(b: &mut Bencher) {
    let input = load_css_normalize();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

fn bench_css_app_cached(b: &mut Bencher) {
    let input = load_css_app();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

fn bench_css_bootstrap_cached(b: &mut Bencher) {
    let input = load_css_bootstrap();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

fn bench_css_bootstrap_parse_only(b: &mut Bencher) {
    let input = load_css_bootstrap();
    let parser = CssParser::stylesheet();
    // Report consumption — grammar uses L1 opaque spans, so throughput
    // reflects span-scanning speed, not full structured parsing.
    let (result, state) = parser.parse_return_state(&input);
    assert!(result.is_some(), "CSS bootstrap parse failed");
    let pct = state.offset * 100 / input.len().max(1);
    if pct < 95 {
        eprintln!(
            "  NOTE: bootstrap parse consumed {pct}% \
             (L1 opaque spans — throughput is span-scan speed)"
        );
    }
    b.bytes = input.len() as u64;
    b.iter(|| {
        parser.parse(&input).unwrap()
    });
}

fn bench_css_bootstrap_to_doc_only(b: &mut Bencher) {
    let input = load_css_bootstrap();
    let parser = CssParser::stylesheet();
    let ast = parser.parse(&input).unwrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        ast.to_doc()
    });
}

fn bench_css_bootstrap_render_only(b: &mut Bencher) {
    let input = load_css_bootstrap();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    let ast = parser.parse(&input).unwrap();
    let doc = ast.to_doc();
    b.bytes = input.len() as u64;
    b.iter(|| {
        pprint_ref(&doc, config.to_printer())
    });
}

// ── JSON cached benchmarks ──────────────────────────────────────────────────

fn bench_json_small_cached(b: &mut Bencher) {
    use gorgeous::json::JsonParser;
    let input = r#"{"name": "Alice", "age": 30, "active": true}"#;
    let config = PrinterConfig::default();
    let parser = JsonParser::value();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

fn bench_json_data_cached(b: &mut Bencher) {
    use gorgeous::json::JsonParser;
    let input = load_json_data();
    let config = PrinterConfig::default();
    let parser = JsonParser::value();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

fn bench_json_canada_cached(b: &mut Bencher) {
    use gorgeous::json::JsonParser;
    let input = load_json_canada();
    let config = PrinterConfig::default();
    let parser = JsonParser::value();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

// ── CSS phase-split benchmarks (parse vs to_doc vs render) ──────────────────

fn bench_css_normalize_parse_only(b: &mut Bencher) {
    let input = load_css_normalize();
    let parser = CssParser::stylesheet();
    let (result, state) = parser.parse_return_state(&input);
    assert!(result.is_some(), "CSS normalize parse failed");
    let pct = state.offset * 100 / input.len().max(1);
    if pct < 95 {
        eprintln!(
            "  NOTE: normalize parse consumed {pct}% \
             (L1 opaque spans — throughput is span-scan speed)"
        );
    }
    b.bytes = input.len() as u64;
    b.iter(|| {
        parser.parse(&input).unwrap()
    });
}

fn bench_css_normalize_to_doc_only(b: &mut Bencher) {
    let input = load_css_normalize();
    let parser = CssParser::stylesheet();
    let ast = parser.parse(&input).unwrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        ast.to_doc()
    });
}

fn bench_css_normalize_render_only(b: &mut Bencher) {
    let input = load_css_normalize();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    let ast = parser.parse(&input).unwrap();
    let doc = ast.to_doc();
    b.bytes = input.len() as u64;
    b.iter(|| {
        pprint_ref(&doc, config.to_printer())
    });
}

fn bench_css_app_parse_only(b: &mut Bencher) {
    let input = load_css_app();
    let parser = CssParser::stylesheet();
    let (result, state) = parser.parse_return_state(&input);
    assert!(result.is_some(), "CSS app parse failed");
    let pct = state.offset * 100 / input.len().max(1);
    if pct < 95 {
        eprintln!(
            "  NOTE: app parse consumed {pct}% \
             (L1 opaque spans — throughput is span-scan speed)"
        );
    }
    b.bytes = input.len() as u64;
    b.iter(|| {
        parser.parse(&input).unwrap()
    });
}

fn bench_css_app_to_doc_only(b: &mut Bencher) {
    let input = load_css_app();
    let parser = CssParser::stylesheet();
    let ast = parser.parse(&input).unwrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        ast.to_doc()
    });
}

fn bench_css_app_render_only(b: &mut Bencher) {
    let input = load_css_app();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    let ast = parser.parse(&input).unwrap();
    let doc = ast.to_doc();
    b.bytes = input.len() as u64;
    b.iter(|| {
        pprint_ref(&doc, config.to_printer())
    });
}

// ── JSON phase-split benchmarks (parse vs to_doc vs render) ──────────────────

fn bench_json_data_parse_only(b: &mut Bencher) {
    use gorgeous::json::JsonParser;
    let input = load_json_data();
    let parser = JsonParser::value();
    let (result, state) = parser.parse_return_state(&input);
    assert!(result.is_some(), "JSON data parse failed");
    let pct = state.offset * 100 / input.len().max(1);
    assert!(pct >= 95, "only consumed {pct}% — grammar is incomplete");
    b.bytes = input.len() as u64;
    b.iter(|| {
        parser.parse(&input).unwrap()
    });
}

fn bench_json_data_to_doc_only(b: &mut Bencher) {
    use gorgeous::json::JsonParser;
    let input = load_json_data();
    let parser = JsonParser::value();
    let ast = parser.parse(&input).unwrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        ast.to_doc()
    });
}

fn bench_json_data_render_only(b: &mut Bencher) {
    use gorgeous::json::JsonParser;
    let input = load_json_data();
    let parser = JsonParser::value();
    let ast = parser.parse(&input).unwrap();
    let doc = ast.to_doc();
    b.bytes = input.len() as u64;
    b.iter(|| {
        pprint_ref(&doc, PrinterConfig::default().to_printer())
    });
}

fn bench_json_canada_parse_only(b: &mut Bencher) {
    use gorgeous::json::JsonParser;
    let input = load_json_canada();
    let parser = JsonParser::value();
    let (result, state) = parser.parse_return_state(&input);
    assert!(result.is_some(), "JSON canada parse failed");
    let pct = state.offset * 100 / input.len().max(1);
    assert!(pct >= 95, "only consumed {pct}% — grammar is incomplete");
    b.bytes = input.len() as u64;
    b.iter(|| {
        parser.parse(&input).unwrap()
    });
}

fn bench_json_canada_to_doc_only(b: &mut Bencher) {
    use gorgeous::json::JsonParser;
    let input = load_json_canada();
    let parser = JsonParser::value();
    let ast = parser.parse(&input).unwrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        ast.to_doc()
    });
}

fn bench_json_canada_render_only(b: &mut Bencher) {
    use gorgeous::json::JsonParser;
    let input = load_json_canada();
    let config = PrinterConfig::default();
    let parser = JsonParser::value();
    let ast = parser.parse(&input).unwrap();
    let doc = ast.to_doc();
    b.bytes = input.len() as u64;
    b.iter(|| {
        pprint_ref(&doc, config.to_printer())
    });
}

// ── Tailwind benchmarks ─────────────────────────────────────────────────────

fn bench_css_tailwind(b: &mut Bencher) {
    let input = load_css_tailwind();
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_css(&input, &config).unwrap();
    });
}

fn bench_css_tailwind_cached(b: &mut Bencher) {
    let input = load_css_tailwind();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

fn bench_css_tailwind_parse_only(b: &mut Bencher) {
    let input = load_css_tailwind();
    let parser = CssParser::stylesheet();
    let (result, state) = parser.parse_return_state(&input);
    assert!(result.is_some(), "CSS tailwind parse failed");
    let pct = state.offset * 100 / input.len().max(1);
    if pct < 95 {
        eprintln!(
            "  NOTE: tailwind parse consumed {pct}% \
             (L1 opaque spans — throughput is span-scan speed)"
        );
    }
    b.bytes = input.len() as u64;
    b.iter(|| {
        parser.parse(&input).unwrap()
    });
}

fn bench_css_tailwind_to_doc_only(b: &mut Bencher) {
    let input = load_css_tailwind();
    let parser = CssParser::stylesheet();
    let ast = parser.parse(&input).unwrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        ast.to_doc()
    });
}

fn bench_css_tailwind_render_only(b: &mut Bencher) {
    let input = load_css_tailwind();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    let ast = parser.parse(&input).unwrap();
    let doc = ast.to_doc();
    b.bytes = input.len() as u64;
    b.iter(|| {
        pprint_ref(&doc, config.to_printer())
    });
}

// ── Biome competitor benchmarks ─────────────────────────────────────────────

fn biome_format_css(input: &str) -> String {
    use biome_css_parser::{CssParserOptions, parse_css};
    use biome_css_formatter::{context::CssFormatOptions, format_node};

    let parsed = parse_css(input, CssParserOptions::default());
    let options = CssFormatOptions::default();
    let formatted = format_node(options, &parsed.syntax()).unwrap();
    formatted.print().unwrap().into_code()
}

fn bench_biome_css_bootstrap(b: &mut Bencher) {
    let input = load_css_bootstrap();
    b.bytes = input.len() as u64;
    // Warmup to verify it works.
    let _ = biome_format_css(&input);
    b.iter(|| {
        biome_format_css(&input)
    });
}

fn bench_biome_css_tailwind(b: &mut Bencher) {
    let input = load_css_tailwind();
    b.bytes = input.len() as u64;
    let _ = biome_format_css(&input);
    b.iter(|| {
        biome_format_css(&input)
    });
}

fn bench_biome_css_app(b: &mut Bencher) {
    let input = load_css_app();
    b.bytes = input.len() as u64;
    let _ = biome_format_css(&input);
    b.iter(|| {
        biome_format_css(&input)
    });
}

// ── Google Sheets formulas ────────────────────────────────────────────────────

const GS_SIMPLE: &str = "=SUM(A1:A10)";

const GS_LET: &str = r#"=LET(data, A1:Z100, filtered, FILTER(data, INDEX(data,,1)>0), count, ROWS(filtered), IF(count>0, MAKEARRAY(count, 3, LAMBDA(r, c, INDEX(filtered, r, c))), "No data"))"#;

const GS_PATHOLOGICAL: &str = r#"=LET(raw, A2:E1000, filtered, FILTER(raw, (INDEX(raw,,3)>100)*(INDEX(raw,,5)="Active")), sorted, SORT(filtered, 3, FALSE), IF(ROWS(sorted)>0, MAP(SEQUENCE(MIN(10, ROWS(sorted))), LAMBDA(i, INDEX(sorted, i, 1)&" - "&TEXT(INDEX(sorted, i, 3), "$#,##0"))), "No results"))"#;

/// Generate a large formula by repeating LET bindings: =LET(a0, SUM(A1:A10), a1, ..., body)
fn generate_large_formula(n_bindings: usize) -> String {
    let mut parts = Vec::with_capacity(n_bindings * 2 + 1);
    for i in 0..n_bindings {
        parts.push(format!("v{}", i));
        parts.push(format!("IF(v{}>0, FILTER(A1:Z100, INDEX(A1:Z100,,{})>0), SUM(A1:A{}))", i, i + 1, i + 10));
    }
    parts.push(format!("v{}", n_bindings - 1));
    format!("=LET({})", parts.join(", "))
}

// ── Google Sheets end-to-end benchmarks ─────────────────────────────────────

fn bench_gs_simple(b: &mut Bencher) {
    let config = PrinterConfig::default();
    b.bytes = GS_SIMPLE.len() as u64;
    b.iter(|| {
        prettify_formula(GS_SIMPLE, &config).unwrap();
    });
}

fn bench_gs_let(b: &mut Bencher) {
    let config = PrinterConfig::default();
    b.bytes = GS_LET.len() as u64;
    b.iter(|| {
        prettify_formula(GS_LET, &config).unwrap();
    });
}

fn bench_gs_pathological(b: &mut Bencher) {
    let config = PrinterConfig::default();
    b.bytes = GS_PATHOLOGICAL.len() as u64;
    b.iter(|| {
        prettify_formula(GS_PATHOLOGICAL, &config).unwrap();
    });
}

fn bench_gs_1kb(b: &mut Bencher) {
    let input = generate_large_formula(10);
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_formula(&input, &config).unwrap();
    });
}

fn bench_gs_10kb(b: &mut Bencher) {
    let input = generate_large_formula(100);
    let config = PrinterConfig::default();
    b.bytes = input.len() as u64;
    b.iter(|| {
        prettify_formula(&input, &config).unwrap();
    });
}

// ── Google Sheets cached benchmarks ─────────────────────────────────────────

fn bench_gs_pathological_cached(b: &mut Bencher) {
    let config = PrinterConfig::default();
    let parser = GoogleSheetsParser::formula();
    b.bytes = GS_PATHOLOGICAL.len() as u64;
    b.iter(|| {
        let ast = parser.parse(GS_PATHOLOGICAL).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

fn bench_gs_1kb_cached(b: &mut Bencher) {
    let input = generate_large_formula(10);
    let config = PrinterConfig::default();
    let parser = GoogleSheetsParser::formula();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

fn bench_gs_10kb_cached(b: &mut Bencher) {
    let input = generate_large_formula(100);
    let config = PrinterConfig::default();
    let parser = GoogleSheetsParser::formula();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), config.to_printer())
    });
}

// ── Google Sheets phase-split benchmarks ────────────────────────────────────

fn bench_gs_simple_parse_only(b: &mut Bencher) {
    let parser = GoogleSheetsParser::formula();
    b.bytes = GS_SIMPLE.len() as u64;
    b.iter(|| {
        parser.parse(GS_SIMPLE).unwrap()
    });
}

fn bench_gs_let_parse_only(b: &mut Bencher) {
    let parser = GoogleSheetsParser::formula();
    b.bytes = GS_LET.len() as u64;
    b.iter(|| {
        parser.parse(GS_LET).unwrap()
    });
}

fn bench_gs_pathological_parse_only(b: &mut Bencher) {
    let parser = GoogleSheetsParser::formula();
    b.bytes = GS_PATHOLOGICAL.len() as u64;
    b.iter(|| {
        parser.parse(GS_PATHOLOGICAL).unwrap()
    });
}

fn bench_gs_pathological_to_doc_only(b: &mut Bencher) {
    let parser = GoogleSheetsParser::formula();
    let ast = parser.parse(GS_PATHOLOGICAL).unwrap();
    b.bytes = GS_PATHOLOGICAL.len() as u64;
    b.iter(|| {
        ast.to_doc()
    });
}

fn bench_gs_pathological_render_only(b: &mut Bencher) {
    let config = PrinterConfig::default();
    let parser = GoogleSheetsParser::formula();
    let ast = parser.parse(GS_PATHOLOGICAL).unwrap();
    let doc = ast.to_doc();
    b.bytes = GS_PATHOLOGICAL.len() as u64;
    b.iter(|| {
        pprint_ref(&doc, config.to_printer())
    });
}

fn bench_gs_1kb_parse_only(b: &mut Bencher) {
    let input = generate_large_formula(10);
    let parser = GoogleSheetsParser::formula();
    b.bytes = input.len() as u64;
    b.iter(|| {
        parser.parse(&input).unwrap()
    });
}

fn bench_gs_1kb_to_doc_only(b: &mut Bencher) {
    let input = generate_large_formula(10);
    let parser = GoogleSheetsParser::formula();
    let ast = parser.parse(&input).unwrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        ast.to_doc()
    });
}

fn bench_gs_1kb_render_only(b: &mut Bencher) {
    let input = generate_large_formula(10);
    let config = PrinterConfig::default();
    let parser = GoogleSheetsParser::formula();
    let ast = parser.parse(&input).unwrap();
    let doc = ast.to_doc();
    b.bytes = input.len() as u64;
    b.iter(|| {
        pprint_ref(&doc, config.to_printer())
    });
}

fn bench_gs_10kb_parse_only(b: &mut Bencher) {
    let input = generate_large_formula(100);
    let parser = GoogleSheetsParser::formula();
    b.bytes = input.len() as u64;
    b.iter(|| {
        parser.parse(&input).unwrap()
    });
}

fn bench_gs_10kb_to_doc_only(b: &mut Bencher) {
    let input = generate_large_formula(100);
    let parser = GoogleSheetsParser::formula();
    let ast = parser.parse(&input).unwrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        ast.to_doc()
    });
}

fn bench_gs_10kb_render_only(b: &mut Bencher) {
    let input = generate_large_formula(100);
    let config = PrinterConfig::default();
    let parser = GoogleSheetsParser::formula();
    let ast = parser.parse(&input).unwrap();
    let doc = ast.to_doc();
    b.bytes = input.len() as u64;
    b.iter(|| {
        pprint_ref(&doc, config.to_printer())
    });
}

// ── CSS parse-only comparison (BBNF fast vs cssparser vs BBNF pretty vs lightningcss) ──

// --- Tier 1: Structural scan (opaque spans / tokenizer) ---

// cssparser: minimal visitor that counts rules and declarations (L0–L1 work)
mod cssparser_visitor {
    use cssparser::{
        AtRuleParser, CowRcStr, DeclarationParser, ParseError, Parser, ParserInput,
        QualifiedRuleParser, RuleBodyItemParser, StyleSheetParser,
    };

    pub struct RuleCounter {
        pub rule_count: usize,
        pub decl_count: usize,
    }

    impl<'i> QualifiedRuleParser<'i> for RuleCounter {
        type Prelude = ();
        type QualifiedRule = ();
        type Error = ();

        fn parse_prelude<'t>(
            &mut self,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self::Prelude, ParseError<'i, ()>> {
            while input.next().is_ok() {}
            Ok(())
        }

        fn parse_block<'t>(
            &mut self,
            _prelude: Self::Prelude,
            _start: &cssparser::ParserState,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self::QualifiedRule, ParseError<'i, ()>> {
            self.rule_count += 1;
            while input.next().is_ok() {}
            Ok(())
        }
    }

    impl<'i> AtRuleParser<'i> for RuleCounter {
        type Prelude = ();
        type AtRule = ();
        type Error = ();

        fn parse_prelude<'t>(
            &mut self,
            _name: CowRcStr<'i>,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self::Prelude, ParseError<'i, ()>> {
            while input.next().is_ok() {}
            Ok(())
        }

        fn parse_block<'t>(
            &mut self,
            _prelude: Self::Prelude,
            _start: &cssparser::ParserState,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self::AtRule, ParseError<'i, ()>> {
            self.rule_count += 1;
            while input.next().is_ok() {}
            Ok(())
        }

        fn rule_without_block(
            &mut self,
            _prelude: Self::Prelude,
            _start: &cssparser::ParserState,
        ) -> Result<Self::AtRule, ()> {
            self.rule_count += 1;
            Ok(())
        }
    }

    impl<'i> DeclarationParser<'i> for RuleCounter {
        type Declaration = ();
        type Error = ();

        fn parse_value<'t>(
            &mut self,
            _name: CowRcStr<'i>,
            input: &mut Parser<'i, 't>,
        ) -> Result<Self::Declaration, ParseError<'i, ()>> {
            self.decl_count += 1;
            while input.next().is_ok() {}
            Ok(())
        }
    }

    impl<'i> RuleBodyItemParser<'i, (), ()> for RuleCounter {
        fn parse_qualified(&self) -> bool {
            true
        }
        fn parse_declarations(&self) -> bool {
            false
        }
    }

    pub fn parse_css(data: &str) -> (usize, usize) {
        let mut input = ParserInput::new(data);
        let mut parser = Parser::new(&mut input);
        let mut counter = RuleCounter {
            rule_count: 0,
            decl_count: 0,
        };
        let rule_parser = StyleSheetParser::new(&mut parser, &mut counter);
        for result in rule_parser {
            let _ = bencher::black_box(result);
        }
        (counter.rule_count, counter.decl_count)
    }
}

// BBNF fast parse-only

fn bench_bbnf_fast_parse_normalize(b: &mut Bencher) {
    let input = load_css_normalize();
    let parser = CssFastParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| parser.parse(&input).unwrap());
}

fn bench_bbnf_fast_parse_bootstrap(b: &mut Bencher) {
    let input = load_css_bootstrap();
    let parser = CssFastParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| parser.parse(&input).unwrap());
}

fn bench_bbnf_fast_parse_tailwind(b: &mut Bencher) {
    let input = load_css_tailwind();
    let parser = CssFastParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| parser.parse(&input).unwrap());
}

// cssparser parse-only

fn bench_cssparser_parse_normalize(b: &mut Bencher) {
    let input = load_css_normalize();
    b.bytes = input.len() as u64;
    b.iter(|| bencher::black_box(cssparser_visitor::parse_css(&input)));
}

fn bench_cssparser_parse_bootstrap(b: &mut Bencher) {
    let input = load_css_bootstrap();
    b.bytes = input.len() as u64;
    b.iter(|| bencher::black_box(cssparser_visitor::parse_css(&input)));
}

fn bench_cssparser_parse_tailwind(b: &mut Bencher) {
    let input = load_css_tailwind();
    b.bytes = input.len() as u64;
    b.iter(|| bencher::black_box(cssparser_visitor::parse_css(&input)));
}

// --- Tier 2: Structural AST (BBNF pretty vs lightningcss) ---

// BBNF pretty parse-only (reuses existing CssParser, aliases for grouping)

fn bench_bbnf_pretty_parse_normalize(b: &mut Bencher) {
    let input = load_css_normalize();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| parser.parse(&input).unwrap());
}

fn bench_bbnf_pretty_parse_bootstrap(b: &mut Bencher) {
    let input = load_css_bootstrap();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| parser.parse(&input).unwrap());
}

fn bench_bbnf_pretty_parse_tailwind(b: &mut Bencher) {
    let input = load_css_tailwind();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| parser.parse(&input).unwrap());
}

// lightningcss parse-only (L2 semantic parse — typed properties, vendor analysis)

fn bench_lightningcss_parse_normalize(b: &mut Bencher) {
    use lightningcss::stylesheet::{ParserOptions, StyleSheet};
    let input = load_css_normalize();
    b.bytes = input.len() as u64;
    b.iter(|| {
        bencher::black_box(StyleSheet::parse(&input, ParserOptions::default()).unwrap())
    });
}

fn bench_lightningcss_parse_bootstrap(b: &mut Bencher) {
    use lightningcss::stylesheet::{ParserOptions, StyleSheet};
    let input = load_css_bootstrap();
    b.bytes = input.len() as u64;
    b.iter(|| {
        bencher::black_box(StyleSheet::parse(&input, ParserOptions::default()).unwrap())
    });
}

// lightningcss errors on synthetic tailwind — omitted

// ── Groups ───────────────────────────────────────────────────────────────────

benchmark_group!(
    json_benches,
    bench_json_small_object,
    bench_json_data_end_to_end,
    bench_json_canada_end_to_end,
);

benchmark_group!(
    json_cached_benches,
    bench_json_small_cached,
    bench_json_data_cached,
    bench_json_canada_cached,
);

benchmark_group!(
    css_benches,
    bench_css_small_rule,
    bench_css_normalize,
    bench_css_app,
    bench_css_bootstrap,
    bench_css_tailwind,
);

benchmark_group!(
    css_cached_benches,
    bench_css_small_rule_cached,
    bench_css_normalize_cached,
    bench_css_app_cached,
    bench_css_bootstrap_cached,
    bench_css_tailwind_cached,
);

benchmark_group!(
    css_phase_benches,
    bench_css_normalize_parse_only,
    bench_css_normalize_to_doc_only,
    bench_css_normalize_render_only,
    bench_css_app_parse_only,
    bench_css_app_to_doc_only,
    bench_css_app_render_only,
    bench_css_bootstrap_parse_only,
    bench_css_bootstrap_to_doc_only,
    bench_css_bootstrap_render_only,
    bench_css_tailwind_parse_only,
    bench_css_tailwind_to_doc_only,
    bench_css_tailwind_render_only,
);

benchmark_group!(
    biome_benches,
    bench_biome_css_app,
    bench_biome_css_bootstrap,
    bench_biome_css_tailwind,
);

benchmark_group!(
    css_parse_comparison,
    // Tier 1: structural scan (opaque spans vs tokenizer)
    bench_bbnf_fast_parse_normalize,
    bench_bbnf_fast_parse_bootstrap,
    bench_bbnf_fast_parse_tailwind,
    bench_cssparser_parse_normalize,
    bench_cssparser_parse_bootstrap,
    bench_cssparser_parse_tailwind,
    // Tier 2: structural AST (typed tree vs semantic parse)
    bench_bbnf_pretty_parse_normalize,
    bench_bbnf_pretty_parse_bootstrap,
    bench_bbnf_pretty_parse_tailwind,
    bench_lightningcss_parse_normalize,
    bench_lightningcss_parse_bootstrap,
);

benchmark_group!(
    json_phase_benches,
    bench_json_data_parse_only,
    bench_json_data_to_doc_only,
    bench_json_data_render_only,
    bench_json_canada_parse_only,
    bench_json_canada_to_doc_only,
    bench_json_canada_render_only,
);

benchmark_group!(
    gs_benches,
    bench_gs_simple,
    bench_gs_let,
    bench_gs_pathological,
    bench_gs_1kb,
    bench_gs_10kb,
);

benchmark_group!(
    gs_cached_benches,
    bench_gs_pathological_cached,
    bench_gs_1kb_cached,
    bench_gs_10kb_cached,
);

benchmark_group!(
    gs_phase_benches,
    bench_gs_simple_parse_only,
    bench_gs_let_parse_only,
    bench_gs_pathological_parse_only,
    bench_gs_pathological_to_doc_only,
    bench_gs_pathological_render_only,
    bench_gs_1kb_parse_only,
    bench_gs_1kb_to_doc_only,
    bench_gs_1kb_render_only,
    bench_gs_10kb_parse_only,
    bench_gs_10kb_to_doc_only,
    bench_gs_10kb_render_only,
);

benchmark_main!(json_benches, json_cached_benches, css_benches, css_cached_benches, css_phase_benches, biome_benches, css_parse_comparison, json_phase_benches, gs_benches, gs_cached_benches, gs_phase_benches);

