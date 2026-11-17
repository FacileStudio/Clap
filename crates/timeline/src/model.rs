//! What a `clap.md` holds, as the timeline shows it.

use serde::{Deserialize, Serialize};

/// A whole project: its stage and its scenes in order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Timeline {
    /// The project title, empty when the headmatter sets none.
    pub title: String,
    /// The stage width in pixels.
    pub width: u32,
    /// The stage height in pixels.
    pub height: u32,
    /// Seconds: the sum of the scenes.
    pub duration: f64,
    /// The scenes, in source order.
    pub scenes: Vec<Scene>,
}

impl Timeline {
    /// The clip with this id.
    pub fn clip(&self, id: &str) -> Option<&Clip> {
        self.scenes.iter().flat_map(|scene| &scene.clips).find(|clip| clip.id == id)
    }
}

/// One scene and the clips it holds.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Scene {
    /// The position of the scene, from 0.
    pub index: usize,
    /// Seconds from the start of the project.
    pub start: f64,
    /// Seconds the scene lasts.
    pub duration: f64,
    /// Whether the frontmatter sets `duration`. When it does not, the scene grows with its clips.
    pub fixed_duration: bool,
    /// `default`, `cover`, `center`, `split` or `raw`.
    pub layout: String,
    /// The source lines of the scene body.
    pub lines: Span,
    /// The frontmatter `transition`: how the scene comes in, one of [`SCENE_TRANSITIONS`].
    ///
    /// [`SCENE_TRANSITIONS`]: crate::SCENE_TRANSITIONS
    pub transition: Option<String>,
    /// The frontmatter `transition_duration`, in seconds. Without it a transition lasts 0.6 s.
    pub transition_duration: Option<f64>,
    /// The clips, in source order.
    pub clips: Vec<Clip>,
}

/// One element that carries `data-start`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Clip {
    /// `data-clip`, else `id`, else `clip-<scene>-<n>` for the n-th clip of its scene. A derived
    /// id moves when a clip before it is added or removed; an op that adds a clip writes
    /// `data-clip`.
    pub id: String,
    /// What the element is.
    pub kind: Kind,
    /// `src`, on a video, an audio or an image.
    pub src: Option<String>,
    /// `data-start`: seconds from the start of the scene.
    pub start: f64,
    /// `data-duration`, or the rest of the scene without it.
    pub duration: f64,
    /// `data-track-index`, 0 without it.
    pub track: i64,
    /// `data-in`: seconds into the media file where the clip starts playing.
    pub media_in: f64,
    /// The index of the scene that holds the clip.
    pub scene: usize,
    /// The source lines of the element, start tag to end tag.
    pub lines: Span,
    /// What a `text` clip shows, trimmed, its HTML escapes read back; `None` for any other kind.
    pub text: Option<String>,
    /// Where the clip sits on the stage and how it comes and goes. In JSON its fields sit on the
    /// clip itself.
    #[serde(flatten)]
    pub look: Look,
}

/// A box on the stage, in frame pixels from the top left corner, turned `rot` degrees clockwise
/// around its centre.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Geometry {
    /// `data-x`.
    pub x: f64,
    /// `data-y`.
    pub y: f64,
    /// `data-w`.
    pub w: f64,
    /// `data-h`.
    pub h: f64,
    /// `data-rot`, in degrees.
    pub rot: f64,
}

/// The stage attributes of a clip. Each one is `None` when the element does not carry it.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct Look {
    /// The box from `data-x`, `data-y`, `data-w`, `data-h` and `data-rot`, when the element
    /// carries at least one of them; a missing one is the full frame's (0, 0, the stage size, 0).
    /// Without any, the clip keeps the box its classes give it.
    pub frame: Option<Geometry>,
    /// `data-fit`: `cover`, `contain` or `fill`.
    pub fit: Option<String>,
    /// `data-opacity`, from 0 to 1.
    pub opacity: Option<f64>,
    /// `data-volume`, from 0 to 1.
    pub volume: Option<f64>,
    /// `data-enter`: one of [`CLIP_TRANSITIONS`].
    ///
    /// [`CLIP_TRANSITIONS`]: crate::CLIP_TRANSITIONS
    pub enter: Option<String>,
    /// `data-exit`: one of [`CLIP_TRANSITIONS`].
    ///
    /// [`CLIP_TRANSITIONS`]: crate::CLIP_TRANSITIONS
    pub exit: Option<String>,
    /// `data-enter-duration`, in seconds.
    pub enter_duration: Option<f64>,
    /// `data-exit-duration`, in seconds.
    pub exit_duration: Option<f64>,
}

/// 1-based source lines, both ends included.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    /// The first line.
    pub first: usize,
    /// The last line.
    pub last: usize,
}

/// What a clip element is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// A `<video>`.
    Video,
    /// An `<audio>`.
    Audio,
    /// An `<img>`.
    Image,
    /// An element holding text and no other element.
    Text,
    /// Any other element.
    Html,
}
