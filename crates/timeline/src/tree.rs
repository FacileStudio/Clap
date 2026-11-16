//! Tags paired into elements.

use std::ops::Range;

use crate::html::{Attr, Tag, tags};

/// One element: its start tag and where it ends.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Element {
    pub tag: Tag,
    /// The bytes between the start tag and the end tag.
    pub inner: Range<usize>,
    /// The byte after the end tag, or after the start tag when the element has none.
    pub end: usize,
}

impl Element {
    /// The attribute called `name`.
    pub(crate) fn attr(&self, name: &str) -> Option<&Attr> {
        self.tag.attrs.iter().find(|attr| attr.name == name)
    }

    /// The value of the attribute called `name`, as written.
    pub(crate) fn value<'a>(&self, source: &'a str, name: &str) -> Option<&'a str> {
        self.attr(name).map(|attr| &source[attr.value.clone()])
    }

    /// The value of `name` as a finite number.
    pub(crate) fn number(&self, source: &str, name: &str) -> Option<f64> {
        self.value(source, name)?.trim().parse::<f64>().ok().filter(|number| number.is_finite())
    }
}

fn shifted(mut tag: Tag, base: usize) -> Tag {
    let moved = |range: &Range<usize>| range.start + base..range.end + base;
    tag.span = moved(&tag.span);
    for attr in &mut tag.attrs {
        attr.value = moved(&attr.value);
        attr.span = moved(&attr.span);
    }
    tag
}

fn close(found: &mut [Element], open: &mut Vec<usize>, tag: &Tag) {
    let Some(depth) = open.iter().rposition(|index| found[*index].tag.name == tag.name) else { return };
    let element = &mut found[open[depth]];
    element.inner = element.tag.span.end..tag.span.start;
    element.end = tag.span.end;
    open.truncate(depth);
}

/// The elements of `text`, in the order their start tags appear. Every byte range counts from
/// `base` bytes before `text`. An element whose end tag is missing ends with its start tag.
pub(crate) fn elements(text: &str, base: usize) -> Vec<Element> {
    let mut found: Vec<Element> = Vec::new();
    let mut open = Vec::new();
    for tag in tags(text).into_iter().map(|tag| shifted(tag, base)) {
        if tag.closing {
            close(&mut found, &mut open, &tag);
            continue;
        }
        if !tag.alone {
            open.push(found.len());
        }
        let end = tag.span.end;
        found.push(Element { tag, inner: end..end, end });
    }
    found
}
