//! Real renders: Chrome and ffmpeg on the title fixture. Skipped, loudly, when either is missing.

use std::collections::HashSet;
use std::net::TcpListener;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap_render::{Capture, Error, Job, Tools, render, render_with};
use serde_json::Value;

const SKIP: &str = "\n================ SKIPPED: clap-render needs chrome-headless-shell, ffmpeg and ffprobe \
    (CLAP_CHROME, CLAP_FFMPEG, CLAP_FFPROBE or PATH) ================\n";

fn tools() -> Option<(Tools, PathBuf)> {
    let found = Tools::locate().ok().zip(clap_render::tools::ffprobe());
    if found.is_none() {
        eprintln!("{SKIP}");
    }
    found
}

fn title(output: &Path) -> Job {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/title");
    Job { width: 1280, height: 720, range: Some(0.0..1.0), ..Job::new(fixture, output) }
}

fn bundle(body: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("index.html"), format!("<!doctype html><body>{body}</body>")).unwrap();
    dir
}

fn probe(ffprobe: &Path, file: &Path) -> Value {
    let entries = "stream=codec_name,width,height,pix_fmt,nb_read_frames,r_frame_rate";
    let output = Command::new(ffprobe)
        .args(["-v", "error", "-select_streams", "v:0", "-count_frames", "-show_entries", entries, "-of", "json"])
        .arg(file)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["streams"][0].take()
}

#[test]
fn one_second_is_thirty_identical_frames_on_every_run() {
    let Some((tools, ffprobe)) = tools() else { return };
    let out = tempfile::tempdir().unwrap();
    let (first, second) = (out.path().join("a.mp4"), out.path().join("b.mp4"));
    let a = render(&title(&first), &tools).unwrap();
    let b = render(&title(&second), &tools).unwrap();
    eprintln!("render: {} frames in {:?} then {:?}, capture {:?}", a.frames, a.elapsed, b.elapsed, a.capture);
    let stream = probe(&ffprobe, &first);
    assert_eq!(stream["nb_read_frames"], "30", "{stream}");
    assert_eq!((&stream["width"], &stream["height"]), (&Value::from(1280), &Value::from(720)), "{stream}");
    assert_eq!((&stream["codec_name"], &stream["pix_fmt"]), (&Value::from("h264"), &Value::from("yuv420p")));
    assert_eq!(stream["r_frame_rate"], "30/1", "{stream}");
    assert_eq!((a.frames, a.duration, a.blocked.len()), (30, 1.0, 0), "{:?}", a.blocked);
    assert_eq!(a.frame_hashes, b.frame_hashes, "two runs must capture the same pixels");
    let distinct: HashSet<&String> = a.frame_hashes.iter().collect();
    assert_eq!(distinct.len(), 30, "every seek must reach the picture");
    assert_eq!(std::fs::read(&first).unwrap(), std::fs::read(&second).unwrap(), "the MP4 bytes must match too");
}

#[test]
fn the_screenshot_path_renders_without_begin_frame_control() {
    let Some((tools, ffprobe)) = tools() else { return };
    let out = tempfile::tempdir().unwrap();
    let file = out.path().join("plain.mp4");
    let job = Job { capture: Capture::Screenshot, range: Some(0.0..0.2), ..title(&file) };
    let report = render(&job, &tools).unwrap();
    assert_eq!((report.frames, report.capture), (6, Capture::Screenshot));
    assert_eq!(probe(&ffprobe, &file)["nb_read_frames"], "6");
    assert_eq!(report.frame_hashes.iter().collect::<HashSet<_>>().len(), 6);
}

#[test]
fn the_page_reaches_nothing_outside_its_bundle() {
    let Some((tools, _)) = tools() else { return };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let page = format!(
        "<img src='http://127.0.0.1:{port}/a.png'><img src='file:///etc/hostname'><img src='inside.svg'>\
         <iframe src='file:///etc/hostname'></iframe>\
         <script>fetch('http://127.0.0.1:{port}/fetch').catch(() => 0);\
         try {{ new WebSocket('ws://127.0.0.1:{port}/ws') }} catch (e) {{}}\
         window.__scene = {{ duration: 1, seek() {{}} }};</script>"
    );
    let dir = bundle(&page);
    std::fs::write(dir.path().join("inside.svg"), "<svg xmlns='http://www.w3.org/2000/svg'/>").unwrap();
    let out = tempfile::tempdir().unwrap();
    let job = Job { width: 320, height: 180, range: Some(0.0..0.1), ..Job::new(dir.path(), out.path().join("o.mp4")) };
    let report = render(&job, &tools).unwrap();
    assert!(listener.accept().is_err(), "the page reached the local listener");
    for refused in [format!("http://127.0.0.1:{port}/a.png"), "file:///etc/hostname".to_owned()] {
        assert!(report.blocked.contains(&refused), "{refused} was not refused: {:?}", report.blocked);
    }
    assert!(!report.blocked.iter().any(|url| url.ends_with("inside.svg")), "{:?}", report.blocked);
}

#[test]
fn a_throwing_seek_and_a_cancel_leave_no_file() {
    let Some((tools, _)) = tools() else { return };
    let out = tempfile::tempdir().unwrap();
    let dir = bundle("<script>window.__scene = { duration: 1, seek() { throw new Error('boom') } };</script>");
    let broken = Job { width: 320, height: 180, ..Job::new(dir.path(), out.path().join("broken.mp4")) };
    let error = render(&broken, &tools).unwrap_err();
    assert!(matches!(&error, Error::Scene(reason) if reason.contains("boom")), "{error}");
    let cancelled = Job { width: 320, height: 180, ..title(&out.path().join("cancelled.mp4")) };
    let mut seen = 0;
    let error = render_with(&cancelled, &tools, &mut |progress| {
        seen = progress.frame;
        if progress.frame == 3 { ControlFlow::Break(()) } else { ControlFlow::Continue(()) }
    })
    .unwrap_err();
    assert!(matches!(error, Error::Cancelled), "{error}");
    assert_eq!(seen, 3);
    assert_eq!(std::fs::read_dir(out.path()).unwrap().count(), 0, "a failed render left a file behind");
}
