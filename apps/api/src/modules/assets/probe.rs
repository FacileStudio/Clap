use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use super::tools::{self, Tools};

/// How long `ffprobe` gets for one file.
pub const TIMEOUT: Duration = Duration::from_secs(30);

/// What `ffprobe` says about one file.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Probe {
    /// The length in milliseconds, when the container has one.
    pub duration_ms: Option<i64>,
    /// The width of the first picture stream, in pixels.
    pub width: Option<i32>,
    /// The height of the first picture stream, in pixels.
    pub height: Option<i32>,
    /// The frame rate of the first picture stream.
    pub fps: Option<f64>,
    /// Whether the file has a sound stream.
    pub has_audio: bool,
    /// Whether the file has a picture stream that is not cover art.
    pub has_video: bool,
}

#[derive(Deserialize, Default)]
struct Disposition {
    #[serde(default)]
    attached_pic: u8,
}

#[derive(Deserialize)]
struct Stream {
    #[serde(default)]
    codec_type: String,
    width: Option<i32>,
    height: Option<i32>,
    avg_frame_rate: Option<String>,
    duration: Option<String>,
    #[serde(default)]
    disposition: Disposition,
}

#[derive(Deserialize, Default)]
struct Format {
    duration: Option<String>,
}

#[derive(Deserialize)]
struct Report {
    #[serde(default)]
    streams: Vec<Stream>,
    #[serde(default)]
    format: Format,
}

fn millis(seconds: Option<&str>) -> Option<i64> {
    let seconds: f64 = seconds?.parse().ok()?;
    (seconds.is_finite() && seconds > 0.0).then(|| (seconds * 1000.0).round() as i64)
}

fn rate(raw: Option<&str>) -> Option<f64> {
    let (above, below) = raw?.split_once('/')?;
    let (above, below): (f64, f64) = (above.parse().ok()?, below.parse().ok()?);
    (above > 0.0 && below > 0.0).then(|| (above / below * 1000.0).round() / 1000.0)
}

/// The arguments that make `ffprobe` describe `path` as JSON, reading only local files.
pub fn args(path: &Path) -> Vec<OsString> {
    let mut args = tools::argv(&["-v", "error", "-protocol_whitelist", "file", "-print_format", "json"]);
    args.extend(tools::argv(&["-show_format", "-show_streams", "-i"]));
    args.push(path.into());
    args
}

/// Reads `ffprobe`'s JSON report.
pub fn parse(json: &[u8]) -> Result<Probe, String> {
    let report: Report =
        serde_json::from_slice(json).map_err(|error| format!("ffprobe answered no report: {error}"))?;
    let picture = report.streams.iter().find(|s| s.codec_type == "video" && s.disposition.attached_pic == 0);
    let longest = report.streams.iter().filter_map(|s| millis(s.duration.as_deref())).max();
    Ok(Probe {
        duration_ms: millis(report.format.duration.as_deref()).or(longest),
        width: picture.and_then(|s| s.width),
        height: picture.and_then(|s| s.height),
        fps: picture.and_then(|s| rate(s.avg_frame_rate.as_deref())),
        has_audio: report.streams.iter().any(|s| s.codec_type == "audio"),
        has_video: picture.is_some(),
    })
}

/// Runs `ffprobe` on `path`, within [`TIMEOUT`].
pub async fn probe(tools: &Tools, path: &Path) -> Result<Probe, String> {
    parse(&tools::run(&tools.ffprobe, &args(path), TIMEOUT).await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPORT: &str = r#"{"streams": [
        {"codec_type": "video", "width": 320, "height": 240, "avg_frame_rate": "30000/1001", "duration": "2.002"},
        {"codec_type": "audio", "avg_frame_rate": "0/0", "duration": "2.0"}],
        "format": {"duration": "2.002000"}}"#;

    #[test]
    fn a_report_gives_duration_size_rate_and_sound() {
        let probe = parse(REPORT.as_bytes()).unwrap();
        assert_eq!((probe.duration_ms, probe.width, probe.height), (Some(2002), Some(320), Some(240)));
        assert_eq!((probe.fps, probe.has_audio, probe.has_video), (Some(29.97), true, true));
    }

    #[test]
    fn cover_art_is_not_a_picture_and_garbage_is_refused() {
        let cover = r#"{"streams": [{"codec_type": "video", "width": 500, "height": 500,
            "disposition": {"attached_pic": 1}}, {"codec_type": "audio", "duration": "3.5"}], "format": {}}"#;
        let probe = parse(cover.as_bytes()).unwrap();
        assert_eq!((probe.has_video, probe.width, probe.duration_ms), (false, None, Some(3500)));
        assert!(parse(b"not json").is_err());
    }

    #[test]
    fn the_file_is_one_whole_argument_and_only_local_files_are_read() {
        let args = args(Path::new("/data/assets/ab/a b;rm -rf"));
        assert_eq!(args.last().unwrap(), "/data/assets/ab/a b;rm -rf");
        assert!(args.windows(2).any(|pair| pair[0] == "-protocol_whitelist" && pair[1] == "file"));
    }
}
