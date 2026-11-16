//! The page side of video-in-video: which media a loaded page holds, and the still each swapped
//! video shows at a frame.

use serde_json::json;

use crate::Error;
use crate::media::{self, Media};
use crate::page::Page;

/// Runs before any script of the page: swaps each `<video>` for an `<img>` the capture loop
/// feeds, silences each `<audio>`, and exposes `window.__clap.scan` and `window.__clap.show`.
pub(crate) const PRELUDE: &str = include_str!("prelude.js");

impl Page<'_> {
    /// The `<video>` and `<audio>` elements of the loaded page, as `prelude.js` found them.
    pub(crate) fn media(&mut self) -> Result<Vec<Media>, Error> {
        let scan = self.eval("window.__clap.scan()")?;
        media::read(&scan, &self.browser.guard)
    }

    /// Hands each swapped video the still it shows now, and waits until every one has loaded.
    pub(crate) fn show(&mut self, stills: &[(usize, &str)]) -> Result<(), Error> {
        if stills.is_empty() {
            return Ok(());
        }
        self.eval(&format!("window.__clap.show({})", json!(stills))).map(|_| ())
    }
}
