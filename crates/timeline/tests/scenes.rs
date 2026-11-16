//! Scene edits, each compared with the exact text it must leave.

use clap_timeline::{Error, Op, apply, read};

const LAUNCH: &str = include_str!("fixtures/launch.md");

const SECOND: &str = "---\nlayout: raw\nduration: 4\n---\n\n<audio data-clip=\"music\" src=\"assets/music.mp3\" data-start=\"0\" data-duration=\"4\" data-volume=\"0.5\"></audio>\n\n";

#[test]
fn a_duration_changes_one_value_or_adds_one_line() {
    let longer = apply(LAUNCH, &[Op::SetSceneDuration { scene: 0, duration: 7.25 }]).unwrap();
    assert_eq!(longer, LAUNCH.replace("duration: 6\n", "duration: 7.25\n"));
    let third = apply(LAUNCH, &[Op::SetSceneDuration { scene: 2, duration: 5.0 }]).unwrap();
    assert_eq!(third, LAUNCH.replace("---\n\n## Thanks", "---\nduration: 5\n---\n\n## Thanks"));
    let bare = apply("# Hi\n", &[Op::SetSceneDuration { scene: 0, duration: 2.0 }]).unwrap();
    assert_eq!(bare, "---\nduration: 2\n---\n# Hi\n");
    let commented =
        apply("---\nduration: 6  # six\n---\n# Hi\n", &[Op::SetSceneDuration { scene: 0, duration: 9.0 }]).unwrap();
    assert_eq!(commented, "---\nduration: 9  # six\n---\n# Hi\n");
    let short = apply(LAUNCH, &[Op::SetSceneDuration { scene: 0, duration: 5.0 }]).unwrap_err();
    assert!(matches!(&short, Error::OutOfBounds(text) if text.contains("clip-0-1")), "{short}");
    assert_eq!(apply(LAUNCH, &[Op::SetSceneDuration { scene: 9, duration: 5.0 }]).unwrap_err(), Error::UnknownScene(9));
}

#[test]
fn a_reorder_moves_whole_lines() {
    let swapped = apply(LAUNCH, &[Op::ReorderScene { scene: 1, to: 2 }]).unwrap();
    let tail = "---\n\n## Thanks\n\nBye.\n";
    assert_eq!(swapped, LAUNCH.replace(&format!("{SECOND}{tail}"), &format!("{tail}{SECOND}")));
    assert_eq!(apply(&swapped, &[Op::ReorderScene { scene: 2, to: 1 }]).unwrap(), LAUNCH);
}

#[test]
fn the_project_keys_stay_on_top_when_the_first_scene_moves() {
    let moved = apply(LAUNCH, &[Op::ReorderScene { scene: 0, to: 2 }]).unwrap();
    assert!(
        moved.starts_with("---\ntitle: Launch\nsize: 1280x720\nlayout: raw\nduration: 4\n---\n\n<audio"),
        "{moved}"
    );
    assert!(moved.contains("Bye.\n---\nlayout: raw\nduration: 6\n---\n\n<style>"), "{moved}");
    let timeline = read(&moved).unwrap();
    assert_eq!((timeline.title.as_str(), timeline.width, timeline.scenes[2].duration), ("Launch", 1280, 6.0));
    assert_eq!(apply(&moved, &[Op::ReorderScene { scene: 2, to: 0 }]).unwrap(), LAUNCH);
}

#[test]
fn a_scene_is_added_and_deleted_back() {
    let body = "<h1 id=\"new\" data-start=\"0\" data-duration=\"1\">New</h1>";
    let added = apply(LAUNCH, &[Op::AddScene { at: 1, duration: 3.0, body: body.to_owned() }]).unwrap();
    let scene = format!("---\nlayout: raw\nduration: 3\n---\n{body}\n");
    assert_eq!(
        added,
        LAUNCH.replacen("---\nlayout: raw\nduration: 4\n", &format!("{scene}---\nlayout: raw\nduration: 4\n"), 1)
    );
    assert_eq!(apply(&added, &[Op::DeleteScene { scene: 1 }]).unwrap(), LAUNCH);
    let first = apply(LAUNCH, &[Op::AddScene { at: 0, duration: 2.0, body: "# Intro\n".to_owned() }]).unwrap();
    assert!(
        first.starts_with(
            "---\ntitle: Launch\nsize: 1280x720\nduration: 2\n---\n# Intro\n---\nlayout: raw\nduration: 6\n---\n"
        ),
        "{first}"
    );
    assert_eq!(apply(&first, &[Op::DeleteScene { scene: 0 }]).unwrap(), LAUNCH);
    let last = apply(LAUNCH, &[Op::AddScene { at: 3, duration: 2.0, body: "End.".to_owned() }]).unwrap();
    assert_eq!(last, format!("{LAUNCH}---\nduration: 2\n---\nEnd.\n"));
}

#[test]
fn a_body_that_would_split_or_break_is_refused() {
    let split = apply(LAUNCH, &[Op::AddScene { at: 1, duration: 3.0, body: "one\n---\ntwo".to_owned() }]).unwrap_err();
    assert_eq!(split.code(), "invalid_result");
    let remote =
        apply(LAUNCH, &[Op::AddScene { at: 1, duration: 3.0, body: "![x](https://example.com/x.png)".to_owned() }]);
    assert_eq!(remote.unwrap_err().code(), "invalid_result");
    assert_eq!(apply("# One\n", &[Op::DeleteScene { scene: 0 }]).unwrap_err().code(), "invalid_op");
}

#[test]
fn a_body_that_opens_like_frontmatter_keeps_meaning_text() {
    let source = "note: first\n\n---\n\n# Second\n";
    let moved = apply(source, &[Op::ReorderScene { scene: 0, to: 1 }]).unwrap();
    assert_eq!(moved, "\n# Second\n---\n---\nnote: first\n\n");
    assert_eq!(
        apply(&moved, &[Op::ReorderScene { scene: 1, to: 0 }]).unwrap(),
        "---\n---\nnote: first\n\n---\n\n# Second\n"
    );
}

#[test]
fn line_endings_and_a_missing_final_newline_survive() {
    let source = "---\r\ntitle: T\r\nduration: 2\r\n---\r\n# One\r\n---\r\n# Two";
    let longer = apply(source, &[Op::SetSceneDuration { scene: 1, duration: 3.0 }]).unwrap();
    assert_eq!(longer, "---\r\ntitle: T\r\nduration: 2\r\n---\r\n# One\r\n---\r\nduration: 3\r\n---\r\n# Two");
    let swapped = apply(source, &[Op::ReorderScene { scene: 0, to: 1 }]).unwrap();
    assert_eq!(swapped, "---\r\ntitle: T\r\n---\r\n# Two\r\n---\r\nduration: 2\r\n---\r\n# One");
    assert_eq!(apply(&swapped, &[Op::ReorderScene { scene: 0, to: 1 }]).unwrap(), source);
}
