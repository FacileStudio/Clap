/** One row of `GET /api/projects`: a project without its source. */
export interface ProjectSummary {
	id: string;
	title: string;
	rev: number;
	updated_at: string;
}

/** A project as `GET /api/projects/{id}` answers it. Every write names the `rev` it read. */
export interface Project extends ProjectSummary {
	source: string;
}

/** One problem the scene parser found in a source the API refused with 422. `line` counts from 1. */
export interface SceneError {
	line: number;
	message: string;
}

/** What a clip shows. */
export type ClipKind = 'video' | 'audio' | 'image' | 'text' | 'html';

/** One clip of a scene. `start` and `duration` are seconds inside the scene; `track` is its `data-track-index`. */
export interface Clip {
	id: string;
	kind: ClipKind;
	src: string | null;
	start: number;
	duration: number;
	track: number;
	media_in: number;
	/** The stage box, null when the clip carries none and keeps the box its classes give it. */
	frame?: Geometry | null;
	fit?: Fit | null;
	opacity?: number | null;
	volume?: number | null;
	enter?: ClipTransition | null;
	exit?: ClipTransition | null;
	enter_duration?: number | null;
	exit_duration?: number | null;
}

/** A box on the stage in frame pixels from the top left, turned `rot` degrees clockwise around its centre. */
export interface Geometry {
	x: number;
	y: number;
	w: number;
	h: number;
	rot: number;
}

/** How a video or an image fills its box (`data-fit`). */
export type Fit = 'cover' | 'contain' | 'fill';

/** How a clip comes in or goes out (`data-enter`, `data-exit`). */
export type ClipTransition =
	| 'fade'
	| 'slide-left'
	| 'slide-right'
	| 'slide-up'
	| 'slide-down'
	| 'zoom-in'
	| 'zoom-out'
	| 'wipe-left'
	| 'wipe-right'
	| 'blur'
	| 'pop';

/** How a scene comes in (frontmatter `transition`). */
export type SceneTransition = 'fade' | 'dip' | 'slide-left' | 'slide-right' | 'wipe-left' | 'wipe-right' | 'zoom';

/** What `update_clip` changes: a field left out keeps its attribute, `null` removes it. */
export interface ClipUpdate {
	x?: number | null;
	y?: number | null;
	w?: number | null;
	h?: number | null;
	rot?: number | null;
	fit?: Fit | null;
	opacity?: number | null;
	volume?: number | null;
	enter?: ClipTransition | null;
	exit?: ClipTransition | null;
	enter_duration?: number | null;
	exit_duration?: number | null;
}

/** One scene of the timeline, with its clips. */
export interface Scene {
	index: number;
	duration: number;
	layout: string;
	clips: Clip[];
	transition?: SceneTransition | null;
	transition_duration?: number | null;
}

/** The model the timeline draws, as `GET /api/projects/{id}/timeline` answers it. */
export interface Timeline {
	scenes: Scene[];
	/** The stage size in pixels. */
	width?: number;
	height?: number;
}

/** One timeline edit of `POST /api/projects/{id}/edits`, tagged by `op`. */
export type Op =
	| { op: 'move_clip'; clip: string; start: number; track?: number }
	| { op: 'trim_clip'; clip: string; start?: number; duration?: number; media_in?: number }
	| { op: 'split_clip'; clip: string; at: number }
	| { op: 'delete_clip'; clip: string }
	| { op: 'add_clip'; scene: number; kind: ClipKind; src?: string; start: number; duration: number; track: number; text?: string }
	| { op: 'set_scene_duration'; scene: number; duration: number }
	| { op: 'reorder_scene'; scene: number; to: number }
	| { op: 'delete_scene'; scene: number }
	| { op: 'add_scene'; at: number; duration: number; body: string }
	| ({ op: 'update_clip'; clip: string } & ClipUpdate)
	| { op: 'set_scene_transition'; scene: number; transition: SceneTransition | null; duration?: number | null };

/** Where an uploaded file stands: still being probed, usable, or refused. */
export type AssetState = 'probing' | 'ready' | 'failed' | 'deleted';

/** A media file of a project. `name` is the relative path the source writes, such as `assets/clip.mp4`. */
export interface Asset {
	name: string;
	kind: string;
	size: number;
	duration_ms: number | null;
	width: number | null;
	height: number | null;
	state: AssetState;
	poster_url: string | null;
	url: string;
	error?: string | null;
}

/** Where a render job stands. */
export type RenderState = 'queued' | 'running' | 'done' | 'failed' | 'cancelled';

/** A render job as `GET /api/renders/{id}` answers it. */
export interface Render {
	id: string;
	state: RenderState;
	frame: number;
	frames: number;
	error: string | null;
	download_url: string | null;
	/** What the server says about a job that waits, such as a render service that went silent. */
	notice?: string | null;
	created_at?: string;
}

/** What `POST /api/projects/{id}/renders` takes. `range` is `[in, out]` in seconds; none renders the whole project. */
export interface RenderRequest {
	width?: number;
	height?: number;
	fps?: number;
	range?: [number, number];
}

/** The share link of a project, as `GET` and `POST /api/projects/{id}/share` answer it. `url` comes once, on creation. */
export interface ShareLink {
	shared: boolean;
	created_at: string | null;
	url?: string;
}

/** The video a share link shows, as `GET /api/public/renders/{token}` answers it. `poster` and `video` are routes. */
export interface PublicRender {
	title: string;
	duration_ms: number;
	width: number;
	height: number;
	poster: string | null;
	video: string;
}

/** The person's Bulle assistant, as `GET /api/assistant` answers it. */
export interface AssistantProfile {
	name: string;
	accent: string | null;
}

/** An agent action waiting for the person's confirmation, as `GET /api/mcp/pending` lists it. `target` is the project id. */
export interface PendingAction {
	id: string;
	kind: string;
	target: string;
	summary: string;
	client_id: string;
	created_at: string;
	expires_at: string;
	state: string;
}

/** The signed-in person, as `GET /api/auth/session` names them. */
export interface SessionUser {
	email: string;
	name?: string;
	avatar_url?: string;
}
