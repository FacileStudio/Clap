//! Share links: one live link per project, shown once, stored hashed, revoked for good.

mod common;

use common::share::{done, link, shared};
use serde_json::{Value, json};

#[tokio::test]
async fn a_link_is_shown_once_stored_hashed_rotated_and_revoked() {
    let Some(pool) = common::pool("clap_share").await else { return };
    let app = common::app(pool).await;
    let roles = common::fixture::roles(&app).await;
    let (id, token) = shared(&app, &roles.member, Some(&roles.space)).await;
    assert_eq!(token.len(), 43, "{token}");
    let path = format!("/api/projects/{id}/share");
    let stored: Vec<String> = sqlx::query_scalar("SELECT token_hash FROM share_links WHERE project_id = $1::uuid")
        .bind(&id)
        .fetch_all(&app.pool)
        .await
        .unwrap();
    assert_eq!(stored, [porte::hash_token(&token)]);
    let again = app.call("POST", &path, Some(&roles.owner), None).await;
    assert_eq!((again.status.as_u16(), again.json()["shared"].clone()), (200, json!(true)));
    assert!(again.json().get("url").is_none(), "{}", again.text());
    let state = app.call("GET", &path, Some(&roles.admin), None).await.json();
    assert!(state["shared"] == json!(true) && state["created_at"].is_string() && state.get("url").is_none());
    let public = format!("/api/public/renders/{token}");
    assert_eq!(app.call("GET", &public, None, None).await.status, 200);
    let rotated = app.call("POST", &format!("{path}?rotate=true"), Some(&roles.member), None).await;
    let fresh = rotated.json()["url"].as_str().unwrap().rsplit('/').next().unwrap().to_owned();
    assert_eq!((rotated.status.as_u16(), fresh == token), (201, false));
    assert_eq!(app.call("GET", &public, None, None).await.status, 404);
    assert_eq!(app.call("GET", &format!("/api/public/renders/{fresh}"), None, None).await.status, 200);
    assert_eq!(app.call("DELETE", &path, Some(&roles.member), None).await.status, 204);
    assert_eq!(app.call("GET", &format!("/api/public/renders/{fresh}"), None, None).await.status, 404);
    assert_eq!(app.call("DELETE", &path, Some(&roles.member), None).await.status, 404);
    let off = app.call("GET", &path, Some(&roles.member), None).await.json();
    assert_eq!((off["shared"].clone(), off["created_at"].clone()), (json!(false), Value::Null));
}

#[tokio::test]
async fn only_an_editor_shares_and_a_trashed_project_is_not_shared() {
    let Some(pool) = common::pool("clap_share_access").await else { return };
    let app = common::app(pool).await;
    let roles = common::fixture::roles(&app).await;
    let id = common::render::project(&app, &roles.owner, Some(&roles.space)).await;
    let path = format!("/api/projects/{id}/share");
    for method in ["POST", "GET", "DELETE"] {
        assert_eq!(app.call(method, &path, None, None).await.status, 401, "{method}");
        assert_eq!(app.call(method, &path, Some(&roles.outsider), None).await.status, 404, "{method}");
    }
    let (shared, token) = shared(&app, &roles.member, Some(&roles.space)).await;
    let trash = Some(json!({"trashed": true}));
    assert_eq!(
        app.call("PATCH", &format!("/api/projects/{shared}"), Some(&roles.owner), trash.clone()).await.status,
        200
    );
    assert_eq!(app.call("GET", &format!("/api/public/renders/{token}"), None, None).await.status, 404);
    assert_eq!(app.call("PATCH", &format!("/api/projects/{id}"), Some(&roles.owner), trash).await.status, 200);
    assert_eq!(app.call("POST", &path, Some(&roles.owner), None).await.status, 412);
}

#[tokio::test]
async fn a_link_shows_the_latest_finished_render_and_nothing_while_none_exists() {
    let Some(pool) = common::pool("clap_share_latest").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = common::render::project(&app, &ada, None).await;
    let token = link(&app, &ada, &id).await;
    let public = format!("/api/public/renders/{token}");
    assert_eq!(app.call("GET", &public, None, None).await.status, 404);
    common::render::queued(&app, &ada, &id, json!({})).await;
    assert_eq!(app.call("GET", &public, None, None).await.status, 404);
    let first = done(&app, &ada, &id, b"first").await;
    let video = format!("{public}/video.mp4");
    assert_eq!(app.call("GET", &video, None, None).await.bytes, b"first");
    done(&app, &ada, &id, b"second").await;
    assert_eq!(app.call("GET", &video, None, None).await.bytes, b"second");
    let one = format!("/api/renders/{}", first["id"].as_str().unwrap());
    assert_eq!(app.call("DELETE", &one, Some(&ada), None).await.status, 204);
    assert_eq!(app.call("GET", &video, None, None).await.bytes, b"second");
}
