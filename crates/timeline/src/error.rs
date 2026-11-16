/// Why a source could not be read or an edit could not be applied.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    /// The source is not a valid project; the text lists every problem with its line.
    #[error("the source does not parse:\n{0}")]
    Parse(String),
    /// No clip has this id.
    #[error("unknown clip `{0}`")]
    UnknownClip(String),
    /// No scene has this index.
    #[error("unknown scene {0}")]
    UnknownScene(usize),
    /// The overlap policy: two clips of one scene that both carry `data-track-index` may not
    /// share a track and a moment.
    #[error("clip `{clip}` would overlap clip `{with}` on track {track}")]
    Overlap {
        /// The clip the op places.
        clip: String,
        /// The clip already there.
        with: String,
        /// The track both would sit on.
        track: i64,
    },
    /// A time that is negative, or past the end of a scene whose frontmatter sets `duration`.
    #[error("out of bounds: {0}")]
    OutOfBounds(String),
    /// An op whose own values make no sense, whatever the source.
    #[error("invalid op: {0}")]
    Invalid(String),
    /// The edit was applied and the result is no longer a valid project. Nothing was returned.
    #[error("the edit produced a source that does not parse:\n{0}")]
    InvalidResult(String),
}

impl Error {
    /// A stable machine name for the error, for an API answer.
    pub fn code(&self) -> &'static str {
        match self {
            Error::Parse(_) => "parse",
            Error::UnknownClip(_) => "unknown_clip",
            Error::UnknownScene(_) => "unknown_scene",
            Error::Overlap { .. } => "overlap",
            Error::OutOfBounds(_) => "out_of_bounds",
            Error::Invalid(_) => "invalid_op",
            Error::InvalidResult(_) => "invalid_result",
        }
    }
}
