//! Pretty backend: interpret `Value` parse tree + `PrettyHints` from IR → pprint `Doc` → formatted string.
//!
//! This is a runtime interpreter — no codegen. It walks the parse tree produced by the
//! bytecode VM, consults the `@pretty` directives stored in `GrammarIR.rules[].meta.pretty`,
//! and builds a pprint `Doc` tree which is then rendered to a string.

use std::borrow::Cow;

use bbnf_ir::interpreter::Value;
use bbnf_ir::{GrammarIR, PrettyHints};
use parse_that::split::split_balanced;
use pprint::Doc;

use crate::PrinterConfig;

/// Format a parse result using the grammar's `@pretty` hints.
///
/// Returns `None` if the value is `Nil` or the input is empty.
pub fn format_value<'a>(
    ir: &'a GrammarIR,
    value: &Value,
    input: &'a str,
    printer: pprint::Printer,
) -> Option<String> {
    let doc = value_to_doc(ir, value, input)?;
    Some(pprint::pprint(doc, printer))
}

/// Format a parse result using `PrinterConfig` (convenience wrapper).
pub fn format_ir(ir: &GrammarIR, value: &Value, input: &str, config: &PrinterConfig) -> Option<String> {
    format_value(ir, value, input, config.to_printer())
}

/// Convert a Value parse tree node into a pprint Doc, consulting PrettyHints.
fn value_to_doc<'a>(ir: &'a GrammarIR, value: &Value, input: &'a str) -> Option<Doc<'a>> {
    match value {
        Value::Nil => None,

        Value::Span(start, end) => {
            let text = &input[*start as usize..*end as usize];
            if text.is_empty() {
                None
            } else {
                Some(Doc::String(Cow::Borrowed(text)))
            }
        }

        Value::Tagged { tag, span, children } => {
            // Look up the rule's pretty hints by name.
            // The `tag` is a StringId (index into ir.strings), not a rule index.
            let rule = ir.rules.iter().find(|r| r.name == *tag);
            let rule_name = ir.strings.get(*tag as usize).map(|s| s.as_str()).unwrap_or("??");
            let hints = rule.and_then(|r| r.meta.pretty.as_ref());
            let span_text = &input[span.0 as usize..span.1 as usize];

            // Recursively convert children to docs.
            let child_docs: Vec<Doc<'a>> = children
                .iter()
                .filter_map(|child| value_to_doc(ir, child, input))
                .collect();

            // Check if children cover the full span. If not (due to >> / << discarding
            // delimiters), use the full span text — the formatter needs the complete content.
            let children_span = children_span_range(children);
            let has_gaps = children_span.map_or(true, |(cs, ce)| cs > span.0 || ce < span.1);

            let _ = rule_name; // used in debug builds

            // Fall back to span text when children don't cover the full span
            // (gaps from >> / << or ?w). But if the node has explicit @pretty hints,
            // respect those hints — the user wants formatting control, not raw text.
            if child_docs.is_empty() || (has_gaps && hints.is_none()) {
                return if span_text.is_empty() {
                    None
                } else {
                    Some(apply_hints(Doc::String(Cow::Borrowed(span_text)), hints))
                };
            }

            // Flatten Array children into their elements when the node has hints.
            // Sequences like `(binding << comma) * , body` produce [Array[bindings], body].
            // The @pretty sep(", ") needs to see each binding as a sibling, not wrapped in an Array.
            let child_docs = if hints.is_some() {
                let mut flat = Vec::with_capacity(child_docs.len());
                for (i, child) in children.iter().enumerate() {
                    match child {
                        Value::Array(items) if items.len() > 1 => {
                            flat.extend(items.iter().filter_map(|v| value_to_doc(ir, v, input)));
                        }
                        _ => {
                            if let Some(d) = child_docs.get(i).and_then(|_| value_to_doc(ir, child, input)) {
                                flat.push(d);
                            }
                        }
                    }
                }
                flat
            } else {
                child_docs
            };

            // For nodes with group + indent (like func_call): construct a wrapping
            // Doc that puts the head (e.g., "LET(") on the first line, args indented
            // on subsequent lines, and the closing delimiter at the original indent.
            // This produces:  LET(\n  arg1, arg2,\n  arg3\n)
            if let Some(h) = hints {
                if h.group && h.indent && children.len() >= 2 && child_docs.len() >= 2 {
                    let last_is_close = match children.last() {
                        Some(Value::Span(s, e)) => (*e - *s) <= 2,
                        _ => false,
                    };
                    if last_is_close {
                        let mut docs_iter = child_docs.into_iter();
                        let head = docs_iter.next().unwrap();
                        let mut inner: Vec<Doc<'a>> = docs_iter.collect();
                        let close = inner.pop().unwrap();

                        let inner_doc = if inner.len() == 1 {
                            inner.into_iter().next().unwrap()
                        } else {
                            combine_docs(inner, None, input)
                        };

                        let break_line = Doc::IfBreak(
                            Box::new(Doc::Hardline),
                            Box::new(Doc::Null),
                        );

                        return Some(Doc::Group(Box::new(
                            head
                            + Doc::Indent(Box::new(break_line.clone() + inner_doc))
                            + break_line
                            + close
                        )));
                    }
                }
            }

            // Combine child docs based on hints.
            let combined = if child_docs.len() == 1 {
                child_docs.into_iter().next().unwrap()
            } else {
                combine_docs(child_docs, hints, input)
            };

            Some(apply_hints(combined, hints))
        }

        Value::Array(items) => {
            let child_docs: Vec<Doc<'a>> = items
                .iter()
                .filter_map(|item| value_to_doc(ir, item, input))
                .collect();

            if child_docs.is_empty() {
                None
            } else if child_docs.len() == 1 {
                Some(child_docs.into_iter().next().unwrap())
            } else {
                // Arrays don't have their own hints — just concatenate.
                Some(Doc::Concat(child_docs))
            }
        }
    }
}

/// Compute the (min_start, max_end) span range of all children.
fn children_span_range(children: &[Value]) -> Option<(u32, u32)> {
    let mut min_start = u32::MAX;
    let mut max_end = 0u32;
    let mut any = false;
    for child in children {
        match child {
            Value::Span(s, e) => { min_start = min_start.min(*s); max_end = max_end.max(*e); any = true; }
            Value::Tagged { span, .. } => { min_start = min_start.min(span.0); max_end = max_end.max(span.1); any = true; }
            Value::Array(items) => {
                if let Some((s, e)) = children_span_range(items) {
                    min_start = min_start.min(s); max_end = max_end.max(e); any = true;
                }
            }
            Value::Nil => {}
        }
    }
    if any { Some((min_start, max_end)) } else { None }
}

/// Combine multiple child docs using PrettyHints.
fn combine_docs<'a>(docs: Vec<Doc<'a>>, hints: Option<&PrettyHints>, input: &'a str) -> Doc<'a> {
    if let Some(hints) = hints {
        // Handle split("...") — format-time balanced splitting for opaque spans.
        if let Some(ref split_delim) = hints.split {
            return combine_with_split(docs, split_delim, hints, input);
        }

        // Handle sep("...") — custom separator.
        // Use SmartJoin when in a group context so items fill lines naturally
        // (e.g., LET name-value pairs stay together on the same line when they fit).
        if let Some(ref sep_str) = hints.sep {
            if hints.group {
                // SmartJoin fills lines: keeps short adjacent items together
                // (e.g., LET name-value pairs on the same line when they fit).
                // IfBreak(trimmed, full) — SmartJoin's break_left handles the
                // actual line break; no Hardline in the separator.
                let trimmed = sep_str.trim_end().to_string();
                let separator = Doc::IfBreak(
                    Box::new(Doc::String(Cow::Owned(trimmed))),
                    Box::new(Doc::String(Cow::Owned(sep_str.to_string()))),
                );
                return Doc::SmartJoin(Box::new((separator, docs)));
            } else {
                let separator = build_separator(sep_str, false);
                return Doc::Join(Box::new((separator, docs)));
            }
        }

        // Handle blankline — double hardline (blank line between elements).
        if hints.blankline {
            return Doc::Join(Box::new((Doc::Hardline + Doc::Hardline, docs)));
        }

        // Handle block — hardline between elements.
        if hints.block {
            return Doc::Join(Box::new((Doc::Hardline, docs)));
        }

        // Handle hardbreak — hardline between elements.
        if hints.hardbreak {
            return Doc::Join(Box::new((Doc::Hardline, docs)));
        }

        // Handle softbreak — softline between elements.
        if hints.softbreak {
            return Doc::Join(Box::new((Doc::Softline, docs)));
        }

        // Handle nobreak — space (never breaks).
        if hints.nobreak {
            return Doc::Join(Box::new((Doc::String(Cow::Borrowed(" ")), docs)));
        }

        // Handle compact — no separator.
        if hints.compact {
            return Doc::Concat(docs);
        }

        // Handle fast — hardline, skip SmartJoin.
        if hints.fast {
            return Doc::Join(Box::new((Doc::Hardline, docs)));
        }
    }

    // Default: concatenate without separator.
    Doc::Concat(docs)
}

/// Build a separator Doc from a sep("...") string.
///
/// In a group context, uses IfBreak: broken mode trims trailing spaces and adds Hardline,
/// flat mode uses the separator as-is.
fn build_separator<'a>(sep_str: &str, in_group: bool) -> Doc<'a> {
    if in_group {
        let trimmed = sep_str.trim_end().to_string();
        Doc::IfBreak(
            Box::new(Doc::String(Cow::Owned(trimmed)) + Doc::Hardline),
            Box::new(Doc::String(Cow::Owned(sep_str.to_string()))),
        )
    } else {
        Doc::String(Cow::Owned(sep_str.to_string()))
    }
}

/// Handle split("...") directive: split opaque span text by a delimiter,
/// then join with the appropriate separator.
fn combine_with_split<'a>(
    docs: Vec<Doc<'a>>,
    delim: &str,
    hints: &PrettyHints,
    _input: &'a str,
) -> Doc<'a> {
    // split() works on Span values — collect all text from docs.
    // For mixed content, fall through to regular join.
    let delim_byte = delim.as_bytes().first().copied().unwrap_or(b',');

    // For each doc that is a String, split it; otherwise keep as-is.
    let mut all_parts: Vec<Doc<'a>> = Vec::new();
    for doc in docs {
        match &doc {
            Doc::String(cow) => {
                let parts = split_balanced(cow, delim_byte);
                for part in parts.iter() {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() {
                        all_parts.push(Doc::String(Cow::Owned(trimmed.to_string())));
                    }
                }
            }
            _ => all_parts.push(doc),
        }
    }

    if all_parts.is_empty() {
        return Doc::Null;
    }

    // Build separator from sep hint or default to delim + space.
    let sep = if let Some(ref sep_str) = hints.sep {
        build_separator(sep_str, hints.group)
    } else {
        let default_sep = format!("{} ", delim);
        build_separator(&default_sep, hints.group)
    };

    Doc::Join(Box::new((sep, all_parts)))
}

/// Apply formatting hints (group, indent, dedent) to a doc.
fn apply_hints<'a>(doc: Doc<'a>, hints: Option<&PrettyHints>) -> Doc<'a> {
    let Some(hints) = hints else {
        return doc;
    };

    // Don't apply any formatting if hints are turned off.
    if hints.off {
        return doc;
    }

    let mut result = doc;

    // Apply indent/dedent.
    if hints.indent {
        result = Doc::Indent(Box::new(result));
    }
    if hints.dedent {
        result = Doc::Dedent(Box::new(result));
    }

    // Apply group.
    if hints.group {
        result = Doc::Group(Box::new(result));
    }

    result
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use bbnf_ir::{GrammarIR, IrNode, IrRule, PrettyHints, RuleMeta};

    /// Build a minimal IR with one rule that has the given pretty hints.
    fn make_ir(hints: PrettyHints) -> GrammarIR {
        GrammarIR {
            entry: 0,
            rules: vec![IrRule {
                id: 0,
                name: 0,
                body: IrNode::Epsilon,
                meta: RuleMeta {
                    pretty: Some(hints),
                    ..RuleMeta::default()
                },
            }],
            strings: vec!["root".into()],
            fns: vec![],
            types: vec![],
            follow_sets: HashMap::new(),
        }
    }

    /// Build a Tagged value with N Span children at distinct offsets.
    fn tagged_spans(input: &str, n: usize) -> Value {
        let chunk = input.len() / n;
        let children: Vec<Value> = (0..n)
            .map(|i| Value::Span((i * chunk) as u32, ((i + 1) * chunk) as u32))
            .collect();
        Value::Tagged {
            tag: 0,
            span: (0, input.len() as u32),
            children,
        }
    }

    /// Format a tagged value with the given hints.
    fn fmt(hints: PrettyHints, input: &str, n: usize, max_width: usize) -> String {
        let ir = make_ir(hints);
        let value = tagged_spans(input, n);
        let printer = pprint::Printer::new(max_width, 2, false);
        format_value(&ir, &value, input, printer).unwrap()
    }

    fn default_hints() -> PrettyHints {
        PrettyHints::default()
    }

    #[test]
    fn hint_blankline() {
        let hints = PrettyHints { blankline: true, ..default_hints() };
        let output = fmt(hints, "aaabbbccc", 3, 80);
        assert!(
            output.contains("\n\n"),
            "blankline should produce double newline, got: {:?}",
            output
        );
    }

    #[test]
    fn hint_block() {
        let hints = PrettyHints { block: true, ..default_hints() };
        let output = fmt(hints, "aaabbb", 2, 80);
        assert!(
            output.contains('\n'),
            "block should produce newlines, got: {:?}",
            output
        );
        // Should NOT produce double newline (that's blankline).
        assert!(
            !output.contains("\n\n"),
            "block should not produce blank lines, got: {:?}",
            output
        );
    }

    #[test]
    fn hint_sep() {
        let hints = PrettyHints {
            sep: Some(", ".to_string()),
            ..default_hints()
        };
        let output = fmt(hints, "aaabbbccc", 3, 80);
        assert!(
            output.contains(", "),
            "sep should produce comma-space separator, got: {:?}",
            output
        );
    }

    #[test]
    fn hint_group_sep() {
        let hints = PrettyHints {
            group: true,
            sep: Some(", ".to_string()),
            ..default_hints()
        };
        // Narrow width forces break.
        let output = fmt(hints, "alphbetagram", 3, 10);
        assert!(
            output.contains('\n'),
            "group sep should break when width exceeded, got: {:?}",
            output
        );
    }

    #[test]
    fn hint_compact() {
        let hints = PrettyHints { compact: true, ..default_hints() };
        let output = fmt(hints, "aaabbb", 2, 80);
        assert_eq!(output, "aaabbb", "compact should concatenate without separator");
    }

    #[test]
    fn hint_indent_group() {
        let hints = PrettyHints {
            indent: true,
            group: true,
            sep: Some(", ".to_string()),
            ..default_hints()
        };
        let output = fmt(hints, "alphbetagram", 3, 10);
        let has_indent = output.lines().skip(1).any(|l| l.starts_with("  "));
        assert!(
            has_indent,
            "indent group should produce indented lines, got: {:?}",
            output
        );
    }

    #[test]
    fn hint_off() {
        // off disables group/indent/dedent wrapping.
        let hints = PrettyHints {
            off: true,
            group: true,
            indent: true,
            ..default_hints()
        };
        let output = fmt(hints, "aaabbb", 2, 80);
        // Should just concatenate — no group or indent applied.
        assert_eq!(output, "aaabbb", "off should disable formatting wrappers");
    }

    #[test]
    fn hint_split() {
        let hints = PrettyHints {
            split: Some(",".to_string()),
            sep: Some(", ".to_string()),
            ..default_hints()
        };
        // Two span children — split breaks the comma-containing spans.
        let input = "a,bc,d";
        let ir = make_ir(hints);
        let value = Value::Tagged {
            tag: 0,
            span: (0, 6),
            children: vec![Value::Span(0, 3), Value::Span(3, 6)],
        };
        let printer = pprint::Printer::new(80, 2, false);
        let output = format_value(&ir, &value, input, printer).unwrap();
        assert!(
            output.contains(", "),
            "split should separate by delimiter, got: {:?}",
            output
        );
    }

    #[test]
    fn hint_softbreak() {
        // Softline in flat mode renders as nothing — items are concatenated.
        let hints = PrettyHints { softbreak: true, ..default_hints() };
        let output = fmt(hints, "aaabbb", 2, 80);
        // In flat mode (no group), softline is empty — same as compact.
        assert_eq!(output, "aaabbb", "softbreak flat should concatenate");
    }

    #[test]
    fn hint_nobreak() {
        let hints = PrettyHints { nobreak: true, ..default_hints() };
        let output = fmt(hints, "aaabbb", 2, 80);
        assert!(
            output.contains(' '),
            "nobreak should join with space, got: {:?}",
            output
        );
        assert!(
            !output.contains('\n'),
            "nobreak should never break, got: {:?}",
            output
        );
    }

    #[test]
    fn hint_fast() {
        let hints = PrettyHints { fast: true, ..default_hints() };
        let output = fmt(hints, "aaabbb", 2, 80);
        assert!(
            output.contains('\n'),
            "fast should produce newlines, got: {:?}",
            output
        );
    }

    #[test]
    fn format_ir_convenience() {
        let hints = PrettyHints { block: true, ..default_hints() };
        let ir = make_ir(hints);
        let value = tagged_spans("aaabbb", 2);
        let config = PrinterConfig::new(80, 2);
        let output = format_ir(&ir, &value, "aaabbb", &config).unwrap();
        assert!(output.contains('\n'), "format_ir should work like format_value");
    }
}
