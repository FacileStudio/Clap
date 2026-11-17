//! Ruler markers: stored by the API beside the source, read by a viewer, written by an editor.

mod common;

use common::fixture::{self, SOURCE};
use serde_json::{Value, json};

#[tokio::test]
async fn markers_are_added_listed_by_time_moved_renamed_and_deleted() {
    let Some(pool) = common::pool("clap_markers").await else { return };
    let app = common::app(pool).await;
    let roles = common::fixture::roles(&app).await;
    let id = fixture::id(&fixture::project(&app, &roles.member, Some(&roles.space), SOURCE).await);
    let path = format!("/api/projects/{id}/markers");
    let drop = app.call("POST", &path, Some(&roles.member), Some(json!({"t": 2.5, "label": "  Drop "}))).await;
    assert_eq!(drop.status, 201, "{}", drop.text());
    let drop = drop.json();
    assert_eq!((drop["t"].clone(), drop["label"].clone()), (json!(2.5), json!("Drop")));
    assert!(drop["id"].is_string() && drop["created_at"].is_string());
    let start = app.call("POST", &path, Some(&roles.owner), Some(json!({"t": 0}))).await.json();
    assert_eq!(start["label"], json!(""));
    let listed = app.call("GET", &path, Some(&roles.admin), None).await.json();
    let times: Vec<f64> = listed["markers"].as_array().unwrap().iter().map(|m| m["t"].as_f64().unwrap()).collect();
    assert_eq!(times, [0.0, 2.5]);
    let one = format!("{path}/{}", drop["id"].as_str().unwrap());
    let moved = app.call("PATCH", &one, Some(&roles.member), Some(json!({"t": 4}))).await.json();
    assert_eq!((moved["t"].clone(), moved["label"].clone()), (json!(4.0), json!("Drop")));
    let named = app.call("PATCH", &one, Some(&roles.member), Some(json!({"label": "Chorus"}))).await.json();
    assert_eq!((named["t"].clone(), named["label"].clone()), (json!(4.0), json!("Chorus")));
    assert_eq!(app.call("DELETE", &one, Some(&roles.member), None).await.status, 204);
    assert_eq!(app.call("DELETE", &one, Some(&roles.member), None).await.status, 404);
    assert_eq!(app.call("PATCH", &one, Some(&roles.member), Some(json!({"t": 1}))).await.status, 404);
    let left = app.call("GET", &path, Some(&roles.member), None).await.json();
    assert_eq!(left["markers"].as_array().unwrap().len(), 1);
}

async fn setup(prefix: &str) -> Option<(common::App, common::fixture::Roles, String, String)> {
    let app = common::app(common::pool(prefix).await?).await;
    let roles = common::fixture::roles(&app).await;
    let id = fixture::id(&fixture::project(&app, &roles.member, Some(&roles.space), SOURCE).await);
    let other = fixture::id(&fixture::project(&app, &roles.member, Some(&roles.space), SOURCE).await);
    Some((app, roles, id, other))
}

#[tokio::test]
async fn a_marker_is_checked_and_reached_only_through_its_project() {
    let Some((app, roles, id, other)) = setup("clap_markers_rules").await else { return };
    let path = format!("/api/projects/{id}/markers");
    for bad in
        [json!({"t": -1}), json!({"t": 1, "label": "x".repeat(201)}), json!({"label": "x"}), json!({"t": 1, "x": 1})]
    {
        let refused = app.call("POST", &path, Some(&roles.member), Some(bad.clone())).await;
        assert!(refused.status.is_client_error() && refused.status != 404, "{bad} {}", refused.status);
    }
    let marker = app.call("POST", &path, Some(&roles.member), Some(json!({"t": 1}))).await.json();
    let mark = marker["id"].as_str().unwrap();
    let foreign = format!("/api/projects/{other}/markers/{mark}");
    assert_eq!(app.call("PATCH", &foreign, Some(&roles.member), Some(json!({"t": 2}))).await.status, 404);
    assert_eq!(app.call("DELETE", &foreign, Some(&roles.member), None).await.status, 404);
    assert_eq!(
        app.call("PATCH", &format!("{path}/nope"), Some(&roles.member), Some(json!({"t": 2}))).await.status,
        404
    );
    let bad_move = app.call("PATCH", &format!("{path}/{mark}"), Some(&roles.member), Some(json!({"t": -2}))).await;
    assert_eq!(bad_move.status, 400);
    for (method, url) in [("GET", path.clone()), ("POST", path.clone()), ("DELETE", format!("{path}/{mark}"))] {
        let body = (method == "POST").then(|| json!({"t": 1}));
        assert_eq!(app.call(method, &url, None, body.clone()).await.status, 401, "{method}");
        assert_eq!(app.call(method, &url, Some(&roles.outsider), body).await.status, 404, "{method}");
    }
}

#[tokio::test]
async fn a_project_holds_at_most_500_markers_and_a_trashed_one_takes_none() {
    let Some((app, roles, id, other)) = setup("clap_markers_cap").await else { return };
    let path = format!("/api/projects/{id}/markers");
    sqlx::query("INSERT INTO markers (project_id, t) SELECT $1::uuid, n FROM generate_series(1, 500) n")
        .bind(&id)
        .execute(&app.pool)
        .await
        .unwrap();
    let full = app.call("POST", &path, Some(&roles.member), Some(json!({"t": 3}))).await;
    assert_eq!(full.status, 412, "{}", full.text());
    let count: Value = app.call("GET", &path, Some(&roles.member), None).await.json();
    assert_eq!(count["markers"].as_array().unwrap().len(), 500);
    let trash = Some(json!({"trashed": true}));
    assert_eq!(app.call("PATCH", &format!("/api/projects/{other}"), Some(&roles.owner), trash).await.status, 200);
    let trashed =
        app.call("POST", &format!("/api/projects/{other}/markers"), Some(&roles.member), Some(json!({"t": 1}))).await;
    assert_eq!(trashed.status, 412);
}
