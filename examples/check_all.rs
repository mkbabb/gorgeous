use gorgeous::PrinterConfig;
use gorgeous::json::prettify_json;
use gorgeous::css::prettify_css;
use gorgeous::bnf::prettify_bnf;
use gorgeous::ebnf::prettify_ebnf;
use gorgeous::bbnf::prettify_bbnf;

fn main() {
    let config = PrinterConfig::default();

    println!("=== JSON (compact input) ===");
    let j = r#"{"name":"Alice","age":30,"items":[1,2,3]}"#;
    println!(">>>\n{}\n<<<", prettify_json(j, &config).unwrap());

    println!("=== JSON (spaced input) ===");
    let j2 = r#"{ "name" : "Alice" , "age" : 30 }"#;
    println!(">>>\n{}\n<<<", prettify_json(j2, &config).unwrap());

    println!("=== CSS (minified) ===");
    let c = "html{line-height:1.15;-webkit-text-size-adjust:100%}body{margin:0}";
    println!(">>>\n{}\n<<<", prettify_css(c, &config).unwrap());

    println!("=== CSS (spaced) ===");
    let c2 = "body { color: red; font-size: 16px; }";
    println!(">>>\n{}\n<<<", prettify_css(c2, &config).unwrap());

    println!("=== CSS idempotency ===");
    let first = prettify_css(c2, &config).unwrap();
    let second = prettify_css(&first, &config).unwrap();
    if first.trim() == second.trim() {
        println!("PASS");
    } else {
        println!("FAIL");
        println!("first:  {:?}", first.trim());
        println!("second: {:?}", second.trim());
    }

    println!("\n=== BNF (compact) ===");
    let b = "<expr>::=<term>|<expr>\"+\"<term>";
    println!(">>>\n{}\n<<<", prettify_bnf(b, &config).unwrap());

    println!("=== EBNF (compact) ===");
    let e = "expr=term,{(\"+\"|\"-\"),term};";
    println!(">>>\n{}\n<<<", prettify_ebnf(e, &config).unwrap());

    println!("=== BBNF (compact) ===");
    let bb = r#"expr = term | expr , "+" , term ;"#;
    println!(">>>\n{}\n<<<", prettify_bbnf(bb, &config).unwrap());
}
