//! `clap render --remote` through the binary, against a mock server.

#[path = "support/mock.rs"]
mod mock;
#[path = "support/project.rs"]
mod project;

use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};

use mock::mock;
use project::{clap, linked_folder, rev, text};

#[test]
fn a_remote_render_is_queued_followed_and_downloaded() {
    let polls = AtomicUsize::new(0);
    let (url, seen) = mock(move |line, _| match line {
        "POST /api/projects/d1/renders" => (202, r#"{"id":"r1","state":"queued","frame":0,"frames":0}"#.to_owned()),
        "GET /api/renders/r1" if polls.fetch_add(1, Ordering::Relaxed) == 0 => {
            (200, r#"{"id":"r1","state":"running","frame":12,"frames":45}"#.to_owned())
        }
        "GET /api/renders/r1" => (200, r#"{"id":"r1","state":"done","frame":45,"frames":45}"#.to_owned()),
        "GET /api/renders/r1/file" => (200, "MP4BYTES".to_owned()),
        _ => (404, r#"{"error":{"code":"not_found","message":"route not found"}}"#.to_owned()),
    });
    let work = linked_folder(&url, &[]);
    let args = ["render", "talk", "--remote", "-o", "talk.mp4", "--fps", "15", "--range", "0:3", "--json"];
    let rendered = clap(&work, &args);
    assert!(rendered.status.success(), "{}", text(&rendered.stderr));
    assert_eq!(fs::read_to_string(work.path().join("talk.mp4")).unwrap(), "MP4BYTES");
    let answer: serde_json::Value = serde_json::from_slice(&rendered.stdout).unwrap();
    assert_eq!((&answer["bytes"], &answer["render"]["frames"]), (&8.into(), &45.into()));
    assert!(text(&rendered.stderr).contains("changed since the last push"), "{}", text(&rendered.stderr));
    let seen = seen.lock().unwrap();
    assert!(seen[0].ends_with(r#"[Bearer s3cret] {"fps":15,"range":[0.0,3.0]}"#), "{seen:?}");
    assert!(!work.path().join("talk.mp4.part").exists());
    assert_eq!(rev(&work.path().join("talk")), 3, "a render does not move the link");
}

#[test]
fn a_remote_render_that_fails_or_is_refused_says_why() {
    let (url, _) = mock(|line, _| match line {
        "POST /api/projects/d1/renders" => (202, r#"{"id":"r2","state":"queued"}"#.to_owned()),
        "GET /api/renders/r2" => {
            (200, r#"{"id":"r2","state":"failed","frame":3,"frames":45,"error":"scene: boom"}"#.to_owned())
        }
        _ => (404, "{}".to_owned()),
    });
    let work = linked_folder(&url, &[]);
    let failed = clap(&work, &["render", "talk", "--remote"]);
    assert_eq!(failed.status.code(), Some(1));
    assert!(text(&failed.stderr).contains("the server render failed: scene: boom"), "{}", text(&failed.stderr));
    assert!(!work.path().join("talk.mp4").exists());
    let missing = r#"{"error":{"code":"failed_precondition","message":"the project does not hold assets/shot.mp4"}}"#;
    let (strict, _) = mock(move |_, _| (412, missing.to_owned()));
    let work = linked_folder(&strict, &[]);
    let refused = clap(&work, &["render", "talk", "--remote"]);
    assert!(
        text(&refused.stderr).contains("does not hold assets/shot.mp4 — run `clap push` first"),
        "{}",
        text(&refused.stderr)
    );
}
