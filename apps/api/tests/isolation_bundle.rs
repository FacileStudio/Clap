//! A bundle never shares an inode with the blobs every project reads, blobs are read-only, and a
//! job directory keeps the worker from removing what the API wrote.

mod common;

use std::os::unix::fs::{MetadataExt, PermissionsExt};

use common::render::{dir, project, queued};
use common::upload::{PNG, png};
use serde_json::json;

fn mode(path: &std::path::Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o7777
}

#[tokio::test]
async fn a_bundle_holds_copies_built_out_of_sight_and_blobs_are_read_only() {
    let Some(pool) = common::pool("clap_isolation_bundle").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = project(&app, &ada, None).await;
    let render = queued(&app, &ada, &id, json!({})).await;
    let sha = api::modules::assets::files::hex(&<sha2::Sha256 as sha2::Digest>::digest(PNG));
    let blob = api::modules::assets::files::path(&app.state.data_dir, &sha);
    let copied = dir(&app, &render).join("bundle/media/assets/dot.png");
    assert_eq!(std::fs::read(&copied).unwrap(), PNG);
    assert_ne!(std::fs::metadata(&copied).unwrap().ino(), std::fs::metadata(&blob).unwrap().ino());
    assert_eq!((std::fs::metadata(&blob).unwrap().nlink(), mode(&blob)), (1, 0o444));
    assert_eq!(mode(&dir(&app, &render)), api::modules::renders::jobfs::JOB_MODE);
    assert_eq!(mode(&dir(&app, &render).join("bundle")), api::modules::renders::jobfs::BUNDLE_MODE);
    let staging = api::modules::renders::stage::staging(&app.state.data_dir);
    assert_eq!(std::fs::read_dir(&staging).map_or(0, |entries| entries.count()), 0, "staging is emptied");
    assert_eq!(png(&app, &ada, &id, "assets/again.png").await.status, 201, "a read-only blob is stored again");
}

#[tokio::test]
async fn the_boot_pass_makes_blobs_stored_before_it_read_only() {
    let Some(pool) = common::pool("clap_isolation_seal").await else { return };
    let app = common::app(pool).await;
    let old = api::modules::assets::files::path(&app.state.data_dir, &"cd".repeat(32));
    std::fs::create_dir_all(old.parent().unwrap()).unwrap();
    std::fs::write(&old, b"GIF89a").unwrap();
    std::fs::set_permissions(&old, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(api::modules::assets::seal::seal(&app.state.data_dir).unwrap(), 1);
    assert_eq!(mode(&old), 0o444);
    assert_eq!(api::modules::assets::seal::seal(&app.state.data_dir).unwrap(), 0);
}
