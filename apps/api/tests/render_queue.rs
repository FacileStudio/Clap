//! The queue cap counts every waiting render of the instance, so this test has a schema of its own, emptied first:
//! the schema outlives a run.

mod common;

use common::render::{project, queued};
use serde_json::json;

#[tokio::test]
async fn a_full_queue_refuses_the_next_render() {
    let Some(pool) = common::pool("clap_render_queue").await else { return };
    sqlx::query("DELETE FROM renders").execute(&pool).await.unwrap();
    let app = common::app_with(pool, &[("RENDER_QUEUE_MAX", "1")]).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    queued(&app, &ada, &id, json!({})).await;
    let full = app.call("POST", &format!("/api/projects/{id}/renders"), Some(&ada), Some(json!({}))).await;
    assert_eq!((full.status.as_u16(), full.json()["error"]["code"].clone()), (429, json!("rate_limited")));
}
