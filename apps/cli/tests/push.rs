//! `clap push` through the binary, against a mock server.

#[path = "support/mock.rs"]
mod mock;
#[path = "support/project.rs"]
mod project;

use std::fs;

use clap_cli::sync::{Link, sha256};
use mock::mock;
use project::{clap, linked_folder, rev, text};

#[test]
fn push_stops_on_a_stale_revision_and_force_refetches_it() {
    let (url, seen) = mock(|line, body| match line {
        "PUT /api/projects/d1/source" if body.contains(r#""rev":5"#) => (200, r#"{"rev":6}"#.to_owned()),
        "PUT /api/projects/d1/source" => {
            (409, r#"{"error":{"code":"already_exists","message":"stale rev"},"rev":5}"#.to_owned())
        }
        "GET /api/projects/d1" => (200, r##"{"id":"d1","title":"T","source":"# Theirs\n","rev":5}"##.to_owned()),
        _ => (200, r#"{"assets":[]}"#.to_owned()),
    });
    let work = linked_folder(&url, &[]);
    let stale = clap(&work, &["push", "talk", "--json"]);
    assert_eq!(stale.status.code(), Some(1));
    assert!(text(&stale.stderr).contains("`clap pull`") && text(&stale.stderr).contains("`clap push --force`"));
    let envelope: serde_json::Value = serde_json::from_slice(&stale.stdout).unwrap();
    assert_eq!(envelope["error"]["code"], "conflict");
    assert_eq!((rev(&work.path().join("talk")), seen.lock().unwrap().len()), (3, 1));
    let forced = clap(&work, &["push", "talk", "--force"]);
    assert!(forced.status.success(), "{}", text(&forced.stderr));
    assert!(text(&forced.stdout).contains("Pushed talk/clap.md to project d1 (revision 6)"));
    assert_eq!(rev(&work.path().join("talk")), 6);
    assert!(seen.lock().unwrap().iter().all(|line| line.contains("[Bearer s3cret]")));
}

#[test]
fn push_syncs_assets_by_hash_and_leaves_what_it_never_had() {
    let listing = r#"{"assets":[{"name":"assets/kept.svg","sha256":"KEPT"},{"name":"assets/dropped.png","sha256":"h"},{"name":"assets/theirs.png","sha256":"z"}]}"#;
    let listing = listing.replace("KEPT", &sha256(b"<svg/>"));
    let (url, seen) = mock(move |line, _| match line {
        "PUT /api/projects/d1/source" => (200, r#"{"rev":4}"#.to_owned()),
        "GET /api/projects/d1/assets" => (200, listing.clone()),
        _ => (200, "{}".to_owned()),
    });
    let work = linked_folder(&url, &[("assets/dropped.png", "h")]);
    fs::write(work.path().join("talk/assets/kept.svg"), "<svg/>").unwrap();
    fs::write(work.path().join("talk/assets/new.svg"), "<svg>new</svg>").unwrap();
    let pushed = clap(&work, &["push", "talk"]);
    let out = text(&pushed.stdout);
    assert!(pushed.status.success(), "{}", text(&pushed.stderr));
    assert!(out.contains("Uploading assets/new.svg") && out.contains("Deleting assets/dropped.png"), "{out}");
    assert!(out.contains("assets/theirs.png only on the server"), "{out}");
    let calls: Vec<String> =
        seen.lock().unwrap().iter().map(|line| line.split(" [").next().unwrap().to_owned()).collect();
    let expected = ["PUT /api/projects/d1/source", "GET /api/projects/d1/assets", "POST /api/projects/d1/assets"];
    assert_eq!(calls[..3], expected);
    assert_eq!(calls[3..], ["DELETE /api/projects/d1/assets/assets/dropped.png"]);
    let link = Link::load(&work.path().join("talk")).unwrap().unwrap();
    assert_eq!(link.assets.keys().collect::<Vec<_>>(), ["assets/kept.svg", "assets/new.svg"]);
}

const REJECTED: &str =
    r#"{"error":{"code":"invalid_argument","message":"line 1: x"},"errors":[{"line":1,"message":"x"}]}"#;

#[test]
fn push_creates_a_project_from_the_folder_and_names_what_the_server_refuses() {
    let (url, seen) = mock(|line, body| match line {
        "POST /api/projects" => (201, r#"{"id":"d7","title":"Mine","rev":1}"#.to_owned()),
        "GET /api/projects/d7/assets" => (200, r#"{"assets":[]}"#.to_owned()),
        "POST /api/projects/d7/assets" if body.contains("notes.html") => {
            (400, r#"{"error":{"code":"invalid_argument","message":"this file type is not accepted"}}"#.to_owned())
        }
        "PUT /api/projects/d7/source" if body.contains("SERVER-REJECTS") => (422, REJECTED.to_owned()),
        "PUT /api/projects/d7/source" => (200, r#"{"rev":2}"#.to_owned()),
        _ => (201, "{}".to_owned()),
    });
    let work = linked_folder(&url, &[]);
    fs::remove_file(work.path().join("talk/.clap.json")).unwrap();
    assert_eq!(clap(&work, &["push", "talk"]).status.code(), Some(1));
    let created = clap(&work, &["push", "talk", "--create"]);
    assert!(created.status.success(), "{}", text(&created.stderr));
    assert!(seen.lock().unwrap()[0].ends_with(r##"{"source":"# Mine\n","title":"talk"}"##));
    let link = Link::load(&work.path().join("talk")).unwrap().unwrap();
    assert_eq!((link.rev, link.project.as_str()), (1, "d7"));
    fs::write(work.path().join("talk/clap.md"), "# SERVER-REJECTS\n").unwrap();
    let invalid = clap(&work, &["push", "talk"]);
    assert!(text(&invalid.stderr).contains("talk/clap.md:1: x"), "{}", text(&invalid.stderr));
    fs::write(work.path().join("talk/assets/notes.html"), "<p>hi</p>").unwrap();
    fs::write(work.path().join("talk/clap.md"), "# Mine\n\n![](assets/notes.html)\n").unwrap();
    let refused = clap(&work, &["push", "talk"]);
    assert_eq!(refused.status.code(), Some(1));
    assert!(text(&refused.stderr).contains("the server refused assets/notes.html: this file type is not accepted"));
}
