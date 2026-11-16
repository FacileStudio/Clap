//! What Clap adds to a compiled scene document: a stage that lays clips out and transitions
//! them at every `seek`, and media that follows it.

/// The stage runtime, wrapped around `window.__scene.seek`. After every seek it lays out each
/// clip from its stage attributes and applies the clip and scene transitions at that time, in
/// the preview and under the renderer alike. In the preview only, it also keeps each shown
/// `<video>` and `<audio>` at `data-in + (t - scene start - data-start)` with its volume, posts
/// `clap:boxes` to the parent and draws a `clap:ghost`. `docs/timeline.md` has the messages.
pub const RUNTIME_JS: &str = include_str!("runtime.js");

/// The stage stylesheet. A raw scene is a black stage with light text, and every element that
/// carries `data-track-index` is positioned, so the track the scene runtime writes as `z-index`
/// stacks clips with no CSS from the author. It defines the clip classes `add_clip` writes:
/// `clap-fill`, `clap-layer`, `clap-title`, `clap-caption` and `clap-lower`. The stage rules sit in
/// `:where()` and the classes are single class selectors placed before the author's styles, so
/// they beat the theme's element rules and lose to any selector an author writes.
pub const STAGE_CSS: &str = include_str!("stage.css");

fn before(html: &str, tag: &str, addition: &str) -> String {
    match html.find(tag) {
        Some(at) => format!("{}{addition}{}", &html[..at], &html[at..]),
        None => format!("{addition}{html}"),
    }
}

/// `html`, a document compiled by `facile-scene`, with [`STAGE_CSS`] at the end of its head and
/// [`RUNTIME_JS`] before `</body>`. The preview, the API's frame and every render use it, so a
/// project looks the same in all three.
pub fn with_runtime(html: &str) -> String {
    let styled = before(html, "</head>", &format!("<style>{STAGE_CSS}</style>\n"));
    let script = format!("<script>{RUNTIME_JS}</script>\n");
    match styled.rfind("</body>") {
        Some(at) => format!("{}{script}{}", &styled[..at], &styled[at..]),
        None => format!("{styled}{script}"),
    }
}
