//! `clap timeline`: the scenes, tracks and clips of a project.

use std::collections::BTreeMap;

use clap_timeline::{Clip, Scene, Timeline};
use serde_json::json;

use crate::cli::FolderArgs;
use crate::commands::print_json;
use crate::diagnose;
use crate::failure::{Failure, Outcome};
use crate::project::Folder;
use crate::ui;

fn kind(clip: &Clip) -> String {
    json!(clip.kind).as_str().unwrap_or_default().to_owned()
}

fn clip_line(clip: &Clip, width: usize) -> String {
    let end = clip.start + clip.duration;
    let src = clip.src.as_deref().map(|src| format!("  {src}")).unwrap_or_default();
    let offset = if clip.media_in > 0.0 { format!("  in {}", clip.media_in) } else { String::new() };
    format!("    {:width$}  {:5}  {} to {} s{offset}{src}", clip.id, kind(clip), clip.start, end)
}

fn scene_lines(scene: &Scene) -> Vec<String> {
    let end = scene.start + scene.duration;
    let mut lines = vec![format!("scene {}  {} to {} s  {}", scene.index, scene.start, end, scene.layout)];
    let width = scene.clips.iter().map(|clip| clip.id.len()).max().unwrap_or(0);
    let mut tracks: BTreeMap<i64, Vec<&Clip>> = BTreeMap::new();
    scene.clips.iter().for_each(|clip| tracks.entry(clip.track).or_default().push(clip));
    for (track, clips) in tracks.iter().rev() {
        lines.push(format!("  track {track}"));
        lines.extend(clips.iter().map(|clip| clip_line(clip, width)));
    }
    lines
}

/// The timeline as text: one block per scene, its tracks from the top one down, one clip per
/// line with its id, kind, times and file.
pub fn text(timeline: &Timeline) -> String {
    let head =
        format!("{}x{}  {} s  {} scenes", timeline.width, timeline.height, timeline.duration, timeline.scenes.len());
    let scenes = timeline.scenes.iter().flat_map(scene_lines);
    std::iter::once(head).chain(scenes).collect::<Vec<_>>().join("\n")
}

/// Reads the timeline of a project that has no problem.
pub fn read(folder: &Folder) -> Outcome<(String, Timeline)> {
    let (source, _) = diagnose::require(folder)?;
    let timeline = clap_timeline::read(&source).map_err(|error| Failure::new(error.code(), error.to_string()))?;
    Ok((source, timeline))
}

/// Prints the timeline, as text or as the JSON `clap-timeline` defines.
pub fn run(args: &FolderArgs) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let (_, timeline) = read(&folder)?;
    if ui::json() {
        print_json(&json!(timeline));
    } else {
        println!("{}", text(&timeline));
    }
    Ok(())
}
