# prettify

Grammar-derived pretty-printers via `#[derive(Parser)]` from `bbnf-derive`.

## Structure

```
src/
  lib.rs               # PrinterConfig, ToDoc + SourceRange traits, range_to_doc()
  json.rs              # JSON prettifier — 7 tests, range formatting
  ebnf.rs              # EBNF prettifier — 4 tests, idempotent multi-rule
  bnf.rs               # BNF prettifier — 5 tests, idempotent multi-rule
  bbnf.rs              # BBNF prettifier — 5 tests, idempotent multi-rule
  css.rs               # CSS prettifier — 7 tests, nested rules, media queries
benches/
  prettify.rs          # 12 benchmarks: JSON (6) + CSS (6)
data/json/             # benchmark datasets (data.json 35KB, canada.json 2.2MB)
data/css/              # benchmark datasets (normalize.css 1.8KB, app.css 6.3KB)
```

## Build

```bash
cargo test
cargo bench --bench prettify
cargo clippy -- -D warnings
```

## Dependencies

All path deps—no registry crates at runtime:

- `parse_that` — `/Users/mkbabb/Programming/parse-that/rust/parse_that`
- `bbnf_derive` — `/Users/mkbabb/Programming/bbnf-lang/rust/bbnf-derive`
- `bbnf` — `/Users/mkbabb/Programming/bbnf-lang/rust/bbnf`
- `pprint` — `/Users/mkbabb/Programming/pprint`

Dev: `bencher` (harness for `[[bench]]`).

## Languages

All five implemented, all tests pass (28 total):

- JSON — 7 tests, range formatting via `prettify_json_range()`
- EBNF — 4 tests, idempotent multi-rule
- BNF — 5 tests, idempotent multi-rule
- BBNF — 5 tests, idempotent multi-rule
- CSS — 7 tests, nested rules, media queries, `css-stylesheet-pretty.bbnf`

## Benchmark Throughput

| Benchmark | Throughput |
|-----------|-----------|
| JSON data.json cached | ~116 MB/s |
| JSON canada.json cached | ~50 MB/s |
| CSS normalize.css cached | ~29 MB/s |
| CSS app.css cached | ~28 MB/s |

## Conventions

- Edition 2024, nightly required (`#![feature(cold_path)]`)
- Each language module: `#[derive(Parser)]` + `impl ToDoc` + `impl SourceRange` + `prettify_X()` entry point
- Grammar files live in `bbnf-lang/grammar/lang/` — `@pretty` directives control doc generation
- CSS grammar: `css-stylesheet-pretty.bbnf` (standalone, no imports)
- `PrinterConfig` controls `max_width`, `indent`, `use_tabs` — passed to `pprint::Printer`
- `range_to_doc()` — partial formatting, emits verbatim source for non-overlapping nodes
- Idempotency: `prettify(prettify(x)) == prettify(x)` — tested for JSON and EBNF
