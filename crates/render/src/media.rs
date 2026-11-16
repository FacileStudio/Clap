//! The `<video>` and `<audio>` elements of a loaded page, and where each sits on the timeline.

use std::path::PathBuf;

use serde_json::Value;

use crate::Error;
use crate::guard::Guard;

/// One media element, as the page reports it once loaded.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Media {
    /// Its position in the page's own list, which `show` takes back.
    pub index: usize,
    pub video: bool,
    /// The file, canonical and inside the bundle.
    pub path: PathBuf,
    /// Seconds from the start of the project to the start of the clip.
    pub at: f64,
    /// Seconds the clip is on, cut at the end of its scene.
    pub duration: f64,
    /// `data-in`: seconds into the file where the clip starts.
    pub offset: f64,
    /// `data-volume`, 1 without it.
    pub volume: f64,
    pub muted: bool,
    /// Seconds the sound rises from silence at the start of the clip (`data-enter="fade"`).
    pub fade_in: f64,
    /// Seconds the sound falls to silence at the end of the clip (`data-exit="fade"`).
    pub fade_out: f64,
    /// Seconds a video keeps its last frame after the clip, while the next scene comes in over
    /// it.
    pub hold: f64,
}

/// The part of one clip a render uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Used {
    /// Seconds from the start of the render.
    pub delay: f64,
    /// Seconds into the file.
    pub seek: f64,
    pub length: f64,
}

impl Media {
    /// The part of the clip inside the seconds `from..to` of the project.
    pub(crate) fn used(&self, from: f64, to: f64) -> Option<Used> {
        let start = self.at.max(from);
        let end = (self.at + self.duration).min(to);
        (end - start > 1e-6).then_some(Used {
            delay: start - from,
            seek: self.offset + start - self.at,
            length: end - start,
        })
    }
}

fn one(index: usize, item: &Value, guard: &Guard) -> Result<Option<Media>, Error> {
    let url = item["url"].as_str().unwrap_or_default();
    if url.is_empty() {
        return Ok(None);
    }
    let path = guard.file(url).ok_or_else(|| Error::Job(format!("media {url} is not a file of the bundle")))?;
    let number = |name: &str| item[name].as_f64().filter(|value| value.is_finite()).unwrap_or(0.0);
    Ok(Some(Media {
        index,
        video: item["video"].as_bool().unwrap_or(false),
        path,
        at: number("at"),
        duration: number("duration"),
        offset: number("offset"),
        volume: number("volume"),
        fade_in: number("fade_in").max(0.0),
        fade_out: number("fade_out").max(0.0),
        hold: number("hold").max(0.0),
        muted: item["muted"].as_bool().unwrap_or(false),
    }))
}

/// Reads what `window.__clap.scan()` answered. A source outside the bundle fails the render.
pub(crate) fn read(scan: &Value, guard: &Guard) -> Result<Vec<Media>, Error> {
    let items = scan.as_array().map(Vec::as_slice).unwrap_or_default();
    let found: Result<Vec<Option<Media>>, Error> =
        items.iter().enumerate().map(|(index, item)| one(index, item, guard)).collect();
    Ok(found?.into_iter().flatten().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clip_is_cut_to_the_rendered_seconds() {
        let media = Media {
            index: 0,
            video: true,
            path: "/a.mp4".into(),
            at: 2.0,
            duration: 4.0,
            offset: 1.0,
            volume: 1.0,
            muted: false,
            fade_in: 0.0,
            fade_out: 0.0,
            hold: 0.0,
        };
        assert_eq!(media.used(0.0, 10.0), Some(Used { delay: 2.0, seek: 1.0, length: 4.0 }));
        assert_eq!(media.used(3.0, 5.0), Some(Used { delay: 0.0, seek: 2.0, length: 2.0 }));
        assert_eq!(media.used(6.0, 10.0), None);
        assert_eq!(media.used(0.0, 2.0), None);
    }
}
