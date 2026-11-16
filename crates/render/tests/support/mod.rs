//! What the stage render tests share: a bundle built the way the API builds one, through
//! `clap_timeline::parse` and `with_runtime`, with a red and a blue picture, and the decoded
//! frames of its render.

use std::path::Path;
use std::process::Command;

use clap_render::{Job, Tools, render};

const SKIP: &str = "\n================ SKIPPED: clap-render needs chrome-headless-shell, ffmpeg and ffprobe \
    (CLAP_CHROME, CLAP_FFMPEG, CLAP_FFPROBE or PATH) ================\n";

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;

pub fn tools() -> Option<Tools> {
    let found = Tools::locate().ok().filter(|_| clap_render::tools::ffprobe().is_some());
    if found.is_none() {
        eprintln!("{SKIP}");
    }
    found
}

fn ffmpeg(tools: &Tools, args: &[&str], output: Option<&Path>) -> Vec<u8> {
    let mut command = Command::new(&tools.ffmpeg);
    command.args(["-hide_banner", "-loglevel", "error", "-y"]).args(args);
    match output {
        Some(path) => command.arg(path),
        None => command.arg("-"),
    };
    let done = command.output().unwrap();
    assert!(done.status.success(), "{}", String::from_utf8_lossy(&done.stderr));
    done.stdout
}

fn bundle(tools: &Tools, source: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let project = clap_timeline::parse(source).unwrap();
    let html = clap_timeline::with_runtime(&scene::compile(&project, &scene::Options::new()));
    std::fs::write(dir.path().join("index.html"), html).unwrap();
    for colour in ["red", "blue"] {
        let picture = dir.path().join(format!("{colour}.png"));
        ffmpeg(tools, &["-f", "lavfi", "-i", &format!("color=c={colour}:s=64x64"), "-frames:v", "1"], Some(&picture));
    }
    let video = ["-f", "lavfi", "-i", "color=c=red:s=64x64:r=30:d=2", "-pix_fmt", "yuv420p"];
    ffmpeg(tools, &video, Some(&dir.path().join("red.mp4")));
    ffmpeg(tools, &["-f", "lavfi", "-i", "sine=frequency=440:duration=2"], Some(&dir.path().join("tone.wav")));
    dir
}

/// One render: the hash and the RGB bytes of every frame, and the file.
pub struct Film {
    pub hashes: Vec<String>,
    pub frames: Vec<Vec<u8>>,
    pub out: tempfile::TempDir,
}

/// Renders `source` at 640x360 and 30 frames a second.
pub fn film(tools: &Tools, source: &str) -> Film {
    let dir = bundle(tools, source);
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("out.mp4");
    let report = render(&Job { width: WIDTH, height: HEIGHT, ..Job::new(dir.path(), &file) }, tools).unwrap();
    let raw = ffmpeg(tools, &["-i", file.to_str().unwrap(), "-f", "rawvideo", "-pix_fmt", "rgb24"], None);
    let size = (WIDTH * HEIGHT * 3) as usize;
    Film { hashes: report.frame_hashes, frames: raw.chunks(size).map(<[u8]>::to_vec).collect(), out }
}

/// The sound of a render as mono 16-bit samples at 48 kHz.
pub fn samples(tools: &Tools, film: &Film) -> Vec<i16> {
    let file = film.out.path().join("out.mp4");
    let args = ["-i", file.to_str().unwrap(), "-vn", "-ac", "1", "-ar", "48000", "-f", "s16le"];
    ffmpeg(tools, &args, None).chunks(2).map(|pair| i16::from_le_bytes([pair[0], pair[1]])).collect()
}

pub fn pixel(frame: &[u8], x: u32, y: u32) -> [u8; 3] {
    let at = ((y * WIDTH + x) * 3) as usize;
    [frame[at], frame[at + 1], frame[at + 2]]
}

pub fn near(found: [u8; 3], wanted: [u8; 3]) -> bool {
    found.iter().zip(wanted).all(|(found, wanted)| found.abs_diff(wanted) <= 40)
}
