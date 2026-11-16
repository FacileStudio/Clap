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
    pub ramp: Ramp,
}

/// How the level of one track rises and falls, in seconds of its clip.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct Ramp {
    /// Where the used part starts, from the start of the clip.
    pub lead: f64,
    /// How long the clip lasts.
    pub length: f64,
    /// Seconds from silence to full level at the start.
    pub rise: f64,
    /// Seconds from full level to silence at the end.
    pub fall: f64,
}

/// The `volume` filter of a track: a constant, or an expression of the time when the clip
/// fades, evaluated per audio frame like the preview evaluates it per seek.
fn level(track: &Track) -> String {
    let Ramp { lead, length, rise, fall } = track.ramp;
    if rise <= 0.0 && fall <= 0.0 {
        return format!("volume={:.4}", track.volume);
    }
    let mut expression = format!("{:.4}", track.volume);
    if rise > 0.0 {
        expression.push_str(&format!("*clip((t+{lead:.6})/{rise:.6},0,1)"));
    }
    if fall > 0.0 {
        expression.push_str(&format!("*clip(({length:.6}-t-{lead:.6})/{fall:.6},0,1)"));
    }
    format!("asetpts=PTS-STARTPTS,volume='{expression}':eval=frame")
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
            let ramp =
                Ramp { lead: used.seek - clip.offset, length: clip.duration, rise: clip.fade_in, fall: clip.fade_out };
            found.push(Track { path: clip.path.clone(), used, volume: clip.volume, ramp });
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
            "[{}:a:0]aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,{},adelay={}S:all=1[a{index}];",
            index + 1,
            level(track),
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
            Track { path: "/b/a.wav".into(), used, volume: 0.5, ramp: Ramp::default() },
            Track { path: "/b/v.mp4".into(), used: Used { delay: 0.0, ..used }, volume: 1.0, ramp: Ramp::default() },
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

    #[test]
    fn a_fading_clip_ramps_its_level_from_the_part_the_render_uses() {
        let used = Used { delay: 0.0, seek: 1.5, length: 2.0 };
        let ramp = Ramp { lead: 0.5, length: 3.0, rise: 1.0, fall: 0.5 };
        let track = Track { path: "/b/a.wav".into(), used, volume: 0.8, ramp };
        assert_eq!(
            level(&track),
            "asetpts=PTS-STARTPTS,volume='0.8000*clip((t+0.500000)/1.000000,0,1)*clip((3.000000-t-0.500000)/0.500000,0,1)':eval=frame"
        );
    }
}
