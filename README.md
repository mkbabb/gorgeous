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

All 31 tests pass. Idempotency verified: `prettify(prettify(x)) == prettify(x)`.

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
          → pprint()       Wadler-Lindig pretty-printer
            → String       formatted output
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
| CSS app.css (6KB) | 50 MB/s | 11 MB/s | 4.5x |
| CSS bootstrap (281KB) | 342 MB/s | 17 MB/s | 20x |
| CSS tailwind (3.8MB) | 36 MB/s | 8 MB/s | 4.5x |
| JSON data.json (35KB) | 115 MB/s | -- | -- |

Phase breakdown (bootstrap 281KB):

| Phase | Throughput |
|-------|-----------|
| parse | 654 MB/s |
| to_doc | 1,450 MB/s |
| render | 1,428 MB/s |
| **e2e (cached)** | **342 MB/s** |

Gorgeous is **4.5--20x faster than biome** depending on file size, with zero
hand-written formatting code -- all formatting is grammar-driven via `@pretty`
directives and `split_balanced()` for format-time balanced splitting.

## Dependencies

All from crates.io:

- [`parse_that`](https://github.com/mkbabb/parse-that) — parser combinators
- [`bbnf_derive`](https://github.com/mkbabb/bbnf-lang) — `#[derive(Parser)]` proc-macro
- [`bbnf`](https://github.com/mkbabb/bbnf-lang) — grammar analysis
- [`pprint`](https://github.com/mkbabb/pprint) — Wadler-Lindig pretty-printer

Dev: `bencher` for `[[bench]]` harness.

## Build

```bash
cargo test --lib                # 31 tests
cargo bench --bench gorgeous    # 12 benchmarks (JSON 6 + CSS 6)
cargo clippy -- -D warnings
```

Requires Rust nightly (`#![feature(cold_path)]`).
