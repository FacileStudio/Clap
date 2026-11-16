//! Assets: streamed to disk, typed by their bytes, named by safe relative paths, capped, listed
//! and deleted.

mod common;

use common::fixture::{self, SOURCE};
use common::media::settled;
use common::upload::{File, PNG, png, upload};
use serde_json::json;

#[tokio::test]
async fn an_upload_is_typed_by_its_bytes_named_listed_and_served() {
    let Some(pool) = common::pool("clap_assets_upload").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::project(&app, &roles.member, Some(&roles.space), SOURCE).await);
    let stored = png(&app, &roles.member, &id, "assets/chart.png").await;
    assert_eq!(stored.status, 201, "{}", stored.text());
    let body = stored.json();
    assert_eq!((body["name"].clone(), body["mime"].clone()), (json!("assets/chart.png"), json!("image/png")));
    assert_eq!((body["kind"].clone(), body["state"].clone()), (json!("image"), json!("probing")));
    assert_eq!(body["url"], json!(format!("/api/projects/{id}/assets/assets/chart.png")));
    assert_eq!(body["size"], json!(PNG.len()));
    let unnamed = File { name: None, file_name: "../../Capture d'écran.png", bytes: PNG };
    assert_eq!(upload(&app, &roles.member, &id, unnamed).await.json()["name"], json!("assets/Capture-d--cran.png"));
    assert_eq!(settled(&app, &roles.owner, &id, "assets/chart.png", false).await["state"], json!("ready"));
    let list = app.call("GET", &format!("/api/projects/{id}/assets"), Some(&roles.owner), None).await.json();
    let names: Vec<&str> = list["assets"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["assets/Capture-d--cran.png", "assets/chart.png"]);
    let served = app.call("GET", body["url"].as_str().unwrap(), Some(&roles.admin), None).await;
    assert_eq!((served.status.as_u16(), served.bytes.as_slice()), (200, PNG));
}

#[tokio::test]
async fn the_same_name_replaces_the_asset_and_a_delete_forgets_it() {
    let Some(pool) = common::pool("clap_assets_replace").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::project(&app, &roles.member, Some(&roles.space), SOURCE).await);
    assert_eq!(png(&app, &roles.member, &id, "assets/chart.png").await.status, 201);
    let font = File { name: Some("assets/chart.png"), file_name: "x", bytes: b"wOF2-replaced" };
    let replaced = upload(&app, &roles.member, &id, font).await.json();
    assert_eq!((replaced["mime"].clone(), replaced["state"].clone()), (json!("font/woff2"), json!("ready")));
    let again = app.call("GET", &format!("/api/projects/{id}/assets"), Some(&roles.owner), None).await.json();
    assert_eq!(again["assets"].as_array().unwrap().len(), 1);
    let path = format!("/api/projects/{id}/assets/assets/chart.png");
    assert_eq!(app.call("GET", &format!("{path}?variant=proxy"), Some(&roles.member), None).await.status, 404);
    assert_eq!(app.call("DELETE", &path, Some(&roles.member), None).await.status, 204);
    assert_eq!(app.call("DELETE", &path, Some(&roles.member), None).await.status, 404);
    assert_eq!(app.call("GET", &path, Some(&roles.member), None).await.status, 404);
}

#[tokio::test]
async fn traversal_and_unsafe_names_are_refused_and_leave_nothing_on_disk() {
    let Some(pool) = common::pool("clap_assets_names").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, SOURCE).await);
    let refused = ["../secret.png", "assets/../../etc/passwd", "/etc/passwd", "assets\\a.png", "a b.png", ".env"];
    for name in refused.into_iter().chain(["https://x.test/a.png", "assets//a.png", "a/./b.png"]) {
        let answer = png(&app, &ada, &id, name).await;
        assert_eq!(answer.status, 400, "{name}: {}", answer.text());
        assert_eq!(answer.json()["error"]["code"], json!("invalid_argument"), "{name}");
    }
    let list = app.call("GET", &format!("/api/projects/{id}/assets"), Some(&ada), None).await.json();
    assert_eq!(list, json!({"assets": []}));
    assert!(!app.state.data_dir.join("assets").exists());
    assert_eq!(std::fs::read_dir(app.state.data_dir.join("uploads")).unwrap().count(), 0);
    for path in ["assets/..%2F..%2Fetc%2Fpasswd", "..%2Fx.png", "%2e%2e/%2e%2e/etc/passwd"] {
        let answer = app.call("GET", &format!("/api/projects/{id}/assets/{path}"), Some(&ada), None).await;
        assert_eq!(answer.status, 404, "{path}");
    }
}

#[tokio::test]
async fn html_scripts_and_unknown_bytes_are_refused_whatever_they_claim() {
    let Some(pool) = common::pool("clap_assets_sniff").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, SOURCE).await);
    for bytes in [&b"<!doctype html><script>alert(1)</script>"[..], b"alert(1)", b"%PDF-1.7", b"PK\x03\x04zip"] {
        let file = File { name: Some("assets/x.png"), file_name: "x.png", bytes };
        let answer = upload(&app, &ada, &id, file).await;
        assert_eq!(answer.status, 400, "{}", answer.text());
    }
    let empty = app.call("POST", &format!("/api/projects/{id}/assets"), Some(&ada), None).await;
    assert_eq!(empty.status, 400, "{}", empty.text());
    assert!(!app.state.data_dir.join("assets").exists());
}

#[tokio::test]
async fn an_asset_and_the_assets_of_a_space_are_capped() {
    let Some(pool) = common::pool("clap_assets_caps").await else { return };
    let app = common::app_with(pool, &[("ASSET_MAX_BYTES", "64"), ("SPACE_ASSETS_MAX_BYTES", "100")]).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::project(&app, &roles.owner, Some(&roles.space), SOURCE).await);
    let sibling = fixture::id(&fixture::project(&app, &roles.member, Some(&roles.space), SOURCE).await);
    let own = fixture::id(&fixture::project(&app, &roles.member, None, SOURCE).await);
    let big = [PNG, &[0u8; 64]].concat();
    let file = File { name: Some("assets/big.png"), file_name: "big.png", bytes: &big };
    assert_eq!(upload(&app, &roles.owner, &id, file).await.status, 413);
    let sized = [PNG, &[0u8; 20]].concat();
    for (project, name, status) in
        [(&id, "a.png", 201), (&id, "a.png", 201), (&sibling, "b.png", 413), (&own, "b.png", 201)]
    {
        let file = File { name: Some(name), file_name: name, bytes: &sized };
        assert_eq!(upload(&app, &roles.member, project, file).await.status, status, "{name}");
    }
    assert_eq!(std::fs::read_dir(app.state.data_dir.join("uploads")).unwrap().count(), 0);
    let trash = Some(json!({"trashed": true}));
    app.call("PATCH", &format!("/api/projects/{id}"), Some(&roles.owner), trash).await;
    let late = File { name: Some("c.png"), file_name: "c.png", bytes: PNG };
    assert_eq!(upload(&app, &roles.owner, &id, late).await.status, 412);
}

#[tokio::test]
async fn the_sweep_removes_only_old_files_no_row_names() {
    let Some(pool) = common::pool("clap_assets_sweep").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, SOURCE).await);
    let kept = png(&app, &ada, &id, "kept.png").await.json()["sha256"].as_str().unwrap().to_owned();
    settled(&app, &ada, &id, "kept.png", false).await;
    let dir = &app.state.data_dir;
    let (orphan, young) = ("ab".repeat(32), "cd".repeat(32));
    let files = api::modules::assets::files::path;
    let poster = dir.join(api::modules::assets::files::poster(&orphan));
    for path in [files(dir, &orphan), files(dir, &young), poster.clone()] {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, b"GIF89a").unwrap();
    }
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 60 * 60);
    for path in [files(dir, &orphan), poster.clone(), files(dir, &kept)] {
        std::fs::File::open(path).unwrap().set_modified(old).unwrap();
    }
    assert_eq!(api::modules::assets::sweep::sweep(&app.state).await.unwrap(), 2);
    assert!(!files(dir, &orphan).exists() && !poster.exists());
    assert!(files(dir, &young).exists() && files(dir, &kept).exists());
}
