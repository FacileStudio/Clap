import type { Op, Timeline } from '$lib/types';
import { applyOps } from './apply';
import { undoTransition, undoUpdate } from './look';
import { clipIds, findClip } from './model';

/** Stands for the id of a clip an `add_clip` creates, until the server names it. */
export const ADDED = '\u0000added';

type Inverters = { [K in Op['op']]: (before: Timeline, op: Extract<Op, { op: K }>) => Op[] | null };

const INVERTERS: Inverters = {
	move_clip: (before, op) => {
		const clip = findClip(before, op.clip)?.clip;
		return clip ? [{ op: 'move_clip', clip: clip.id, start: clip.start, track: clip.track }] : null;
	},
	trim_clip: (before, op) => {
		const clip = findClip(before, op.clip)?.clip;
		if (!clip) return null;
		return [{ op: 'trim_clip', clip: clip.id, start: clip.start, duration: clip.duration, media_in: clip.media_in }];
	},
	add_clip: () => [{ op: 'delete_clip', clip: ADDED }],
	duplicate_clip: () => [{ op: 'delete_clip', clip: ADDED }],
	paste_clip: () => [{ op: 'delete_clip', clip: ADDED }],
	set_clip_text: (before, op) => {
		const text = findClip(before, op.clip)?.clip.text;
		return text ? [{ op: 'set_clip_text', clip: op.clip, text }] : null;
	},
	duplicate_scene: (_, op) => [{ op: 'delete_scene', scene: op.scene + 1 }],
	set_scene_duration: (before, op) => {
		const scene = before.scenes[op.scene];
		return scene ? [{ op: 'set_scene_duration', scene: op.scene, duration: scene.duration }] : null;
	},
	reorder_scene: (_, op) => [{ op: 'reorder_scene', scene: op.to, to: op.scene }],
	add_scene: (_, op) => [{ op: 'delete_scene', scene: op.at }],
	update_clip: (before, op) => {
		const clip = findClip(before, op.clip)?.clip;
		return clip ? [undoUpdate(clip, op)] : null;
	},
	set_scene_transition: (before, op) => {
		const scene = before.scenes[op.scene];
		return scene ? [undoTransition(scene, op.scene)] : null;
	},
	split_clip: () => null,
	delete_clip: () => null,
	delete_scene: () => null
};

/**
 * The ops that undo `ops` on `before`, last first; null when one of them destroys text an op cannot write
 * back (a delete, a split), in which case undo restores the source it kept.
 */
export function invertOps(before: Timeline, ops: readonly Op[]): Op[] | null {
	const inverse: Op[] = [];
	let state = before;
	for (const op of ops) {
		const undo = (INVERTERS[op.op] as (before: Timeline, op: Op) => Op[] | null)(state, op);
		if (!undo) return null;
		inverse.unshift(...undo);
		state = applyOps(state, [op]);
	}
	return inverse;
}

/** `inverse` with each `ADDED` placeholder replaced by a clip that `after` has and `before` had not, last first. */
export function nameAdded(inverse: readonly Op[], before: Timeline, after: Timeline): Op[] {
	const known = new Set(clipIds(before));
	const added = clipIds(after).filter((id) => !known.has(id));
	return inverse.map((op) => (op.op === 'delete_clip' && op.clip === ADDED ? { ...op, clip: added.pop() ?? ADDED } : op));
}
