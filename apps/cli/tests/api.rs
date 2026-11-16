//! The project and asset calls against a mock server.

#[path = "support/mock.rs"]
mod mock;

use std::net::TcpListener;

use clap_cli::api::{ApiError, Client, SourceError, Upload};
use mock::mock;

const INVALID: &str = r#"{"error":{"code":"invalid_argument","message":"line 2: unknown key `colour`"},
    "errors":[{"line":2,"message":"unknown key `colour`"}]}"#;

fn projects(line: &str, body: &str) -> (u16, String) {
    let project = r#"{"id":"d1","title":"Q3","source":"Hello\n","rev":4}"#;
    match line {
        "GET /api/projects" => (200, r#"[{"id":"d1","title":"Q3"},{"id":7,"title":"Seven","rev":2}]"#.to_owned()),
        "POST /api/projects" => (201, r#"{"id":"d2","title":"New"}"#.to_owned()),
        "GET /api/projects/d1" => (200, project.to_owned()),
        "PUT /api/projects/d1/source" if body.contains("colour") => (422, INVALID.to_owned()),
        "PUT /api/projects/d1/source" if body.contains(r#""rev":4"#) => (200, r#"{"rev":5}"#.to_owned()),
        "PUT /api/projects/d1/source" => {
            (409, r#"{"error":{"code":"already_exists","message":"the project moved"},"rev":4}"#.to_owned())
        }
        _ => (500, "boom".to_owned()),
    }
}

#[test]
fn projects_are_listed_created_and_read_with_the_bearer_token() {
    let (url, seen) = mock(projects);
    let client = Client::new(&url, Some("t0k"));
    let listed = client.projects().unwrap();
    assert_eq!((listed[0].id.as_str(), listed[1].id.as_str(), listed[1].rev), ("d1", "7", Some(2)));
    assert_eq!(client.create_project("New", "# New\n").unwrap().id, "d2");
    let project = client.project("d1").unwrap();
    assert_eq!((project.source.as_str(), project.rev), ("Hello\n", 4));
    let seen = seen.lock().unwrap();
    assert!(seen.iter().all(|line| line.contains("[Bearer t0k]")), "{seen:?}");
    assert!(seen[1].ends_with(r##"{"source":"# New\n","title":"New"}"##), "{seen:?}");
}

#[test]
fn a_list_wrapped_in_an_object_is_read_too() {
    let (url, _) = mock(|_, _| (200, r#"{"projects":[{"id":"a","title":"A"}],"total":1}"#.to_owned()));
    assert_eq!(Client::new(&url, None).projects().unwrap()[0].id, "a");
}

#[test]
fn a_write_names_its_revision_and_a_stale_one_is_a_conflict() {
    let (url, _) = mock(projects);
    let client = Client::new(&url, Some("t0k"));
    assert_eq!(client.put_source("d1", "New\n", 4).unwrap(), 5);
    let stale = client.put_source("d1", "New\n", 3).unwrap_err();
    assert!(stale.is_conflict());
    let expected =
        ApiError::Refused { status: 409, code: "already_exists".to_owned(), message: "the project moved".to_owned() };
    assert_eq!(stale, expected);
    let invalid = client.put_source("d1", "---\ncolour: red\n---\n# x\n", 4).unwrap_err();
    let problem = SourceError { line: 2, message: "unknown key `colour`".to_owned() };
    assert_eq!(invalid, ApiError::Invalid(vec![problem]));
}

#[test]
fn an_error_without_the_envelope_keeps_its_status_and_body() {
    let (url, _) = mock(projects);
    let error = Client::new(&url, None).project("missing").unwrap_err();
    assert_eq!(error, ApiError::Refused { status: 500, code: "http_500".to_owned(), message: "boom".to_owned() });
    assert!(!error.is_conflict());
}

#[test]
fn a_server_that_is_not_there_is_unreachable() {
    let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
    let error = Client::new(&format!("http://127.0.0.1:{port}"), None).projects().unwrap_err();
    assert!(matches!(error, ApiError::Unreachable(_)), "{error:?}");
}

fn assets(line: &str, _: &str) -> (u16, String) {
    match line {
        "GET /api/projects/d1/assets" => (
            200,
            r#"{"assets":[{"name":"assets/chart.svg","mime":"image/svg+xml","size":5,"sha256":"abc"}]}"#.to_owned(),
        ),
        "GET /api/projects/d1/assets/assets/img/a%20b.png" => (200, "PNGDATA".to_owned()),
        "POST /api/projects/d1/assets" => (201, "{}".to_owned()),
        "DELETE /api/projects/d1/assets/assets/old.png" => (204, String::new()),
        _ => (404, r#"{"error":{"code":"not_found","message":"no such asset"}}"#.to_owned()),
    }
}

#[test]
fn assets_are_listed_uploaded_downloaded_and_deleted() {
    let (url, seen) = mock(assets);
    let client = Client::new(&url, Some("t0k"));
    let listed = client.assets("d1").unwrap();
    assert_eq!((listed[0].name.as_str(), listed[0].size, listed[0].sha256.as_str()), ("assets/chart.svg", 5, "abc"));
    let mut fetched = Vec::new();
    let mut steps = Vec::new();
    client
        .download_asset("d1", "assets/img/a b.png", &mut fetched, &mut |done, total| steps.push((done, total)))
        .unwrap();
    assert_eq!((fetched.as_slice(), steps.last()), (&b"PNGDATA"[..], Some(&(7, 7))));
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("new.svg"), "<svg/>").unwrap();
    let file = dir.path().join("new.svg");
    let form = Upload { name: "assets/img/new.svg", kind: "image/svg+xml", path: &file, boundary: "clap-b0undary" };
    let mut sent = 0;
    client.upload_asset("d1", &form, &mut |done, _| sent = done).unwrap();
    assert!(sent > 6, "{sent}");
    client.delete_asset("d1", "assets/old.png").unwrap();
    let missing = client.delete_asset("d1", "nope.png").unwrap_err();
    assert!(matches!(missing, ApiError::Refused { status: 404, .. }), "{missing:?}");
    let upload = seen.lock().unwrap()[2].clone();
    assert!(upload.contains("name=\"name\"\r\n\r\nassets/img/new.svg\r\n"), "{upload}");
    assert!(upload.contains("name=\"file\"; filename=\"new.svg\""), "{upload}");
    assert!(upload.contains("Content-Type: image/svg+xml\r\n\r\n<svg/>\r\n--clap-b0undary--"), "{upload}");
    let missing_file = Upload { path: &dir.path().join("gone.svg"), ..form };
    assert!(matches!(client.upload_asset("d1", &missing_file, &mut |_, _| ()), Err(ApiError::Unreadable(_))));
}
