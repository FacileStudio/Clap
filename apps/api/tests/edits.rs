//! Timeline edits over REST: the rev rule, the engine's refusals, and the cap on one edit.

mod common;

use common::Person;
use common::fixture;
use serde_json::{Value, json};

const CLIPS: &str = "---\nlayout: raw\nduration: 4\n---\n\n\
<h1 id=\"title\" data-start=\"0\" data-duration=\"2\" data-track-index=\"1\">Hi</h1>\n\
<p id=\"sub\" data-start=\"2\" data-duration=\"2\" data-track-index=\"1\">Bye</p>\n";

#[tokio::test]
async fn timeline_edits_patch_the_source_under_the_rev_rule() {
    let Some(pool) = common::pool("clap_projects_edits").await else { return };
    let app = common::app(pool).await;
    let roles = fixture::roles(&app).await;
    let id = fixture::id(&fixture::project(&app, &roles.member, Some(&roles.space), CLIPS).await);
    let path = format!("/api/projects/{id}/edits");
    let post = async |who: &Person, body: Value| app.call("POST", &path, Some(who), Some(body)).await;
    let op = json!({"op": "move_clip", "clip": "title", "start": 0.5});
    let stale = post(&roles.member, json!({"rev": 4, "ops": [op]})).await;
    assert_eq!((stale.status.as_u16(), stale.json()["rev"].clone()), (409, json!(1)));
    assert_eq!(post(&roles.member, json!({"rev": 1, "ops": [{"op": "explode"}]})).await.status, 400);
    assert_eq!(post(&roles.outsider, json!({"rev": 1, "ops": [op]})).await.status, 404);
    let clash = post(&roles.member, json!({"rev": 1, "ops": [{"op": "move_clip", "clip": "title", "start": 1}]})).await;
    assert_eq!((clash.status.as_u16(), clash.json()["timeline_error"].clone()), (422, json!("overlap")));
    assert_eq!(clash.json()["error"]["code"], json!("invalid_argument"));
    let ops = json!([{"op": "trim_clip", "clip": "title", "duration": 1.5}, op]);
    let edited = post(&roles.member, json!({"rev": 1, "ops": ops})).await;
    assert_eq!(edited.status, 200, "{}", edited.text());
    let body = edited.json();
    let expected = CLIPS.replace("data-start=\"0\" data-duration=\"2\"", "data-start=\"0.5\" data-duration=\"1.5\"");
    assert_eq!(
        (body["source"].clone(), body["rev"].clone(), body["clips"].clone()),
        (json!(expected), json!(2), json!(["title"]))
    );
    assert_eq!(body["timeline"]["scenes"][0]["clips"][0]["start"], json!(0.5));
    let read = app.call("GET", &format!("/api/projects/{id}/timeline"), Some(&roles.owner), None).await.json();
    assert_eq!((read["rev"].clone(), read["timeline"].clone()), (json!(2), body["timeline"].clone()));
    assert_eq!(app.call("GET", &format!("/api/projects/{id}/timeline"), Some(&roles.outsider), None).await.status, 404);
}

#[tokio::test]
async fn an_edit_of_more_than_five_hundred_ops_is_refused_before_it_runs() {
    let Some(pool) = common::pool("clap_projects_edit_cap").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, CLIPS).await);
    let ops: Vec<Value> = (0..501).map(|n| json!({"op": "move_clip", "clip": "title", "start": n % 2})).collect();
    let refused =
        app.call("POST", &format!("/api/projects/{id}/edits"), Some(&ada), Some(json!({"rev": 1, "ops": ops}))).await;
    assert_eq!(refused.status, 400, "{}", refused.text());
    assert!(refused.json()["error"]["message"].as_str().unwrap().contains("at most 500"));
}

#[tokio::test]
async fn a_copied_clip_and_scene_are_reported_by_their_new_ids() {
    let Some(pool) = common::pool("clap_projects_edit_copies").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, CLIPS).await);
    let ops = json!([{"op": "duplicate_clip", "clip": "title", "start": 2, "track": 2}, {"op": "duplicate_scene", "scene": 0}]);
    let copied =
        app.call("POST", &format!("/api/projects/{id}/edits"), Some(&ada), Some(json!({"rev": 1, "ops": ops}))).await;
    assert_eq!(copied.status, 200, "{}", copied.text());
    let body = copied.json();
    assert_eq!(body["clips"], json!(["title-copy", "title-copy-2", "sub-copy", "title-copy-copy"]));
    assert_eq!(body["timeline"]["scenes"].as_array().map(Vec::len), Some(2));
}

#[tokio::test]
async fn a_pasted_clip_must_load_files_the_project_holds_and_text_is_set_in_place() {
    let Some(pool) = common::pool("clap_projects_edit_paste").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, CLIPS).await);
    let path = format!("/api/projects/{id}/edits");
    let image = "<img src=\"assets/dot.png\" data-start=\"0\" data-duration=\"1\" data-track-index=\"2\">";
    let paste = json!([{"op": "paste_clip", "scene": 0, "start": 1, "element": image}]);
    let missing = app.call("POST", &path, Some(&ada), Some(json!({"rev": 1, "ops": paste}))).await;
    assert_eq!((missing.status.as_u16(), missing.json()["timeline_error"].clone()), (422, json!("invalid_op")));
    assert_eq!(common::upload::png(&app, &ada, &id, "assets/dot.png").await.status, 201);
    let pasted = app.call("POST", &path, Some(&ada), Some(json!({"rev": 1, "ops": paste}))).await;
    assert_eq!((pasted.status.as_u16(), pasted.json()["clips"].clone()), (200, json!(["clip-copy"])));
    let words = json!([{"op": "set_clip_text", "clip": "sub", "text": "Bye & co"}]);
    let set = app.call("POST", &path, Some(&ada), Some(json!({"rev": 2, "ops": words}))).await.json();
    assert!(set["source"].as_str().unwrap().contains(">Bye &amp; co</p>"), "{set}");
    assert_eq!(set["clips"], json!(["sub"]));
    let video = json!([{"op": "set_clip_text", "clip": "clip-copy", "text": "x"}]);
    let refused = app.call("POST", &path, Some(&ada), Some(json!({"rev": 3, "ops": video}))).await;
    assert_eq!(refused.status, 422);
}
