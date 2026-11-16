//! The stage attributes of a clip and the transition of a scene: what `update_clip` and
//! `set_scene_transition` write, what `read` answers, and what `parse` hands the scene format.

use clap_timeline::{Error, Geometry, Op, Update, apply, parse, read};
use serde_json::json;

const LAUNCH: &str = include_str!("fixtures/launch.md");

fn boxed(x: f64, y: f64, w: f64, h: f64) -> Update {
    Update {
        x: Some(Some(x)),
        y: Some(Some(y)),
        w: Some(Some(w)),
        h: Some(Some(h)),
        rot: Some(Some(0.0)),
        ..Update::default()
    }
}

fn update(clip: &str, update: Update) -> Op {
    Op::UpdateClip { clip: clip.to_owned(), update }
}

#[test]
fn an_update_writes_the_box_after_data_start_and_null_takes_it_away() {
    let placed = apply(
        LAUNCH,
        &[update("sub", Update { fit: Some(Some("contain".into())), ..boxed(100.0, 100.0, 200.0, 200.5) })],
    )
    .unwrap();
    let written = "id=\"sub\" data-start=\"2\" data-x=\"100\" data-y=\"100\" data-w=\"200\" data-h=\"200.5\" data-rot=\"0\" data-fit=\"contain\" data-duration";
    assert_eq!(placed, LAUNCH.replace("id=\"sub\" data-start=\"2\" data-duration", written));
    let sub = read(&placed).unwrap().clip("sub").cloned().unwrap();
    assert_eq!(sub.look.frame, Some(Geometry { x: 100.0, y: 100.0, w: 200.0, h: 200.5, rot: 0.0 }));
    assert_eq!(sub.look.fit.as_deref(), Some("contain"));
    let none = Update {
        x: Some(None),
        y: Some(None),
        w: Some(None),
        h: Some(None),
        rot: Some(None),
        fit: Some(None),
        ..Update::default()
    };
    assert_eq!(apply(&placed, &[update("sub", none)]).unwrap(), LAUNCH);
    let music = apply(LAUNCH, &[update("music", Update { volume: Some(None), ..Update::default() })]).unwrap();
    assert_eq!(music, LAUNCH.replace(" data-volume=\"0.5\"", ""));
}

#[test]
fn an_update_changes_a_value_in_place_and_leaves_an_equal_one_alone() {
    let faded = Update {
        enter: Some(Some("fade".into())),
        enter_duration: Some(Some(0.25)),
        opacity: Some(Some(0.5)),
        ..Update::default()
    };
    let once = apply(LAUNCH, &[update("shot", faded.clone())]).unwrap();
    assert_eq!(apply(&once, &[update("shot", faded)]).unwrap(), once);
    let later =
        apply(&once, &[update("shot", Update { enter: Some(Some("pop".into())), ..Update::default() })]).unwrap();
    assert_eq!(later, once.replace("data-enter=\"fade\"", "data-enter=\"pop\""));
    let shot = read(&later).unwrap().clip("shot").cloned().unwrap();
    assert_eq!(
        (shot.look.enter.as_deref(), shot.look.enter_duration, shot.look.opacity),
        (Some("pop"), Some(0.25), Some(0.5))
    );
}

#[test]
fn an_update_refuses_values_the_runtime_cannot_draw() {
    for bad in [
        Update { w: Some(Some(0.0)), ..Update::default() },
        Update { x: Some(Some(f64::NAN)), ..Update::default() },
        Update { opacity: Some(Some(1.5)), ..Update::default() },
        Update { fit: Some(Some("stretch".into())), ..Update::default() },
        Update { exit: Some(Some("spin".into())), ..Update::default() },
        Update { exit_duration: Some(Some(-1.0)), ..Update::default() },
    ] {
        let error = apply(LAUNCH, &[update("sub", bad.clone())]).unwrap_err();
        assert_eq!(error.code(), "invalid_op", "{bad:?}");
    }
    assert_eq!(apply(LAUNCH, &[update("nope", Update::default())]).unwrap_err(), Error::UnknownClip("nope".into()));
}

#[test]
fn update_and_transition_ops_are_json_with_null_for_removal() {
    let text = r#"[{"op":"update_clip","clip":"a","x":10,"fit":null},
        {"op":"set_scene_transition","scene":1,"transition":"fade","duration":0.8},
        {"op":"set_scene_transition","scene":1,"transition":null}]"#;
    let ops: Vec<Op> = serde_json::from_str(text).unwrap();
    let expected = Update { x: Some(Some(10.0)), fit: Some(None), ..Update::default() };
    assert_eq!(ops[0], update("a", expected));
    assert_eq!(ops[1], Op::SetSceneTransition { scene: 1, transition: Some("fade".into()), duration: Some(Some(0.8)) });
    assert_eq!(ops[2], Op::SetSceneTransition { scene: 1, transition: None, duration: None });
    assert_eq!(
        serde_json::to_value(&ops[0]).unwrap(),
        json!({"op": "update_clip", "clip": "a", "x": 10.0, "fit": null})
    );
    assert_eq!(
        serde_json::to_value(&ops[2]).unwrap(),
        json!({"op": "set_scene_transition", "scene": 1, "transition": null})
    );
}

#[test]
fn a_transition_is_a_scene_key_that_travels_and_goes_away_cleanly() {
    let set = |scene, transition: Option<&str>, duration| Op::SetSceneTransition {
        scene,
        transition: transition.map(str::to_owned),
        duration,
    };
    let dip = apply(LAUNCH, &[set(1, Some("dip"), Some(Some(1.0)))]).unwrap();
    assert_eq!(dip, LAUNCH.replace("duration: 4\n---", "duration: 4\ntransition: dip\ntransition_duration: 1\n---"));
    let scene = &read(&dip).unwrap().scenes[1];
    assert_eq!((scene.transition.as_deref(), scene.transition_duration), (Some("dip"), Some(1.0)));
    let fade = apply(&dip, &[set(1, Some("fade"), Some(None))]).unwrap();
    assert_eq!(fade, LAUNCH.replace("duration: 4\n---", "duration: 4\ntransition: fade\n---"));
    assert_eq!(apply(&fade, &[set(1, None, None)]).unwrap(), LAUNCH);
    let bare = apply(LAUNCH, &[set(2, Some("zoom"), None)]).unwrap();
    assert_eq!(bare, LAUNCH.replace("---\n\n## Thanks", "---\ntransition: zoom\n---\n\n## Thanks"));
    assert_eq!(apply(&bare, &[set(2, None, None)]).unwrap(), LAUNCH);
    let moved = apply(&dip, &[Op::ReorderScene { scene: 1, to: 0 }]).unwrap();
    assert_eq!(read(&moved).unwrap().scenes[0].transition.as_deref(), Some("dip"));
    assert_eq!(apply(LAUNCH, &[set(0, Some("spin"), None)]).unwrap_err().code(), "invalid_op");
    assert_eq!(apply(LAUNCH, &[set(0, None, Some(Some(1.0)))]).unwrap_err().code(), "invalid_op");
}

#[test]
fn parse_folds_the_transition_into_the_section_class() {
    let source = "# One\n\n---\ntransition: fade\nclass: warm\ntransition_duration: 0.8\n---\n\n# Two\n";
    let project = parse(source).unwrap();
    assert_eq!(project.scenes()[1].class, "warm clap-transition-fade clap-transition-ms-800");
    let html = scene::compile(&project, &scene::Options::new());
    assert!(html.contains("class=\"scene layout-default warm clap-transition-fade clap-transition-ms-800\""), "{html}");
    assert!(scene::parse(source).is_err());
    let alone = parse("---\ntransition: slide-left\n---\n# Hi\n").unwrap();
    assert_eq!(alone.scenes()[0].class, "clap-transition-slide-left");
    let errors = parse("# One\n\n---\nlayout: raw\ntransition: spin\ncolour: red\n---\n\n<p>Two</p>\n").unwrap_err();
    let lines: Vec<usize> = errors.0.iter().map(|error| error.line).collect();
    assert_eq!(lines, [5, 6], "{errors}");
    assert!(parse("---\ntransition: fade\ntransition: dip\n---\n# Hi\n").unwrap_err().to_string().contains("line 3"));
    assert!(read("---\ntransition_duration: 0\n---\n# Hi\n").is_err());
}
