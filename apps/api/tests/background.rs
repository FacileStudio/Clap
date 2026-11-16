//! Two API processes share one database during a deploy: one holds the background lock, the
//! other waits for it; and the boot pass leaves young uploads alone.

mod common;

use std::time::Duration;

use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn one_process_holds_the_background_lock_and_the_next_gets_it_when_it_lets_go() {
    let Some(pool) = common::pool("clap_background_lock").await else { return };
    let token = CancellationToken::new();
    let first = api::background::hold(&pool, &token).await.expect("the first process takes the lock");
    let waiting = tokio::time::timeout(Duration::from_secs(2), api::background::hold(&pool, &token)).await;
    assert!(waiting.is_err(), "a second process waits while the first holds the lock");
    drop(first);
    let next = tokio::time::timeout(Duration::from_secs(20), api::background::hold(&pool, &token)).await;
    let _second = next.expect("the lock came back").expect("held");
    token.cancel();
    let after = tokio::time::timeout(Duration::from_secs(5), api::background::hold(&pool, &token)).await;
    assert!(after.expect("a cancelled wait ends").is_none());
}

#[tokio::test]
async fn the_boot_pass_clears_only_uploads_and_staged_bundles_a_day_old() {
    let Some(pool) = common::pool("clap_background_leftovers").await else { return };
    let app = common::app(pool).await;
    let (uploads, staging) = (app.state.data_dir.join("uploads"), app.state.data_dir.join("staging"));
    for dir in [uploads.join("old"), staging.join("old/bundle")] {
        std::fs::create_dir_all(&dir).unwrap();
    }
    std::fs::write(uploads.join("young"), b"in flight").unwrap();
    let old = std::time::SystemTime::now() - Duration::from_secs(2 * 24 * 60 * 60);
    for path in [uploads.join("old"), staging.join("old")] {
        std::fs::File::open(&path).unwrap().set_modified(old).unwrap();
    }
    api::modules::assets::leftovers::clear(&app.state.data_dir).await;
    assert!(!uploads.join("old").exists() && !staging.join("old").exists());
    assert!(uploads.join("young").is_file(), "an upload another process is receiving is left alone");
}
