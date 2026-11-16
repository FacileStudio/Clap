//! Renders: the status file the API reads back from the worker, and what it keeps.

mod common;

use axum::body::Body;
use common::render::{dir, finished, mirrored, project, queued, report};
use http::{Request, header};
use serde_json::{Value, json};

async fn served_by_range(app: &common::App, who: &common::Person, file: &str) {
    let request = Request::get(file)
        .header(header::AUTHORIZATION, format!("Bearer {}", who.token))
        .header(header::RANGE, "bytes=4-11")
        .body(Body::empty())
        .unwrap();
    let part = common::http::raw(&app.router, request).await;
    assert_eq!((part.status.as_u16(), part.bytes.as_slice()), (206, &b"ftypisom"[..]));
    assert_eq!(part.headers["content-type"], "video/mp4");
    assert_eq!(part.headers["content-disposition"], "attachment; filename=\"Untitled.mp4\"");
}

async fn running(app: &common::App, who: &common::Person, render: &Value) {
    let status =
        json!({"state": "running", "frame": 12, "frames": 60, "error": null, "updated_at": "2026-11-16T10:00:00Z"});
    report(app, render, status);
    let seen = mirrored(app, who, render).await;
    assert_eq!((seen["state"].clone(), seen["frame"].clone()), (json!("running"), json!(12)));
    assert!(seen["started_at"].is_string() && seen["finished_at"].is_null(), "{seen}");
    report(app, render, json!({"state": "exploded", "frame": 99}));
    assert_eq!(mirrored(app, who, render).await["frame"], json!(12));
}

#[tokio::test]
async fn the_status_file_is_mirrored_and_a_finished_file_is_kept_and_served_by_range() {
    let Some(pool) = common::pool("clap_renders_mirror").await else { return };
    let app = common::app(pool).await;
    let roles = common::fixture::roles(&app).await;
    let id = project(&app, &roles.owner, Some(&roles.space)).await;
    let render = queued(&app, &roles.member, &id, json!({})).await;
    assert_eq!(mirrored(&app, &roles.member, &render).await["state"], json!("queued"));
    running(&app, &roles.member, &render).await;
    let file = format!("/api/renders/{}/file", render["id"].as_str().unwrap());
    assert_eq!(app.call("GET", &file, Some(&roles.member), None).await.status, 412);
    finished(&app, &render, b"\0\0\0\x18ftypisom-rendered");
    let done = mirrored(&app, &roles.member, &render).await;
    assert_eq!(
        (done["state"].clone(), done["size"].clone(), done["download_url"].clone()),
        (json!("done"), json!(21), json!(file))
    );
    assert!(!dir(&app, &render).exists());
    served_by_range(&app, &roles.admin, &file).await;
    let one = format!("/api/renders/{}", render["id"].as_str().unwrap());
    for path in [one.clone(), file.clone(), format!("{one}/events")] {
        assert_eq!(app.call("GET", &path, Some(&roles.outsider), None).await.status, 404, "{path}");
    }
    assert_eq!(app.call("DELETE", &one, Some(&roles.outsider), None).await.status, 404);
    assert_eq!(app.call("DELETE", &one, Some(&roles.member), None).await.status, 204);
    assert_eq!(app.call("GET", &one, Some(&roles.member), None).await.status, 404);
    assert!(!app.state.data_dir.join("outputs").join(format!("{}.mp4", render["id"].as_str().unwrap())).exists());
}

#[tokio::test]
async fn a_failure_and_a_cancel_close_the_render() {
    let Some(pool) = common::pool("clap_renders_close").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let failing = queued(&app, &ada, &id, json!({})).await;
    report(&app, &failing, json!({"state": "failed", "frame": 3, "frames": 60, "error": "chrome exited"}));
    let failed = mirrored(&app, &ada, &failing).await;
    assert_eq!((failed["state"].clone(), failed["error"].clone()), (json!("failed"), json!("chrome exited")));
    assert!(failed["finished_at"].is_string() && dir(&app, &failing).join("status.json").is_file());
    let age = "UPDATE renders SET finished_at = now() - interval '25 hours' WHERE id = $1::uuid";
    sqlx::query(age).bind(failing["id"].as_str().unwrap()).execute(&app.pool).await.unwrap();
    let stopping = queued(&app, &ada, &id, json!({})).await;
    let one = format!("/api/renders/{}", stopping["id"].as_str().unwrap());
    let asked = app.call("DELETE", &one, Some(&ada), None).await;
    assert_eq!((asked.status.as_u16(), asked.json()["state"].clone()), (202, json!("queued")));
    assert!(dir(&app, &stopping).join("cancel").is_file());
    report(&app, &stopping, json!({"state": "cancelled", "frame": 0, "frames": 60, "error": null}));
    assert_eq!(mirrored(&app, &ada, &stopping).await["state"], json!("cancelled"));
    let list = app.call("GET", &format!("/api/projects/{id}/renders"), Some(&ada), None).await.json();
    let states: Vec<&str> = list["renders"].as_array().unwrap().iter().map(|r| r["state"].as_str().unwrap()).collect();
    assert_eq!(states, ["cancelled", "failed"]);
    assert_eq!(api::modules::renders::sweep::swept(&app.state).await.unwrap(), 1);
    assert!(!dir(&app, &failing).exists() && dir(&app, &stopping).exists());
    assert_eq!(app.call("DELETE", &one, Some(&ada), None).await.status, 204);
    assert!(!dir(&app, &stopping).exists());
}

#[tokio::test]
async fn a_link_in_place_of_the_finished_file_fails_the_render() {
    let Some(pool) = common::pool("clap_renders_link").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let linked = queued(&app, &ada, &id, json!({})).await;
    let secret = app.state.data_dir.join("secret");
    std::fs::write(&secret, b"not yours").unwrap();
    std::os::unix::fs::symlink(&secret, dir(&app, &linked).join("out.mp4")).unwrap();
    report(&app, &linked, json!({"state": "done", "frame": 60, "frames": 60}));
    let refused = mirrored(&app, &ada, &linked).await;
    assert_eq!((refused["state"].clone(), refused["download_url"].clone()), (json!("failed"), Value::Null));
    assert!(!app.state.data_dir.join("outputs").exists());
}

#[tokio::test]
async fn the_sweep_fails_lost_and_stale_jobs_and_removes_what_no_row_names() {
    let Some(pool) = common::pool("clap_renders_sweep").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let (waiting, lost, stale) = (
        queued(&app, &ada, &id, json!({})).await,
        queued(&app, &ada, &id, json!({})).await,
        queued(&app, &ada, &id, json!({})).await,
    );
    std::fs::remove_dir_all(dir(&app, &lost)).unwrap();
    let aged = |render: &Value, age: &str| {
        let query = format!("UPDATE renders SET created_at = now() - interval '{age}' WHERE id = $1::uuid");
        let (pool, id) = (app.pool.clone(), render["id"].as_str().unwrap().to_owned());
        async move { sqlx::query(sqlx::AssertSqlSafe(query)).bind(id).execute(&pool).await.unwrap() }
    };
    aged(&lost, "5 minutes").await;
    aged(&stale, "7 hours").await;
    let orphan = app.state.data_dir.join("renders").join(uuid::Uuid::new_v4().to_string());
    let output = app.state.data_dir.join("outputs").join(format!("{}.mp4", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(orphan.join("bundle")).unwrap();
    std::fs::create_dir_all(output.parent().unwrap()).unwrap();
    std::fs::write(&output, b"old").unwrap();
    assert_eq!(api::modules::renders::sweep::swept(&app.state).await.unwrap(), 2);
    assert!(!orphan.exists() && !output.exists() && !dir(&app, &stale).exists());
    assert!(dir(&app, &waiting).join("job.json").is_file());
    assert_eq!(mirrored(&app, &ada, &waiting).await["state"], json!("queued"));
    assert!(mirrored(&app, &ada, &lost).await["error"].as_str().unwrap().contains("lost"));
    assert!(mirrored(&app, &ada, &stale).await["error"].as_str().unwrap().contains("too long"));
}
