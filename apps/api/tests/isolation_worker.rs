//! A compromised worker controls every path under `DATA_DIR/renders`. These plant the links,
//! hard links and FIFOs it could, and check the API neither writes nor reads through them.

mod common;

use std::os::unix::fs::{MetadataExt, symlink};
use std::time::Duration;

use common::render::{dir, mirrored, project, queued, report};
use common::upload::PNG;
use serde_json::{Value, json};

fn blob(app: &common::App) -> std::path::PathBuf {
    let sha = api::modules::assets::files::hex(&<sha2::Sha256 as sha2::Digest>::digest(PNG));
    api::modules::assets::files::path(&app.state.data_dir, &sha)
}

#[tokio::test]
async fn a_cancel_through_a_planted_link_or_a_swapped_job_directory_writes_nothing() {
    let Some(pool) = common::pool("clap_isolation_cancel").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let (linked, swapped) = (queued(&app, &ada, &id, json!({})).await, queued(&app, &ada, &id, json!({})).await);
    symlink(blob(&app), dir(&app, &linked).join("cancel")).unwrap();
    let one = |render: &Value| format!("/api/renders/{}", render["id"].as_str().unwrap());
    assert_eq!(app.call("DELETE", &one(&linked), Some(&ada), None).await.status, 500);
    assert_eq!(std::fs::read(blob(&app)).unwrap(), PNG, "the blob behind the link is untouched");
    let assets = app.state.data_dir.join("assets");
    std::fs::rename(dir(&app, &swapped), app.state.data_dir.join("aside")).unwrap();
    symlink(&assets, dir(&app, &swapped)).unwrap();
    assert_eq!(app.call("DELETE", &one(&swapped), Some(&ada), None).await.status, 500);
    assert!(!assets.join("cancel").exists(), "nothing was created through the swapped directory");
}

#[tokio::test]
async fn a_status_behind_a_link_or_a_fifo_is_ignored_without_hanging() {
    let Some(pool) = common::pool("clap_isolation_status").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let (linked, piped) = (queued(&app, &ada, &id, json!({})).await, queued(&app, &ada, &id, json!({})).await);
    let fake = app.state.data_dir.join("fake.json");
    std::fs::write(&fake, json!({"state": "failed", "error": "SECRET"}).to_string()).unwrap();
    symlink(&fake, dir(&app, &linked).join("status.json")).unwrap();
    let made = std::process::Command::new("mkfifo").arg(dir(&app, &piped).join("status.json")).status().unwrap();
    assert!(made.success());
    let tick = tokio::time::timeout(Duration::from_secs(10), api::modules::renders::mirror::tick(&app.state)).await;
    assert_eq!(tick.expect("a FIFO hung the mirror").unwrap(), 0);
    for render in [&linked, &piped] {
        let seen = mirrored(&app, &ada, render).await;
        assert_eq!(seen["state"], json!("queued"), "{seen}");
        assert!(!seen.to_string().contains("SECRET"));
    }
}

#[tokio::test]
async fn the_finished_file_is_copied_once_and_a_hard_linked_one_is_refused() {
    let Some(pool) = common::pool("clap_isolation_copy").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let (good, linked) = (queued(&app, &ada, &id, json!({})).await, queued(&app, &ada, &id, json!({})).await);
    std::fs::write(dir(&app, &good).join("out.mp4"), b"good-video").unwrap();
    let written = std::fs::metadata(dir(&app, &good).join("out.mp4")).unwrap().ino();
    std::fs::write(dir(&app, &linked).join("out.mp4"), b"linked").unwrap();
    std::fs::hard_link(dir(&app, &linked).join("out.mp4"), dir(&app, &linked).join("other")).unwrap();
    for render in [&good, &linked] {
        report(&app, render, json!({"state": "done", "frame": 60, "frames": 60}));
    }
    assert_eq!(mirrored(&app, &ada, &good).await["state"], json!("done"));
    let kept =
        api::modules::renders::status::output(&app.state.data_dir, good["id"].as_str().unwrap().parse().unwrap());
    let metadata = std::fs::metadata(&kept).unwrap();
    assert_eq!((std::fs::read(&kept).unwrap(), metadata.nlink()), (b"good-video".to_vec(), 1));
    assert_ne!(metadata.ino(), written, "the API keeps a copy, never the worker's inode");
    let refused = mirrored(&app, &ada, &linked).await;
    assert_eq!((refused["state"].clone(), refused["download_url"].clone()), (json!("failed"), Value::Null));
}

#[tokio::test]
async fn a_kept_render_swapped_for_a_link_is_never_served() {
    let Some(pool) = common::pool("clap_isolation_serve").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let render = common::share::done(&app, &ada, &id, b"video").await;
    let token = common::share::link(&app, &ada, &id).await;
    let kept =
        api::modules::renders::status::output(&app.state.data_dir, render["id"].as_str().unwrap().parse().unwrap());
    let secret = app.state.data_dir.join("secret");
    std::fs::write(&secret, b"SECRET").unwrap();
    std::fs::remove_file(&kept).unwrap();
    symlink(&secret, &kept).unwrap();
    for path in [render["download_url"].as_str().unwrap().to_owned(), format!("/api/public/renders/{token}/video.mp4")]
    {
        let answer = app.call("GET", &path, Some(&ada), None).await;
        assert_eq!(answer.status, 500, "{path}");
        assert!(!answer.text().contains("SECRET"), "{path}");
    }
}
