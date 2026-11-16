//! The filmstrip of a video: one frame every few seconds, 90 pixels high, side by side in one
//! JPEG, so the timeline draws a trimmed clip from the frames it really shows.

use std::ffi::OsString;
use std::path::Path;

/// The most frames a strip holds.
pub const MAX_TILES: i64 = 60;

/// The shortest time between two frames of a strip, in milliseconds.
pub const MIN_STEP_MS: i64 = 2000;

/// The height of every frame of a strip, in pixels.
pub const HEIGHT: u32 = 90;

/// How a strip cuts a video: how many frames, and the milliseconds between two.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    /// How many frames.
    pub tiles: i64,
    /// The milliseconds between two frames.
    pub step_ms: i64,
}

impl Plan {
    /// The seconds between two frames, as the API answers it.
    pub fn step(self) -> f64 {
        self.step_ms as f64 / 1000.0
    }
}

/// The strip of a video `duration_ms` long: a frame every `max(2 s, duration / 60)`, at most
/// [`MAX_TILES`], frame `i` taken at `i * step`. No length, no strip.
pub fn plan(duration_ms: Option<i64>) -> Option<Plan> {
    let duration = duration_ms.filter(|duration| *duration > 0)?;
    let step_ms = MIN_STEP_MS.max((duration + MAX_TILES - 1) / MAX_TILES);
    let tiles = ((duration + step_ms - 1) / step_ms).clamp(1, MAX_TILES);
    Some(Plan { tiles, step_ms })
}

/// The arguments that write the strip of `input` cut by `plan` to `output`.
pub fn strip_args(input: &Path, plan: Plan, output: &Path) -> Vec<OsString> {
    let filter = format!("fps=1000/{},scale=-2:{HEIGHT},setsar=1,tile={}x1", plan.step_ms, plan.tiles);
    let after = ["-map", "0:v:0", "-an", "-vf", &filter, "-frames:v", "1", "-q:v", "6", "-f", "mjpeg"];
    crate::framed(&[], input, &after, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_strip_takes_a_frame_every_two_seconds_and_never_more_than_sixty() {
        assert_eq!(plan(None), None);
        assert_eq!(plan(Some(0)), None);
        assert_eq!(plan(Some(500)), Some(Plan { tiles: 1, step_ms: 2000 }));
        assert_eq!(plan(Some(2000)), Some(Plan { tiles: 1, step_ms: 2000 }));
        assert_eq!(plan(Some(8000)), Some(Plan { tiles: 4, step_ms: 2000 }));
        assert_eq!(plan(Some(8001)), Some(Plan { tiles: 5, step_ms: 2000 }));
        assert_eq!(plan(Some(120_000)), Some(Plan { tiles: 60, step_ms: 2000 }));
        assert_eq!(plan(Some(3_600_000)), Some(Plan { tiles: 60, step_ms: 60_000 }));
        assert_eq!(plan(Some(3_600_001)), Some(Plan { tiles: 60, step_ms: 60_001 }));
        let words: Vec<String> =
            strip_args(Path::new("/d/in put"), Plan { tiles: 4, step_ms: 2000 }, Path::new("/d/o.jpg"))
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
        assert!(words.windows(2).any(|pair| pair == ["-max_pixels", "8847360"]), "{words:?}");
        assert!(words.contains(&"fps=1000/2000,scale=-2:90,setsar=1,tile=4x1".to_owned()), "{words:?}");
        assert_eq!(
            (words[words.len() - 1].as_str(), words.iter().position(|word| word == "/d/in put")),
            ("/d/o.jpg", Some(11))
        );
    }
}
