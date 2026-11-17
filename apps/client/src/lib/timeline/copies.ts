import type { Op, Scene, Timeline } from '$lib/types';
import { freeTrack, growOps } from './adds';
import { clipIds, findClip, locate, round } from './model';

type DuplicateClip = Extract<Op, { op: 'duplicate_clip' }>;
type DuplicateScene = Extract<Op, { op: 'duplicate_scene' }>;

/** The name the server gives a copy of `id`: `<id>-copy`, else `<id>-copy-<n>` from 2, the first one not `taken`. */
export function copyName(taken: ReadonlySet<string>, id: string): string {
	if (!taken.has(`${id}-copy`)) return `${id}-copy`;
	let n = 2;
	while (taken.has(`${id}-copy-${n}`)) n += 1;
	return `${id}-copy-${n}`;
}

/** Writes a `duplicate_clip` into the timeline, as the server will read the edited text back. */
export function duplicateClip(timeline: Timeline, op: DuplicateClip): void {
	const found = findClip(timeline, op.clip);
	const scene = found ? timeline.scenes[op.scene ?? found.scene.index] : undefined;
	if (!found || !scene) return;
	const id = copyName(new Set(clipIds(timeline)), op.clip);
	scene.clips.push({ ...structuredClone(found.clip), id, start: op.start, track: op.track ?? found.clip.track });
}

/** Writes a `duplicate_scene` into the timeline: a copy right after the scene, every clip renamed as a copy. */
export function duplicateScene(timeline: Timeline, op: DuplicateScene): void {
	const scene = timeline.scenes[op.scene];
	if (!scene) return;
	const taken = new Set(clipIds(timeline));
	const copy: Scene = structuredClone(scene);
	for (const clip of copy.clips) {
		clip.id = copyName(taken, clip.id);
		taken.add(clip.id);
	}
	timeline.scenes.splice(op.scene + 1, 0, copy);
}

/**
 * The ops that copy clip `id` to project time `time`: on the first free track at or above its own, the scene made
 * longer first when the copy would run past its end. None when the clip is gone.
 */
export function pasteOps(timeline: Timeline, id: string, time: number): Op[] {
	const clip = findClip(timeline, id)?.clip;
	if (!clip || timeline.scenes.length === 0) return [];
	const { scene, local } = locate(timeline, time);
	const start = round(local);
	const track = freeTrack(timeline.scenes[scene].clips, start, clip.duration, clip.track);
	return [...growOps(timeline, scene, start + clip.duration), { op: 'duplicate_clip', clip: id, scene, start, track }];
}

/** The ops that copy clip `id` right after itself, in its own scene. */
export function duplicateOps(timeline: Timeline, id: string): Op[] {
	const found = findClip(timeline, id);
	if (!found) return [];
	const { scene, clip } = found;
	const start = round(clip.start + clip.duration);
	const track = freeTrack(scene.clips, start, clip.duration, clip.track);
	const at = timeline.scenes.indexOf(scene);
	return [...growOps(timeline, at, start + clip.duration), { op: 'duplicate_clip', clip: id, scene: at, start, track }];
}
