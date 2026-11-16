//! A project created without a source starts with the starter, in the caller's language.

mod common;

use axum::body::Body;
use common::{App, Person};
use http::{Request, header};
use serde_json::{Value, json};

async fn created(app: &App, who: &Person, accept: Option<&str>, body: Value) -> String {
    let mut request = Request::post("/api/projects")
        .header(header::HOST, "clap.test")
        .header(header::AUTHORIZATION, format!("Bearer {}", who.token))
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(accept) = accept {
        request = request.header(header::ACCEPT_LANGUAGE, accept);
    }
    let answer = common::http::raw(&app.router, request.body(Body::from(body.to_string())).unwrap()).await;
    assert_eq!(answer.status, 201, "{}", answer.text());
    answer.json()["source"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn a_project_without_a_source_starts_in_the_callers_language() {
    let Some(pool) = common::pool("clap_projects_starter").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let french = created(&app, &ada, None, json!({})).await;
    assert!(french.contains("lang: fr\n") && french.contains("Votre vidéo commence ici"), "{french}");
    let german = created(&app, &ada, Some("de-CH,de;q=0.9,en;q=0.5"), json!({"title": "Plan"})).await;
    assert!(german.contains("Ihr Video beginnt hier"), "{german}");
    let asked = created(&app, &ada, Some("de"), json!({"lang": "es"})).await;
    assert!(asked.contains("Tu vídeo empieza aquí"), "{asked}");
    let given = created(&app, &ada, Some("en"), json!({"source": "# Mine\n"})).await;
    assert_eq!(given, "# Mine\n");
    let timeline = clap_timeline::read(&french).unwrap();
    assert_eq!((timeline.duration, timeline.scenes[0].clips.len()), (4.0, 2));
}
