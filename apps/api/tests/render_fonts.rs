//! A rendered scene lays its text out in a proportional face. In a slim image fontconfig once
//! resolved `sans-serif` and `system-ui` to Liberation Mono, and every text of a video came out
//! in a typewriter face. The scene measures ten `i` against ten `M` in the theme's own font stack
//! and paints the frame green only when they differ as a proportional face makes them.

mod common;

use common::render::queued;
use common::worker::{alive, settled};
use serde_json::json;

const SOURCE: &str = "---\ntitle: Fonts\nsize: 320x180\nlayout: raw\nduration: 1\n---\n\n\
<div id=\"probe\" style=\"position:absolute;inset:0;background:#ff0000\"></div>\n\
<span id=\"narrow\" style=\"position:absolute;top:0;left:0;white-space:pre\">iiiiiiiiii</span>\n\
<span id=\"wide\" style=\"position:absolute;top:60px;left:0;white-space:pre\">MMMMMMMMMM</span>\n\
<script>\n  const ratio = document.getElementById(\"narrow\").offsetWidth / document.getElementById(\"wide\").offsetWidth;\n\
  document.getElementById(\"probe\").style.background = ratio < 0.6 ? \"#00ff00\" : \"#ff0000\";\n</script>\n";

fn corner(video: &std::path::Path) -> Vec<u8> {
    let crop = "crop=10:10:300:160,scale=1:1";
    let out = std::process::Command::new("ffmpeg")
        .args(["-nostdin", "-v", "error", "-sseof", "-0.2", "-i"])
        .arg(video)
        .args(["-frames:v", "1", "-vf", crop, "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    out.stdout
}

#[tokio::test]
async fn the_theme_text_renders_in_a_proportional_face() {
    let Some(pool) = common::pool("clap_render_fonts").await else { return };
    let Some((binary, chrome)) = common::worker::ready() else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = common::fixture::id(&common::fixture::project(&app, &ada, None, SOURCE).await);
    let _worker = common::worker::start(&binary, &chrome, &app.state.data_dir.join("renders"));
    alive(&app).await;
    let done = settled(&app, &ada, &queued(&app, &ada, &id, json!({"fps": 10})).await).await;
    assert_eq!(done["state"], json!("done"), "{done}");
    let video =
        api::modules::renders::status::output(&app.state.data_dir, done["id"].as_str().unwrap().parse().unwrap());
    let pixel = corner(&video);
    assert!(pixel[1] > 180 && pixel[0] < 90, "the probe is not green, so the face is not proportional: {pixel:?}");
}
