//! What Clap adds to a compiled scene document: media that follows `seek`.

/// A script for the preview. `facile-scene`'s runtime hides a clip outside its time, but it does
/// not drive media: after every `window.__scene.seek(t)`, this sets each shown `<video>` and
/// `<audio>` to `data-in + (t - scene start - data-start)` and applies `data-volume`. A parent
/// frame posts `{type: "clap:playing", playing}` to let media play between seeks. The renderer
/// sets `window.__clapRender` and the script then does nothing: a render swaps frames in itself.
pub const RUNTIME_JS: &str = include_str!("runtime.js");

/// `html`, a document compiled by `facile-scene`, with [`RUNTIME_JS`] added before `</body>`.
pub fn with_runtime(html: &str) -> String {
    let script = format!("<script>{RUNTIME_JS}</script>\n");
    match html.rfind("</body>") {
        Some(at) => format!("{}{script}{}", &html[..at], &html[at..]),
        None => format!("{html}{script}"),
    }
}
