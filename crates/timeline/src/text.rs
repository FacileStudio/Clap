//! The words of a text clip: read from its element, and written back by `set_clip_text`.

use std::ops::Range;

use crate::Error;
use crate::model::Kind;
use crate::patch::Patch;
use crate::scan::Sketch;
use crate::tree::Element;

fn trimmed(source: &str, inner: Range<usize>) -> Range<usize> {
    let text = &source[inner.clone()];
    let start = inner.start + (text.len() - text.trim_start().len());
    let end = inner.end - (text.len() - text.trim_end().len());
    start..end.max(start)
}

/// The text a text clip shows: what sits between its tags, trimmed, with the escapes HTML
/// writes for `&`, `<`, `>`, `"` and `'` read back.
pub(crate) fn text_of(element: &Element, source: &str) -> String {
    let raw = &source[trimmed(source, element.inner.clone())];
    let decoded = raw.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'");
    decoded.replace("&amp;", "&")
}

/// `text` as the content of an element: `&`, `<` and `>` escaped.
pub(crate) fn escaped(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// `set_clip_text`. The trimmed content of a text clip becomes `text`, escaped; the spaces and
/// lines around it stay. Refused for any other kind of clip and for an empty text.
pub(crate) fn set(source: &str, sketch: &Sketch, id: &str, text: &str) -> Result<Vec<Patch>, Error> {
    let (_, item) = sketch.find(id)?;
    if item.clip.kind != Kind::Text {
        return Err(Error::Invalid(format!("clip `{id}` is not a text clip: only a text clip has text to set")));
    }
    let text = text.trim();
    if text.is_empty() {
        return Err(Error::Invalid(format!("clip `{id}` needs some text: delete the clip instead")));
    }
    if text_of(&item.element, source) == text {
        return Ok(Vec::new());
    }
    Ok(vec![Patch { at: trimmed(source, item.element.inner.clone()), text: escaped(text) }])
}
