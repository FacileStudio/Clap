//! Properties over generated sources: an op then its inverse returns the original text, and
//! whatever an op answers still parses.

mod support;

use clap_timeline::{Kind, Op, Update, apply, read};
use proptest::prelude::*;
use support::{quarter, scenes, source};

fn any_op() -> impl Strategy<Value = Op> {
    let clip = (0usize..3, 0usize..4).prop_map(|(scene, nth)| format!("c{scene}x{nth}"));
    let time = (0u32..40).prop_map(quarter);
    prop_oneof![
        (clip.clone(), time.clone(), prop::option::of(0i64..4)).prop_map(|(clip, start, track)| Op::MoveClip {
            clip,
            start,
            track
        }),
        (clip.clone(), time.clone(), time.clone()).prop_map(|(clip, a, b)| Op::TrimClip {
            clip,
            start: Some(a),
            duration: Some(b),
            media_in: None
        }),
        (clip.clone(), time.clone()).prop_map(|(clip, at)| Op::SplitClip { clip, at }),
        clip.clone().prop_map(|clip| Op::DeleteClip { clip }),
        (clip.clone(), prop::option::of(0usize..4), time.clone(), prop::option::of(0i64..6))
            .prop_map(|(clip, scene, start, track)| Op::DuplicateClip { clip, scene, start, track }),
        (0usize..4).prop_map(|scene| Op::DuplicateScene { scene }),
        (0usize..4, time.clone()).prop_map(|(scene, duration)| Op::SetSceneDuration { scene, duration }),
        (0usize..4, 0usize..4).prop_map(|(scene, to)| Op::ReorderScene { scene, to }),
        (0usize..4).prop_map(|scene| Op::DeleteScene { scene }),
        (0usize..4, time).prop_map(|(at, duration)| Op::AddScene {
            at,
            duration,
            body: "<p data-start=\"0\">New</p>".into()
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    #[test]
    fn whatever_an_op_answers_still_parses(spec in scenes(), ops in prop::collection::vec(any_op(), 1..6)) {
        let original = source(&spec);
        let scenes = read(&original).unwrap().scenes.len();
        for op in ops {
            let Ok(edited) = apply(&original, std::slice::from_ref(&op)) else { continue };
            let timeline = read(&edited);
            prop_assert!(timeline.is_ok(), "{op:?} left a source that does not parse: {edited}");
            let grown = matches!(op, Op::AddScene { .. } | Op::DuplicateScene { .. }) as usize;
            let shrunk = matches!(op, Op::DeleteScene { .. }) as usize;
            prop_assert_eq!(timeline.unwrap().scenes.len() + shrunk, scenes + grown);
        }
    }

    #[test]
    fn clip_ops_undo_to_the_same_text(spec in scenes(), pick in 0usize..16, start in 0u32..8, length in 1u32..8, offset in 0u32..8) {
        let original = source(&spec);
        let timeline = read(&original).unwrap();
        let clips: Vec<_> = timeline.scenes.iter().flat_map(|scene| &scene.clips).collect();
        prop_assume!(!clips.is_empty());
        let clip = clips[pick % clips.len()];
        let id = clip.id.clone();
        let moved = apply(&original, &[Op::MoveClip { clip: id.clone(), start: quarter(start), track: Some(9) }]).unwrap();
        let back = Op::MoveClip { clip: id.clone(), start: clip.start, track: Some(clip.track) };
        prop_assert_eq!(apply(&moved, &[back]).unwrap(), original.clone());
        let media = (clip.kind == Kind::Video).then_some(quarter(offset));
        let trim = Op::TrimClip { clip: id.clone(), start: Some(quarter(start)), duration: Some(quarter(length)), media_in: media };
        let undo = Op::TrimClip { clip: id, start: Some(clip.start), duration: Some(clip.duration), media_in: media.map(|_| clip.media_in) };
        let trimmed = apply(&original, &[trim]);
        prop_assume!(trimmed.is_ok());
        prop_assert_eq!(apply(&trimmed.unwrap(), &[undo]).unwrap(), original);
    }

    #[test]
    fn scene_ops_undo_to_the_same_text(spec in scenes(), from in 0usize..3, to in 0usize..4, length in 16u32..40) {
        let original = source(&spec);
        let count = spec.len();
        let (from, to) = (from % count, to % count);
        let moved = apply(&original, &[Op::ReorderScene { scene: from, to }]).unwrap();
        prop_assert_eq!(apply(&moved, &[Op::ReorderScene { scene: to, to: from }]).unwrap(), original.clone());
        let before = f64::from(spec[from].0 + 4);
        let longer = apply(&original, &[Op::SetSceneDuration { scene: from, duration: quarter(length) }]).unwrap();
        prop_assert_eq!(apply(&longer, &[Op::SetSceneDuration { scene: from, duration: before }]).unwrap(), original.clone());
        let at = to % (count + 1);
        let added = apply(&original, &[Op::AddScene { at, duration: 2.5, body: "<h1>Added</h1>\n".into() }]).unwrap();
        prop_assert_eq!(apply(&added, &[Op::DeleteScene { scene: at }]).unwrap(), original.clone());
        let clip = Op::AddClip { scene: from, kind: Kind::Audio, src: Some("assets/a.mp3".into()), start: 0.0, duration: 1.0, track: 99, text: None };
        let with_clip = apply(&original, &[clip]).unwrap();
        prop_assert_eq!(apply(&with_clip, &[Op::DeleteClip { clip: "audio-1".into() }]).unwrap(), original);
    }

    #[test]
    fn stage_ops_undo_to_the_same_text(spec in scenes(), pick in 0usize..16, x in -40i32..400, side in 1u32..400, scene in 0usize..3, which in 0usize..7) {
        let original = source(&spec);
        let timeline = read(&original).unwrap();
        let clips: Vec<_> = timeline.scenes.iter().flat_map(|scene| &scene.clips).collect();
        prop_assume!(!clips.is_empty());
        let clip = clips[pick % clips.len()];
        let some = |value: f64| Some(Some(value));
        let update = Update { x: some(f64::from(x)), y: some(0.5), w: some(f64::from(side)), h: some(f64::from(side) / 2.0),
            rot: some(-12.5), fit: Some(Some("contain".into())), opacity: some(0.25), enter: Some(Some("pop".into())),
            exit_duration: some(0.75), ..Update::default() };
        let placed = apply(&original, &[Op::UpdateClip { clip: clip.id.clone(), update }]).unwrap();
        let undo = Update { x: Some(None), y: Some(None), w: Some(None), h: Some(None), rot: Some(None), fit: Some(None),
            opacity: Some(None), enter: Some(None), exit_duration: Some(None), ..Update::default() };
        prop_assert_eq!(apply(&placed, &[Op::UpdateClip { clip: clip.id.clone(), update: undo }]).unwrap(), original.clone());
        let index = scene % spec.len();
        let kind = clap_timeline::SCENE_TRANSITIONS[which].to_owned();
        let set = Op::SetSceneTransition { scene: index, transition: Some(kind), duration: Some(Some(quarter(side % 8 + 1))) };
        let cut = Op::SetSceneTransition { scene: index, transition: None, duration: None };
        let with = apply(&original, &[set]).unwrap();
        let written = read(&with).unwrap().scenes[index].transition.clone();
        prop_assert_eq!(written.as_deref(), Some(clap_timeline::SCENE_TRANSITIONS[which]));
        prop_assert_eq!(apply(&with, &[cut]).unwrap(), original);
    }
}
