//! `clap render` through the binary: a real MP4 from a starter, and what it says when it cannot.

#[path = "support/bin.rs"]
mod bin;

use std::fs;
use std::process::Command;

use clap_render::Tools;

const SKIP: &str = "\n================ SKIPPED: `clap render` needs chrome-headless-shell, ffmpeg and ffprobe \
    (CLAP_CHROME, CLAP_FFMPEG, CLAP_FFPROBE or PATH) ================\n";

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn new_then_check_then_render_writes_an_mp4_ffprobe_accepts() {
    let Some((_, ffprobe)) = Tools::locate().ok().zip(clap_render::tools::ffprobe()) else {
        return eprintln!("{SKIP}");
    };
    let work = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| bin::command(work.path(), work.path(), args).output().unwrap();
    assert!(run(&["new", "demo", "--template", "lower-third"]).status.success());
    assert!(run(&["check", "demo"]).status.success());
    let args =
        ["render", "demo", "-o", "out.mp4", "--width", "320", "--height", "180", "--fps", "10", "--range", "0:1.5"];
    let rendered = run(&[&args[..], &["--json"]].concat());
    assert!(rendered.status.success(), "{}", text(&rendered.stderr));
    let answer: serde_json::Value = serde_json::from_slice(&rendered.stdout).unwrap();
    assert_eq!((&answer["render"]["frames"], &answer["render"]["duration"]), (&15.into(), &1.5.into()), "{answer}");
    let probe =
        ["-v", "error", "-count_frames", "-show_entries", "stream=codec_name,width,nb_read_frames", "-of", "csv=p=0"];
    let probed = Command::new(ffprobe).args(probe).arg(work.path().join("out.mp4")).output().unwrap();
    assert_eq!(text(&probed.stdout).trim(), "h264,320,15", "{}", text(&probed.stderr));
    let left: Vec<_> =
        fs::read_dir(work.path().join("demo")).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(left.len(), 2, "the render left its bundle in the project folder: {left:?}");
}

#[test]
fn render_says_which_binary_is_missing_and_how_to_name_it() {
    let work = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        bin::command(work.path(), work.path(), args).env("CLAP_CHROME", "/nonexistent/chrome").output().unwrap()
    };
    assert!(run(&["new", "demo"]).status.success());
    let missing = run(&["render", "demo"]);
    assert_eq!(missing.status.code(), Some(1));
    let said = text(&missing.stderr);
    assert!(said.contains("chrome-headless-shell not found") && said.contains("set CLAP_CHROME to"), "{said}");
    assert!(!work.path().join("demo.mp4").exists());
    let bad = run(&["render", "demo", "--range", "5"]);
    assert!(text(&bad.stderr).contains("write start:end in seconds"), "{}", text(&bad.stderr));
    let remote = run(&["render", "demo", "--remote"]);
    assert!(text(&remote.stderr).contains("is not linked to a project"), "{}", text(&remote.stderr));
}
