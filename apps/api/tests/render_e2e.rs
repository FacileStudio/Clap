//! One real render, end to end: the API queues a job, the worker binary renders it with Chrome
//! and ffmpeg over the same renders directory, and the API serves the MP4. Chrome's sandbox is
//! off in this test's worker only, because CI runners refuse it user namespaces; the compose
//! file never turns it off. Skips with a banner without the worker binary, Chrome or ffmpeg.

mod common;

use std::time::{Duration, Instant};

use common::render::{mirrored, queued};
use common::{App, Person};
use serde_json::{Value, json};

const SOURCE: &str = "---\ntitle: One second\nsize: 320x180\nlayout: raw\nduration: 1\n---\n\n\
<img id=\"dot\" src=\"assets/dot.png\" data-start=\"0\" data-duration=\"1\" data-track-index=\"0\">\n\
<h1 id=\"title\" data-start=\"0\" data-duration=\"1\" data-track-index=\"1\">Hello</h1>\n\
<script>\n  scene.timeline((tl) => tl.from(\"#title\", { opacity: 0, duration: 0.5 }));\n</script>\n";

async fn alive(app: &App) {
    let heartbeat = app.state.data_dir.join("renders/.worker");
    std::fs::remove_file(&heartbeat).ok();
    let started = Instant::now();
    while !heartbeat.is_file() {
        assert!(started.elapsed() < Duration::from_secs(30), "the worker wrote no heartbeat in 30 s");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

async fn settled(app: &App, who: &Person, render: &Value) -> Value {
    let started = Instant::now();
    loop {
        let seen = mirrored(app, who, render).await;
        if ["done", "failed", "cancelled"].contains(&seen["state"].as_str().unwrap_or_default()) {
            return seen;
        }
        assert!(started.elapsed() < Duration::from_secs(300), "the render did not finish in 5 minutes: {seen}");
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[tokio::test]
async fn the_api_and_the_worker_render_one_second_to_an_mp4_the_api_serves() {
    let Some(pool) = common::pool("clap_render_e2e").await else { return };
    let Some((binary, chrome)) = common::worker::ready() else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = common::fixture::id(&common::fixture::project(&app, &ada, None, SOURCE).await);
    assert_eq!(common::upload::png(&app, &ada, &id, "assets/dot.png").await.status, 201);
    let _worker = common::worker::start(&binary, &chrome, &app.state.data_dir.join("renders"));
    alive(&app).await;
    let render = queued(&app, &ada, &id, json!({"fps": 10})).await;
    let done = settled(&app, &ada, &render).await;
    assert_eq!((done["state"].clone(), done["frames"].clone()), (json!("done"), json!(10)), "{done}");
    let file = app.call("GET", done["download_url"].as_str().unwrap(), Some(&ada), None).await;
    assert_eq!((file.status.as_u16(), &file.bytes[4..8]), (200, &b"ftyp"[..]));
    assert_eq!(file.headers["content-type"], "video/mp4");
    let saved = app.state.data_dir.join("served.mp4");
    std::fs::write(&saved, &file.bytes).unwrap();
    let tools = api::modules::assets::tools::Tools::from_vars(&tronc::env::Vars::process());
    let probe = api::modules::assets::probe::probe(&tools, &saved).await.unwrap();
    assert_eq!((probe.width, probe.height), (Some(320), Some(180)), "{probe:?}");
    assert!(probe.duration_ms.is_some_and(|ms| (900..=1100).contains(&ms)), "{probe:?}");
    common::media::poster(&app, done["id"].as_str().unwrap()).await;
}
