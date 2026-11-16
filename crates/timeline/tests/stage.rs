//! The stage stylesheet and the starter a new project begins with.

use clap_timeline::{CLAP_GUIDE, Kind, read, with_runtime};

#[test]
fn the_starter_speaks_the_callers_language_and_reads_as_one_timed_scene() {
    for (locale, lang, title) in [
        ("", "fr", "Votre vidéo commence ici"),
        ("en-GB,en;q=0.9", "en", "Your video starts here"),
        ("it-IT, de;q=0.8, en;q=0.5", "de", "Ihr Video beginnt hier"),
        ("es", "es", "Tu vídeo empieza aquí"),
        ("en;q=0, ja", "fr", "Votre vidéo commence ici"),
    ] {
        assert_eq!(clap_timeline::starter_language(locale), lang, "{locale}");
        let source = clap_timeline::starter(locale);
        let timeline = read(&source).unwrap();
        let clips: Vec<(&str, Kind, i64)> =
            timeline.scenes[0].clips.iter().map(|clip| (clip.id.as_str(), clip.kind, clip.track)).collect();
        assert_eq!((timeline.duration, clips), (4.0, vec![("title", Kind::Text, 1), ("hint", Kind::Text, 2)]));
        assert!(source.contains(&format!("lang: {lang}\n")) && source.contains(title), "{source}");
        assert_eq!(scene::parse(&source).unwrap().config().lang, lang);
    }
}

#[test]
fn the_stage_stylesheet_lands_in_the_head_and_defines_every_class_add_clip_writes() {
    let html = with_runtime("<html><head><title>t</title></head><body><main></main></body></html>");
    let style = html.find(clap_timeline::STAGE_CSS).unwrap();
    assert!(style < html.find("</head>").unwrap() && html.find("clap:playing").unwrap() > html.find("<main>").unwrap());
    for class in ["clap-fill", "clap-layer", "clap-title", "clap-caption", "clap-lower"] {
        assert!(clap_timeline::STAGE_CSS.contains(&format!(".{class}{{")), "{class}");
        assert!(CLAP_GUIDE.contains(&format!("`{class}`")), "the guide documents {class}");
    }
}
