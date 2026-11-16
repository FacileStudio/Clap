//! A tolerant reader of HTML tags that keeps the byte range of everything it reads, so an edit
//! can replace one attribute value and nothing else.

use std::ops::Range;

/// Elements that never have an end tag.
const VOID: [&str; 8] = ["img", "br", "hr", "input", "source", "track", "meta", "link"];
/// Elements whose content is not HTML.
const RAW: [&str; 2] = ["script", "style"];

/// One attribute of a start tag.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Attr {
    /// The name, lowercase.
    pub name: String,
    /// The bytes of the value, quotes excluded. Empty, at the end of the name, without a value.
    pub value: Range<usize>,
    /// The bytes of the whole attribute.
    pub span: Range<usize>,
}

/// A start tag or an end tag.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Tag {
    /// The element name, lowercase.
    pub name: String,
    /// The bytes of the tag, `<` to `>`.
    pub span: Range<usize>,
    pub attrs: Vec<Attr>,
    /// An end tag: `</name>`.
    pub closing: bool,
    /// A start tag that opens nothing: a void element or `<name />`.
    pub alone: bool,
}

struct Reader<'a> {
    text: &'a str,
    at: usize,
}

impl Reader<'_> {
    fn skip(&mut self, stop: impl Fn(u8) -> bool) -> Range<usize> {
        let start = self.at;
        let bytes = self.text.as_bytes();
        while self.at < bytes.len() && !stop(bytes[self.at]) {
            self.at += 1;
        }
        start..self.at
    }

    fn byte(&self) -> Option<u8> {
        self.text.as_bytes().get(self.at).copied()
    }

    fn value(&mut self) -> Range<usize> {
        match self.byte() {
            Some(quote @ (b'"' | b'\'')) => {
                self.at += 1;
                let value = self.skip(|b| b == quote);
                self.at = (self.at + 1).min(self.text.len());
                value
            }
            _ => self.skip(|b| b.is_ascii_whitespace() || b == b'>'),
        }
    }

    fn attr(&mut self) -> Option<Attr> {
        self.skip(|b| !b.is_ascii_whitespace() && b != b'/');
        let name = self.skip(|b| b.is_ascii_whitespace() || matches!(b, b'=' | b'>' | b'/'));
        if name.is_empty() {
            return None;
        }
        let mut value = name.end..name.end;
        let rest = self.at;
        self.skip(|b| !b.is_ascii_whitespace());
        if self.byte() == Some(b'=') {
            self.at += 1;
            self.skip(|b| !b.is_ascii_whitespace());
            value = self.value();
        } else {
            self.at = rest;
        }
        Some(Attr { name: self.text[name.clone()].to_ascii_lowercase(), value, span: name.start..self.at })
    }

    fn tag(&mut self) -> Option<Tag> {
        let start = self.at;
        let closing = self.text[start..].starts_with("</");
        self.at += 1 + usize::from(closing);
        let name = self.skip(|b| !b.is_ascii_alphanumeric() && b != b'-');
        if name.is_empty() || !self.text.as_bytes()[name.start].is_ascii_alphabetic() {
            return None;
        }
        let name = self.text[name].to_ascii_lowercase();
        let mut attrs = Vec::new();
        while let Some(attr) = self.attr() {
            attrs.push(attr);
        }
        let slashed = self.text[..self.at].ends_with('/');
        self.at = (self.at + 1).min(self.text.len());
        let alone = !closing && (slashed || VOID.contains(&name.as_str()));
        Some(Tag { name, span: start..self.at, attrs, closing, alone })
    }

    fn after(&mut self, end: &str) {
        let found = self.text[self.at..].to_ascii_lowercase().find(end);
        self.at = found.map_or(self.text.len(), |offset| self.at + offset);
    }
}

/// Every tag of `text`, in order. Comments and the content of `<script>` and `<style>` are
/// skipped.
pub(crate) fn tags(text: &str) -> Vec<Tag> {
    let mut reader = Reader { text, at: 0 };
    let mut found = Vec::new();
    while let Some(offset) = text[reader.at..].find('<') {
        reader.at += offset;
        if text[reader.at..].starts_with("<!--") {
            reader.after("-->");
            continue;
        }
        let Some(tag) = reader.tag() else { continue };
        if !tag.closing && RAW.contains(&tag.name.as_str()) {
            reader.after(&format!("</{}", tag.name));
        }
        found.push(tag);
    }
    found
}
