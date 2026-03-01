#![feature(cold_path)]

use bencher::{benchmark_group, benchmark_main, Bencher};
use pprint::pprint as render;
use prettify::json::prettify_json;
use prettify::css::{prettify_css, CssParser};
use prettify::{PrinterConfig, ToDoc};

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

// ── CSS cached benchmarks (parser built once, reused) ───────────────────────

fn bench_css_small_rule_cached(b: &mut Bencher) {
    let input = "body { color: red; font-size: 16px; margin: 0; padding: 0; }";
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(input).unwrap();
        render(ast.to_doc(), Some(config.to_printer()))
    });
}

fn bench_css_normalize_cached(b: &mut Bencher) {
    let input = load_css_normalize();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), Some(config.to_printer()))
    });
}

fn bench_css_app_cached(b: &mut Bencher) {
    let input = load_css_app();
    let config = PrinterConfig::default();
    let parser = CssParser::stylesheet();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), Some(config.to_printer()))
    });
}

// ── JSON cached benchmarks ──────────────────────────────────────────────────

fn bench_json_small_cached(b: &mut Bencher) {
    use prettify::json::JsonParser;
    let input = r#"{"name": "Alice", "age": 30, "active": true}"#;
    let config = PrinterConfig::default();
    let parser = JsonParser::value();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(input).unwrap();
        render(ast.to_doc(), Some(config.to_printer()))
    });
}

fn bench_json_data_cached(b: &mut Bencher) {
    use prettify::json::JsonParser;
    let input = load_json_data();
    let config = PrinterConfig::default();
    let parser = JsonParser::value();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), Some(config.to_printer()))
    });
}

fn bench_json_canada_cached(b: &mut Bencher) {
    use prettify::json::JsonParser;
    let input = load_json_canada();
    let config = PrinterConfig::default();
    let parser = JsonParser::value();
    b.bytes = input.len() as u64;
    b.iter(|| {
        let ast = parser.parse(&input).unwrap();
        render(ast.to_doc(), Some(config.to_printer()))
    });
}

// ── CSS phase-split benchmarks (parse vs to_doc vs render) ──────────────────

fn bench_css_app_parse_only(b: &mut Bencher) {
    let input = load_css_app();
    let parser = CssParser::stylesheet();
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
        render(doc.clone(), Some(config.to_printer()))
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
);

benchmark_group!(
    css_cached_benches,
    bench_css_small_rule_cached,
    bench_css_normalize_cached,
    bench_css_app_cached,
);

benchmark_group!(
    css_phase_benches,
    bench_css_app_parse_only,
    bench_css_app_to_doc_only,
    bench_css_app_render_only,
);

benchmark_main!(json_benches, json_cached_benches, css_benches, css_cached_benches, css_phase_benches);

