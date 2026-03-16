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
            // However, gaps that are purely whitespace (from `?w` / TrimWs) are harmless —
            // TrimWs advances the offset past whitespace without producing a child value,
            // so the parent span extends beyond the children, but the gap is just whitespace.
            // In that case, prefer the structured children so formatting hints can break lines.
            let children_span = children_span_range(children);
            let has_gaps = children_span.map_or(true, |(cs, ce)| {
                let start_gap = &input[span.0 as usize..cs as usize];
                let end_gap = &input[ce as usize..span.1 as usize];
                // Only meaningful gap if non-whitespace content is missing from children
                !start_gap.bytes().all(|b| b.is_ascii_whitespace())
                    || !end_gap.bytes().all(|b| b.is_ascii_whitespace())
            });

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
                for child in children.iter() {
                    match child {
                        Value::Array(items) if items.len() > 1 => {
                            flat.extend(items.iter().filter_map(|v| value_to_doc(ir, v, input)));
                        }
                        _ => {
                            if let Some(d) = value_to_doc(ir, child, input) {
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
                        Some(Value::Span(s, e)) => {
                            let text = &input[*s as usize..*e as usize];
                            matches!(text, ")" | "}" | "]" | ">")
                        }
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
        // Check `off` before anything else — it suppresses all formatting.
        if hints.off {
            return Doc::Concat(docs);
        }

        // Handle split("...") — format-time balanced splitting for opaque spans.
        if let Some(ref split_delim) = hints.split {
            return combine_with_split(docs, split_delim, hints, input);
        }

        // Handle sep("...") — custom separator.
        if let Some(ref sep_str) = hints.sep {
            if hints.hardbreak || hints.block {
                // Non-filling: each item on its own line with trimmed separator.
                // e.g., LET args: `scale, DURATION,\n  psus, FILTER(...),`
                let trimmed = sep_str.trim_end().to_string();
                let separator = Doc::String(Cow::Owned(trimmed)) + Doc::Hardline;
                return Doc::Join(Box::new((separator, docs)));
            } else if hints.group {
                // SmartJoin fills lines: keeps short adjacent items together
                // (e.g., function args on the same line when they fit).
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
