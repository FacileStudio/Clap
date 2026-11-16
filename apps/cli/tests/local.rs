//! The offline commands, through the binary.

#[path = "support/bin.rs"]
mod bin;

use std::fs;
use std::path::Path;
use std::process::Output;

fn clap(cwd: &Path, args: &[&str]) -> Output {
    bin::command(cwd, cwd, args).output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn new_makes_a_valid_project_from_every_starter() {
    let work = tempfile::tempdir().unwrap();
    let listed = text(&clap(work.path(), &["starters"]).stdout);
    let names: Vec<&str> = listed.lines().filter_map(|line| line.split_whitespace().next()).collect();
    assert_eq!(names, ["title", "lower-third", "caption", "chart-race", "logo-reveal"]);
    for name in &names {
        let made = clap(work.path(), &["new", name, "--template", name]);
        assert!(made.status.success() && work.path().join(name).join("assets").is_dir(), "{}", text(&made.stderr));
        let checked = clap(work.path(), &["check", name]);
        assert!(checked.status.success(), "{name}: {}", text(&checked.stdout));
    }
    let source = fs::read_to_string(work.path().join("caption/clap.md")).unwrap();
    assert!(
        source.starts_with("---\ntitle: \"caption\"\nsize: 1920x1080\nlayout: raw\nduration: 6\n---\n"),
        "{source}"
    );
    assert!(text(&clap(work.path(), &["check", "caption"]).stdout).contains("is valid (1 scenes, 3 clips, 6 s)"));
    assert_eq!(clap(work.path(), &["new", "title"]).status.code(), Some(1));
    let unknown = clap(work.path(), &["new", "other", "--template", "nope"]);
    assert!(text(&unknown.stderr).contains("run `clap starters`") && !work.path().join("other").exists());
}

#[test]
fn check_prints_every_problem_with_file_and_line_and_speaks_json() {
    let work = tempfile::tempdir().unwrap();
    clap(work.path(), &["new", "demo"]);
    let file = work.path().join("demo/clap.md");
    let source = fs::read_to_string(&file).unwrap();
    fs::write(&file, source.replace("layout: raw", "colour: red") + "\n<img src=\"assets/gone.png\">\n").unwrap();
    let broken = clap(work.path(), &["check", "demo"]);
    assert_eq!(broken.status.code(), Some(1));
    assert_eq!(text(&broken.stdout).lines().next(), Some("demo/clap.md:4: unknown key `colour`"));
    fs::write(&file, source + "\n<img src=\"assets/gone.png\">\n").unwrap();
    let missing = clap(work.path(), &["check", "demo", "--json"]);
    let answer: serde_json::Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert_eq!((missing.status.code(), &answer["valid"]), (Some(1), &serde_json::json!(false)));
    assert_eq!((&answer["errors"][0]["code"], &answer["timeline"]["clips"]), (&"missing_asset".into(), &1.into()));
    let nothing = clap(work.path(), &["check"]);
    assert!(text(&nothing.stderr).contains("no clap.md in ."), "{}", text(&nothing.stderr));
}

#[test]
fn timeline_prints_scenes_tracks_and_clips() {
    let work = tempfile::tempdir().unwrap();
    clap(work.path(), &["new", "demo", "--template", "caption"]);
    let printed = text(&clap(work.path(), &["timeline", "demo"]).stdout);
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines[..3], ["1920x1080  6 s  1 scenes", "scene 0  0 to 6 s  raw", "  track 0"], "{printed}");
    assert_eq!(lines[4], "    st-caption-2  text   2 to 4 s", "{printed}");
    let json: serde_json::Value =
        serde_json::from_slice(&clap(work.path(), &["timeline", "demo", "--json"]).stdout).unwrap();
    assert_eq!(json["scenes"][0]["clips"][2]["id"], "st-caption-3");
}

#[test]
fn edit_patches_the_file_and_refuses_what_does_not_apply() {
    let work = tempfile::tempdir().unwrap();
    clap(work.path(), &["new", "demo", "--template", "caption"]);
    let file = work.path().join("demo/clap.md");
    let before = fs::read_to_string(&file).unwrap();
    let ops = r#"[{"op":"set_scene_duration","scene":0,"duration":7},
        {"op":"move_clip","clip":"st-caption-3","start":4.5},
        {"op":"trim_clip","clip":"st-caption-3","duration":1.5}]"#;
    let edited = clap(work.path(), &["edit", "demo", "--ops", ops]);
    assert!(text(&edited.stdout).contains("Applied 3 operations to demo/clap.md"), "{}", text(&edited.stderr));
    let golden = before
        .replace(
            "id=\"st-caption-3\" data-start=\"4\" data-duration=\"2\"",
            "id=\"st-caption-3\" data-start=\"4.5\" data-duration=\"1.5\"",
        )
        .replace("duration: 6\n", "duration: 7\n");
    assert_eq!(fs::read_to_string(&file).unwrap(), golden);
    fs::write(work.path().join("ops.json"), r#"{"op":"delete_clip","clip":"nope"}"#).unwrap();
    let unknown = clap(work.path(), &["edit", "demo", "--ops", "@ops.json", "--json"]);
    let envelope: serde_json::Value = serde_json::from_slice(&unknown.stdout).unwrap();
    assert_eq!((unknown.status.code(), &envelope["error"]["code"]), (Some(1), &"unknown_clip".into()));
    let past = clap(work.path(), &["edit", "demo", "--ops", r#"{"op":"move_clip","clip":"st-caption-1","start":9}"#]);
    assert!(text(&past.stderr).contains("out of bounds"), "{}", text(&past.stderr));
    assert_eq!(clap(work.path(), &["edit", "demo", "--ops", "not json"]).status.code(), Some(1));
    assert_eq!(fs::read_to_string(&file).unwrap(), golden);
}

#[test]
fn version_usage_errors_and_the_guide_follow_the_standard() {
    let work = tempfile::tempdir().unwrap();
    let version = clap(work.path(), &["--version"]);
    assert_eq!(text(&version.stdout), format!("clap {}\n", env!("CARGO_PKG_VERSION")));
    assert_eq!(clap(work.path(), &["frobnicate"]).status.code(), Some(2));
    assert_eq!(clap(work.path(), &["render", "--range"]).status.code(), Some(2));
    let guide = text(&clap(work.path(), &["guide"]).stdout);
    assert!(guide.starts_with("# The scene format") && guide.contains("# Clap: a video is a scene file"));
    assert!(guide.contains("## From the command line") && guide.contains("`clap edit [dir] --ops"));
}
