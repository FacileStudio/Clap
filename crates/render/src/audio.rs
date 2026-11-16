//! The sound of a render: every `<audio>` and every video that has a sound stream, each at its
//! place on the timeline, mixed by one ffmpeg filter graph.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::Error;
use crate::media::{Media, Used};
use crate::proc::run;
use crate::tools::Tools;

/// The sample rate of the mix.
const RATE: f64 = 48_000.0;

/// One input of the mix.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Track {
    pub path: PathBuf,
    pub used: Used,
    pub volume: f64,
}

fn sounds(tools: &Tools, media: &Media) -> Result<bool, Error> {
    let mut command = Command::new(&tools.ffmpeg);
    command.args(["-hide_banner", "-nostdin", "-i"]).arg(&media.path);
    let said = run(&mut command, Duration::from_secs(30), &mut || Ok(()))?.said;
    let name = media.path.file_name().unwrap_or_default().to_string_lossy().into_owned();
    match said.lines().any(|line| line.contains("Stream #") && line.contains("Audio:")) {
        false if !media.video => Err(Error::Job(format!("{name} has no sound stream"))),
        heard => Ok(heard),
    }
}

/// The tracks to mix for the seconds `from..to` of the project. A muted element, a volume of
/// zero and a video without sound give none.
pub(crate) fn tracks(tools: &Tools, media: &[Media], from: f64, to: f64) -> Result<Vec<Track>, Error> {
    let mut found = Vec::new();
    for clip in media.iter().filter(|clip| !clip.muted && clip.volume > 0.0) {
        let Some(used) = clip.used(from, to) else { continue };
        if sounds(tools, clip)? {
            found.push(Track { path: clip.path.clone(), used, volume: clip.volume });
        }
    }
    Ok(found)
}

/// The input arguments of `tracks`: each file opened at its offset for its length. They follow
/// the picture, so track `n` is ffmpeg's input `n + 1`.
pub(crate) fn inputs(tracks: &[Track]) -> Vec<OsString> {
    let mut args = Vec::new();
    for track in tracks {
        let (seek, length) = (format!("{:.6}", track.used.seek), format!("{:.6}", track.used.length));
        args.extend(["-ss", &seek, "-t", &length, "-i"].map(OsString::from));
        args.push(track.path.clone().into_os_string());
    }
    args
}

/// The filter graph that mixes `tracks` into `[mix]`, exactly `seconds` long: each is brought to
/// 48 kHz stereo, scaled by its volume and delayed to its place, then all are summed.
pub(crate) fn graph(tracks: &[Track], seconds: f64) -> String {
    let samples = |seconds: f64| (seconds * RATE).round() as u64;
    let mut graph = String::new();
    for (index, track) in tracks.iter().enumerate() {
        graph.push_str(&format!(
            "[{}:a:0]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,volume={:.4},adelay={}S:all=1[a{index}];",
            index + 1,
            track.volume,
            samples(track.used.delay)
        ));
    }
    let mixed: String = (0..tracks.len()).map(|index| format!("[a{index}]")).collect();
    let count = tracks.len();
    graph.push_str(&format!(
        "{mixed}amix=inputs={count}:normalize=0:dropout_transition=0,apad,atrim=end_sample={}[mix]",
        samples(seconds)
    ));
    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_track_is_delayed_to_its_place_and_the_mix_is_cut_to_length() {
        let used = Used { delay: 0.5, seek: 1.0, length: 2.0 };
        let tracks = [
            Track { path: "/b/a.wav".into(), used, volume: 0.5 },
            Track { path: "/b/v.mp4".into(), used: Used { delay: 0.0, ..used }, volume: 1.0 },
        ];
        let graph = graph(&tracks, 2.0);
        assert!(graph.starts_with("[1:a:0]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,volume=0.5000,adelay=24000S:all=1[a0];"), "{graph}");
        assert!(
            graph.ends_with("[a0][a1]amix=inputs=2:normalize=0:dropout_transition=0,apad,atrim=end_sample=96000[mix]"),
            "{graph}"
        );
        let args: Vec<String> = inputs(&tracks).iter().map(|arg| arg.to_string_lossy().into_owned()).collect();
        assert_eq!(args[..6], ["-ss", "1.000000", "-t", "2.000000", "-i", "/b/a.wav"]);
    }
}
