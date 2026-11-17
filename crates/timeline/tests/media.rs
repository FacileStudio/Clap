//! `rename_media`, compared with the exact text it must leave, and undone by the rename back.

mod support;

use clap_timeline::{Error, Op, apply, read};
use proptest::prelude::*;
use support::{scenes, source};

const LAUNCH: &str = include_str!("fixtures/launch.md");

const PLACES: &str = "---\nlayout: raw\nduration: 4\n---\n\n<style>\n  @import \"assets/a.png\";\n  \
.bg { background: url(\"./assets/a.png?v=2\"); }\n</style>\n\
<img src=\"assets/a.png\" srcset=\"assets/a.png 1x, assets/b.png 2x\" alt=\"assets/a.png\" data-start=\"0\">\n\
<video src='assets/a.png#t=1' poster=assets/a.png data-start=\"1\"></video>\n\
<!-- <img src=\"assets/a.png\"> -->\n<a href=\"assets/a.png\">keep</a>\n\n---\n\n\
![Chart](assets/a.png \"title\")\n\n```html\n<img src=\"assets/a.png\">\n```\n";

fn rename(from: &str, to: &str) -> Op {
    Op::RenameMedia { from: from.to_owned(), to: to.to_owned() }
}

#[test]
fn every_place_the_compiler_loads_from_changes_and_nothing_else() {
    let renamed = apply(PLACES, &[rename("assets/a.png", "media/c.png")]).unwrap();
    let expected = "---\nlayout: raw\nduration: 4\n---\n\n<style>\n  @import \"media/c.png\";\n  \
.bg { background: url(\"./media/c.png?v=2\"); }\n</style>\n\
<img src=\"media/c.png\" srcset=\"media/c.png 1x, assets/b.png 2x\" alt=\"assets/a.png\" data-start=\"0\">\n\
<video src='media/c.png#t=1' poster=media/c.png data-start=\"1\"></video>\n\
<!-- <img src=\"assets/a.png\"> -->\n<a href=\"assets/a.png\">keep</a>\n\n---\n\n\
![Chart](media/c.png \"title\")\n\n```html\n<img src=\"assets/a.png\">\n```\n";
    assert_eq!(renamed, expected);
    assert_eq!(apply(&renamed, &[rename("media/c.png", "assets/a.png")]).unwrap(), PLACES);
}

#[test]
fn a_clip_follows_its_file_and_a_source_without_it_stays_as_it_is() {
    let renamed = apply(LAUNCH, &[rename("assets/shot.mp4", "assets/launch-shot.mp4")]).unwrap();
    assert_eq!(renamed, LAUNCH.replace("src=\"assets/shot.mp4\"", "src=\"assets/launch-shot.mp4\""));
    let clip = read(&renamed).unwrap().clip("shot").cloned().unwrap();
    assert_eq!(clip.src.as_deref(), Some("assets/launch-shot.mp4"));
    assert_eq!(apply(LAUNCH, &[rename("assets/none.mp4", "assets/other.mp4")]).unwrap(), LAUNCH);
}

#[test]
fn a_bad_or_taken_name_is_refused() {
    let taken = apply(LAUNCH, &[rename("assets/shot.mp4", "assets/music.mp3")]).unwrap_err();
    assert_eq!((taken.code(), taken), ("name_taken", Error::NameTaken("assets/music.mp3".into())));
    for bad in ["../x.mp4", "assets/a b.mp4", "/abs.mp4", "https://x.test/a.mp4", ""] {
        let refused = apply(LAUNCH, &[rename("assets/shot.mp4", bad)]).unwrap_err();
        assert_eq!(refused.code(), "invalid_op", "{bad}");
    }
    assert_eq!(apply(LAUNCH, &[rename("assets/shot.mp4", "assets/shot.mp4")]).unwrap_err().code(), "invalid_op");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn a_rename_undoes_to_the_same_text(spec in scenes(), pick in 0usize..4) {
        let original = source(&spec);
        let from = format!("assets/v{pick}.mp4");
        let renamed = apply(&original, &[rename(&from, "assets/renamed.mp4")]).unwrap();
        prop_assert_eq!(&renamed, &original.replace(&format!("\"{from}\""), "\"assets/renamed.mp4\""));
        let (before, after) = (read(&original).unwrap(), read(&renamed).unwrap());
        for (old, new) in before.scenes.iter().flat_map(|scene| &scene.clips).zip(after.scenes.iter().flat_map(|scene| &scene.clips)) {
            let moved = old.src.as_deref() == Some(from.as_str());
            prop_assert_eq!(new.src.as_deref(), if moved { Some("assets/renamed.mp4") } else { old.src.as_deref() });
            prop_assert_eq!((&old.id, old.start, old.duration), (&new.id, new.start, new.duration));
        }
        prop_assert_eq!(apply(&renamed, &[rename("assets/renamed.mp4", &from)]).unwrap(), original);
    }
}
