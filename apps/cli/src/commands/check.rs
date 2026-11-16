//! `clap check`: every problem of a project as `file:line: message`.

use clap_timeline::Timeline;
use serde_json::json;

use crate::cli::FolderArgs;
use crate::commands::print_json;
use crate::diagnose::{self, Diagnostic, Report};
use crate::failure::{Failure, Outcome};
use crate::project::Folder;
use crate::ui;

fn timeline(report: &mut Report) -> Option<Timeline> {
    report.project.as_ref()?;
    match clap_timeline::read(&report.source) {
        Ok(timeline) => Some(timeline),
        Err(error) => {
            report.problems.push(Diagnostic { line: 1, code: error.code().to_owned(), message: error.to_string() });
            None
        }
    }
}

fn summary(timeline: &Timeline) -> (usize, usize, f64) {
    let clips = timeline.scenes.iter().map(|scene| scene.clips.len()).sum();
    (timeline.scenes.len(), clips, timeline.duration)
}

/// Prints the problems on stdout, which is what this command is for, and fails when there
/// is one. A project is valid when the scene parser and the timeline both read it and every
/// file it names is in the folder.
pub fn run(args: &FolderArgs) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let mut report = diagnose::inspect(&folder)?;
    let (scenes, clips, seconds) = timeline(&mut report).as_ref().map(summary).unwrap_or_default();
    let (label, valid) = (folder.label(), report.problems.is_empty());
    if ui::json() {
        let counts = json!({ "scenes": scenes, "clips": clips, "duration": seconds });
        print_json(&json!({ "file": label, "valid": valid, "errors": report.problems, "timeline": counts }));
    } else {
        diagnose::lines(&label, &report.problems).iter().for_each(|line| println!("{line}"));
    }
    if valid {
        ui::success(&format!("{label} is valid ({scenes} scenes, {clips} clips, {seconds} s)"));
        return Ok(());
    }
    ui::error(&format!("{label} has {}", diagnose::count(report.problems.len())));
    Err(Failure::quiet("invalid"))
}
