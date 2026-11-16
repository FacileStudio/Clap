use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::upload::png;
use super::{App, Person, fixture};

pub const SOURCE: &str = "---\nsize: 320x240\nlayout: raw\nduration: 2\n---\n\n\
<style>.back { background: url(assets/back.png); }</style>\n\
<img id=\"dot\" src=\"assets/dot.png\" data-start=\"0\" data-duration=\"2\" data-track-index=\"1\">\n\
<img id=\"again\" src=\"./assets/dot.png#frame\">\n";

pub async fn project(app: &App, who: &Person, space: Option<&str>) -> String {
    let id = fixture::id(&fixture::project(app, who, space, SOURCE).await);
    for name in ["assets/dot.png", "assets/back.png"] {
        assert_eq!(png(app, who, &id, name).await.status, 201);
    }
    id
}

pub async fn queued(app: &App, who: &Person, project: &str, request: Value) -> Value {
    let answer = app.call("POST", &format!("/api/projects/{project}/renders"), Some(who), Some(request)).await;
    assert_eq!(answer.status, 202, "{}", answer.text());
    answer.json()
}

pub fn dir(app: &App, render: &Value) -> PathBuf {
    app.state.data_dir.join("renders").join(render["id"].as_str().unwrap())
}

pub fn report(app: &App, render: &Value, status: Value) {
    let dir = dir(app, render);
    std::fs::write(dir.join("status.json.tmp"), status.to_string()).unwrap();
    std::fs::rename(dir.join("status.json.tmp"), dir.join("status.json")).unwrap();
}

pub async fn mirrored(app: &App, who: &Person, render: &Value) -> Value {
    api::modules::renders::mirror::tick(&app.state).await.unwrap();
    let path = format!("/api/renders/{}", render["id"].as_str().unwrap());
    app.call("GET", &path, Some(who), None).await.json()
}

pub fn finished(app: &App, render: &Value, bytes: &[u8]) {
    std::fs::write(dir(app, render).join("out.mp4"), bytes).unwrap();
    let frames = render["frames"].clone();
    report(app, render, json!({"state": "done", "frame": frames, "frames": frames, "error": null, "updated_at": "x"}));
}

pub fn heartbeat(data_dir: &Path, at: time::OffsetDateTime) {
    let stamp = at.format(&time::format_description::well_known::Rfc3339).unwrap();
    std::fs::write(data_dir.join("renders").join(".worker"), stamp).unwrap();
}

pub fn jobs(app: &App) -> usize {
    let entries = std::fs::read_dir(app.state.data_dir.join("renders")).unwrap();
    let names = entries.map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned());
    names.filter(|name| name != ".worker" && !name.starts_with(api::modules::assets::media::PREFIX)).count()
}
