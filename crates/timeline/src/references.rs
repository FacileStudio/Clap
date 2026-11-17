//! The places a scene body names a file to load, as the scene compiler finds them: resource
//! attributes, CSS `url()` and `@import`, and markdown images.

use std::ops::Range;

use crate::html::{Attr, Tag, tags};

/// The elements whose `href` loads a file.
const LINKING: [&str; 4] = ["link", "image", "use", "script"];

fn loads(tag: &Tag, attr: &Attr) -> bool {
    match attr.name.as_str() {
        "src" | "poster" | "srcset" => true,
        "href" | "xlink:href" => LINKING.contains(&tag.name.as_str()),
        "data" => tag.name == "object",
        _ => false,
    }
}

fn trimmed(text: &str, span: Range<usize>) -> Range<usize> {
    let quote = |c: char| c.is_whitespace() || c == '"' || c == '\'';
    let inner = text[span.clone()].trim_matches(quote);
    let start = span.start + text[span].find(inner).unwrap_or_default();
    start..start + inner.len()
}

fn candidates(text: &str, attr: &Attr, found: &mut Vec<Range<usize>>) {
    if attr.name != "srcset" {
        return found.push(attr.value.clone());
    }
    let mut start = attr.value.start;
    for part in text[attr.value.clone()].split(',') {
        let url = part.split_whitespace().next().unwrap_or_default();
        let at = start + part.find(url).unwrap_or_default();
        found.push(trimmed(text, at..at + url.len()));
        start += part.len() + 1;
    }
}

fn styles(text: &str, found: &mut Vec<Range<usize>>) {
    for (at, _) in text.match_indices("url(") {
        let open = at + 4;
        let close = text[open..].find(')').map_or(text.len(), |end| open + end);
        found.push(trimmed(text, open..close));
    }
    for (at, _) in text.match_indices("@import") {
        let rest = text[at + 7..].trim_start();
        let quoted = rest.starts_with(['"', '\'']).then(|| rest[1..].find(['"', '\''])).flatten();
        if let Some(end) = quoted {
            let start = text.len() - rest.len() + 1;
            found.push(start..start + end);
        }
    }
}

fn images(text: &str, found: &mut Vec<Range<usize>>) {
    for (at, _) in text.match_indices("![") {
        let line = text[at..].find('\n').map_or(text.len(), |end| at + end);
        let Some(open) = text[at..line].find("](").map(|offset| at + offset + 2) else { continue };
        let rest = &text[open..line];
        let skipped = rest.len() - rest.trim_start().len();
        let bracketed = usize::from(rest.trim_start().starts_with('<'));
        let start = open + skipped + bracketed;
        let length = text[start..line].find(|c: char| c.is_whitespace() || c == ')' || c == '>');
        found.push(start..start + length.unwrap_or(line - start));
    }
}

/// Every byte range of `text` that names a file the compiled scene loads: the resource
/// attributes, CSS `url()` and `@import`, and in a `markdown` body the image destinations.
pub(crate) fn references(text: &str, markdown: bool) -> Vec<Range<usize>> {
    let mut found = Vec::new();
    for tag in tags(text) {
        for attr in tag.attrs.iter().filter(|attr| loads(&tag, attr)) {
            candidates(text, attr, &mut found);
        }
    }
    styles(text, &mut found);
    if markdown {
        images(text, &mut found);
    }
    found.retain(|span| !span.is_empty());
    found.sort_by_key(|span| span.start);
    found.dedup_by_key(|span| span.start);
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_loading_place_is_found_and_nothing_else() {
        let text = "<img srcset=\"a.png 1x, ./b.png 2x\" alt=\"c.png\"><a href=\"d.png\">x</a>\
                    <use href=\"e.svg#i\"/><p style=\"background: url( 'f.png' )\">@import \"g.css\";</p>\n![x](<h.png> \"t\")";
        let found: Vec<&str> = references(text, true).into_iter().map(|span| &text[span]).collect();
        assert_eq!(found, ["a.png", "./b.png", "e.svg#i", "f.png", "g.css", "h.png"]);
        assert_eq!(references("![x](h.png)", false), Vec::<Range<usize>>::new());
    }
}
