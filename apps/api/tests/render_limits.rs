//! The bounds on what one person and one instance hold: open renders per person, silent jobs,
//! kept renders, kept revisions, and uploads reserved before they are read.

mod common;

use common::render::{mirrored, project, queued, report};
use common::share::done;
use common::upload::{File, PNG, upload};
use serde_json::json;

fn ago(minutes: i64) -> String {
    let at = time::OffsetDateTime::now_utc() - time::Duration::minutes(minutes);
    at.format(&time::format_description::well_known::Rfc3339).unwrap()
}

#[tokio::test]
async fn a_person_has_two_renders_open_and_another_person_is_not_held_back() {
    let Some(pool) = common::pool("clap_limits_user").await else { return };
    let app = common::app_with(pool, &[("RENDER_USER_MAX", "2")]).await;
    let (ada, bob) = (app.person("ada").await, app.person("bob").await);
    let (mine, theirs) = (project(&app, &ada, None).await, project(&app, &bob, None).await);
    queued(&app, &ada, &mine, json!({})).await;
    queued(&app, &ada, &mine, json!({})).await;
    let third = app.call("POST", &format!("/api/projects/{mine}/renders"), Some(&ada), Some(json!({}))).await;
    assert_eq!(third.status, 429, "{}", third.text());
    assert!(third.json()["error"]["message"].as_str().unwrap().contains("you already have 2"));
    queued(&app, &bob, &theirs, json!({})).await;
}

#[tokio::test]
async fn a_running_job_the_worker_stopped_reporting_on_is_given_up() {
    let Some(pool) = common::pool("clap_limits_silent").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let (silent, busy) = (queued(&app, &ada, &id, json!({})).await, queued(&app, &ada, &id, json!({})).await);
    report(&app, &silent, json!({"state": "running", "frame": 3, "frames": 60, "updated_at": ago(20)}));
    report(&app, &busy, json!({"state": "running", "frame": 3, "frames": 60, "updated_at": ago(0)}));
    for render in [&silent, &busy] {
        assert_eq!(mirrored(&app, &ada, render).await["state"], json!("running"));
    }
    api::modules::renders::sweep::swept(&app.state).await.unwrap();
    let given_up = mirrored(&app, &ada, &silent).await;
    assert!(given_up["error"].as_str().unwrap().contains("stopped reporting"), "{given_up}");
    assert_eq!(mirrored(&app, &ada, &busy).await["state"], json!("running"));
}

#[tokio::test]
async fn a_project_keeps_its_three_newest_renders_and_a_posted_one_a_week() {
    let Some(pool) = common::pool("clap_limits_retention").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let mut ids = Vec::new();
    for n in 0..5 {
        ids.push(done(&app, &ada, &id, format!("video-{n}").as_bytes()).await["id"].as_str().unwrap().to_owned());
    }
    let posted: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO renders (requested_by, rev, width, height, fps, state, finished_at) \
         VALUES ($1, 0, 320, 240, 30, 'done', now() - interval '8 days') RETURNING id",
    )
    .bind(ada.user_id)
    .fetch_one(&app.pool)
    .await
    .unwrap();
    api::modules::renders::sweep::swept(&app.state).await.unwrap();
    let list = app.call("GET", &format!("/api/projects/{id}/renders"), Some(&ada), None).await.json();
    let kept: Vec<&str> = list["renders"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert_eq!(kept, [ids[4].as_str(), ids[3].as_str(), ids[2].as_str()]);
    let output = |render: &str| api::modules::renders::status::output(&app.state.data_dir, render.parse().unwrap());
    assert!(!output(&ids[0]).exists() && output(&ids[4]).exists());
    let left: i64 = sqlx::query_scalar("SELECT count(*) FROM renders WHERE id = $1")
        .bind(posted)
        .fetch_one(&app.pool)
        .await
        .unwrap();
    assert_eq!(left, 0);
}

#[tokio::test]
async fn a_project_keeps_its_fifty_newest_revisions() {
    let Some(pool) = common::pool("clap_limits_revisions").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    sqlx::query(
        "INSERT INTO project_revisions (project_id, rev, source, origin) \
         SELECT $1::uuid, n, 'x', 'cli' FROM generate_series(2, 61) AS n",
    )
    .bind(&id)
    .execute(&app.pool)
    .await
    .unwrap();
    assert_eq!(api::modules::projects::purge::revisions(&app.state).await.unwrap(), 11);
    let (count, oldest, newest): (i64, i64, i64) =
        sqlx::query_as("SELECT count(*), min(rev), max(rev) FROM project_revisions WHERE project_id = $1::uuid")
            .bind(&id)
            .fetch_one(&app.pool)
            .await
            .unwrap();
    assert_eq!((count, oldest, newest), (50, 12, 61));
}

#[tokio::test]
async fn an_upload_longer_than_the_room_left_is_refused_before_it_is_read() {
    let Some(pool) = common::pool("clap_limits_reserve").await else { return };
    let app = common::app_with(pool, &[("SPACE_ASSETS_MAX_BYTES", "1000")]).await;
    let ada = app.person("ada").await;
    let id = common::fixture::id(&common::fixture::project(&app, &ada, None, common::render::SOURCE).await);
    let big = [PNG, &[0u8; 80 << 10]].concat();
    let refused = upload(&app, &ada, &id, File { name: Some("big.png"), file_name: "big.png", bytes: &big }).await;
    assert_eq!(refused.status, 413, "{}", refused.text());
    assert!(refused.json()["error"]["message"].as_str().unwrap().contains("left for uploads"));
    let incoming = app.state.data_dir.join("uploads");
    assert_eq!(std::fs::read_dir(&incoming).map_or(0, |entries| entries.count()), 0, "nothing was written");
    let small = File { name: Some("dot.png"), file_name: "dot.png", bytes: PNG };
    assert_eq!(upload(&app, &ada, &id, small).await.status, 201);
}
