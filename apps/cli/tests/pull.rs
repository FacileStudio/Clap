//! `clap pull` through the binary, against a mock server.

#[path = "support/mock.rs"]
mod mock;
#[path = "support/project.rs"]
mod project;

use std::fs;

use clap_cli::sync::sha256;
use mock::mock;
use project::{clap, linked_folder, rev, text};

fn library(line: &str, _: &str) -> (u16, String) {
    let listing = format!(r#"{{"assets":[{{"name":"assets/chart.svg","size":6,"sha256":"{}"}}]}}"#, sha256(b"<svg/>"));
    match line {
        "GET /api/projects/d9" => {
            (200, r##"{"id":"d9","title":"Q3 Review","source":"# Remote\n","rev":7}"##.to_owned())
        }
        "GET /api/projects/d9/assets" => (200, listing),
        "GET /api/projects/d9/assets/assets/chart.svg" => (200, "<svg/>".to_owned()),
        "GET /api/projects/evil/assets" => (200, r#"[{"name":"../../escaped.txt","sha256":"x"}]"#.to_owned()),
        "GET /api/projects/evil" => (200, r##"{"id":"evil","title":"Evil","source":"# x\n","rev":1}"##.to_owned()),
        _ => (404, r#"{"error":{"code":"not_found","message":"project not found"}}"#.to_owned()),
    }
}

#[test]
fn pull_writes_the_project_and_refuses_to_lose_local_edits() {
    let (url, _) = mock(library);
    let work = linked_folder(&url, &[]);
    let pulled = clap(&work, &["pull", &format!("{url}/projects/d9")]);
    assert!(pulled.status.success(), "{}", text(&pulled.stderr));
    let dir = work.path().join("q3-review");
    assert_eq!(fs::read_to_string(dir.join("clap.md")).unwrap(), "# Remote\n");
    assert_eq!(fs::read_to_string(dir.join("assets/chart.svg")).unwrap(), "<svg/>");
    assert_eq!(rev(&dir), 7);
    assert!(text(&clap(&work, &["pull", "q3-review"]).stdout).contains("q3-review is up to date (revision 7)"));
    fs::write(dir.join("clap.md"), "# Edited here\n").unwrap();
    let refused = clap(&work, &["pull", "q3-review"]);
    assert_eq!(refused.status.code(), Some(1));
    assert!(text(&refused.stderr).contains("clap.md changed here since the last sync"), "{}", text(&refused.stderr));
    assert_eq!(fs::read_to_string(dir.join("clap.md")).unwrap(), "# Edited here\n");
    assert!(clap(&work, &["pull", "q3-review", "--force"]).status.success());
    assert_eq!(fs::read_to_string(dir.join("clap.md")).unwrap(), "# Remote\n");
}

#[test]
fn pull_refuses_an_asset_name_that_leaves_the_folder_and_reports_a_missing_project() {
    let (url, _) = mock(library);
    let work = linked_folder(&url, &[]);
    let evil = clap(&work, &["pull", "evil", "inside/project"]);
    assert_eq!(evil.status.code(), Some(1));
    assert!(text(&evil.stderr).contains("unsafe name"), "{}", text(&evil.stderr));
    assert!(!work.path().join("escaped.txt").exists() && !work.path().join("inside").exists());
    let missing = clap(&work, &["pull", "nope"]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(text(&missing.stderr).contains("the server answered 404: project not found"), "{}", text(&missing.stderr));
}
