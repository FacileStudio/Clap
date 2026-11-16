//! `clap render`: the project to an MP4 on this machine, with `clap-render`.

use std::ops::{ControlFlow, Range};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use clap_render::{Error, Job, Report, Tools, render_with};
use serde_json::json;

use crate::cli::RenderArgs;
use crate::commands::{bundle, print_json};
use crate::failure::{Failure, INTERRUPTED, Outcome};
use crate::progress::Bar;
use crate::project::Folder;
use crate::{diagnose, ui};

/// Set by Ctrl+C. The render stops at the next frame and removes what it started.
pub static STOP: AtomicBool = AtomicBool::new(false);

/// Makes Ctrl+C set [`STOP`] instead of killing the process, so Chrome and ffmpeg are closed.
pub fn catch_interrupt() {
    let _ = ctrlc::set_handler(|| STOP.store(true, Ordering::Relaxed));
}

/// Reads `--range a:b`.
pub fn range(text: &str) -> Outcome<Range<f64>> {
    let parsed =
        text.split_once(':').and_then(|(start, end)| Some(start.trim().parse().ok()?..end.trim().parse().ok()?));
    parsed.filter(|range: &Range<f64>| range.start >= 0.0 && range.end > range.start).ok_or_else(|| {
        Failure::new("bad_range", format!("`{text}` is not a range — write start:end in seconds, such as 2:7.5"))
    })
}

fn missing(error: Error) -> Failure {
    let how = match &error {
        Error::MissingTool("chrome-headless-shell", _) => {
            "set CLAP_CHROME to a chrome-headless-shell binary, or run `npx playwright install chromium-headless-shell`"
        }
        Error::MissingTool(..) => "install ffmpeg, or set CLAP_FFMPEG to its binary",
        _ => "see the message above",
    };
    Failure::new("missing_tool", format!("{error} — {how}"))
}

fn failed(error: Error) -> Failure {
    match error {
        Error::Cancelled => Failure::new(INTERRUPTED, "render cancelled, nothing was written"),
        other => Failure::new("render_failed", format!("the render failed: {other}")),
    }
}

fn report(output: &PathBuf, done: &Report) {
    let seconds = done.elapsed.as_secs_f64();
    if ui::json() {
        let media = json!({ "videos": done.videos, "audio_tracks": done.audio_tracks });
        let facts = json!({ "frames": done.frames, "duration": done.duration, "seconds": seconds, "media": media });
        return print_json(&json!({ "output": output, "render": facts, "blocked": done.blocked }));
    }
    for url in &done.blocked {
        ui::warn(&format!("the page asked for {url} and was refused: a render loads only the project's files"));
    }
    ui::success(&format!(
        "Wrote {} ({} frames, {} s, in {seconds:.1} s)",
        output.display(),
        done.frames,
        done.duration
    ));
}

/// Renders the project to `--output`, or to `<folder name>.mp4` in the current folder.
pub fn run(args: &RenderArgs) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let (_, project) = diagnose::require(&folder)?;
    let range = args.range.as_deref().map(range).transpose()?;
    let tools = Tools::locate().map_err(missing)?;
    let output = args.output.clone().unwrap_or_else(|| PathBuf::from(format!("{}.mp4", folder.name())));
    let bundle = bundle::build(&folder, &project)?;
    let size = project.config().size;
    let job = Job {
        width: args.width.unwrap_or(size.width),
        height: args.height.unwrap_or(size.height),
        fps: args.fps.unwrap_or(30),
        range,
        ..Job::new(bundle.path(), &output)
    };
    catch_interrupt();
    let mut bar = Bar::new("Rendering");
    let rendered = render_with(&job, &tools, &mut |progress| {
        bar.count(u64::from(progress.frame), u64::from(progress.frames), "frames");
        if STOP.load(Ordering::Relaxed) { ControlFlow::Break(()) } else { ControlFlow::Continue(()) }
    });
    bar.finish();
    report(&output, &rendered.map_err(failed)?);
    Ok(())
}
