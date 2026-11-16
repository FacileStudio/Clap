//! `/mcp` against tronc's conformance kit, with `clap_delete_project` confirmed.

mod common;

use std::sync::Arc;

use common::fixture::SOURCE;
use common::mcp::{Served, args, served};
use serde_json::json;
use tronc::mcptest::{self, ClientOptions, ConfirmCase, Target, connect};

async fn runs(state: api::state::AppState, id: String) -> usize {
    let trashed: bool = sqlx::query_scalar("SELECT trashed_at IS NOT NULL FROM projects WHERE id = $1::uuid")
        .bind(&id)
        .fetch_one(&state.pool)
        .await
        .unwrap();
    usize::from(trashed)
}

async fn fresh(state: api::state::AppState, router: axum::Router, token: String) -> Target {
    let project =
        common::send(&router, "POST", "/api/projects", Some(&token), Some(json!({"source": SOURCE}))).await.json();
    let id = project["id"].as_str().unwrap().to_owned();
    let target = args(json!({"project_id": id}));
    let count = move || -> mcptest::Fut<usize> { Box::pin(runs(state.clone(), id.clone())) };
    Target { args: target, runs: Arc::new(count) }
}

fn delete_case(served: &Served) -> ConfirmCase {
    let (state, router, token) = (served.app.state.clone(), served.app.router.clone(), served.person.token.clone());
    let setup = move || -> mcptest::Fut<Target> { Box::pin(fresh(state.clone(), router.clone(), token.clone())) };
    ConfirmCase { tool: "clap_delete_project".into(), setup: Arc::new(setup) }
}

#[tokio::test]
async fn the_mcp_endpoint_passes_tronc_conformance_with_the_delete_confirmed() {
    let Some(served) = served("clap_mcp_kit").await else { return };
    let user = served.person.user_id;
    let token = Arc::new(move |scopes: &[&str]| {
        let scopes = if scopes.is_empty() { "clap.read,clap.write".to_owned() } else { scopes.join(",") };
        format!("{user}|{scopes}")
    });
    let confirm = vec![delete_case(&served)];
    mcptest::run(&mcptest::Config { url: served.url.clone(), token, confirm }).await;
}

#[tokio::test]
async fn the_eleven_tools_are_listed_under_their_fixed_names() {
    let Some(served) = served("clap_mcp_names").await else { return };
    let token = format!("{}|clap.read,clap.write", served.person.user_id);
    let client = connect(&served.url, &token, ClientOptions::default()).await;
    let tools = client.list_tools(None).await.unwrap().tools;
    let mut names: Vec<String> = tools.iter().map(|tool| tool.name.to_string()).collect();
    names.sort();
    let expected = [
        "clap_create_project",
        "clap_delete_project",
        "clap_edit_project",
        "clap_format_guide",
        "clap_get_project",
        "clap_list_assets",
        "clap_list_projects",
        "clap_render",
        "clap_render_status",
        "clap_timeline_edit",
        "clap_write_project",
    ];
    assert_eq!(names, expected);
    let timeline = tools.iter().find(|tool| tool.name == "clap_timeline_edit").unwrap();
    let schema = serde_json::to_value(&timeline.input_schema).unwrap();
    let mut required: Vec<&str> = schema["required"].as_array().unwrap().iter().map(|v| v.as_str().unwrap()).collect();
    required.sort_unstable();
    assert_eq!(required, ["ops", "project_id", "rev"]);
    assert_eq!(schema["properties"]["ops"]["type"], json!("array"));
    for op in ["move_clip", "trim_clip", "split_clip", "delete_clip", "add_clip", "add_scene", "reorder_scene"] {
        assert!(timeline.description.as_deref().unwrap_or_default().contains(op), "{op}");
    }
}
