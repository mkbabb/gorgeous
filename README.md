# gorgeous

Grammar-driven pretty-printers for structured languages. Each language is defined
by a BBNF grammar file — `#[derive(Parser)]` generates the parser, `@pretty`
directives control formatting, and `pprint` renders the final output.

## Usage

```rust
use gorgeous::{prettify_json, prettify_css, PrinterConfig};

let config = PrinterConfig { max_width: 80, indent: 2, use_tabs: false };

let formatted_json = prettify_json(input, &config);
let formatted_css = prettify_css(input, &config);
```

Partial formatting via source range:

```rust
use gorgeous::{prettify_json_range, PrinterConfig};

let config = PrinterConfig { max_width: 80, indent: 2, use_tabs: false };
let formatted = prettify_json_range(input, 100..200, &config);
```

## Languages

| Language | Tests | Grammar source | Entry point |
|----------|------:|----------------|-------------|
| JSON | 9 | `json.bbnf` | `prettify_json()` |
| CSS | 8 | `css-stylesheet-pretty.bbnf` | `prettify_css()` |
| EBNF | 4 | `ebnf.bbnf` | `prettify_ebnf()` |
| BNF | 5 | `bnf.bbnf` | `prettify_bnf()` |
| BBNF | 5 | `bbnf.bbnf` | `prettify_bbnf()` |
| Google Sheets | 6 | `google-sheets.bbnf` | `prettify_formula()` |

All 37 tests pass. Idempotency verified: `prettify(prettify(x)) == prettify(x)`.

## CLI

Built-in languages auto-detect by extension; arbitrary languages JIT-compile from
any `.bbnf` grammar.

```bash
gorg input.json                        # built-in, auto-detect by extension
gorg --lang css input.css              # built-in, explicit language
gorg --grammar my.bbnf input.txt       # JIT: any grammar, instant prettifier
gorg --grammar my.bbnf -r expr in.txt  # JIT with explicit entry rule
echo '{}' | gorg --lang json           # stdin
gorg input.css -o output.css           # write to file
gorg -w 120 -i 2 input.json            # custom width + indent
gorg --clear-cache                     # purge JIT cache
```

The JIT pipeline parses the `.bbnf`, generates a Cargo project with
`#[derive(Parser, prettify)]`, compiles it once, and caches the binary by content
hash in `~/.cache/gorgeous/`. Second run is instant.

## Architecture

```
Grammar (.bbnf)
  → #[derive(Parser)]     proc-macro codegen (bbnf-derive)
    → AST (enum)           typed parse tree
      → to_doc()           @pretty-directed Doc emission
        → Doc tree         pprint intermediate representation
          → pprint()       Wadler-Lindig-inspired pretty-printer
            → String       formatted output

Alternative: VM interpreter (feature = "vm")
  Grammar (.bbnf) → bbnf-ir bytecode → runtime interpret → Doc tree → pprint()
```

The `@pretty` directives annotate grammar rules with formatting hints:

| Hint | Effect |
|------|--------|
| `group` | Wrap in `Doc::Group` -- try flat, break if too wide |
| `block` | Hardline-separated items |
| `indent` | Indent children |
| `blankline` | Double hardline between items |
| `softbreak` | Softline separators |
| `nobreak` | Space-only separators (no line breaks) |
| `fast` | `Join` instead of `SmartJoin` -- linear, no DP justification |
| `sep("...")` | Custom separator string; with `group`, uses `IfBreak(trimmed + Hardline, sep)` |
| `split("...")` | Format-time balanced splitting of opaque Spans via `split_balanced()` |

`range_to_doc()` enables partial formatting: nodes outside the target range emit
verbatim source text, nodes inside emit formatted `Doc` trees.

## Performance

End-to-end cached throughput (parse + to_doc + render):

| Benchmark | Gorgeous | Biome | Speedup |
|-----------|----------|-------|---------|
| CSS app.css (6KB) | 54 MB/s | 10 MB/s | 5.4x |
| CSS normalize (6KB) | 67 MB/s | — | — |
| CSS bootstrap (281KB) | 415 MB/s | 16 MB/s | 25.9x |
| CSS tailwind (3.8MB) | 45 MB/s | 14 MB/s | 3.2x |
| JSON data.json (35KB) | 115 MB/s | — | — |
| JSON canada.json (2.2MB) | 26 MB/s | — | — |

Phase breakdown (bootstrap 281KB):

| Phase | Throughput |
|-------|-----------|
| parse | 2,843 MB/s |
| to_doc | 1,038 MB/s |
| render | 1,140 MB/s |

The derive path uses zero hand-written formatting code—all formatting is
grammar-driven via `@pretty` directives and `split_balanced()` for format-time
balanced splitting. The VM path interprets `bbnf-ir` bytecode at runtime for
grammars not compiled via `#[derive(Parser)]`.

## Dependencies

All from crates.io:

- [`parse_that`](https://github.com/mkbabb/parse-that) — parser combinators
- [`bbnf_derive`](https://github.com/mkbabb/bbnf-lang) — `#[derive(Parser)]` proc-macro
- [`bbnf`](https://github.com/mkbabb/bbnf-lang) — grammar analysis
- [`pprint`](https://github.com/mkbabb/pprint) — Wadler-Lindig pretty-printer

Dev: `bencher` for `[[bench]]` harness, `biome_css_parser`/`biome_css_formatter` v0.4.0 (benchmark competitor).

## Build

```bash
cargo test --lib                # 37 tests
cargo test --lib --features vm  # + VM tests
cargo bench --bench gorgeous    # 51 benchmarks (JSON + CSS + GS + biome, phase splits)
cargo clippy -- -D warnings
```

Requires Rust nightly (`#![feature(cold_path)]`).
