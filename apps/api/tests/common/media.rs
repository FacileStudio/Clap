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
