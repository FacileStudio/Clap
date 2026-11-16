//! The source a new project starts with, in the language of the person who creates it.

/// The languages the starter is written in, with its title and its hint. The first is the default.
const WORDS: [(&str, &str, &str); 4] = [
    ("fr", "Votre vidéo commence ici", "Déposez un média sur la timeline, ou modifiez ce texte dans Source."),
    ("en", "Your video starts here", "Drop a clip on the timeline, or edit this text in Source."),
    (
        "de",
        "Ihr Video beginnt hier",
        "Ziehen Sie ein Medium auf die Zeitleiste oder ändern Sie diesen Text im Quelltext.",
    ),
    ("es", "Tu vídeo empieza aquí", "Arrastra un medio a la línea de tiempo o edita este texto en Fuente."),
];

const TEMPLATE: &str = include_str!("starter.md");

fn quality(part: &str) -> f64 {
    let weight = part.split(';').skip(1).find_map(|param| param.trim().strip_prefix("q="));
    weight.and_then(|value| value.trim().parse().ok()).unwrap_or(1.0)
}

/// The language of `locale` the starter is written in: the best one an `Accept-Language` value,
/// or a single tag such as `de-CH`, names. French when it names none of them.
pub fn starter_language(locale: &str) -> &'static str {
    let mut asked: Vec<(f64, &str)> = locale.split(',').map(|part| (quality(part), part)).collect();
    asked.sort_by(|a, b| b.0.total_cmp(&a.0));
    let primary = |part: &str| part.split([';', '-', '_']).next().unwrap_or_default().trim().to_ascii_lowercase();
    let wanted = asked.iter().filter(|(weight, _)| *weight > 0.0).map(|(_, part)| primary(part));
    wanted.into_iter().find_map(|lang| WORDS.iter().find(|words| words.0 == lang)).map_or(WORDS[0].0, |words| words.0)
}

/// A one-scene project of 4 seconds on a dark stage: a centered title that rises in and a hint
/// line under it, written in the language [`starter_language`] picks from `locale`.
pub fn starter(locale: &str) -> String {
    let lang = starter_language(locale);
    let (_, title, hint) = WORDS.iter().find(|words| words.0 == lang).copied().unwrap_or(WORDS[0]);
    TEMPLATE.replace("{{lang}}", lang).replace("{{title}}", title).replace("{{hint}}", hint)
}
