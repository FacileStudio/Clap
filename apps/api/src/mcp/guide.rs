use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

/// What Clap adds to the scene format reference: time-based scenes, clips, tracks, media and the
/// determinism rules.
pub const CLAP_GUIDE: &str = include_str!("guide.md");

/// The whole guide: `scene::FORMAT_GUIDE`, then [`CLAP_GUIDE`].
pub fn text() -> String {
    format!("{}\n{CLAP_GUIDE}", scene::FORMAT_GUIDE)
}

/// `clap_format_guide`.
#[derive(Deserialize, JsonSchema)]
pub struct GuideIn {}

/// What `clap_format_guide` answers.
#[derive(Serialize, JsonSchema)]
pub struct GuideOut {
    /// The format reference, in markdown.
    pub guide: String,
}

/// `clap_format_guide`: the scene format reference and Clap's own section.
pub async fn guide(_call: Call, _input: GuideIn) -> Result<Answer<GuideOut>, ToolError> {
    Ok(GuideOut { guide: text() }.into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_guide_is_the_scene_reference_then_claps_section_and_its_example_parses() {
        let guide = super::text();
        assert!(guide.starts_with(scene::FORMAT_GUIDE));
        for heading in ["## Clips", "## Tracks", "## Media", "## Determinism"] {
            assert!(guide.contains(heading), "{heading}");
        }
        let example = super::CLAP_GUIDE.split("```md\n").nth(1).unwrap().split("```").next().unwrap();
        let project = scene::parse(example).unwrap();
        assert_eq!((project.scenes().len(), project.duration()), (1, 6.0));
    }
}
