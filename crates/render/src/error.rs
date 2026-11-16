use std::time::Duration;

/// Why a render failed.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A binary the renderer needs is not installed.
    #[error("{0} not found: set {1} or put it on PATH")]
    MissingTool(&'static str, &'static str),
    /// The job cannot be rendered as asked.
    #[error("invalid job: {0}")]
    Job(String),
    /// Chrome refused a command, crashed or closed its pipe.
    #[error("chrome: {0}")]
    Chrome(String),
    /// Chrome did not answer in time.
    #[error("chrome did not answer {0} within {1:?}")]
    Timeout(String, Duration),
    /// The page broke the `window.__scene` contract or threw.
    #[error("scene: {0}")]
    Scene(String),
    /// ffmpeg failed.
    #[error("ffmpeg: {0}")]
    Encode(String),
    /// The caller's progress callback asked to stop.
    #[error("render cancelled")]
    Cancelled,
    /// A file or a pipe failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
