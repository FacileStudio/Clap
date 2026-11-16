//! Real media: a clip made with ffmpeg is uploaded, then probed and given a poster, a filmstrip,
//! a waveform and a proxy by the worker binary, never by the API, whose `ffmpeg` here does not
//! exist, and served by range. Needs the worker, Chrome, `ffmpeg` and `ffprobe`; prints a banner
//! and passes without them.

mod common;

use axum::body::Body;
use common::fixture;
use common::media::{clip, installed, settled};
use common::upload::{File, upload};
use http::{Request, header};
use serde_json::json;

const NO_DECODER: &[(&str, &str)] = &[("CLAP_FFMPEG", "/nonexistent/ffmpeg")];

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
    let app = common::app_with(pool, NO_DECODER).await;
    let Some(_worker) = common::worker::media(&app).await else { return };
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
    previewed(&app, &ada, &asset).await;
}

fn size_of(dir: &std::path::Path, bytes: &[u8]) -> (i64, i64) {
    let path = dir.join(format!("strip-{}.jpg", uuid::Uuid::new_v4()));
    std::fs::write(&path, bytes).unwrap();
    let args = ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0"];
    let out = std::process::Command::new("ffprobe").args(args).arg(&path).output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let (width, height) = text.split_once(',').unwrap_or_else(|| panic!("no size: {text}"));
    (width.parse().unwrap(), height.parse().unwrap())
}

async fn previewed(app: &common::App, who: &common::Person, asset: &serde_json::Value) {
    assert_eq!((asset["strip_tiles"].clone(), asset["strip_step"].clone()), (json!(1), json!(2.0)), "{asset}");
    let strip = app.call("GET", asset["strip_url"].as_str().unwrap(), Some(who), None).await;
    assert_eq!((strip.status.as_u16(), &strip.bytes[..3]), (200, &[0xff, 0xd8, 0xff][..]));
    assert_eq!(strip.headers["content-type"], "image/jpeg");
    assert_eq!(size_of(&app.state.data_dir, &strip.bytes), (120, 90), "one 4:3 frame, 90 pixels high");
    let peaks = app.call("GET", asset["peaks_url"].as_str().unwrap(), Some(who), None).await;
    assert_eq!(peaks.headers["content-type"], "application/json");
    let values: Vec<f64> = serde_json::from_value(peaks.json()["peaks"].clone()).unwrap();
    let loud = values.iter().filter(|value| (0.08..0.2).contains(*value)).count();
    assert!((150..=1000).contains(&values.len()) && loud * 10 >= values.len() * 9, "a sine at 1/8: {values:?}");
}

async fn forget_previews(app: &common::App, sha: &str) {
    for suffix in ["strip.jpg", "peaks.json"] {
        std::fs::remove_file(app.state.data_dir.join(api::modules::assets::files::derived(sha, suffix))).unwrap();
    }
    let query = "UPDATE assets SET strip = NULL, strip_tiles = NULL, strip_step = NULL, peaks = NULL WHERE sha256 = $1";
    sqlx::query(query).bind(sha).execute(&app.state.pool).await.unwrap();
}

#[tokio::test]
async fn boot_makes_the_filmstrip_and_the_waveform_of_a_ready_asset_that_has_none() {
    let Some(pool) = common::pool("clap_media_resume").await else { return };
    if !installed() {
        return;
    }
    let app = common::app_with(pool, NO_DECODER).await;
    let Some(_worker) = common::worker::media(&app).await else { return };
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::project(&app, &ada, None, SOURCE).await);
    let bytes = clip(&app.state.data_dir);
    let file = File { name: Some("assets/clip.mp4"), file_name: "clip.mp4", bytes: &bytes };
    assert_eq!(upload(&app, &ada, &id, file).await.status, 201);
    let sha = settled(&app, &ada, &id, "assets/clip.mp4", true).await["sha256"].as_str().unwrap().to_owned();
    forget_previews(&app, &sha).await;
    let bare = settled(&app, &ada, &id, "assets/clip.mp4", true).await;
    assert!(bare["strip_url"].is_null() && bare["peaks_url"].is_null() && bare["strip_tiles"].is_null(), "{bare}");
    api::modules::assets::worker::resume(&app.state).await;
    for _ in 0..300 {
        let asset = settled(&app, &ada, &id, "assets/clip.mp4", true).await;
        if asset["strip_url"].is_string() && asset["peaks_url"].is_string() {
            return previewed(&app, &ada, &asset).await;
        }
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }
    panic!("boot made no filmstrip or waveform");
}

#[tokio::test]
async fn a_file_that_only_looks_like_a_video_fails_with_a_reason() {
    let Some(pool) = common::pool("clap_media_failed").await else { return };
    if !installed() {
        return;
    }
    let app = common::app_with(pool, NO_DECODER).await;
    let Some(_worker) = common::worker::media(&app).await else { return };
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
