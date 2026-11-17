//! The files of a project: the rules of an asset name, the places a source names one, and
//! `rename_media`, which points every one of those places at a new name.

use crate::Error;
use crate::body::readable;
use crate::patch::Patch;
use crate::references::references;
use crate::scan::Sketch;

/// The longest asset name, in bytes.
pub const MAX_ASSET_NAME: usize = 200;

fn segment(part: &str) -> bool {
    !part.is_empty() && !part.starts_with('.') && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

/// Whether `name` is an asset name: a relative path of at most [`MAX_ASSET_NAME`] bytes whose
/// segments hold ASCII letters, digits, `.`, `_` and `-` and never start with a dot. So no `..`,
/// no absolute path, no backslash, no scheme, nothing a URL or a header must escape. Every such
/// name is a path the scene format accepts.
pub fn valid_asset_name(name: &str) -> bool {
    name.len() <= MAX_ASSET_NAME && name.split('/').all(segment)
}

/// The asset name a source reference means, and the `#fragment` it carries: a leading `./` and
/// any `?query` are dropped.
pub fn asset_reference(path: &str) -> (&str, &str) {
    let path = path.strip_prefix("./").unwrap_or(path);
    let (rest, fragment) = path.find('#').map_or((path, ""), |at| path.split_at(at));
    (rest.split('?').next().unwrap_or(rest), fragment)
}

/// The project files an HTML fragment loads, by asset name, in order: what each resource
/// attribute, CSS `url()` and `@import` names once `asset_reference` reads it, fragments and
/// inline `data:` URIs left out. A name that breaks [`valid_asset_name`] is listed as written.
pub fn asset_names(html: &str) -> Vec<String> {
    let inline = |url: &str| url.get(..5).is_some_and(|head| head.eq_ignore_ascii_case("data:"));
    let urls =
        references(html, false).into_iter().map(|span| &html[span]).filter(|url| !url.starts_with('#') && !inline(url));
    urls.map(|url| asset_reference(url).0.to_owned()).collect()
}

fn found(text: &str, at: usize, raw: bool, (from, to): (&str, &str)) -> Result<Vec<Patch>, Error> {
    let mut patches = Vec::new();
    for span in references(text, !raw) {
        let written = &text[span.clone()];
        let name = asset_reference(written).0;
        if name == to {
            return Err(Error::NameTaken(to.to_owned()));
        }
        if name == from {
            let start = at + span.start + if written.starts_with("./") { 2 } else { 0 };
            patches.push(Patch { at: start..start + from.len(), text: to.to_owned() });
        }
    }
    Ok(patches)
}

/// `rename_media`. Every reference to the file `from` names `to` instead, its `./`, `?query` and
/// `#fragment` kept. Refused when either name is not an asset name, when both are the same, and
/// when the source already references `to`. A source that never names `from` is left as it is.
pub(crate) fn rename(source: &str, sketch: &Sketch, from: &str, to: &str) -> Result<Vec<Patch>, Error> {
    if let Some(bad) = [from, to].into_iter().find(|name| !valid_asset_name(name)) {
        return Err(Error::Invalid(format!(
            "`{bad}` is not an asset name: a relative path of letters, digits, `.`, `_` and `-`, such as assets/clip.mp4"
        )));
    }
    if from == to {
        return Err(Error::Invalid(format!("`{to}` is already the name of the file")));
    }
    let mut patches = Vec::new();
    for (unit, stage) in sketch.doc.units.iter().zip(&sketch.stages) {
        let raw = stage.scene.layout == scene::Layout::Raw.name();
        patches.extend(found(&readable(source, unit, raw), unit.at, raw, (from, to))?);
    }
    Ok(patches)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFUSED: [&str; 12] = [
        "",
        "../secret.png",
        "assets/../../etc/passwd",
        "/etc/passwd",
        "assets//a.png",
        "assets/.env",
        "C:\\a.png",
        "assets\\a.png",
        "https://x.test/a.png",
        "a b.png",
        "a%2e%2e/b.png",
        "assets/a.png?x=1",
    ];

    #[test]
    fn names_are_plain_relative_paths_the_scene_format_accepts() {
        for name in ["assets/chart.png", "logo.svg", "fonts/Inter-Bold_2.woff2", "a/b/c/d.mp4"] {
            assert!(valid_asset_name(name), "{name}");
            assert!(scene::parse(&format!("<img src=\"{name}\">\n")).is_ok(), "{name}");
        }
        for name in REFUSED {
            assert!(!valid_asset_name(name), "{name}");
        }
        assert!(!valid_asset_name(&format!("assets/{}.png", "a".repeat(MAX_ASSET_NAME))));
    }

    #[test]
    fn a_reference_names_its_asset_and_keeps_its_fragment() {
        assert_eq!(asset_reference("./assets/a.svg#icon"), ("assets/a.svg", "#icon"));
        assert_eq!(asset_reference("assets/a.png?v=2"), ("assets/a.png", ""));
        assert_eq!(asset_reference("a.svg?v=2#x"), ("a.svg", "#x"));
    }
}
