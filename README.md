# prettify

Grammar-derived pretty-printers for structured languages. Each language is defined
by a BBNF grammar file — `#[derive(Parser)]` generates the parser, `@pretty`
directives control formatting, and `pprint` renders the final output.

## Usage

```rust
use prettify::{prettify_json, prettify_css, PrinterConfig};

let config = PrinterConfig { max_width: 80, indent: 2, use_tabs: false };

let formatted_json = prettify_json(input, &config);
let formatted_css = prettify_css(input, &config);
```

Partial formatting via source range:

```rust
use prettify::{prettify_json_range, PrinterConfig};

let config = PrinterConfig { max_width: 80, indent: 2, use_tabs: false };
let formatted = prettify_json_range(input, 100..200, &config);
```

## Languages

| Language | Tests | Grammar source | Entry point |
|----------|------:|----------------|-------------|
| JSON | 7 | `json-pretty.bbnf` | `prettify_json()` |
| CSS | 7 | `css-stylesheet-pretty.bbnf` | `prettify_css()` |
| EBNF | 4 | `ebnf-pretty.bbnf` | `prettify_ebnf()` |
| BNF | 5 | `bnf-pretty.bbnf` | `prettify_bnf()` |
| BBNF | 5 | `bbnf-pretty.bbnf` | `prettify_bbnf()` |

All 28 tests pass. Idempotency verified: `prettify(prettify(x)) == prettify(x)`.

## Architecture

```
Grammar (.bbnf)
  → #[derive(Parser)]     proc-macro codegen (bbnf-derive)
    → AST (enum)           typed parse tree
      → to_doc()           @pretty-directed Doc emission
        → Doc tree         pprint intermediate representation
          → pprint()       Wadler-Lindig pretty-printer
            → String       formatted output
```

The `@pretty` directives annotate grammar rules with formatting hints:

| Hint | Effect |
|------|--------|
| `group` | Wrap in `Doc::Group` — try flat, break if too wide |
| `block` | Hardline-separated items |
| `indent` | Indent children |
| `blankline` | Double hardline between items |
| `softbreak` | Softline separators |
| `nobreak` | Space-only separators (no line breaks) |
| `fast` | `Join` instead of `SmartJoin` — linear, no DP justification |

`range_to_doc()` enables partial formatting: nodes outside the target range emit
verbatim source text, nodes inside emit formatted `Doc` trees.

## Performance

Phase-split CSS throughput (normalize.css + app.css):

| Phase | Throughput |
|-------|-----------|
| Parse | ~31 MB/s |
| to_doc | ~28 MB/s |
| pprint render | ~94 MB/s |

End-to-end cached throughput:

| Benchmark | Throughput |
|-----------|-----------|
| JSON data.json | ~116 MB/s |
| JSON canada.json | ~50 MB/s |
| CSS normalize.css | ~29 MB/s |
| CSS app.css | ~28 MB/s |

## Dependencies

All path dependencies — no registry crates at runtime:

- [`parse_that`](https://github.com/mkbabb/parse-that) — parser combinators
- [`bbnf-derive`](https://github.com/mkbabb/bbnf-lang) — `#[derive(Parser)]` proc-macro
- [`bbnf`](https://github.com/mkbabb/bbnf-lang) — grammar analysis
- [`pprint`](https://github.com/mkbabb/pprint) — Wadler-Lindig pretty-printer

Dev: `bencher` for `[[bench]]` harness.

## Build

```bash
cargo test                      # 28 tests
cargo bench --bench prettify    # 12 benchmarks (JSON 6 + CSS 6)
cargo clippy -- -D warnings
```

Requires Rust nightly (`#![feature(cold_path)]`).
