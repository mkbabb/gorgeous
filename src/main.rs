#![feature(cold_path)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use gorgeous::PrinterConfig;

// ---------------------------------------------------------------------------
// Built-in language detection + formatting
// ---------------------------------------------------------------------------

fn detect_language(path: &str) -> Option<&'static str> {
    match Path::new(path).extension()?.to_str()? {
        "json" | "jsonc" | "json5" => Some("json"),
        "css" | "scss" => Some("css"),
        "ebnf" => Some("ebnf"),
        "bnf" => Some("bnf"),
        "bbnf" => Some("bbnf"),
        _ => None,
    }
}

fn format_builtin(lang: &str, input: &str, config: &PrinterConfig) -> Result<String, String> {
    let result = match lang {
        "json" => gorgeous::json::prettify_json(input, config),
        "css" => gorgeous::css::prettify_css(input, config),
        "ebnf" => gorgeous::ebnf::prettify_ebnf(input, config),
        "bnf" => gorgeous::bnf::prettify_bnf(input, config),
        "bbnf" => gorgeous::bbnf::prettify_bbnf(input, config),
        _ => return Err(format!("unknown language: {lang}")),
    };
    result.ok_or_else(|| "parse error".to_string())
}

// ---------------------------------------------------------------------------
// JIT grammar pipeline
// ---------------------------------------------------------------------------

/// Extract rule names from a BBNF grammar source string.
fn extract_rule_names(grammar_src: &str) -> Vec<String> {
    let parser = bbnf::BBNFGrammar::grammar_with_imports();
    let Some(parsed) = parser.parse(grammar_src) else {
        return Vec::new();
    };
    parsed
        .rules
        .keys()
        .filter_map(|expr| bbnf::get_nonterminal_name(expr).map(String::from))
        .collect()
}

/// Stable hash of grammar content → hex string (for cache key).
fn grammar_hash(content: &str) -> String {
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// Where cached binaries live: `~/.cache/gorgeous/<hash>/formatter`
fn cache_dir(hash: &str) -> PathBuf {
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home).join(".cache")
        });
    base.join("gorgeous").join(hash)
}

/// Crate versions baked in at compile time (match gorgeous's own deps).
const DEP_PARSE_THAT: &str = "0.1.1";
const DEP_BBNF_DERIVE: &str = "0.2.0";
const DEP_BBNF: &str = "0.2.0";
const DEP_PPRINT: &str = "0.3.1";

/// Generate a temporary Cargo project that `#[derive(Parser)]` from the grammar.
fn generate_project(
    dir: &Path,
    grammar_content: &str,
    rule_names: &[String],
) -> Result<(), String> {
    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| format!("mkdir: {e}"))?;

    // Copy grammar into the project root
    std::fs::write(dir.join("grammar.bbnf"), grammar_content)
        .map_err(|e| format!("write grammar: {e}"))?;

    // Cargo.toml — deps from crates.io, no local paths
    let cargo_toml = format!(
        r#"[package]
name = "gorgeous-jit"
version = "0.0.0"
edition = "2024"
publish = false

[dependencies]
parse_that = "{DEP_PARSE_THAT}"
bbnf_derive = "{DEP_BBNF_DERIVE}"
bbnf = "{DEP_BBNF}"
pprint = "{DEP_PPRINT}"

[profile.release]
opt-level = 2
lto = "thin"
"#
    );
    std::fs::write(dir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("write Cargo.toml: {e}"))?;

    // rust-toolchain.toml (ensure nightly)
    std::fs::write(
        dir.join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"nightly\"\n",
    )
    .map_err(|e| format!("write toolchain: {e}"))?;

    // Generate main.rs with match arms for every rule
    // Last rule is typically the entry point (leaf rules first, root rule last).
    let first_rule = rule_names.last().map(|s| s.as_str()).unwrap_or("start");

    let match_arms: String = rule_names
        .iter()
        .map(|name| {
            format!(
                "            \"{name}\" => {{\n                let ast = GrammarParser::{name}().parse(&input)?;\n                ast.to_doc()\n            }}"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let rule_list = rule_names.join(", ");

    let main_rs = format!(
        r##"#![feature(cold_path)]

use std::io::{{self, Read, Write}};
use bbnf_derive::Parser;
use pprint::pprint as render;

#[derive(Parser)]
#[parser(path = "grammar.bbnf", prettify)]
pub struct GrammarParser;

impl<'a> GrammarParserEnum<'a> {{
    fn to_doc_trait(&self) -> pprint::Doc<'a> {{
        GrammarParserEnum::to_doc(self)
    }}
}}

fn main() {{
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut rule: Option<String> = None;
    let mut width: usize = 80;
    let mut indent: usize = 4;
    let mut use_tabs = false;
    let mut input_path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {{
        match args[i].as_str() {{
            "--rule" | "-r" => {{
                i += 1;
                rule = args.get(i).cloned();
            }}
            "--width" | "-w" => {{
                i += 1;
                width = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(80);
            }}
            "--indent" | "-i" => {{
                i += 1;
                indent = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(4);
            }}
            "--tabs" => use_tabs = true,
            _ => input_path = Some(args[i].clone()),
        }}
        i += 1;
    }}

    let rule = rule.as_deref().unwrap_or("{first_rule}");

    let input = match &input_path {{
        Some(path) => std::fs::read_to_string(path).unwrap_or_else(|e| {{
            eprintln!("error reading {{path}}: {{e}}");
            std::process::exit(1);
        }}),
        None => {{
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap();
            buf
        }}
    }};

    let doc: Option<pprint::Doc<'_>> = (|| {{
        Some(match rule {{
{match_arms}
            other => {{
                eprintln!("unknown rule: {{other}}");
                eprintln!("available: [{rule_list}]");
                std::process::exit(1);
            }}
        }})
    }})();

    match doc {{
        Some(d) => {{
            let printer = pprint::Printer::new(width, indent, use_tabs);
            let output = render(d, printer);
            io::stdout().write_all(output.as_bytes()).unwrap();
        }}
        None => {{
            eprintln!("parse error");
            std::process::exit(1);
        }}
    }}
}}
"##
    );

    std::fs::write(src_dir.join("main.rs"), main_rs).map_err(|e| format!("write main.rs: {e}"))?;
    Ok(())
}

/// Compile the generated project.
fn compile_project(dir: &Path) -> Result<PathBuf, String> {
    let output = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--quiet")
        .current_dir(dir)
        .output()
        .map_err(|e| format!("cargo build: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("compilation failed:\n{stderr}"));
    }

    let binary = dir.join("target/release/gorgeous-jit");
    if !binary.exists() {
        return Err("binary not found after compilation".into());
    }
    Ok(binary)
}

/// Run the cached JIT binary, forwarding config + input.
fn run_jit_binary(
    binary: &Path,
    rule: Option<&str>,
    input_path: Option<&str>,
    input_stdin: Option<&str>,
    config: &PrinterConfig,
) -> Result<String, String> {
    let mut cmd = Command::new(binary);
    cmd.arg("--width").arg(config.max_width.to_string());
    cmd.arg("--indent").arg(config.indent.to_string());
    if config.use_tabs {
        cmd.arg("--tabs");
    }
    if let Some(r) = rule {
        cmd.arg("--rule").arg(r);
    }

    if let Some(path) = input_path {
        cmd.arg(path);
    }

    let output = if input_stdin.is_some() {
        use std::process::Stdio;
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn: {e}"))?;
        if let Some(stdin_data) = input_stdin {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(stdin_data.as_bytes())
                .map_err(|e| format!("stdin write: {e}"))?;
        }
        child.wait_with_output().map_err(|e| format!("wait: {e}"))?
    } else {
        cmd.output().map_err(|e| format!("exec: {e}"))?
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    String::from_utf8(output.stdout).map_err(|e| format!("utf8: {e}"))
}

/// Full JIT pipeline: parse grammar → generate → compile → cache → run.
fn format_grammar(
    grammar_path: &str,
    rule: Option<&str>,
    input_path: Option<&str>,
    input_stdin: Option<&str>,
    config: &PrinterConfig,
) -> Result<String, String> {
    let grammar_content = std::fs::read_to_string(grammar_path)
        .map_err(|e| format!("error reading grammar {grammar_path}: {e}"))?;

    let rule_names = extract_rule_names(&grammar_content);
    if rule_names.is_empty() {
        return Err("no rules found in grammar (parse failed?)".into());
    }

    eprintln!(
        "gorg: grammar has {} rules: {}",
        rule_names.len(),
        rule_names.join(", ")
    );

    let hash = grammar_hash(&grammar_content);
    let dir = cache_dir(&hash);
    let binary = dir.join("target/release/gorgeous-jit");

    if !binary.exists() {
        eprintln!("gorg: compiling grammar (first run)…");
        generate_project(&dir, &grammar_content, &rule_names)?;
        compile_project(&dir)?;
        eprintln!("gorg: cached at {}", dir.display());
    }

    run_jit_binary(&binary, rule, input_path, input_stdin, config)
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

fn usage() -> ! {
    eprintln!("Usage: gorg [OPTIONS] <FILE>");
    eprintln!();
    eprintln!("  BUILT-IN   gorg input.json");
    eprintln!("  GRAMMAR    gorg --grammar my.bbnf input.txt");
    eprintln!("  STDIN      echo '{{}}' | gorg --lang json");
    eprintln!("  WRITE      gorg input.css -o output.css");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -l, --lang <LANG>       Language: json, css, ebnf, bnf, bbnf");
    eprintln!("  -g, --grammar <FILE>    Use a .bbnf grammar file (JIT-compiled)");
    eprintln!("  -r, --rule <RULE>       Entry rule (default: first rule in grammar)");
    eprintln!("  -w, --width <N>         Max line width (default: 80)");
    eprintln!("  -i, --indent <N>        Indent size (default: 4)");
    eprintln!("  --tabs                  Use tabs instead of spaces");
    eprintln!("  -o, --output <FILE>     Write to file instead of stdout");
    eprintln!("  --clear-cache           Remove all cached JIT binaries");
    std::process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }

    let mut lang: Option<String> = None;
    let mut grammar_path: Option<String> = None;
    let mut rule: Option<String> = None;
    let mut width: usize = 80;
    let mut indent: usize = 4;
    let mut use_tabs = false;
    let mut output_path: Option<String> = None;
    let mut input_path: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-l" | "--lang" => {
                i += 1;
                lang = Some(args.get(i).unwrap_or_else(|| usage()).clone());
            }
            "-g" | "--grammar" => {
                i += 1;
                grammar_path = Some(args.get(i).unwrap_or_else(|| usage()).clone());
            }
            "-r" | "--rule" => {
                i += 1;
                rule = Some(args.get(i).unwrap_or_else(|| usage()).clone());
            }
            "-w" | "--width" => {
                i += 1;
                width = args
                    .get(i)
                    .unwrap_or_else(|| usage())
                    .parse()
                    .unwrap_or_else(|_| usage());
            }
            "-i" | "--indent" => {
                i += 1;
                indent = args
                    .get(i)
                    .unwrap_or_else(|| usage())
                    .parse()
                    .unwrap_or_else(|_| usage());
            }
            "--tabs" => use_tabs = true,
            "-o" | "--output" => {
                i += 1;
                output_path = Some(args.get(i).unwrap_or_else(|| usage()).clone());
            }
            "--clear-cache" => {
                let dir = cache_dir("").parent().unwrap().to_path_buf();
                if dir.exists() {
                    std::fs::remove_dir_all(&dir).ok();
                    eprintln!("gorg: cache cleared");
                }
                std::process::exit(0);
            }
            "-h" | "--help" => usage(),
            s if s.starts_with('-') => {
                eprintln!("unknown option: {s}");
                usage();
            }
            _ => input_path = Some(args[i].clone()),
        }
        i += 1;
    }

    let config = PrinterConfig {
        max_width: width,
        indent,
        use_tabs,
    };

    // Grammar file mode → JIT pipeline (input may be file or stdin)
    if let Some(ref gpath) = grammar_path {
        let stdin_input = if input_path.is_none() {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error reading stdin: {e}");
                std::process::exit(1);
            });
            Some(buf)
        } else {
            None
        };

        match format_grammar(
            gpath,
            rule.as_deref(),
            input_path.as_deref(),
            stdin_input.as_deref(),
            &config,
        ) {
            Ok(output) => emit(&output, output_path.as_deref()),
            Err(e) => {
                eprintln!("gorg: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    // Built-in mode
    let input = match &input_path {
        Some(path) => std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error reading {path}: {e}");
            std::process::exit(1);
        }),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error reading stdin: {e}");
                std::process::exit(1);
            });
            buf
        }
    };

    let lang = lang
        .as_deref()
        .or_else(|| input_path.as_deref().and_then(detect_language))
        .unwrap_or_else(|| {
            eprintln!("cannot detect language — use --lang or --grammar");
            std::process::exit(1);
        });

    match format_builtin(lang, &input, &config) {
        Ok(output) => emit(&output, output_path.as_deref()),
        Err(e) => {
            eprintln!("gorg: {e}");
            std::process::exit(1);
        }
    }
}

fn emit(output: &str, path: Option<&str>) {
    if let Some(path) = path {
        std::fs::write(path, output).unwrap_or_else(|e| {
            eprintln!("error writing {path}: {e}");
            std::process::exit(1);
        });
    } else {
        let stdout = io::stdout().lock();
        let mut writer = BufWriter::new(stdout);
        writer.write_all(output.as_bytes()).unwrap();
    }
}
