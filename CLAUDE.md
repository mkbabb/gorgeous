# prettify

Grammar-derived pretty-printers via `#[derive(Parser)]` from `bbnf-derive`.

## Structure

```
src/
  lib.rs               # PrinterConfig, ToDoc + SourceRange traits, range_to_doc()
  json.rs              # JSON prettifier — 7 tests, range formatting
  ebnf.rs              # EBNF prettifier — 4 tests, idempotent multi-rule
  bnf.rs               # BNF prettifier — stub (returns None)
  bbnf.rs              # BBNF prettifier — stub (returns None)
benches/
  prettify.rs          # JSON benchmarks — small object, data.json, canada.json
data/json/             # benchmark datasets (data.json, canada.json)
```

## Build

```bash
cargo test
cargo bench --bench prettify
cargo clippy -- -D warnings
```

## Dependencies

All path deps — no registry crates at runtime:

- `parse_that` — `/Users/mkbabb/Programming/parse-that/rust/parse_that`
- `bbnf_derive` — `/Users/mkbabb/Programming/bbnf-lang/rust/bbnf-derive`
- `bbnf` — `/Users/mkbabb/Programming/bbnf-lang/rust/bbnf`
- `pprint` — `/Users/mkbabb/Programming/pprint`

Dev: `bencher` (harness for `[[bench]]`).

## Languages

- JSON — fully implemented, 7 tests, range formatting via `prettify_json_range()`
- EBNF — fully implemented, 4 tests, idempotent multi-rule
- BNF — stub (returns `None`) — blocked on codegen for complex grammars
- BBNF — stub (returns `None`) — blocked on codegen for compound rules (`>>` / `<<`)
- CSS — not started

## Conventions

- Edition 2024, nightly required (`#![feature(cold_path)]`)
- Each language module: `#[derive(Parser)]` + `impl ToDoc` + `impl SourceRange` + `prettify_X()` entry point
- Grammar files live in `bbnf-lang/grammar/lang/` — `@pretty` directives control doc generation
- `PrinterConfig` controls `max_width`, `indent`, `use_tabs` — passed to `pprint::Printer`
- `range_to_doc()` — partial formatting, emits verbatim source for non-overlapping nodes
- Idempotency: `prettify(prettify(x)) == prettify(x)` — tested for JSON and EBNF
