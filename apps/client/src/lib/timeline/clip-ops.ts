import type { Clip, Op, Timeline } from '$lib/types';
import { trimOps } from './edits';
import { findClip, locate, round } from './model';

/** The ops that delete clip `id` and pull the later clips of its track and scene back by its length. */
export function rippleOps(timeline: Timeline, id: string): Op[] {
	const found = findClip(timeline, id);
	if (!found) return [];
	const { scene, clip } = found;
	const later = scene.clips.filter((one) => one.track === clip.track && one.start >= clip.start + clip.duration && one.id !== id);
	const moves: Op[] = later
		.sort((a, b) => a.start - b.start)
		.map((one) => ({ op: 'move_clip', clip: one.id, start: round(Math.max(0, one.start - clip.duration)) }));
	return [{ op: 'delete_clip', clip: id }, ...moves];
}

/**
 * The op that moves clip `id` to the nearest track in direction `step` (1 up, -1 down) where nothing else plays
 * while it does; null when there is none, below track 0.
 */
export function laneOps(timeline: Timeline, id: string, step: 1 | -1): Op[] | null {
	const found = findClip(timeline, id);
	if (!found) return null;
	const { scene, clip } = found;
	const busy = (track: number) =>
		scene.clips.some((one) => one !== clip && one.track === track && one.start < clip.start + clip.duration && clip.start < one.start + one.duration);
	let track = clip.track + step;
	while (track >= 0 && busy(track)) track += step;
	return track < 0 ? null : [{ op: 'move_clip', clip: id, start: clip.start, track }];
}

/** Where the playhead at project time `time` cuts clip `id`, in scene time; null when it is not strictly inside. */
function cutAt(timeline: Timeline, id: string, time: number): { clip: Clip; at: number } | null {
	const found = findClip(timeline, id);
	if (!found) return null;
	const { scene, local } = locate(timeline, time);
	const at = round(local);
	const inside = timeline.scenes[scene] === found.scene && at > found.clip.start && at < found.clip.start + found.clip.duration;
	return inside ? { clip: found.clip, at } : null;
}

/** The ops that trim clip `id` so it starts (`side` start) or ends (`side` end) at the playhead; null when it is not under it. */
export function trimToOps(timeline: Timeline, id: string, time: number, side: 'start' | 'end'): Op[] | null {
	const cut = cutAt(timeline, id, time);
	if (!cut) return null;
	const { clip, at } = cut;
	const end = clip.start + clip.duration;
	const box = side === 'start' ? { start: at, duration: round(end - at) } : { start: clip.start, duration: round(at - clip.start) };
	return trimOps(clip, { ...box, track: clip.track });
}

/** The op that gives clip `id` a fade in and out, or with `clear` removes both of its transitions. */
export function fadeOps(id: string, clear: boolean): Op[] {
	const preset = clear ? null : 'fade';
	return [{ op: 'update_clip', clip: id, enter: preset, exit: preset }];
}
