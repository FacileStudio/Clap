//! Generated sources for the property tests: scenes of raw clips of three kinds.

use proptest::prelude::*;

/// One generated clip: start and duration in quarter seconds, and which element it is.
pub type ClipSpec = (u32, u32, u8);

pub fn quarter(n: u32) -> f64 {
    f64::from(n) / 4.0
}

fn element(scene: usize, nth: usize, (start, duration, kind): ClipSpec) -> String {
    let timing = format!(
        "data-start=\"{}\" data-duration=\"{}\" data-track-index=\"{nth}\"",
        quarter(start),
        quarter(duration + 1)
    );
    match kind % 3 {
        0 => format!("  <p id=\"c{scene}x{nth}\" {timing}>Line {nth}</p>\n"),
        1 => {
            format!("<video data-clip=\"c{scene}x{nth}\" src=\"assets/v{nth}.mp4\" {timing} data-in=\"0.5\"></video>\n")
        }
        _ => format!("<div class=\"clip\" id=\"c{scene}x{nth}\" {timing}>\n  <b>Bold</b>\n</div>\n"),
    }
}

pub fn source(scenes: &[(u32, Vec<ClipSpec>)]) -> String {
    let mut out = String::from("---\ntitle: Generated\nsize: 1280x720\n");
    for (index, (duration, clips)) in scenes.iter().enumerate() {
        let open = if index == 0 { "" } else { "---\n" };
        out.push_str(&format!("{open}layout: raw\nduration: {}\n---\n\n<!-- scene {index} -->\n", duration + 4));
        let elements: Vec<String> = clips.iter().enumerate().map(|(nth, clip)| element(index, nth, *clip)).collect();
        out.push_str(&elements.concat());
        out.push_str("<script>\n  const tag = '<p data-start=\"9\">';\n</script>\n\n");
    }
    out
}

pub fn scenes() -> impl Strategy<Value = Vec<(u32, Vec<ClipSpec>)>> {
    let clip = (0u32..8, 0u32..8, 0u8..3);
    prop::collection::vec((0u32..8, prop::collection::vec(clip, 0..4)), 1..4)
}
