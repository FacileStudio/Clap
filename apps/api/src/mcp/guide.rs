use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tronc::mcp::{Answer, Call, ToolError};

/// `clap_format_guide`.
#[derive(Deserialize, JsonSchema)]
pub struct GuideIn {}

/// What `clap_format_guide` answers.
#[derive(Serialize, JsonSchema)]
pub struct GuideOut {
    /// The format reference, in markdown.
    pub guide: String,
}

/// `clap_format_guide`: the scene format reference and Clap's own section, both from
/// `clap_timeline::format_guide`, which the `clap guide` command prints too.
pub async fn guide(_call: Call, _input: GuideIn) -> Result<Answer<GuideOut>, ToolError> {
    Ok(GuideOut { guide: clap_timeline::format_guide() }.into())
}
