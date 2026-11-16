//! `clap render --remote`: the pushed project rendered by the server, then downloaded.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use serde_json::{Map, Value, json};

use crate::api::{ApiError, Client, Render};
use crate::cli::{Globals, RenderArgs};
use crate::commands::render::{STOP, catch_interrupt, range};
use crate::commands::{linked_session, print_json};
use crate::failure::{Failure, INTERRUPTED, Outcome};
use crate::progress::Bar;
use crate::project::Folder;
use crate::sync::{Link, sha256};
use crate::ui;

const POLL: Duration = Duration::from_secs(1);

fn spec(args: &RenderArgs) -> Outcome<Value> {
    let mut spec = Map::new();
    for (key, value) in [("width", args.width), ("height", args.height), ("fps", args.fps)] {
        value.into_iter().for_each(|value| drop(spec.insert(key.to_owned(), json!(value))));
    }
    if let Some(text) = &args.range {
        let range = range(text)?;
        spec.insert("range".to_owned(), json!([range.start, range.end]));
    }
    Ok(Value::Object(spec))
}

fn refused(error: ApiError) -> Failure {
    match error {
        ApiError::Refused { status: 412, message, .. } => {
            Failure::new("missing_assets", format!("the server cannot render yet: {message} — run `clap push` first"))
        }
        ApiError::Refused { status: 429, message, .. } => {
            Failure::new("queue_full", format!("the server's render queue is full: {message} — try again in a moment"))
        }
        ApiError::Refused { status: 400, message, .. } => {
            Failure::new("render_refused", format!("the server refused this render: {message}"))
        }
        other => other.into(),
    }
}

fn follow(client: &Client, started: Render) -> Outcome<Render> {
    let mut bar = Bar::new("Rendering on the server");
    let mut render = started;
    while !render.is_final() {
        if STOP.load(Ordering::Relaxed) {
            bar.finish();
            let _ = client.cancel_render(&render.id);
            return Err(Failure::new(INTERRUPTED, format!("render {} cancelled on the server", render.id)));
        }
        thread::sleep(POLL);
        render = client.render(&render.id)?;
        bar.count(render.frame, render.frames, "frames");
    }
    bar.finish();
    match render.state.as_str() {
        "done" => Ok(render),
        state => {
            let reason = render.error.clone().unwrap_or_else(|| "no reason given".to_owned());
            Err(Failure::new("render_failed", format!("the server render {state}: {reason}")))
        }
    }
}

fn download(client: &Client, id: &str, output: &Path) -> Outcome<u64> {
    let partial = PathBuf::from(format!("{}.part", output.display()));
    let mut bar = Bar::new("Downloading");
    let fetched = File::create(&partial)
        .map_err(Failure::from)
        .and_then(|mut file| Ok(client.download_render(id, &mut file, &mut |done, total| bar.bytes(done, total))?));
    bar.finish();
    match fetched.and_then(|bytes| Ok(fs::rename(&partial, output).map(|()| bytes)?)) {
        Ok(bytes) => Ok(bytes),
        Err(failure) => {
            let _ = fs::remove_file(&partial);
            Err(failure)
        }
    }
}

/// Queues a render of the project this folder is linked to, follows it, and downloads the
/// MP4. The server renders what was last pushed, not the folder.
pub fn run(args: &RenderArgs, globals: &Globals) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let link = Link::load(&folder.dir)?.ok_or_else(|| {
        let hint = "run `clap push --create` first, or drop --remote to render here";
        Failure::new("not_linked", format!("{} is not linked to a project — {hint}", folder.dir.display()))
    })?;
    if sha256(folder.source()?.as_bytes()) != link.source_sha256 {
        ui::warn(&format!("{} changed since the last push: the server renders revision {}", folder.label(), link.rev));
    }
    let client = linked_session(globals.server.as_deref(), &link)?;
    catch_interrupt();
    let started = client.start_render(&link.project, &spec(args)?).map_err(refused)?;
    ui::step(&format!("Queued render {} of project {}", started.id, link.project));
    let done = follow(&client, started)?;
    let output = args.output.clone().unwrap_or_else(|| PathBuf::from(format!("{}.mp4", folder.name())));
    let bytes = download(&client, &done.id, &output)?;
    if ui::json() {
        print_json(&json!({ "output": output, "bytes": bytes, "render": done }));
        return Ok(());
    }
    ui::success(&format!("Wrote {} ({} frames, {:.1} MiB)", output.display(), done.frames, bytes as f64 / 1_048_576.0));
    Ok(())
}
