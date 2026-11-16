//! The edits a timeline accepts.

use serde::{Deserialize, Deserializer, Serialize};

use crate::model::Kind;

/// One edit. In JSON: `{"op": "move_clip", "clip": "intro", "start": 1.5}`. Times are seconds;
/// a clip time counts from the start of its scene.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Op {
    /// Sets `data-start`, and `data-track-index` when `track` is given.
    MoveClip {
        /// The clip id.
        clip: String,
        /// The new start.
        start: f64,
        /// The new track.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        track: Option<i64>,
    },
    /// Sets any of `data-start`, `data-duration` and `data-in`.
    TrimClip {
        /// The clip id.
        clip: String,
        /// The new start.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        start: Option<f64>,
        /// The new duration.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration: Option<f64>,
        /// The new media offset.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        media_in: Option<f64>,
    },
    /// Cuts a clip in two at scene time `at`. The second half is a copy of the element, placed
    /// after it, whose id is `<id>-b`.
    SplitClip {
        /// The clip id.
        clip: String,
        /// Where to cut, strictly inside the clip.
        at: f64,
    },
    /// Removes the element.
    DeleteClip {
        /// The clip id.
        clip: String,
    },
    /// Adds an element after the last clip of the scene. Its id is `<kind>-<n>`.
    AddClip {
        /// The scene index.
        scene: usize,
        /// What to add.
        kind: Kind,
        /// The file to play or show: required for a video, an audio and an image.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        src: Option<String>,
        /// The start.
        start: f64,
        /// The duration.
        duration: f64,
        /// The track.
        track: i64,
        /// The text of a `text` clip, or the inner HTML of an `html` clip.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    /// Sets the `duration` key of a scene's frontmatter.
    SetSceneDuration {
        /// The scene index.
        scene: usize,
        /// The new duration.
        duration: f64,
    },
    /// Moves a scene so that it sits at index `to`.
    ReorderScene {
        /// The scene index.
        scene: usize,
        /// The index the scene ends up at.
        to: usize,
    },
    /// Removes a scene. The last scene of a project cannot be removed.
    DeleteScene {
        /// The scene index.
        scene: usize,
    },
    /// Inserts a scene at index `at`. A body that starts with `<` gets `layout: raw`.
    AddScene {
        /// The index the scene ends up at, up to the number of scenes.
        at: usize,
        /// The scene duration.
        duration: f64,
        /// The scene body, without frontmatter.
        body: String,
    },
    /// Sets or removes the stage attributes of a clip. A field left out keeps its attribute; a
    /// JSON `null` removes it.
    UpdateClip {
        /// The clip id.
        clip: String,
        /// What changes.
        #[serde(flatten)]
        update: Update,
    },
    /// Sets the frontmatter `transition` of a scene, and `transition_duration` when `duration`
    /// is given. A `null` transition removes both keys; a `null` duration removes that key.
    SetSceneTransition {
        /// The scene index.
        scene: usize,
        /// One of [`SCENE_TRANSITIONS`](crate::SCENE_TRANSITIONS), or `None` for a cut.
        #[serde(default)]
        transition: Option<String>,
        /// Seconds the transition lasts.
        #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
        duration: Option<Option<f64>>,
    },
}

/// What `update_clip` changes. `None` keeps an attribute, `Some(None)` removes it and
/// `Some(Some(value))` writes it. Box values are frame pixels, `rot` degrees, times seconds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Update {
    /// `data-x`.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub x: Option<Option<f64>>,
    /// `data-y`.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub y: Option<Option<f64>>,
    /// `data-w`, above 0.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub w: Option<Option<f64>>,
    /// `data-h`, above 0.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub h: Option<Option<f64>>,
    /// `data-rot`.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub rot: Option<Option<f64>>,
    /// `data-fit`, one of [`FITS`](crate::FITS).
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub fit: Option<Option<String>>,
    /// `data-opacity`, from 0 to 1.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub opacity: Option<Option<f64>>,
    /// `data-volume`, from 0 to 1.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub volume: Option<Option<f64>>,
    /// `data-enter`, one of [`CLIP_TRANSITIONS`](crate::CLIP_TRANSITIONS).
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub enter: Option<Option<String>>,
    /// `data-exit`, one of [`CLIP_TRANSITIONS`](crate::CLIP_TRANSITIONS).
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub exit: Option<Option<String>>,
    /// `data-enter-duration`, above 0.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub enter_duration: Option<Option<f64>>,
    /// `data-exit-duration`, above 0.
    #[serde(default, deserialize_with = "present", skip_serializing_if = "Option::is_none")]
    pub exit_duration: Option<Option<f64>>,
}

fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(input: D) -> Result<Option<Option<T>>, D::Error> {
    Option::<T>::deserialize(input).map(Some)
}
