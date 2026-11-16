//! The model read from a source, and the JSON the client and the API exchange.

use clap_timeline::{Error, Kind, Op, read, with_runtime};
use serde_json::json;

const LAUNCH: &str = include_str!("fixtures/launch.md");

#[test]
fn a_source_reads_into_scenes_and_clips() {
    let timeline = read(LAUNCH).unwrap();
    assert_eq!(
        (timeline.title.as_str(), timeline.width, timeline.height, timeline.duration),
        ("Launch", 1280, 720, 14.0)
    );
    let scenes: Vec<_> = timeline.scenes.iter().map(|s| (s.index, s.start, s.duration, s.layout.as_str())).collect();
    assert_eq!(scenes, [(0, 0.0, 6.0, "raw"), (1, 6.0, 4.0, "raw"), (2, 10.0, 4.0, "default")]);
    assert_eq!((timeline.scenes[0].fixed_duration, timeline.scenes[2].fixed_duration), (true, false));
    let clips: Vec<_> =
        timeline.scenes[0].clips.iter().map(|c| (c.id.as_str(), c.kind, c.start, c.duration, c.track)).collect();
    let expected =
        [("shot", Kind::Video, 0.0, 4.0, 0), ("clip-0-1", Kind::Html, 0.0, 6.0, 1), ("sub", Kind::Text, 2.0, 3.0, 2)];
    assert_eq!(clips, expected);
    let shot = timeline.clip("shot").unwrap();
    assert_eq!(
        (shot.src.as_deref(), shot.media_in, shot.lines.first, shot.lines.last),
        (Some("assets/shot.mp4"), 1.0, 12, 12)
    );
    assert_eq!((timeline.scenes[0].clips[1].lines.first, timeline.scenes[0].clips[1].lines.last), (13, 15));
    let music = timeline.clip("music").unwrap();
    assert_eq!((music.kind, music.scene, music.track, music.lines.first), (Kind::Audio, 1, 0, 28));
}

#[test]
fn the_json_is_what_the_client_draws() {
    let value = serde_json::to_value(read(LAUNCH).unwrap()).unwrap();
    let clip = &value["scenes"][0]["clips"][0];
    let expected = json!({"id": "shot", "kind": "video", "src": "assets/shot.mp4", "start": 0.0, "duration": 4.0,
        "track": 0, "media_in": 1.0, "scene": 0, "lines": {"first": 12, "last": 12}});
    assert_eq!(clip, &expected);
    assert_eq!(value["scenes"][2]["lines"], json!({"first": 31, "last": 34}));
}

#[test]
fn ops_are_tagged_json() {
    let text = r#"[{"op":"move_clip","clip":"a","start":1.5},
        {"op":"trim_clip","clip":"a","duration":2,"media_in":0.5},
        {"op":"split_clip","clip":"a","at":1},
        {"op":"delete_clip","clip":"a"},
        {"op":"add_clip","scene":0,"kind":"video","src":"a.mp4","start":0,"duration":2,"track":1},
        {"op":"set_scene_duration","scene":0,"duration":5},
        {"op":"reorder_scene","scene":0,"to":1},
        {"op":"delete_scene","scene":1},
        {"op":"add_scene","at":0,"duration":3,"body":"Hello"}]"#;
    let ops: Vec<Op> = serde_json::from_str(text).unwrap();
    assert_eq!(ops[0], Op::MoveClip { clip: "a".into(), start: 1.5, track: None });
    assert_eq!(ops[1], Op::TrimClip { clip: "a".into(), start: None, duration: Some(2.0), media_in: Some(0.5) });
    assert_eq!(serde_json::to_value(&ops[0]).unwrap(), json!({"op": "move_clip", "clip": "a", "start": 1.5}));
    assert_eq!(ops.len(), 9);
}

#[test]
fn html_in_code_and_comments_is_not_a_clip() {
    let source = "# Notes\n\n```html\n<p data-start=\"1\">code</p>\n```\n\n<!-- <p data-start=\"2\">gone</p> -->\n\n\
        <p data-start=\"3\" data-duration=\"1\">real</p>\n\n<script>const html = '<p data-start=\"4\">';</script>\n";
    let timeline = read(source).unwrap();
    let starts: Vec<f64> = timeline.scenes[0].clips.iter().map(|clip| clip.start).collect();
    assert_eq!(starts, [3.0]);
}

#[test]
fn a_bad_source_is_a_parse_error() {
    let error = read("---\ncolour: red\n---\n# Hi\n").unwrap_err();
    assert!(matches!(&error, Error::Parse(text) if text.contains("line 2")), "{error}");
    assert_eq!(error.code(), "parse");
}

#[test]
fn the_preview_runtime_lands_before_the_end_of_the_body() {
    let html = with_runtime("<html><body><main></main></body></html>");
    assert!(html.contains("clap:playing") && html.ends_with("</script>\n</body></html>"), "{html}");
}
