//! The SSE stream: where the project is, then every change, whichever door wrote it.

mod common;

use axum::body::Body;
use common::fixture::{self, SOURCE};
use http::{Request, header};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

type Stream = axum::body::Body;

async fn open(app: &common::App, who: &common::Person, project: &str) -> axum::response::Response {
    stream(app, who, &format!("/api/projects/{project}/events")).await
}

async fn stream(app: &common::App, who: &common::Person, path: &str) -> axum::response::Response {
    let request = Request::get(path)
        .header(header::HOST, "clap.test")
        .header(header::AUTHORIZATION, format!("Bearer {}", who.token))
        .body(Body::empty())
        .unwrap();
    app.router.clone().oneshot(request).await.unwrap()
}

async fn next(stream: &mut Stream) -> Option<String> {
    let wait = std::time::Duration::from_secs(5);
    let frame = tokio::time::timeout(wait, stream.frame()).await.expect("no event within 5 s")?;
    Some(String::from_utf8(frame.unwrap().into_data().unwrap().to_vec()).unwrap())
}

#[tokio::test]
async fn an_open_stream_hears_where_the_project_is_then_every_write() {
    let Some(pool) = common::pool("clap_events").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::project(&app, &roles.owner, Some(&roles.space), SOURCE).await);
    let other = fixture::id(&fixture::project(&app, &roles.owner, Some(&roles.space), SOURCE).await);
    let response = open(&app, &roles.member, &id).await;
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let mut stream = response.into_body();
    assert_eq!(next(&mut stream).await.unwrap(), "event: changed\ndata: {\"rev\":1}\n\n");
    let write = |project: &str, rev: i64| {
        let path = format!("/api/projects/{project}/source");
        let body = json!({"source": "# Changed\n", "rev": rev});
        let (app, who) = (&app, &roles.owner);
        async move { app.call("PUT", &path, Some(who), Some(body)).await.status }
    };
    assert_eq!(write(&other, 1).await, 200);
    assert_eq!(write(&id, 1).await, 200);
    assert_eq!(next(&mut stream).await.unwrap(), "event: changed\ndata: {\"rev\":2}\n\n");
    assert_eq!(write(&id, 1).await, 409);
    let rename = Some(json!({"title": "Renamed"}));
    assert_eq!(app.call("PATCH", &format!("/api/projects/{id}"), Some(&roles.owner), rename).await.status, 200);
    assert_eq!(next(&mut stream).await.unwrap(), "event: changed\ndata: {\"rev\":2}\n\n");
}

#[tokio::test]
async fn a_stream_is_refused_to_strangers_and_ends_when_access_is_lost() {
    let Some(pool) = common::pool("clap_events_access").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::project(&app, &roles.owner, Some(&roles.space), SOURCE).await);
    assert_eq!(open(&app, &roles.outsider, &id).await.status(), 404);
    let mut stream = open(&app, &roles.member, &id).await.into_body();
    assert!(next(&mut stream).await.unwrap().contains("\"rev\":1"));
    sqlx::query("DELETE FROM space_members WHERE subject = $1")
        .bind(&roles.member.subject)
        .execute(&app.pool)
        .await
        .unwrap();
    let body = json!({"source": "# Changed\n", "rev": 1});
    app.call("PUT", &format!("/api/projects/{id}/source"), Some(&roles.owner), Some(body)).await;
    assert_eq!(next(&mut stream).await, None);
}

#[tokio::test]
async fn an_open_stream_hears_each_asset_arrive_settle_and_go() {
    let Some(pool) = common::pool("clap_events_assets").await else { return };
    let app = common::app(pool).await;
    let Some(_worker) = common::worker::media(&app).await else { return };
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, SOURCE).await);
    let mut stream = open(&app, &ada, &id).await.into_body();
    assert!(next(&mut stream).await.unwrap().contains("\"rev\":1"));
    assert_eq!(common::upload::png(&app, &ada, &id, "assets/dot.png").await.status, 201);
    let probing = "event: asset\ndata: {\"name\":\"assets/dot.png\",\"state\":\"probing\"}\n\n";
    assert_eq!(next(&mut stream).await.unwrap(), probing);
    assert_eq!(next(&mut stream).await.unwrap(), probing.replace("probing", "ready"));
    let path = format!("/api/projects/{id}/assets/assets/dot.png");
    assert_eq!(app.call("DELETE", &path, Some(&ada), None).await.status, 204);
    assert_eq!(next(&mut stream).await.unwrap(), probing.replace("probing", "deleted"));
}

#[tokio::test]
async fn a_render_stream_follows_the_render_to_its_end() {
    let Some(pool) = common::pool("clap_events_render").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = common::render::project(&app, &ada, None).await;
    let render = common::render::queued(&app, &ada, &id, json!({})).await;
    let path = format!("/api/renders/{}/events", render["id"].as_str().unwrap());
    let mut events = stream(&app, &ada, &path).await.into_body();
    let state = |frame: String| -> serde_json::Value {
        let data = frame.strip_prefix("event: status\ndata: ").unwrap_or_else(|| panic!("{frame}"));
        serde_json::from_str(data.trim()).unwrap()
    };
    assert_eq!(state(next(&mut events).await.unwrap())["state"], json!("queued"));
    common::render::report(&app, &render, json!({"state": "running", "frame": 7, "frames": 60}));
    common::render::mirrored(&app, &ada, &render).await;
    let running = state(next(&mut events).await.unwrap());
    assert_eq!((running["state"].clone(), running["frame"].clone()), (json!("running"), json!(7)));
    common::render::finished(&app, &render, b"mp4");
    common::render::mirrored(&app, &ada, &render).await;
    let done = state(next(&mut events).await.unwrap());
    assert_eq!((done["state"].clone(), done["download_url"].is_string()), (json!("done"), true));
    assert_eq!(next(&mut events).await, None);
}
