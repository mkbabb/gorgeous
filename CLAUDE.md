# gorgeous

Grammar-driven pretty-printers via `#[derive(Parser)]` from `bbnf-derive`.
CLI with built-in languages + JIT compilation from arbitrary `.bbnf` grammars.

## Structure

```
src/
  lib.rs               # PrinterConfig, ToDoc + SourceRange traits, range_to_doc()
  main.rs              # CLI binary — built-in languages + JIT grammar pipeline
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
cargo test --lib
cargo build                        # binary: target/debug/gorgeous
cargo bench --bench prettify
cargo clippy -- -D warnings
```

## CLI

```bash
gorgeous input.json                                    # built-in, auto-detect
gorgeous --lang css input.css                          # built-in, explicit
gorgeous --grammar my.bbnf input.txt                   # JIT from grammar
gorgeous --grammar my.bbnf --rule expr input.txt       # JIT, explicit entry rule
echo '{}' | gorgeous --lang json                       # stdin
gorgeous input.json -o output.json                     # write to file
gorgeous --clear-cache                                 # purge JIT cache
```

JIT pipeline: parse `.bbnf` → extract rules → generate temp Cargo project with
`#[derive(Parser, prettify)]` → `cargo build --release` → cache binary by
content hash in `~/.cache/gorgeous/<hash>/` → exec. Second run is instant.

## Dependency Graph

```
pprint_derive → pprint → parse_that → bbnf → bbnf_derive
                                                  ↓
                                              gorgeous  ← all of the above
```

gorgeous is the leaf of the Rust crate graph — depends on everything.
Cargo.toml uses crates.io version-only deps; local dev via `.cargo/config.toml` `[patch.crates-io]`.

## Dependencies

All from crates.io:

- `parse_that` — parser combinator library
- `bbnf_derive` — proc-macro: `#[derive(Parser)]` from `.bbnf` files
- `bbnf` — grammar parser (used at runtime for JIT rule extraction)
- `pprint` — Wadler-Lindig pretty-printer

Dev: `bencher` (harness for `[[bench]]`).

## Languages

All five built-in, all tests pass (28 total):

- JSON — 7 tests, range formatting via `prettify_json_range()`
- EBNF — 4 tests, idempotent multi-rule
- BNF — 5 tests, idempotent multi-rule
- BBNF — 5 tests, idempotent multi-rule
- CSS — 7 tests, nested rules, media queries, `css-stylesheet-pretty.bbnf`

## Benchmark Throughput

| Benchmark | Throughput |
|-----------|-----------|
| JSON data.json cached | ~119 MB/s |
| JSON canada.json cached | ~53 MB/s |
| CSS normalize.css cached | ~22 MB/s |
| CSS app.css cached | ~22 MB/s |
| CSS app.css to_doc only | ~117 MB/s |
| CSS app.css render only | ~93 MB/s |

## Conventions

- Edition 2024, nightly required (`#![feature(cold_path)]`)
- Crate name `gorgeous`, lib name `gorgeous`, binary name `gorgeous`
- Each language module: `#[derive(Parser)]` + `impl ToDoc` + `impl SourceRange` + `prettify_X()` entry point
- Grammar files bundled in `grammar/` — `@pretty` directives control doc generation
- CSS grammar: `css-stylesheet-pretty.bbnf` (standalone, no imports)
- `PrinterConfig` controls `max_width`, `indent`, `use_tabs` — passed to `pprint::Printer`
- `range_to_doc()` — partial formatting, emits verbatim source for non-overlapping nodes
- Idempotency: `prettify(prettify(x)) == prettify(x)` — tested for JSON and EBNF
- JIT: `DefaultHasher` for grammar content hashing, cached in `~/.cache/gorgeous/<hash>/`
