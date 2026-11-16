import type { Point } from '$lib/stage/geometry';
import { dropBox } from '$lib/stage/place';
import type { ClipKind, Op, Timeline } from '$lib/types';
import { placeOps, type Placeable } from './adds';
import { fullFrame } from './look';
import { clipIds, locate, round } from './model';

const MIN_STILL = 0.5;

/** The id the server gives the next `kind` clip it adds: `<kind>-<n>`, the first one the timeline does not use. */
export function nextClipId(timeline: Timeline, kind: ClipKind): string {
	const taken = new Set(clipIds(timeline));
	let n = 1;
	while (taken.has(`${kind}-${n}`)) n += 1;
	return `${kind}-${n}`;
}

/** A placeable file with its natural size in pixels, when known. */
export type Sized = Placeable & { width: number | null; height: number | null };

/**
 * The ops that drop `media` on the stage at frame point `at`, at project time `time`: added above every clip
 * playing then, in a box centred on the point at the media's ratio, fitted whole. A still lasts to the scene's end;
 * a sound gets no box.
 */
export function stageDropOps(timeline: Timeline, media: Sized, time: number, at: Point): Op[] {
	if (timeline.scenes.length === 0) return [];
	const { scene, local } = locate(timeline, time);
	const playing = timeline.scenes[scene]?.clips.filter((clip) => clip.start <= local && local < clip.start + clip.duration) ?? [];
	const top = playing.reduce((high, clip) => Math.max(high, clip.track + 1), 0);
	const rest = round((timeline.scenes[scene]?.duration ?? 0) - local);
	const still = media.duration === null && media.kind === 'image' && rest >= MIN_STILL;
	const ops = placeOps(timeline, still ? { ...media, duration: rest } : media, time, top);
	if (media.kind === 'audio' || ops.length === 0) return ops;
	const { w: width, h: height } = fullFrame(timeline);
	const box = dropBox({ width, height }, at, media);
	return [...ops, { op: 'update_clip', clip: nextClipId(timeline, media.kind), ...box, fit: 'contain' }];
}
