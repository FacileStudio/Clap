//! `paste_clip` and `set_clip_text`, compared with the exact text they must leave, and undone.

mod support;

use clap_timeline::{Error, Kind, Op, apply, read};
use proptest::prelude::*;
use support::{quarter, scenes, source};

const LAUNCH: &str = include_str!("fixtures/launch.md");
const SUB: &str = "<p id=\"sub\" data-start=\"2\" data-duration=\"3\" data-track-index=\"2\">Today, everywhere.</p>";

fn paste(scene: usize, start: f64, track: Option<i64>, element: &str) -> Op {
    Op::PasteClip { scene, start, track, element: element.to_owned() }
}

fn text(clip: &str, text: &str) -> Op {
    Op::SetClipText { clip: clip.to_owned(), text: text.to_owned() }
}

fn element_of(source: &str, id: &str) -> String {
    let lines = read(source).unwrap().clip(id).unwrap().lines;
    let all: Vec<&str> = source.lines().collect();
    all[lines.first - 1..lines.last].join("\n").trim().to_owned()
}

#[test]
fn a_cut_clip_pasted_back_lands_as_a_named_copy() {
    let cut = apply(LAUNCH, &[Op::DeleteClip { clip: "sub".into() }]).unwrap();
    let pasted = apply(&cut, &[paste(0, 2.0, None, &format!("  {SUB}\n"))]).unwrap();
    let copy = SUB.replace("data-start=\"2\"", "data-start=\"2\" data-clip=\"sub-copy\"");
    assert_eq!(pasted, LAUNCH.replace(SUB, &copy));
    assert_eq!(apply(&pasted, &[Op::DeleteClip { clip: "sub-copy".into() }]).unwrap(), cut);
    let moved = apply(LAUNCH, &[paste(1, 0.5, Some(5), SUB)]).unwrap();
    let clip = read(&moved).unwrap().clip("sub-copy").cloned().unwrap();
    assert_eq!((clip.scene, clip.start, clip.duration, clip.track), (1, 0.5, 3.0, 5));
    assert_eq!(clip.text.as_deref(), Some("Today, everywhere."));
}

#[test]
fn a_paste_is_checked_like_an_added_clip() {
    let blocked = apply(LAUNCH, &[paste(0, 2.0, Some(0), SUB)]).unwrap_err();
    assert_eq!(blocked, Error::Overlap { clip: "sub-copy".into(), with: "shot".into(), track: 0 });
    assert_eq!(apply(LAUNCH, &[paste(1, 3.0, None, SUB)]).unwrap_err().code(), "out_of_bounds");
    assert_eq!(apply(LAUNCH, &[paste(9, 0.0, None, SUB)]).unwrap_err(), Error::UnknownScene(9));
    for bad in [
        "<p data-start=\"0\">a</p><p>b</p>",
        "text <p>a</p>",
        "",
        "<img src=\"https://x.test/a.png\" data-start=\"0\">",
        "<img src=\"../a.png\" data-start=\"0\">",
    ] {
        assert_eq!(apply(LAUNCH, &[paste(1, 0.0, None, bad)]).unwrap_err().code(), "invalid_op", "{bad}");
    }
    let unnamed = apply(LAUNCH, &[paste(1, 0.0, None, "<p data-start=\"1\" data-duration=\"1\">Hi</p>")]).unwrap();
    assert_eq!(read(&unnamed).unwrap().scenes[1].clips.last().unwrap().id, "clip-copy");
}

#[test]
fn a_text_clip_takes_new_words_escaped_and_keeps_its_spaces() {
    let set = apply(LAUNCH, &[text("sub", " Rain & <shine> ")]).unwrap();
    assert_eq!(set, LAUNCH.replace(">Today, everywhere.<", ">Rain &amp; &lt;shine&gt;<"));
    assert_eq!(read(&set).unwrap().clip("sub").unwrap().text.as_deref(), Some("Rain & <shine>"));
    assert_eq!(apply(&set, &[text("sub", "Today, everywhere.")]).unwrap(), LAUNCH);
    let spaced = "---\nduration: 2\n---\n<h1 data-clip=\"t\" data-start=\"0\">\n  Hello\n</h1>\n";
    assert_eq!(apply(spaced, &[text("t", "Bye")]).unwrap(), spaced.replace("Hello", "Bye"));
    assert_eq!(apply(LAUNCH, &[text("sub", "Today, everywhere.")]).unwrap(), LAUNCH);
    assert_eq!(apply(LAUNCH, &[text("shot", "x")]).unwrap_err().code(), "invalid_op");
    assert_eq!(apply(LAUNCH, &[text("sub", "  ")]).unwrap_err().code(), "invalid_op");
    assert_eq!(apply(LAUNCH, &[text("nope", "x")]).unwrap_err().code(), "unknown_clip");
    assert_eq!(read(LAUNCH).unwrap().clip("shot").unwrap().kind, Kind::Video);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_pasted_copy_and_new_words_undo_to_the_same_text(spec in scenes(), pick in 0usize..16, start in 0u32..8, scene in 0usize..3) {
        let original = source(&spec);
        let timeline = read(&original).unwrap();
        let clips: Vec<_> = timeline.scenes.iter().flat_map(|scene| &scene.clips).collect();
        prop_assume!(!clips.is_empty());
        let clip = clips[pick % clips.len()];
        let index = scene % spec.len();
        let element = element_of(&original, &clip.id);
        let pasted = apply(&original, &[paste(index, quarter(start), Some(99), &element)]);
        prop_assume!(pasted.is_ok());
        let pasted = pasted.unwrap();
        let name = format!("{}-copy", clip.id);
        let copy = read(&pasted).unwrap().clip(&name).cloned().unwrap();
        prop_assert_eq!((copy.kind, copy.start, copy.duration, copy.track), (clip.kind, quarter(start), clip.duration, 99));
        prop_assert_eq!(apply(&pasted, &[Op::DeleteClip { clip: name }]).unwrap(), original.clone());
        if let Some(words) = &clip.text {
            let set = apply(&original, &[text(&clip.id, "New words")]).unwrap();
            prop_assert_eq!(apply(&set, &[text(&clip.id, words)]).unwrap(), original);
        }
    }
}
