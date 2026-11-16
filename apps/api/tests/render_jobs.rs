//! Queueing a render: the job directory the API writes for the worker, and what it refuses.

mod common;

use common::render::{dir, project, queued};
use common::upload::PNG;
use serde_json::{Value, json};

fn references(html: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (open, close) in [("src=\"", '"'), ("url(", ')')] {
        for (at, _) in html.match_indices(open) {
            let rest = &html[at + open.len()..];
            found.push(rest[..rest.find(close).unwrap()].to_owned());
        }
    }
    found
}

fn job_file(app: &common::App, render: &Value) -> Value {
    let job: Value = serde_json::from_slice(&std::fs::read(dir(app, render).join("job.json")).unwrap()).unwrap();
    let mut keys: Vec<&str> = job.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["created_at", "fps", "height", "id", "range", "width"]);
    let stamp = time::format_description::well_known::Rfc3339;
    assert!(time::OffsetDateTime::parse(job["created_at"].as_str().unwrap(), &stamp).is_ok(), "{job}");
    job
}

fn whole_bundle(bundle: &std::path::Path) {
    let html = std::fs::read_to_string(bundle.join("index.html")).unwrap();
    let named = references(&html);
    assert_eq!(named, ["media/assets/dot.png", "media/assets/dot.png#frame", "media/assets/back.png"]);
    for reference in named {
        let file = reference.split('#').next().unwrap();
        assert_eq!(std::fs::read(bundle.join(file)).unwrap(), PNG, "{file}");
    }
    assert!(!html.contains("/api/") && !html.contains("?frame="), "{html}");
}

#[tokio::test]
async fn a_queued_render_writes_a_whole_bundle_then_a_valid_job_file() {
    let Some(pool) = common::pool("clap_renders_enqueue").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let render = queued(&app, &ada, &id, json!({"fps": 24, "range": [0.5, 1.5]})).await;
    assert_eq!(
        (render["state"].clone(), render["frames"].clone(), render["rev"].clone()),
        (json!("queued"), json!(24), json!(1))
    );
    assert_eq!((render["download_url"].clone(), render["range"].clone()), (Value::Null, json!([0.5, 1.5])));
    let job = job_file(&app, &render);
    assert_eq!(
        (job["id"].clone(), job["width"].clone(), job["height"].clone()),
        (render["id"].clone(), json!(320), json!(240))
    );
    assert_eq!((job["fps"].clone(), job["range"].clone()), (json!(24), json!([0.5, 1.5])));
    whole_bundle(&dir(&app, &render).join("bundle"));
    let left: Vec<String> =
        std::fs::read_dir(dir(&app, &render)).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    assert_eq!(left.len(), 2, "{left:?}");
    let whole = queued(&app, &ada, &id, json!({})).await;
    let job = job_file(&app, &whole);
    assert_eq!(
        (job["range"].clone(), job["fps"].clone(), whole["frames"].clone()),
        (Value::Null, json!(30), json!(60))
    );
}

#[tokio::test]
async fn a_render_past_a_cap_or_with_a_missing_file_is_refused_and_queues_nothing() {
    let Some(pool) = common::pool("clap_renders_refused").await else { return };
    let app = common::app_with(pool, &[("RENDER_QUEUE_MAX", "1")]).await;
    let roles = common::fixture::roles(&app).await;
    let id = project(&app, &roles.owner, Some(&roles.space)).await;
    let path = format!("/api/projects/{id}/renders");
    for request in
        [json!({"width": 4000, "height": 2000}), json!({"fps": 120}), json!({"range": [1, 9]}), json!({"width": 321})]
    {
        let refused = app.call("POST", &path, Some(&roles.member), Some(request.clone())).await;
        assert_eq!(refused.status, 400, "{request}: {}", refused.text());
    }
    assert_eq!(app.call("POST", &path, Some(&roles.member), Some(json!({"crf": 1}))).await.status, 400);
    assert_eq!(app.call("POST", &path, Some(&roles.outsider), Some(json!({}))).await.status, 404);
    let gone = format!("/api/projects/{id}/assets/assets/back.png");
    assert_eq!(app.call("DELETE", &gone, Some(&roles.member), None).await.status, 204);
    let missing = app.call("POST", &path, Some(&roles.member), Some(json!({}))).await;
    assert_eq!(missing.status, 412, "{}", missing.text());
    assert!(missing.json()["error"]["message"].as_str().unwrap().contains("assets/back.png"));
    assert!(!app.state.data_dir.join("renders").exists());
    assert_eq!(app.call("GET", &path, Some(&roles.member), None).await.json(), json!({"renders": []}));
    assert_eq!(common::upload::png(&app, &roles.member, &id, "assets/back.png").await.status, 201);
    queued(&app, &roles.member, &id, json!({})).await;
    let full = app.call("POST", &path, Some(&roles.member), Some(json!({}))).await;
    assert_eq!((full.status.as_u16(), full.json()["error"]["code"].clone()), (429, json!("rate_limited")));
}
