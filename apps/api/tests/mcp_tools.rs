//! The tools themselves: the same service functions as REST, the same `rev` rule, the same event.

mod common;

use common::fixture::SOURCE;
use common::mcp::{Served, args, refused, served};
use serde_json::{Value, json};
use tronc::mcptest::{self, Client, ClientOptions, call, connect, decode};

async fn client(served: &Served, scopes: &str) -> Client {
    connect(&served.url, &format!("{}|{scopes}", served.person.user_id), ClientOptions::default()).await
}

async fn created(served: &Served, client: &Client) -> String {
    let input = json!({"title": "Plan", "source": SOURCE});
    let project: Value = decode(&call(client, "clap_create_project", args(input)).await);
    let id = project["id"].as_str().unwrap().to_owned();
    assert_eq!(
        (project["rev"].clone(), project["scenes"].clone(), project["title"].clone()),
        (json!(1), json!(2), json!("Plan"))
    );
    assert_eq!(project["url"], json!(format!("{}/projects/{id}", served.url.trim_end_matches("/mcp"))));
    id
}

#[tokio::test]
async fn an_agent_creates_reads_edits_and_rewrites_a_project_under_the_rev_rule() {
    let Some(served) = served("clap_mcp_tools").await else { return };
    let client = client(&served, "clap.read,clap.write").await;
    let id = created(&served, &client).await;
    let read: Value = decode(&call(&client, "clap_get_project", args(json!({"project_id": id}))).await);
    assert_eq!(
        (read["source"].clone(), read["rev"].clone(), read["truncated"].clone()),
        (json!(SOURCE), json!(1), json!(false))
    );
    let edit = |old: &str, rev: i64| json!({"project_id": id, "old_string": old, "new_string": "Up 15%", "rev": rev});
    assert!(refused(&client, "clap_edit_project", edit("Up 99%", 1)).await.contains("not in the source"));
    assert!(refused(&client, "clap_edit_project", edit("Q3 review", 1)).await.contains("2 times"));
    assert!(refused(&client, "clap_edit_project", edit("Up 12%", 7)).await.contains("is at rev 1, not 7"));
    let edited: Value = decode(&call(&client, "clap_edit_project", args(edit("Up 12%", 1))).await);
    assert_eq!((edited["rev"].clone(), edited["url"].clone()), (json!(2), read["url"].clone()));
    let write = |source: &str, rev: i64| json!({"project_id": id, "source": source, "rev": rev});
    assert!(refused(&client, "clap_write_project", write("# Lost\n", 1)).await.contains("is at rev 2, not 1"));
    let invalid = refused(&client, "clap_write_project", write("---\ncolour: red\n---\n\n# A\n", 2)).await;
    assert!(invalid.contains("line 2: unknown key `colour`") && invalid.contains("clap_format_guide"), "{invalid}");
    let written: Value = decode(&call(&client, "clap_write_project", args(write("# Whole\n", 2))).await);
    assert_eq!((written["rev"].clone(), written["scenes"].clone()), (json!(3), json!(1)));
    let origins: Vec<String> =
        sqlx::query_scalar("SELECT origin FROM project_revisions WHERE project_id = $1::uuid ORDER BY rev")
            .bind(&id)
            .fetch_all(&served.app.pool)
            .await
            .unwrap();
    assert_eq!(origins, ["mcp", "mcp", "mcp"]);
    let rest = served.app.call("GET", &format!("/api/projects/{id}"), Some(&served.person), None).await.json();
    assert_eq!((rest["source"].clone(), rest["rev"].clone()), (json!("# Whole\n"), json!(3)));
}

#[tokio::test]
async fn an_agent_lists_projects_and_assets_and_reads_the_guide() {
    let Some(served) = served("clap_mcp_reads").await else { return };
    let client = client(&served, "clap.read,clap.write").await;
    let id = created(&served, &client).await;
    let stranger = served.app.person("stranger").await;
    let theirs = common::fixture::id(&common::fixture::project(&served.app, &stranger, None, SOURCE).await);
    let list: Value = decode(&call(&client, "clap_list_projects", args(json!({}))).await);
    assert_eq!((list["total"].clone(), list["projects"][0]["id"].clone()), (json!(1), json!(id)));
    assert!(list["projects"][0]["url"].as_str().unwrap().ends_with(&format!("/projects/{id}")));
    assert!(refused(&client, "clap_get_project", json!({"project_id": theirs})).await.contains("Project not found"));
    assert!(refused(&client, "clap_list_assets", json!({"project_id": theirs})).await.contains("Project not found"));
    assert_eq!(common::upload::png(&served.app, &served.person, &id, "assets/dot.png").await.status, 201);
    let assets: Value = decode(&call(&client, "clap_list_assets", args(json!({"project_id": id}))).await);
    assert_eq!(
        (assets["assets"][0]["name"].clone(), assets["assets"][0]["kind"].clone()),
        (json!("assets/dot.png"), json!("image"))
    );
    let guide: Value = decode(&call(&client, "clap_format_guide", args(json!({}))).await);
    let text = guide["guide"].as_str().unwrap();
    assert!(
        text.starts_with(scene::FORMAT_GUIDE) && text.contains("data-track-index") && text.contains("## Determinism")
    );
}

#[tokio::test]
async fn an_agent_edits_the_timeline_under_the_rev_rule_and_hears_which_clips_changed() {
    let Some(served) = served("clap_mcp_timeline").await else { return };
    let client = client(&served, "clap.read,clap.write").await;
    let id = created(&served, &client).await;
    let stale = json!({"project_id": id, "rev": 9, "ops": [{"op": "delete_scene", "scene": 1}]});
    assert!(refused(&client, "clap_timeline_edit", stale).await.contains("is at rev 1, not 9"));
    let unknown = json!({"project_id": id, "rev": 1, "ops": [{"op": "delete_clip", "clip": "nope"}]});
    assert!(refused(&client, "clap_timeline_edit", unknown).await.contains("unknown_clip"));
    let body = "<h1 id=\"end\" data-start=\"0\" data-duration=\"2\">End</h1>";
    let ops = json!([{"op": "add_scene", "at": 2, "duration": 4, "body": body}, {"op": "move_clip", "clip": "end", "start": 1}]);
    let edited: Value =
        decode(&call(&client, "clap_timeline_edit", args(json!({"project_id": id, "rev": 1, "ops": ops}))).await);
    assert_eq!(
        (edited["rev"].clone(), edited["scenes"].clone(), edited["clips"].clone()),
        (json!(2), json!(3), json!(["end"]))
    );
    let origin: String =
        sqlx::query_scalar("SELECT origin FROM project_revisions WHERE project_id = $1::uuid AND rev = 2")
            .bind(&id)
            .fetch_one(&served.app.pool)
            .await
            .unwrap();
    assert_eq!(origin, "mcp");
}

#[tokio::test]
async fn an_agent_queues_a_render_and_follows_it_to_the_download_link() {
    let Some(served) = served("clap_mcp_render").await else { return };
    let client = client(&served, "clap.read,clap.write").await;
    let (app, who) = (&served.app, &served.person);
    let id = common::render::project(app, who, None).await;
    let big = refused(&client, "clap_render", json!({"project_id": id, "width": 4000, "height": 2000})).await;
    assert!(big.contains("at most 1920x1080"), "{big}");
    let queued: Value = decode(&call(&client, "clap_render", args(json!({"project_id": id, "fps": 24}))).await);
    assert_eq!(
        (queued["state"].clone(), queued["frames"].clone(), queued["rev"].clone()),
        (json!("queued"), json!(48), json!(1))
    );
    assert!(queued["url"].as_str().unwrap().ends_with(&format!("/projects/{id}")));
    let job = common::render::dir(app, &queued).join("job.json");
    assert!(job.is_file());
    common::render::finished(app, &queued, b"mp4");
    api::modules::renders::mirror::tick(&app.state).await.unwrap();
    let status = json!({"render_id": queued["id"]});
    let done: Value = decode(&call(&client, "clap_render_status", args(status)).await);
    let origin = served.url.trim_end_matches("/mcp");
    assert_eq!(done["state"], json!("done"));
    assert_eq!(done["download_url"], json!(format!("{origin}/api/renders/{}/file", queued["id"].as_str().unwrap())));
    let unknown = json!({"render_id": uuid::Uuid::new_v4().to_string()});
    assert!(refused(&client, "clap_render_status", unknown).await.contains("Render not found"));
}

#[tokio::test]
async fn a_read_token_reads_and_is_refused_every_write_tool() {
    let Some(served) = served("clap_mcp_scopes").await else { return };
    let reader = client(&served, "clap.read").await;
    let list: Value = decode(&call(&reader, "clap_list_projects", args(json!({}))).await);
    assert_eq!(list["total"], json!(0));
    let token = format!("{}|clap.read", served.person.user_id);
    let writes =
        ["clap_create_project", "clap_write_project", "clap_edit_project", "clap_timeline_edit", "clap_render"];
    for tool in writes.into_iter().chain(["clap_delete_project"]) {
        let body =
            json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": tool, "arguments": {}}});
        let (status, headers) = mcptest::post(&served.url, &token, &body.to_string(), &[]).await;
        let challenge = headers.get("www-authenticate").and_then(|value| value.to_str().ok()).unwrap_or_default();
        assert_eq!(status, 403, "{tool}");
        assert!(challenge.contains("insufficient_scope") && challenge.contains("clap.write"), "{tool}: {challenge}");
    }
    let projects: i64 = sqlx::query_scalar("SELECT count(*) FROM projects").fetch_one(&served.app.pool).await.unwrap();
    assert_eq!(projects, 0);
}

#[tokio::test]
async fn a_delete_waits_for_the_person_and_only_their_browser_session_confirms() {
    let Some(served) = served("clap_mcp_delete").await else { return };
    let client = client(&served, "clap.read,clap.write").await;
    let id = created(&served, &client).await;
    let pending: Value = decode(&call(&client, "clap_delete_project", args(json!({"project_id": id}))).await);
    assert_eq!(pending["status"], json!("pending_confirmation"));
    let action = pending["confirm_url"].as_str().unwrap().rsplit('=').next().unwrap().to_owned();
    let path = format!("/api/mcp/pending/{action}/confirm");
    let project = format!("/api/projects/{id}");
    let live = || served.app.call("GET", &project, Some(&served.person), None);
    assert_eq!(live().await.json()["trashed_at"], Value::Null);
    let bearer = served.app.call("POST", &path, Some(&served.person), None).await;
    assert_eq!(bearer.status, 403, "{}", bearer.text());
    let login = common::http::browser_login(&served.app.kits.sessions, served.person.user_id).await;
    let router = &served.app.router;
    assert_eq!(common::http::without_csrf(router, "POST", &path, &login).await.status, 403);
    let list = common::http::with_cookie(router, "GET", "/api/mcp/pending", &login).await.json();
    assert_eq!((list["actions"][0]["id"].clone(), list["actions"][0]["target"].clone()), (json!(action), json!(id)));
    let done = common::http::with_cookie(router, "POST", &path, &login).await;
    assert_eq!(done.json(), json!({"status": "done"}), "{}", done.text());
    assert!(live().await.json()["trashed_at"].is_string());
    assert!(refused(&client, "clap_delete_project", json!({"project_id": id})).await.contains("already in the trash"));
}
