//! Real renders of Clap's stage: a clip placed by `data-x` and its siblings, a clip and a sound
//! that fade in, and scenes that come in through black or over the scene before. Skipped,
//! loudly, when Chrome, ffmpeg or ffprobe is missing.

mod support;

use support::{Film, film, near, pixel, samples, tools};

#[test]
fn a_placed_picture_covers_its_box_and_nothing_else_the_same_way_twice() {
    let Some(tools) = tools() else { return };
    let source = "---\nsize: 640x360\nlayout: raw\nduration: 1\n---\n<img data-clip=\"red\" src=\"red.png\" alt=\"\" \
        data-start=\"0\" data-x=\"100\" data-y=\"100\" data-w=\"200\" data-h=\"200\" data-rot=\"0\" data-fit=\"fill\">\n";
    let Film { hashes, frames, .. } = film(&tools, source);
    let first = &frames[0];
    assert!(near(pixel(first, 200, 200), [255, 0, 0]), "inside: {:?}", pixel(first, 200, 200));
    assert!(near(pixel(first, 105, 105), [255, 0, 0]), "near the corner: {:?}", pixel(first, 105, 105));
    for (x, y) in [(50, 50), (310, 200), (200, 310), (500, 300)] {
        assert!(near(pixel(first, x, y), [0, 0, 0]), "outside at ({x}, {y}): {:?}", pixel(first, x, y));
    }
    assert_eq!(film(&tools, source).hashes, hashes, "two runs must capture the same pixels");
}

#[test]
fn a_clip_that_fades_in_starts_black_and_is_whole_after_its_enter() {
    let Some(tools) = tools() else { return };
    let source = "---\nsize: 640x360\nlayout: raw\nduration: 1\n---\n<img data-clip=\"red\" src=\"red.png\" alt=\"\" \
        data-start=\"0\" data-x=\"0\" data-y=\"0\" data-w=\"640\" data-h=\"360\" data-rot=\"0\" data-fit=\"cover\" \
        data-enter=\"fade\">\n";
    let frames = film(&tools, source).frames;
    assert!(near(pixel(&frames[0], 320, 180), [0, 0, 0]), "frame 0: {:?}", pixel(&frames[0], 320, 180));
    let [red, _, _] = pixel(&frames[7], 320, 180);
    assert!((60..200).contains(&red), "halfway in: {red}");
    assert!(near(pixel(&frames[15], 320, 180), [255, 0, 0]), "after 0.5 s: {:?}", pixel(&frames[15], 320, 180));
}

#[test]
fn a_sound_that_fades_in_and_out_rises_from_and_falls_to_silence() {
    let Some(tools) = tools() else { return };
    let source = "---\nsize: 640x360\nlayout: raw\nduration: 2\n---\n<audio data-clip=\"tone\" src=\"tone.wav\" \
        data-start=\"0\" data-duration=\"2\" data-enter=\"fade\" data-enter-duration=\"1\" data-exit=\"fade\"></audio>\n";
    let sound = samples(&tools, &film(&tools, source));
    let loudest = |from: f64, to: f64| {
        let range = (from * 48_000.0) as usize..((to * 48_000.0) as usize).min(sound.len());
        sound[range].iter().map(|sample| sample.unsigned_abs()).max().unwrap_or(0)
    };
    let (start, rising, full, end) = (loudest(0.0, 0.02), loudest(0.45, 0.55), loudest(1.1, 1.4), loudest(1.97, 2.0));
    assert!(full > 1_500, "the tone plays: {full}");
    assert!(start < full / 10 && end < full / 4, "it starts and ends near silence: {start}, {end} of {full}");
    assert!(rising > full / 3 && rising < full * 2 / 3, "halfway up at 0.5 s: {rising} of {full}");
}

const TWO: &str = "---\nsize: 640x360\nlayout: raw\nduration: 1\n---\n<img src=\"red.png\" alt=\"\" data-start=\"0\" \
    data-x=\"0\" data-y=\"0\" data-w=\"640\" data-h=\"360\" data-fit=\"cover\">\n\n---\nlayout: raw\nduration: 1\n\
    transition: TRANSITION\n---\n<img src=\"blue.png\" alt=\"\" data-start=\"0\" data-x=\"0\" data-y=\"0\" \
    data-w=\"640\" data-h=\"360\" data-fit=\"cover\">\n";

#[test]
fn a_dip_goes_through_black_at_the_boundary() {
    let Some(tools) = tools() else { return };
    let frames = film(&tools, &TWO.replace("TRANSITION", "dip")).frames;
    assert!(near(pixel(&frames[30], 320, 180), [0, 0, 0]), "at the boundary: {:?}", pixel(&frames[30], 320, 180));
    assert!(near(pixel(&frames[29], 320, 180), [0, 0, 0]), "just before: {:?}", pixel(&frames[29], 320, 180));
    assert!(near(pixel(&frames[15], 320, 180), [255, 0, 0]), "before the dip: {:?}", pixel(&frames[15], 320, 180));
    assert!(near(pixel(&frames[45], 320, 180), [0, 0, 255]), "after the dip: {:?}", pixel(&frames[45], 320, 180));
}

#[test]
fn a_crossfade_draws_the_scene_before_under_the_one_coming_in() {
    let Some(tools) = tools() else { return };
    let Film { hashes, frames, .. } = film(&tools, &TWO.replace("TRANSITION", "fade"));
    let [red, _, blue] = pixel(&frames[39], 320, 180);
    assert!(red > 60 && blue > 60, "halfway through the fade both show: {:?}", pixel(&frames[39], 320, 180));
    assert!(near(pixel(&frames[30], 320, 180), [255, 0, 0]), "at the boundary: {:?}", pixel(&frames[30], 320, 180));
    assert!(near(pixel(&frames[50], 320, 180), [0, 0, 255]), "after the fade: {:?}", pixel(&frames[50], 320, 180));
    assert_eq!(film(&tools, &TWO.replace("TRANSITION", "fade")).hashes, hashes, "a crossfade renders the same twice");
}

#[test]
fn a_video_holds_its_last_frame_while_the_next_scene_fades_in_over_it() {
    let Some(tools) = tools() else { return };
    let source = TWO.replace("TRANSITION", "fade").replacen("<img src=\"red.png\"", "<video src=\"red.mp4\"", 1);
    let source = source.replacen("data-fit=\"cover\">", "data-fit=\"cover\"></video>", 1);
    let frames = film(&tools, &source).frames;
    let [red, _, blue] = pixel(&frames[39], 320, 180);
    assert!(red > 60 && blue > 60, "the held still shows under the fade: {:?}", pixel(&frames[39], 320, 180));
}
