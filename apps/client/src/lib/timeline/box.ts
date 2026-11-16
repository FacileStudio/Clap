import type { Clip } from '$lib/types';
import { clamp, round } from './model';

/** Where a clip sits in its scene. */
export interface Box {
	start: number;
	duration: number;
	track: number;
}

/** The shortest clip an edit may leave, in seconds. */
export const MIN_CLIP = 0.1;

/** `box` moved by `dt` seconds and `rows` tracks, kept inside a scene `limit` seconds long and on track 0 or above. */
export function moveBox(box: Box, limit: number, dt: number, rows = 0): Box {
	const start = clamp(box.start + dt, 0, Math.max(0, limit - box.duration));
	return { start: round(start), duration: box.duration, track: Math.max(0, box.track + rows) };
}

/** `box` with its start edge moved by `dt`: the end stays, and the start never goes below `floor`. */
export function trimStart(box: Box, dt: number, floor = 0): Box {
	const end = box.start + box.duration;
	const start = clamp(box.start + dt, Math.max(0, floor), end - MIN_CLIP);
	return { start: round(start), duration: round(end - start), track: box.track };
}

/** `box` with its end edge moved by `dt`, kept inside a scene `limit` seconds long. */
export function trimEnd(box: Box, limit: number, dt: number): Box {
	const end = clamp(box.start + box.duration + dt, box.start + MIN_CLIP, Math.max(limit, box.start + MIN_CLIP));
	return { start: box.start, duration: round(end - box.start), track: box.track };
}

/** How early the start of `clip` may be trimmed: a video or a sound has no media before its first frame. */
export function startFloor(clip: Clip): number {
	const media = clip.kind === 'video' || clip.kind === 'audio';
	return media ? Math.max(0, clip.start - clip.media_in) : 0;
}
