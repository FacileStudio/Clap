use std::path::Path;
use std::process::Command;
use std::time::Duration;

use serde_json::Value;

use super::{App, Person};

pub fn installed() -> bool {
    let runs = |name: &str| Command::new(name).arg("-version").output().is_ok_and(|out| out.status.success());
    let found = runs("ffmpeg") && runs("ffprobe");
    if !found {
        eprintln!("\n=== SKIPPED: ffmpeg or ffprobe is not installed, so the media tests did not run ===\n");
    }
    found
}

pub fn clip(dir: &Path) -> Vec<u8> {
    let path = dir.join("fixture.mp4");
    let picture = "testsrc=duration=2:size=320x240:rate=25";
    let sound = "sine=frequency=440:duration=2";
    let made = Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-y", "-f", "lavfi", "-i", picture, "-f", "lavfi", "-i", sound])
        .args(["-c:v", "libx264", "-preset", "ultrafast", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(made.status.success(), "{}", String::from_utf8_lossy(&made.stderr));
    std::fs::read(path).unwrap()
}

pub async fn settled(app: &App, who: &Person, project: &str, name: &str, proxied: bool) -> Value {
    for _ in 0..600 {
        let list = app.call("GET", &format!("/api/projects/{project}/assets"), Some(who), None).await.json();
        let found = list["assets"].as_array().unwrap().iter().find(|asset| asset["name"] == name).cloned();
        let asset = found.unwrap_or_else(|| panic!("no asset {name}: {list}"));
        let waiting =
            asset["state"] == "probing" || (proxied && asset["state"] == "ready" && asset["proxy_url"].is_null());
        if !waiting {
            return asset;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    panic!("the asset {name} was still being looked at after two minutes");
}

pub async fn poster(app: &App, render: &str) -> std::path::PathBuf {
    let path = api::modules::public::poster::path(&app.state.data_dir, render.parse().unwrap());
    for _ in 0..150 {
        if path.is_file() {
            return path;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    panic!("no poster for {render} after 30 s");
}

pub const REPORT: &str = r#"{"streams": [{"codec_type": "video", "width": 320, "height": 240,
    "avg_frame_rate": "25/1", "duration": "2.0"}], "format": {"duration": "2.0"}}"#;

pub const FAKE_MP4: &[u8] = b"\0\0\0\x18ftypisom-and-nothing-else";

pub async fn job(app: &App, input: &[u8]) -> std::path::PathBuf {
    let renders = app.state.data_dir.join("renders");
    for _ in 0..150 {
        let entries = std::fs::read_dir(&renders).unwrap().flatten().map(|entry| entry.path());
        let queued = entries.filter(|path| {
            path.file_name().is_some_and(|name| name.to_string_lossy().starts_with("media-"))
                && path.join("job.json").is_file()
                && !path.join("status.json").exists()
        });
        for dir in queued.collect::<Vec<_>>() {
            match std::fs::read(dir.join("bundle/input")).is_ok_and(|bytes| bytes == input) {
                true => return dir,
                false => finish(&dir),
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    panic!("the API queued no media job on those bytes in 30 s");
}

pub fn finish(dir: &Path) {
    let now = time::OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap();
    let status = serde_json::json!({"state": "done", "frame": 5, "frames": 5, "error": null, "updated_at": now});
    std::fs::write(dir.join("status.json.tmp"), status.to_string()).unwrap();
    std::fs::rename(dir.join("status.json.tmp"), dir.join("status.json")).unwrap();
}

pub async fn closed(dir: &Path) {
    for _ in 0..150 {
        if !dir.exists() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    panic!("the API did not close the media job in 30 s");
}
