//! Every motion starter of `starters/` parses, compiles and renders. Skipped, loudly, when
//! Chrome or ffmpeg is missing.

use std::collections::HashSet;
use std::path::Path;

use clap_render::{Job, Tools, render};

const SKIP: &str = "\n================ SKIPPED: the starters need chrome-headless-shell and ffmpeg \
    (CLAP_CHROME, CLAP_FFMPEG or PATH) ================\n";

const STARTERS: [(&str, f64); 5] =
    [("title", 4.0), ("lower-third", 5.0), ("caption", 6.0), ("chart-race", 6.0), ("logo-reveal", 3.0)];

#[test]
fn every_starter_is_one_raw_scene_that_only_uses_the_timeline_helper() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../starters");
    let listed: HashSet<String> = STARTERS.iter().map(|(name, _)| format!("{name}.md")).collect();
    let found: HashSet<String> =
        std::fs::read_dir(&dir).unwrap().map(|entry| entry.unwrap().file_name().into_string().unwrap()).collect();
    assert_eq!(found.iter().filter(|name| *name != "README.md").cloned().collect::<HashSet<_>>(), listed);
    for (name, seconds) in STARTERS {
        let source = std::fs::read_to_string(dir.join(format!("{name}.md"))).unwrap();
        let project = scene::parse(&source).unwrap_or_else(|errors| panic!("{name}: {errors}"));
        let scenes = project.scenes();
        assert_eq!((scenes.len(), scenes[0].layout, scenes[0].duration), (1, scene::Layout::Raw, seconds), "{name}");
        for banned in
            ["Date", "Math.random", "setTimeout", "requestAnimationFrame", "fetch(", "transition:", "animation:"]
        {
            assert!(!source.contains(banned), "{name} uses {banned}");
        }
        assert!(source.contains("scene.timeline("), "{name} has no timeline");
    }
}

#[test]
fn every_starter_renders_and_moves() {
    let Ok(tools) = Tools::locate() else { return eprintln!("{SKIP}") };
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../starters");
    for (name, seconds) in STARTERS {
        let source = std::fs::read_to_string(dir.join(format!("{name}.md"))).unwrap();
        let bundle = tempfile::tempdir().unwrap();
        let html = scene::compile(&scene::parse(&source).unwrap(), &scene::Options::new());
        std::fs::write(bundle.path().join("index.html"), html).unwrap();
        let job = Job { width: 320, height: 180, fps: 5, ..Job::new(bundle.path(), bundle.path().join("out.mp4")) };
        let report = render(&job, &tools).unwrap_or_else(|error| panic!("{name}: {error}"));
        let distinct = report.frame_hashes.iter().collect::<HashSet<_>>().len();
        assert_eq!((report.frames, report.blocked.len()), ((seconds * 5.0) as u32, 0), "{name}: {:?}", report.blocked);
        assert!(distinct >= 4, "{name} drew only {distinct} different frames");
    }
}
