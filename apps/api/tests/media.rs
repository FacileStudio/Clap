//! Real media: a clip made with ffmpeg is uploaded, probed, given a poster and a proxy, and
//! served by range. Needs `ffmpeg` and `ffprobe`; prints a banner and passes without them.

mod common;

use axum::body::Body;
use common::fixture;
use common::media::{clip, installed, settled};
use common::upload::{File, upload};
use http::{Request, header};
use serde_json::json;

const SOURCE: &str = "---\nsize: 320x240\nlayout: raw\nduration: 2\n---\n\n\
<video id=\"shot\" src=\"assets/clip.mp4\" data-start=\"0\" data-duration=\"2\" data-track-index=\"0\"></video>\n";

async fn ranged(app: &common::App, who: &common::Person, path: &str, range: &str) -> common::Answer {
    let request = Request::get(path)
        .header(header::AUTHORIZATION, format!("Bearer {}", who.token))
        .header(header::RANGE, range)
        .body(Body::empty())
        .unwrap();
    common::http::raw(&app.router, request).await
}

fn described(asset: &serde_json::Value) {
    assert_eq!(asset["state"], json!("ready"), "{asset}");
    assert_eq!(
        (asset["width"].clone(), asset["height"].clone(), asset["fps"].clone()),
        (json!(320), json!(240), json!(25.0))
    );
    assert_eq!(asset["has_audio"], json!(true));
    assert!((1900..=2200).contains(&asset["duration_ms"].as_i64().unwrap()), "{asset}");
}

#[tokio::test]
async fn an_uploaded_clip_is_probed_given_a_poster_and_a_proxy_and_served_by_range() {
    let Some(pool) = common::pool("clap_media").await else { return };
    if !installed() {
        return;
    }
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, SOURCE).await);
    let bytes = clip(&app.state.data_dir);
    let file = File { name: Some("assets/clip.mp4"), file_name: "clip.mp4", bytes: &bytes };
    let stored = upload(&app, &ada, &id, file).await;
    assert_eq!(stored.status, 201, "{}", stored.text());
    assert_eq!((stored.json()["kind"].clone(), stored.json()["state"].clone()), (json!("video"), json!("probing")));
    let asset = settled(&app, &ada, &id, "assets/clip.mp4", true).await;
    described(&asset);
    let poster = app.call("GET", asset["poster_url"].as_str().unwrap(), Some(&ada), None).await;
    assert_eq!((poster.status.as_u16(), &poster.bytes[..3]), (200, &[0xff, 0xd8, 0xff][..]));
    assert_eq!(poster.headers["content-type"], "image/jpeg");
    let part = ranged(&app, &ada, asset["url"].as_str().unwrap(), "bytes=4-7").await;
    assert_eq!((part.status.as_u16(), part.bytes.as_slice()), (206, &b"ftyp"[..]));
    assert_eq!(part.headers["content-range"], format!("bytes 4-7/{}", bytes.len()));
    assert_eq!(part.headers["content-type"], "video/mp4");
    let proxy = ranged(&app, &ada, asset["proxy_url"].as_str().unwrap(), "bytes=4-7").await;
    assert_eq!((proxy.status.as_u16(), proxy.bytes.as_slice()), (206, &b"ftyp"[..]));
    let frame = app.call("GET", &format!("/api/projects/{id}/frame"), Some(&ada), None).await.text();
    assert!(frame.contains("&amp;variant=proxy\""), "the frame plays the proxy: {frame}");
}

#[tokio::test]
async fn a_file_that_only_looks_like_a_video_fails_with_a_reason() {
    let Some(pool) = common::pool("clap_media_failed").await else { return };
    if !installed() {
        return;
    }
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, SOURCE).await);
    let file =
        File { name: Some("assets/clip.mp4"), file_name: "clip.mp4", bytes: b"\0\0\0\x18ftypisom-and-nothing-else" };
    assert_eq!(upload(&app, &ada, &id, file).await.status, 201);
    let asset = settled(&app, &ada, &id, "assets/clip.mp4", false).await;
    assert_eq!(asset["state"], json!("failed"), "{asset}");
    assert!(asset["error"].as_str().unwrap().contains("ffprobe"), "{asset}");
    assert!(asset["poster_url"].is_null() && asset["duration_ms"].is_null());
}
