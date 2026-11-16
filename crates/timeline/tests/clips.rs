//! Clip edits, each compared with the exact text it must leave.

use clap_timeline::{Error, Kind, Op, apply};

const LAUNCH: &str = include_str!("fixtures/launch.md");

fn id(name: &str) -> String {
    name.to_owned()
}

#[test]
fn a_move_changes_one_value() {
    let moved = apply(LAUNCH, &[Op::MoveClip { clip: id("sub"), start: 2.5, track: None }]).unwrap();
    assert_eq!(moved, LAUNCH.replace("id=\"sub\" data-start=\"2\"", "id=\"sub\" data-start=\"2.5\""));
    let lane = apply(LAUNCH, &[Op::MoveClip { clip: id("sub"), start: 2.0, track: Some(5) }]).unwrap();
    assert_eq!(lane, LAUNCH.replace("data-track-index=\"2\"", "data-track-index=\"5\""));
    let music = apply(LAUNCH, &[Op::MoveClip { clip: id("music"), start: 0.0, track: Some(3) }]).unwrap();
    assert_eq!(
        music,
        LAUNCH.replace(
            "data-start=\"0\" data-duration=\"4\" data-volume",
            "data-start=\"0\" data-track-index=\"3\" data-duration=\"4\" data-volume"
        )
    );
}

#[test]
fn a_trim_sets_what_it_names() {
    let op = Op::TrimClip { clip: id("shot"), start: Some(0.5), duration: Some(3.5), media_in: Some(1.5) };
    let expected = LAUNCH.replace(
        "data-start=\"0\" data-duration=\"4\" data-track-index=\"0\" data-in=\"1\"",
        "data-start=\"0.5\" data-duration=\"3.5\" data-track-index=\"0\" data-in=\"1.5\"",
    );
    assert_eq!(apply(LAUNCH, &[op]).unwrap(), expected);
    let offset = Op::TrimClip { clip: id("music"), start: None, duration: None, media_in: Some(2.0) };
    let expected = LAUNCH.replace(
        "data-start=\"0\" data-duration=\"4\" data-volume",
        "data-start=\"0\" data-in=\"2\" data-duration=\"4\" data-volume",
    );
    assert_eq!(apply(LAUNCH, &[offset]).unwrap(), expected);
}

#[test]
fn a_split_copies_the_element_after_itself() {
    let split = apply(LAUNCH, &[Op::SplitClip { clip: id("shot"), at: 1.5 }]).unwrap();
    let first = "<video id=\"shot\" src=\"assets/shot.mp4\" data-start=\"0\" data-duration=\"1.5\" data-track-index=\"0\" data-in=\"1\"></video>\n";
    let second = "<video id=\"shot-b\" src=\"assets/shot.mp4\" data-start=\"1.5\" data-duration=\"2.5\" data-track-index=\"0\" data-in=\"2.5\"></video>\n";
    let whole = "<video id=\"shot\" src=\"assets/shot.mp4\" data-start=\"0\" data-duration=\"4\" data-track-index=\"0\" data-in=\"1\"></video>\n";
    assert_eq!(split, LAUNCH.replace(whole, &format!("{first}{second}")));
    let block = apply(LAUNCH, &[Op::SplitClip { clip: id("clip-0-1"), at: 2.0 }]).unwrap();
    let copy = "</div>\n<div class=\"clip\" data-start=\"2\" data-clip=\"clip-0-1-b\" data-duration=\"4\" data-track-index=\"1\">\n  <h1 id=\"t\">Launch day</h1>\n</div>\n";
    let expected = LAUNCH
        .replace("data-duration=\"6\" data-track-index=\"1\"", "data-duration=\"2\" data-track-index=\"1\"")
        .replacen("</div>\n", copy, 1);
    assert_eq!(block, expected);
}

#[test]
fn an_added_clip_is_deleted_back_to_the_same_text() {
    let add = |kind, src: Option<&str>, text: Option<&str>| Op::AddClip {
        scene: 0,
        kind,
        src: src.map(str::to_owned),
        start: 4.0,
        duration: 2.0,
        track: 0,
        text: text.map(str::to_owned),
    };
    let added = apply(LAUNCH, &[add(Kind::Video, Some("assets/b.mp4"), None)]).unwrap();
    let line = "Today, everywhere.</p>\n<video data-clip=\"video-1\" src=\"assets/b.mp4\" data-start=\"4\" data-duration=\"2\" data-track-index=\"0\"></video>\n";
    assert_eq!(added, LAUNCH.replace("Today, everywhere.</p>\n", line));
    assert_eq!(apply(&added, &[Op::DeleteClip { clip: id("video-1") }]).unwrap(), LAUNCH);
    let text = apply(LAUNCH, &[add(Kind::Text, None, Some("a < b"))]).unwrap();
    assert!(text.contains("<div class=\"clip\" data-clip=\"text-1\" data-start=\"4\" data-duration=\"2\" data-track-index=\"0\">a &lt; b</div>"));
    let missing = apply(LAUNCH, &[add(Kind::Audio, None, None)]).unwrap_err();
    assert_eq!(missing.code(), "invalid_op");
    let remote = apply(LAUNCH, &[add(Kind::Image, Some("https://example.com/a.png"), None)]).unwrap_err();
    assert!(matches!(&remote, Error::InvalidResult(text) if text.contains("remote URL")), "{remote}");
}

#[test]
fn a_first_clip_lands_before_the_note_or_at_the_end() {
    let add = |scene| Op::AddClip {
        scene,
        kind: Kind::Text,
        src: None,
        start: 0.0,
        duration: 1.0,
        track: 0,
        text: Some("Hi".into()),
    };
    let source = "---\nlayout: raw\nduration: 2\n---\n<h1>Title</h1>\n\n<!-- note -->\n\n---\n\nBye.";
    let element = |n| {
        format!(
            "<div class=\"clip\" data-clip=\"text-{n}\" data-start=\"0\" data-duration=\"1\" data-track-index=\"0\">Hi</div>"
        )
    };
    let first = apply(source, &[add(0)]).unwrap();
    assert_eq!(first, source.replace("<h1>Title</h1>\n", &format!("<h1>Title</h1>\n{}\n", element(1))));
    let last = apply(source, &[add(1)]).unwrap();
    assert_eq!(last, format!("{source}\n{}", element(1)));
    assert_eq!(apply(&last, &[Op::DeleteClip { clip: id("text-1") }]).unwrap(), source);
}

#[test]
fn a_delete_takes_the_whole_element() {
    let gone = apply(LAUNCH, &[Op::DeleteClip { clip: id("clip-0-1") }]).unwrap();
    let block = "<div class=\"clip\" data-start=\"0\" data-duration=\"6\" data-track-index=\"1\">\n  <h1 id=\"t\">Launch day</h1>\n</div>\n";
    assert_eq!(gone, LAUNCH.replace(block, ""));
    let inline = "---\nlayout: raw\nduration: 2\n---\n<p>a <b id=\"x\" data-start=\"0\">b</b> c</p>\n";
    assert_eq!(
        apply(inline, &[Op::DeleteClip { clip: id("x") }]).unwrap(),
        inline.replace("<b id=\"x\" data-start=\"0\">b</b>", "")
    );
}

#[test]
fn bad_placements_are_typed_errors() {
    let unknown = apply(LAUNCH, &[Op::DeleteClip { clip: id("nope") }]).unwrap_err();
    assert_eq!(unknown, Error::UnknownClip(id("nope")));
    let past = apply(LAUNCH, &[Op::MoveClip { clip: id("sub"), start: 4.0, track: None }]).unwrap_err();
    assert_eq!(past.code(), "out_of_bounds");
    let before = apply(LAUNCH, &[Op::MoveClip { clip: id("sub"), start: -1.0, track: None }]).unwrap_err();
    assert_eq!(before.code(), "out_of_bounds");
    let overlap = apply(LAUNCH, &[Op::MoveClip { clip: id("sub"), start: 2.0, track: Some(0) }]).unwrap_err();
    assert_eq!(overlap, Error::Overlap { clip: id("sub"), with: id("shot"), track: 0 });
    let free = apply(
        LAUNCH,
        &[
            Op::MoveClip { clip: id("sub"), start: 4.0, track: Some(0) },
            Op::TrimClip { clip: id("sub"), start: None, duration: Some(2.0), media_in: None },
        ],
    );
    assert_eq!(free.unwrap_err().code(), "out_of_bounds");
    let cut = apply(LAUNCH, &[Op::SplitClip { clip: id("sub"), at: 2.0 }]).unwrap_err();
    assert_eq!(cut.code(), "out_of_bounds");
}
