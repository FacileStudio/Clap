import type { Clip, Op, Scene, Timeline } from '$lib/types';
import { nextClipId } from './stage-drop';
import { setTransition, updateClip } from './look';
import { findClip, round } from './model';

type Handlers = { [K in Op['op']]: (timeline: Timeline, op: Extract<Op, { op: K }>) => void };

let serial = 0;

/** The id an added clip carries until the server names it. */
export const provisional = (): string => `new-${(serial += 1)}`;

const defined = <T extends object>(values: T): Partial<T> =>
	Object.fromEntries(Object.entries(values).filter(([, value]) => value !== undefined)) as Partial<T>;

function split(timeline: Timeline, op: Extract<Op, { op: 'split_clip' }>): void {
	const found = findClip(timeline, op.clip);
	if (!found || op.at <= found.clip.start || op.at >= found.clip.start + found.clip.duration) return;
	const { scene, clip } = found;
	const head = round(op.at - clip.start);
	const tail: Clip = { ...clip, id: provisional(), start: op.at, duration: round(clip.duration - head) };
	if (clip.kind === 'video' || clip.kind === 'audio') tail.media_in = round(clip.media_in + head);
	clip.duration = head;
	scene.clips.splice(scene.clips.indexOf(clip) + 1, 0, tail);
}

function reorder(timeline: Timeline, from: number, to: number): void {
	const [moved] = timeline.scenes.splice(from, 1);
	if (moved) timeline.scenes.splice(to, 0, moved);
}

const HANDLERS: Handlers = {
	move_clip: (timeline, op) => Object.assign(findClip(timeline, op.clip)?.clip ?? {}, defined({ start: op.start, track: op.track })),
	trim_clip: (timeline, { clip, start, duration, media_in }) =>
		Object.assign(findClip(timeline, clip)?.clip ?? {}, defined({ start, duration, media_in })),
	split_clip: split,
	delete_clip: (timeline, op) => {
		const found = findClip(timeline, op.clip);
		if (found) found.scene.clips.splice(found.scene.clips.indexOf(found.clip), 1);
	},
	add_clip: (timeline, { scene, kind, src, start, duration, track }) => {
		timeline.scenes[scene]?.clips.push({ id: nextClipId(timeline, kind), kind, src: src ?? null, start, duration, track, media_in: 0 });
	},
	set_scene_duration: (timeline, op) => Object.assign(timeline.scenes[op.scene] ?? {}, { duration: op.duration }),
	reorder_scene: (timeline, op) => reorder(timeline, op.scene, op.to),
	delete_scene: (timeline, op) => void timeline.scenes.splice(op.scene, 1),
	add_scene: (timeline, op) => {
		const scene: Scene = { index: op.at, duration: op.duration, layout: 'raw', clips: [] };
		timeline.scenes.splice(op.at, 0, scene);
	},
	update_clip: updateClip,
	set_scene_transition: setTransition
};

/** `timeline` after `ops`, as the editor shows it before the server answers. The argument is left untouched. */
export function applyOps(timeline: Timeline, ops: readonly Op[]): Timeline {
	const next = structuredClone(timeline);
	for (const op of ops) (HANDLERS[op.op] as (timeline: Timeline, op: Op) => void)(next, op);
	next.scenes.forEach((scene, index) => (scene.index = index));
	return next;
}
