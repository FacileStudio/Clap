//! Renaming an asset: the row and every reference in the source move together, as one revision.

mod common;

use common::render::{SOURCE, project};
use common::upload::{PNG, png};
use serde_json::{Value, json};

async fn rev(app: &common::App, who: &common::Person, id: &str) -> i64 {
    app.call("GET", &format!("/api/projects/{id}"), Some(who), None).await.json()["rev"].as_i64().unwrap()
}

#[tokio::test]
async fn a_rename_moves_the_asset_and_its_references_in_one_revision() {
    let Some(pool) = common::pool("clap_rename").await else { return };
    let app = common::app(pool).await;
    let roles = common::fixture::roles(&app).await;
    let id = project(&app, &roles.member, Some(&roles.space)).await;
    let at = rev(&app, &roles.member, &id).await;
    let path = format!("/api/projects/{id}/assets/assets/dot.png");
    let renamed =
        app.call("PATCH", &path, Some(&roles.member), Some(json!({"name": "media/spot.png", "rev": at}))).await;
    assert_eq!(renamed.status, 200, "{}", renamed.text());
    let body = renamed.json();
    assert_eq!((body["rev"].clone(), body["asset"]["name"].clone()), (json!(at + 1), json!("media/spot.png")));
    assert_eq!(body["asset"]["url"], json!(format!("/api/projects/{id}/assets/media/spot.png")));
    let stored = app.call("GET", &format!("/api/projects/{id}"), Some(&roles.owner), None).await.json();
    let expected =
        SOURCE.replace("\"assets/dot.png\"", "\"media/spot.png\"").replace("./assets/dot.png#", "./media/spot.png#");
    assert_eq!(stored["source"], json!(expected));
    let revisions: Vec<(i64, String)> =
        sqlx::query_as("SELECT rev, origin FROM project_revisions WHERE project_id = $1::uuid ORDER BY rev")
            .bind(&id)
            .fetch_all(&app.pool)
            .await
            .unwrap();
    assert_eq!(revisions.last().unwrap(), &(at + 1, "cli".to_owned()));
    assert_eq!(app.call("GET", &path, Some(&roles.member), None).await.status, 404);
    let served = app.call("GET", &format!("/api/projects/{id}/assets/media/spot.png"), Some(&roles.admin), None).await;
    assert_eq!((served.status.as_u16(), served.bytes.as_slice()), (200, PNG));
    let list = app.call("GET", &format!("/api/projects/{id}/assets"), Some(&roles.owner), None).await.json();
    let names: Vec<&str> = list["assets"].as_array().unwrap().iter().map(|a| a["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["assets/back.png", "media/spot.png"]);
}

fn ask(name: &str, rev: i64) -> Option<Value> {
    Some(json!({"name": name, "rev": rev}))
}

async fn setup(prefix: &str) -> Option<(common::App, common::fixture::Roles, String, i64)> {
    let app = common::app(common::pool(prefix).await?).await;
    let roles = common::fixture::roles(&app).await;
    let id = project(&app, &roles.member, Some(&roles.space)).await;
    let at = rev(&app, &roles.member, &id).await;
    Some((app, roles, id, at))
}

#[tokio::test]
async fn a_stale_unknown_bad_or_unauthorized_rename_changes_nothing() {
    let Some((app, roles, id, at)) = setup("clap_rename_refused").await else { return };
    let path = format!("/api/projects/{id}/assets/assets/dot.png");
    let stale = app.call("PATCH", &path, Some(&roles.member), ask("assets/new.png", at - 1)).await;
    assert_eq!((stale.status.as_u16(), stale.json()["rev"].clone()), (409, json!(at)));
    let unknown = format!("/api/projects/{id}/assets/assets/none.png");
    assert_eq!(app.call("PATCH", &unknown, Some(&roles.member), ask("assets/new.png", at)).await.status, 404);
    for bad in ["../x.png", "assets/a b.png", "https://x.test/a.png"] {
        let refused = app.call("PATCH", &path, Some(&roles.member), ask(bad, at)).await;
        let code = refused.json()["timeline_error"].clone();
        assert_eq!((refused.status.as_u16(), code), (422, json!("invalid_op")), "{bad}");
    }
    assert_eq!(app.call("PATCH", &path, None, ask("assets/new.png", at)).await.status, 401);
    assert_eq!(app.call("PATCH", &path, Some(&roles.outsider), ask("assets/new.png", at)).await.status, 404);
    assert_eq!(rev(&app, &roles.member, &id).await, at);
    let stored = app.call("GET", &format!("/api/projects/{id}"), Some(&roles.owner), None).await.json();
    assert_eq!(stored["source"], json!(SOURCE));
}

#[tokio::test]
async fn a_taken_name_is_a_conflict_and_an_unreferenced_file_moves_alone() {
    let Some((app, roles, id, at)) = setup("clap_rename_taken").await else { return };
    let path = format!("/api/projects/{id}/assets/assets/dot.png");
    assert_eq!(png(&app, &roles.member, &id, "assets/loose.png").await.status, 201);
    let held = app.call("PATCH", &path, Some(&roles.member), ask("assets/loose.png", at)).await;
    assert_eq!((held.status.as_u16(), held.json()["error"]["code"].clone()), (409, json!("already_exists")));
    let back = format!("/api/projects/{id}/assets/assets/back.png");
    assert_eq!(app.call("DELETE", &back, Some(&roles.member), None).await.status, 204);
    let loose = format!("/api/projects/{id}/assets/assets/loose.png");
    let named = app.call("PATCH", &loose, Some(&roles.member), ask("assets/back.png", at)).await;
    let message = named.json()["error"]["message"].as_str().unwrap_or_default().to_owned();
    assert!(named.status == 409 && message.contains("already references"), "{}", named.text());
    let moved = app.call("PATCH", &loose, Some(&roles.member), ask("assets/kept.png", at)).await;
    assert_eq!((moved.status.as_u16(), moved.json()["rev"].clone()), (200, json!(at + 1)));
    let same = app.call("GET", &format!("/api/projects/{id}"), Some(&roles.owner), None).await.json();
    assert_eq!(same["source"], json!(SOURCE));
}
