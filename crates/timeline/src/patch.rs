//! Text replacements, and the two an edit is made of: one attribute, one number.

use std::ops::Range;

use crate::tree::Element;

/// Replaces the bytes `at` with `text`.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Patch {
    pub at: Range<usize>,
    pub text: String,
}

/// The bytes `within` of `source` with `patches` applied. Patches outside `within` are ignored;
/// patches at one position keep their order.
pub(crate) fn patched(source: &str, within: Range<usize>, mut patches: Vec<Patch>) -> String {
    patches.retain(|patch| patch.at.start >= within.start && patch.at.end <= within.end);
    patches.sort_by_key(|patch| patch.at.start);
    let mut out = String::new();
    let mut done = within.start;
    for patch in patches {
        out.push_str(&source[done..patch.at.start]);
        out.push_str(&patch.text);
        done = patch.at.end;
    }
    out.push_str(&source[done..within.end]);
    out
}

/// Seconds as the source writes them: milliseconds at most, no trailing zero.
pub(crate) fn seconds(value: f64) -> String {
    let text = format!("{:.3}", value + 0.0);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() || text == "-0" { "0".to_owned() } else { text.to_owned() }
}

/// What goes inside a double-quoted attribute.
pub(crate) fn quoted(value: &str) -> String {
    value.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")
}

/// Sets attribute `name` of `element` to `value`. A missing attribute is added after
/// `data-start`, or after the tag name.
pub(crate) fn attribute(element: &Element, name: &str, value: &str) -> Patch {
    let value = quoted(value);
    match element.attr(name) {
        Some(attr) if attr.span.end == attr.value.end => {
            Patch { at: attr.span.clone(), text: format!("{name}=\"{value}\"") }
        }
        Some(attr) => Patch { at: attr.value.clone(), text: value },
        None => {
            let after_name = element.tag.span.start + 1 + element.tag.name.len();
            let anchor = element.attr("data-start").map_or(after_name, |start| start.span.end);
            Patch { at: anchor..anchor, text: format!(" {name}=\"{value}\"") }
        }
    }
}

/// Sets a numeric attribute, or nothing when it already holds that number.
pub(crate) fn number(source: &str, element: &Element, name: &str, value: f64) -> Option<Patch> {
    let written = seconds(value);
    let same = element.number(source, name).is_some_and(|now| seconds(now) == written);
    (!same).then(|| attribute(element, name, &written))
}
