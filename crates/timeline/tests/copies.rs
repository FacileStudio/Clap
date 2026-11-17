//! The copy edits, each compared with the exact text it must leave, and undone by a delete.

mod support;

use clap_timeline::{Error, Op, apply, read};
use proptest::prelude::*;
use support::{quarter, scenes, source};

const LAUNCH: &str = include_str!("fixtures/launch.md");

fn copy(clip: &str, scene: Option<usize>, start: f64, track: Option<i64>) -> Op {
    Op::DuplicateClip { clip: clip.to_owned(), scene, start, track }
}

#[test]
fn a_copy_is_the_element_renamed_after_the_last_clip() {
    let copied = apply(LAUNCH, &[copy("shot", None, 2.0, Some(3))]).unwrap();
    let line = "<video id=\"shot\" src=\"assets/shot.mp4\" data-start=\"2\" data-clip=\"shot-copy\" data-duration=\"4\" data-track-index=\"3\" data-in=\"1\"></video>";
    let after = "Today, everywhere.</p>\n";
    assert_eq!(copied, LAUNCH.replace(after, &format!("{after}{line}\n")));
    assert_eq!(apply(&copied, &[Op::DeleteClip { clip: "shot-copy".into() }]).unwrap(), LAUNCH);
    let again = apply(&copied, &[copy("shot", None, 0.0, Some(4))]).unwrap();
    assert_eq!(read(&again).unwrap().scenes[0].clips.last().unwrap().id, "shot-copy-2");
}

#[test]
fn a_copy_crosses_scenes_and_keeps_its_attributes() {
    let copied = apply(LAUNCH, &[copy("music", Some(0), 1.0, None)]).unwrap();
    let timeline = read(&copied).unwrap();
    let pasted = timeline.scenes[0].clips.last().unwrap();
    assert_eq!((pasted.id.as_str(), pasted.start, pasted.duration), ("music-copy", 1.0, 4.0));
    assert_eq!(pasted.look.volume, Some(0.5));
    assert_eq!(apply(&copied, &[Op::DeleteClip { clip: "music-copy".into() }]).unwrap(), LAUNCH);
    let late = apply(LAUNCH, &[copy("music", Some(1), 2.0, None)]).unwrap_err();
    assert_eq!(late.code(), "out_of_bounds");
    let blocked = apply(LAUNCH, &[copy("sub", None, 2.0, Some(0))]).unwrap_err();
    assert_eq!(blocked, Error::Overlap { clip: "sub-copy".into(), with: "shot".into(), track: 0 });
    assert_eq!(apply(LAUNCH, &[copy("sub", Some(7), 0.0, None)]).unwrap_err(), Error::UnknownScene(7));
}

#[test]
fn a_scene_copy_renames_its_clips_and_leaves_the_project_keys() {
    let copied = apply(LAUNCH, &[Op::DuplicateScene { scene: 0 }]).unwrap();
    let timeline = read(&copied).unwrap();
    assert_eq!((timeline.title.as_str(), timeline.scenes.len()), ("Launch", 4));
    let ids: Vec<_> = timeline.scenes[1].clips.iter().map(|clip| clip.id.as_str()).collect();
    assert_eq!(ids, ["shot-copy", "clip-0-1-copy", "sub-copy"]);
    assert!(
        copied.contains("Voice-over: it is launch day. -->\n\n---\nlayout: raw\nduration: 6\n---\n\n<style>"),
        "{copied}"
    );
    assert_eq!(apply(&copied, &[Op::DeleteScene { scene: 1 }]).unwrap(), LAUNCH);
    let last = apply(LAUNCH, &[Op::DuplicateScene { scene: 2 }]).unwrap();
    assert_eq!(last, format!("{LAUNCH}---\n\n## Thanks\n\nBye.\n"));
    assert_eq!(apply(LAUNCH, &[Op::DuplicateScene { scene: 3 }]).unwrap_err(), Error::UnknownScene(3));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn copies_undo_to_the_same_text(spec in scenes(), pick in 0usize..16, start in 0u32..8, scene in 0usize..3) {
        let original = source(&spec);
        let timeline = read(&original).unwrap();
        let index = scene % spec.len();
        let twice = apply(&original, &[Op::DuplicateScene { scene: index }]).unwrap();
        prop_assert_eq!(apply(&twice, &[Op::DeleteScene { scene: index + 1 }]).unwrap(), original.clone());
        let clips: Vec<_> = timeline.scenes.iter().flat_map(|scene| &scene.clips).collect();
        prop_assume!(!clips.is_empty());
        let clip = clips[pick % clips.len()];
        let copied = apply(&original, &[copy(&clip.id, Some(index), quarter(start), Some(99))]);
        prop_assume!(copied.is_ok());
        let copied = copied.unwrap();
        let named = read(&copied).unwrap().scenes[index].clips.last().unwrap().id.clone();
        prop_assert_eq!(named, format!("{}-copy", clip.id));
        prop_assert_eq!(apply(&copied, &[Op::DeleteClip { clip: format!("{}-copy", clip.id) }]).unwrap(), original);
    }
}
