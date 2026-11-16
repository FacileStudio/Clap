//! The motion starters of `starters/`, built into the binary.

/// One ready-made scene.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Starter {
    /// What `--template` takes.
    pub name: &'static str,
    /// How long the scene lasts.
    pub seconds: u32,
    /// What it is, in one line.
    pub summary: &'static str,
    /// The scene: a frontmatter block, then the composition.
    #[serde(skip)]
    pub source: &'static str,
}

macro_rules! starter {
    ($name:literal, $seconds:literal, $summary:literal) => {
        Starter {
            name: $name,
            seconds: $seconds,
            summary: $summary,
            source: include_str!(concat!("../../../starters/", $name, ".md")),
        }
    };
}

/// Every starter, in the order `clap starters` lists them.
pub const STARTERS: [Starter; 5] = [
    starter!("title", 4, "A large title, a subtitle and an accent bar that rise in and fade out"),
    starter!("lower-third", 5, "A name plate that slides in at the bottom left, holds, and leaves"),
    starter!("caption", 6, "Three captions shown one after the other as clips"),
    starter!("chart-race", 6, "Four bars that grow, then change length and rank"),
    starter!("logo-reveal", 3, "A mark that spins in, then the name"),
];

/// The starter called `name`.
pub fn find(name: &str) -> Option<&'static Starter> {
    STARTERS.iter().find(|starter| starter.name == name)
}

/// A whole project that begins with `starter`: its scene under a headmatter titled `title`.
pub fn project(starter: &Starter, title: &str) -> String {
    let safe: String = title.chars().filter(|letter| !letter.is_control() && *letter != '"').collect();
    let scene = starter.source.strip_prefix("---\n").unwrap_or(starter.source);
    format!("---\ntitle: \"{safe}\"\nsize: 1920x1080\n{scene}")
}
