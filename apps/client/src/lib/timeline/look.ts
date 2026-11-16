import type { Clip, ClipUpdate, Geometry, Op, Scene, Timeline } from '$lib/types';
import { findClip } from './model';

type UpdateOp = Extract<Op, { op: 'update_clip' }>;
type TransitionOp = Extract<Op, { op: 'set_scene_transition' }>;

const BOX = ['x', 'y', 'w', 'h', 'rot'] as const;
const LOOK = ['fit', 'opacity', 'volume', 'enter', 'exit', 'enter_duration', 'exit_duration'] as const;

/** The full-frame box of `timeline`'s stage, the one a clip without `data-x` and its siblings fills. */
export function fullFrame(timeline: Timeline): Geometry {
	return { x: 0, y: 0, w: timeline.width ?? 1280, h: timeline.height ?? 720, rot: 0 };
}

/** Writes an `update_clip` into the clip it names, as the server will read the edited text back. */
export function updateClip(timeline: Timeline, op: UpdateOp): void {
	const clip = findClip(timeline, op.clip)?.clip;
	if (!clip) return;
	const given = BOX.filter((key) => op[key] !== undefined);
	if (given.length > 0) {
		const box = { ...(clip.frame ?? fullFrame(timeline)) };
		for (const key of given) box[key] = op[key] ?? fullFrame(timeline)[key];
		const cleared = given.length === BOX.length && given.every((key) => op[key] === null);
		clip.frame = cleared ? null : box;
	}
	const look = clip as unknown as Record<string, unknown>;
	for (const key of LOOK) if (op[key] !== undefined) look[key] = op[key];
}

/** The `update_clip` that puts back what `op` changes on `clip`: its previous values, `null` where it had none. */
export function undoUpdate(clip: Clip, op: UpdateOp): UpdateOp {
	const undo: ClipUpdate = {};
	const target = undo as Record<string, unknown>;
	for (const key of BOX) if (op[key] !== undefined) target[key] = clip.frame ? clip.frame[key] : null;
	for (const key of LOOK) if (op[key] !== undefined) target[key] = clip[key] ?? null;
	return { op: 'update_clip', clip: clip.id, ...undo };
}

/** Writes a `set_scene_transition` into its scene: a `null` transition clears both keys. */
export function setTransition(timeline: Timeline, op: TransitionOp): void {
	const scene = timeline.scenes[op.scene];
	if (!scene) return;
	scene.transition = op.transition;
	if (op.transition === null || op.duration === null) scene.transition_duration = null;
	else if (op.duration !== undefined) scene.transition_duration = op.duration;
}

/** The `set_scene_transition` that puts `scene` back as it was. */
export function undoTransition(scene: Scene, index: number): TransitionOp {
	return { op: 'set_scene_transition', scene: index, transition: scene.transition ?? null, duration: scene.transition_duration ?? null };
}
