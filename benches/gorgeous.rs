#![feature(cold_path)]

use bencher::{benchmark_group, benchmark_main, Bencher};
use pprint::{pprint as render, pprint_ref};
use gorgeous::json::prettify_json;
use gorgeous::css::{prettify_css, CssParser};
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
    json_phase_benches,
    bench_json_data_parse_only,
    bench_json_data_to_doc_only,
    bench_json_data_render_only,
    bench_json_canada_parse_only,
);

benchmark_main!(json_benches, json_cached_benches, css_benches, css_cached_benches, css_phase_benches, biome_benches, json_phase_benches);

