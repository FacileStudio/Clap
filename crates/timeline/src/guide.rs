//! The format guide Clap gives people and agents: the scene reference, then Clap's own section.

/// What Clap adds to the scene format reference: time-based scenes, clips, tracks, media, the
/// determinism rules and the timeline operations. The API's `clap_format_guide` tool and the
/// `clap guide` command both print it after [`scene::FORMAT_GUIDE`].
pub const CLAP_GUIDE: &str = include_str!("guide.md");

/// The whole guide: `scene::FORMAT_GUIDE`, then [`CLAP_GUIDE`].
pub fn format_guide() -> String {
    format!("{}\n{CLAP_GUIDE}", scene::FORMAT_GUIDE)
}
