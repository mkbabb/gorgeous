# Gorgeous Execution Paths

Three paths format source code: AOT (built-in languages), JIT (arbitrary grammars), and VM (WASM runtime).

## Pipeline Overview

```
              CLI (gorg)                    WASM module
              │       │                     │         │
         built-in   --grammar          AOT wrappers  VM wrapper
              │       │                     │         │
              ▼       ▼                     ▼         ▼
           [AOT]    [JIT]               [AOT]       [VM]
         compiled  compile-on-     pre-compiled  bbnf-ir bytecode
         at build  first-run       into WASM     interpreter
              │       │                     │         │
              ▼       ▼                     ▼         ▼
         parse → to_doc → pprint::pprint() → formatted string
```

## 1. AOT — Built-in Languages

Compile-time code generation via `#[derive(Parser)]`. Grammar baked into the binary.

**Flow:**
```
.bbnf grammar
  ↓  (compile time — bbnf-derive proc macro)
Monomorphic Rust code: enum variants, parse methods, to_doc(), source_range()
  ↓  (runtime)
Parser::rule().parse(input) → Enum<'a>       // zero-copy parse
  ↓
enum.to_doc() → pprint::Doc<'a>              // @pretty-directed Doc tree
  ↓
pprint::pprint(doc, printer) → String        // Wadler-Lindig layout
```

**6 Built-in Formatters:**

| Language | Module | Grammar | Entry Rule |
|----------|--------|---------|------------|
| JSON | `src/json.rs` | `grammar/lang/json.bbnf` | `value()` |
| CSS | `src/css.rs` | `grammar/css/css-stylesheet-pretty.bbnf` | `stylesheet()` |
| EBNF | `src/ebnf.rs` | `grammar/lang/ebnf.bbnf` | `grammar()` |
| BNF | `src/bnf.rs` | `grammar/lang/bnf.bbnf` | `grammar()` |
| BBNF | `src/bbnf.rs` | `grammar/lang/bbnf.bbnf` | `grammar()` |
| Google Sheets | `src/google_sheets.rs` | `grammar/lang/google-sheets.bbnf` | `formula()` |

Each module follows the same pattern:
```rust
#[derive(Parser)]
#[parser(path = "grammar/lang/json.bbnf", prettify)]
pub struct JsonParser;

pub fn prettify_json(input: &str, config: &PrinterConfig) -> Option<String> {
    let ast = JsonParser::value().parse(input)?;
    let doc = ast.to_doc();
    Some(pprint::pprint(doc, config.to_printer()))
}
```

**Range formatting** — `prettify_*_range()` variants use `range_to_doc()`:
nodes outside the range emit verbatim source, nodes inside get formatted.

## 2. JIT — `--grammar` Flag

Compiles arbitrary `.bbnf` to a Rust binary on first run, caches by content hash.

**Flow:**
```
gorg --grammar my.bbnf input.txt
  ↓
1. Parse grammar, extract rule names
  ↓
2. Content hash → cache key (16-hex)
  ↓
3. Check ~/.cache/gorgeous/<hash>/target/release/gorgeous-jit
  ↓  (cache miss)
4. Generate Cargo project:
   - Cargo.toml (parse_that, bbnf_derive, bbnf, pprint deps)
   - main.rs with #[derive(Parser)] + match arms for every rule
   - grammar.bbnf copied into project
   - rust-toolchain.toml → nightly
  ↓
5. cargo build --release
  ↓  (cache hit or after build)
6. Execute binary: gorgeous-jit --rule <RULE> --width <N> [input]
  ↓
7. Return formatted output
```

**Cache behavior:**
- One binary per unique grammar content hash
- Second run is instant (subprocess execution only)
- `--clear-cache` removes `~/.cache/gorgeous/`

## 3. VM — Runtime Bytecode Interpreter

Feature-gated (`vm` feature). Used by WASM for custom grammars in the playground.

**Flow:**
```
compile_grammar(source)
  ↓
bbnf pipeline → GrammarIR → bbnf-ir::compile() → BytecodeProgram
  ↓
parse_with_grammar(handle, input) → Value tree
  ↓
value_to_doc(ir, value, input) → pprint::Doc    // consults @pretty hints from IR
  ↓
pprint::pprint(doc, printer) → String
```

**Not used by CLI.** The CLI uses AOT (built-in) or JIT (subprocess).
The VM exists for WASM where runtime grammar compilation is needed.

## WASM Module

Compiled from `bbnf-lang/wasm/`. Exports three categories of functions:

### AOT Formatters (pre-compiled)
```
format_json(input, max_width, indent, use_tabs) → string?
format_css(input, max_width, indent, use_tabs) → string?
format_bnf(input, max_width, indent, use_tabs) → string?
format_ebnf(input, max_width, indent, use_tabs) → string?
format_bbnf(input, max_width, indent, use_tabs) → string?
```

### VM Functions (custom grammars)
```
compile_grammar(grammar, entry_rule?) → handle
parse_with_grammar(handle, input) → ParseResult
format_with_grammar(handle, input, max_width, indent, use_tabs) → string?
free_grammar(handle)
```

### LSP Functions (17 features)
Shared `bbnf-analysis` crate, called directly from the playground:
```
hover_at_offset, completions, semantic_tokens_full, inlay_hints,
goto_definition, find_references, prepare_rename, rename,
document_symbols, folding_ranges, code_lens, selection_range,
code_actions, format_document, format_range, on_type_format,
full_sync
```

## @pretty Directives

Grammar annotations that control formatting. Applied during `to_doc()` codegen (AOT)
or `value_to_doc()` interpretation (VM).

| Directive | Effect |
|-----------|--------|
| `group` | Wrap in `Doc::Group` — flat if fits, break otherwise |
| `block` | Hardline separators between children |
| `blankline` | Double hardline separators |
| `indent` / `dedent` | Indent/dedent children |
| `softbreak` | Softline (space if flat, newline if broken) |
| `nobreak` | Space-only, never break |
| `compact` | Concatenate with no separator |
| `fast` | Use `LinearJoin` — no text-justify pre-pass |
| `sep("...")` | Custom separator; with `group` uses `IfBreak(trimmed+Hardline, sep)` |
| `split("...")` | Format-time balanced splitting of opaque Spans |
| `off` | Disable formatting for this rule |

## CLI Reference

```
gorg [OPTIONS] [FILE]

Options:
  -l, --lang <LANG>       Explicit language (json, css, ebnf, bnf, bbnf)
  -g, --grammar <FILE>    BBNF grammar file (JIT path)
  -r, --rule <RULE>       Entry rule (default: last rule in grammar)
  -w, --width <N>         Max line width (default: 80)
  -i, --indent <N>        Indent size (default: 4)
  --tabs                  Use tabs instead of spaces
  -o, --output <FILE>     Write to file instead of stdout
  --clear-cache           Remove cached JIT binaries

Language detection:
  .json .jsonc .json5 → json
  .css .scss           → css
  .ebnf                → ebnf
  .bnf                 → bnf
  .bbnf                → bbnf
```
