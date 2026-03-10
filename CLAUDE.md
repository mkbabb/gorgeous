# gorgeous

Grammar-driven pretty-printers via `#[derive(Parser)]` from `bbnf-derive`.
CLI with built-in languages + JIT compilation from arbitrary `.bbnf` grammars.

## Structure

```
src/
  lib.rs               # PrinterConfig, ToDoc + SourceRange traits, range_to_doc()
  main.rs              # CLI binary — built-in languages + JIT grammar pipeline
  json.rs              # JSON prettifier — 9 tests, range formatting
  ebnf.rs              # EBNF prettifier — 4 tests, idempotent multi-rule
  bnf.rs               # BNF prettifier — 5 tests, idempotent multi-rule
  bbnf.rs              # BBNF prettifier — 5 tests, idempotent multi-rule
  css.rs               # CSS prettifier — 8 tests, nested rules, media queries
  tests/
  biome_compare.rs     # Output comparison tests: biome vs gorgeous
  biome_compare2.rs    # Output size ratio tests across files
benches/
  gorgeous.rs          # 32 benchmarks: JSON + CSS + biome, phase splits
data/json/             # benchmark datasets (data.json 35KB, canada.json 2.2MB)
data/css/              # benchmark datasets (normalize 1.8KB, app 6.3KB, bootstrap 281KB, tailwind 3.8MB)
```

## Build

```bash
cargo test --lib
cargo build                        # binary: target/debug/gorg
cargo bench --bench gorgeous
cargo clippy -- -D warnings
```

## CLI

```bash
gorg input.json                                        # built-in, auto-detect
gorg --lang css input.css                              # built-in, explicit
gorg --grammar my.bbnf input.txt                       # JIT from grammar
gorg --grammar my.bbnf --rule expr input.txt           # JIT, explicit entry rule
echo '{}' | gorg --lang json                           # stdin
gorg input.json -o output.json                         # write to file
gorg --clear-cache                                     # purge JIT cache
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
- `pprint` — Wadler-Lindig pretty-printer (uses `rustc-hash` FxHashMap internally)
- `mimalloc` — global allocator

Dev: `bencher` (harness for `[[bench]]`), `biome_css_parser`/`biome_css_formatter` v0.4.0 (benchmark competitor).

## Languages

All five built-in, all tests pass (31 total):

- JSON — 9 tests, range formatting via `prettify_json_range()`
- EBNF — 4 tests, idempotent multi-rule
- BNF — 5 tests, idempotent multi-rule
- BBNF — 5 tests, idempotent multi-rule
- CSS — 8 tests, nested rules, media queries, `css-stylesheet-pretty.bbnf`

## Benchmark Throughput

| Benchmark | Gorgeous | Biome | Speedup |
|-----------|----------|-------|---------|
| CSS app.css (6KB) | 50 MB/s | 11 MB/s | 4.5x |
| CSS bootstrap (281KB) | 342 MB/s | 17 MB/s | 20x |
| CSS tailwind (3.8MB) | 36 MB/s | 8 MB/s | 4.5x |
| JSON data.json (35KB) | 115 MB/s | — | — |

Phase breakdown (bootstrap): parse 654 MB/s, to_doc 1,450 MB/s, render 1,428 MB/s.

## Conventions

- Edition 2024, nightly required (`#![feature(cold_path)]`)
- Crate name `gorgeous`, lib name `gorgeous`, binary name `gorg`
- Each language module: `#[derive(Parser)]` + `impl ToDoc` + `impl SourceRange` + `prettify_X()` entry point
- Grammar files bundled in `grammar/` — `@pretty` directives control doc generation
- CSS formatting is purely grammar-driven — `@pretty selectorSpan split(",") group sep(", ")` handles selector splitting via `split_balanced()` at format time (zero manual overrides)
- CSS grammar: `css-stylesheet-pretty.bbnf` (standalone, no imports); `css-fast.bbnf` available for JIT
- `PrinterConfig` controls `max_width`, `indent`, `use_tabs` — passed to `pprint::Printer`
- `range_to_doc()` — partial formatting, emits verbatim source for non-overlapping nodes
- Idempotency: `prettify(prettify(x)) == prettify(x)` — tested for JSON and EBNF
- `mimalloc` as global allocator (`#[global_allocator]`)
- `pprint` uses `rustc-hash` FxHashMap for internal hash maps
- JIT: `DefaultHasher` for grammar content hashing, cached in `~/.cache/gorgeous/<hash>/`
