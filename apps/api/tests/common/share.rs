use serde_json::{Value, json};

use super::render::{finished, mirrored, project, queued};
use super::{App, Person};

pub const VIDEO: &[u8] = b"\0\0\0\x18ftypisom-shared-render";

pub async fn link(app: &App, who: &Person, project: &str) -> String {
    let link = app.call("POST", &format!("/api/projects/{project}/share"), Some(who), None).await;
    assert_eq!(link.status, 201, "{}", link.text());
    let url = link.json()["url"].as_str().unwrap().to_owned();
    assert!(url.starts_with(&format!("{}/v/", super::HOST)), "{url}");
    url.rsplit('/').next().unwrap().to_owned()
}

pub async fn done(app: &App, who: &Person, project: &str, video: &[u8]) -> Value {
    let render = queued(app, who, project, json!({"fps": 24})).await;
    finished(app, &render, video);
    let render = mirrored(app, who, &render).await;
    assert_eq!(render["state"], json!("done"), "{render}");
    render
}

pub async fn shared(app: &App, who: &Person, space: Option<&str>) -> (String, String) {
    let id = project(app, who, space).await;
    sqlx::query("UPDATE projects SET title = $2 WHERE id = $1::uuid")
        .bind(&id)
        .bind("Launch \"day\" <b>")
        .execute(&app.pool)
        .await
        .unwrap();
    done(app, who, &id, VIDEO).await;
    let token = link(app, who, &id).await;
    (id, token)
}
