import type { Asset, ClipKind, Op, Timeline } from '$lib/types';
import { locate, round, trackRows } from './model';

const MEDIA: ClipKind[] = ['video', 'audio', 'image'];
const STILL = 5;

/** The kind of clip an uploaded file makes, or null when it is not media the timeline can place. */
export function kindOfAsset(asset: Asset): ClipKind | null {
	return MEDIA.find((kind) => asset.kind.startsWith(kind)) ?? null;
}

function freeTrack(timeline: Timeline, clip: Extract<Op, { op: 'add_clip' }>): number {
	const others = timeline.scenes[clip.scene]?.clips ?? [];
	const busy = (track: number) =>
		others.some((one) => one.track === track && one.start < clip.start + clip.duration && clip.start < one.start + one.duration);
	let track = clip.track;
	while (busy(track)) track += 1;
	return track;
}

/**
 * The ops that add `wanted` to its scene: on the first track at or above the wanted one where nothing plays at
 * that moment, and making the scene longer first when the clip would run past its end.
 */
export function addOps(timeline: Timeline, wanted: Extract<Op, { op: 'add_clip' }>): Op[] {
	const clip = { ...wanted, track: freeTrack(timeline, wanted) };
	const end = round(clip.start + clip.duration);
	const grow: Op[] = end > (timeline.scenes[clip.scene]?.duration ?? 0) ? [{ op: 'set_scene_duration', scene: clip.scene, duration: end }] : [];
	return [...grow, clip];
}

/** The ops that place `asset` at project time `time` on `track`; none when the file is not placeable media. */
export function dropOps(timeline: Timeline, asset: Asset, time: number, track: number): Op[] {
	const kind = kindOfAsset(asset);
	if (!kind || timeline.scenes.length === 0) return [];
	const { scene, local } = locate(timeline, time);
	const duration = round(asset.duration_ms ? asset.duration_ms / 1000 : STILL);
	return addOps(timeline, { op: 'add_clip', scene, kind, src: asset.name, start: round(local), duration, track });
}

/** The ops that add a text clip saying `text` at project time `time`, on a free track above the others. */
export function textOps(timeline: Timeline, time: number, text: string): Op[] {
	if (timeline.scenes.length === 0) return [];
	const { scene, local } = locate(timeline, time);
	const track = trackRows(timeline)[1] ?? 0;
	return addOps(timeline, { op: 'add_clip', scene, kind: 'text', start: round(local), duration: 3, track, text });
}
