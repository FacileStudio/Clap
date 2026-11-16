import type { Clip, Scene, Timeline } from '$lib/types';

/** Rounds seconds to the millisecond, the precision the source keeps. */
export const round = (seconds: number): number => Math.round(seconds * 1000) / 1000;

/** Keeps `value` between `low` and `high`; `low` wins when they cross. */
export const clamp = (value: number, low: number, high: number): number => Math.max(low, Math.min(value, high));

/** The second each scene starts at, scenes laid end to end. */
export function sceneStarts(timeline: Timeline): number[] {
	let at = 0;
	return timeline.scenes.map((scene) => {
		const start = at;
		at += scene.duration;
		return start;
	});
}

/** The length of the whole project in seconds. */
export function total(timeline: Timeline): number {
	return timeline.scenes.reduce((sum, scene) => sum + scene.duration, 0);
}

/** The scene that holds the project time `time`, and the time inside it. The last scene holds its own end. */
export function locate(timeline: Timeline, time: number): { scene: number; local: number } {
	const starts = sceneStarts(timeline);
	let scene = 0;
	starts.forEach((start, index) => {
		if (time >= start) scene = index;
	});
	return { scene, local: Math.max(0, time - (starts[scene] ?? 0)) };
}

/** The clip `id` with its scene, or null when the timeline has no such clip. */
export function findClip(timeline: Timeline, id: string): { scene: Scene; clip: Clip } | null {
	for (const scene of timeline.scenes) {
		const clip = scene.clips.find((known) => known.id === id);
		if (clip) return { scene, clip };
	}
	return null;
}

/** The track rows the timeline shows, top first: every used track, track 0, and one free track above. */
export function trackRows(timeline: Timeline): number[] {
	const used = new Set<number>([0]);
	for (const scene of timeline.scenes) for (const clip of scene.clips) used.add(clip.track);
	const top = Math.max(...used) + 1;
	return [top, ...[...used].sort((a, b) => b - a)];
}

/** Every clip id of the timeline, in scene then source order. */
export function clipIds(timeline: Timeline): string[] {
	return timeline.scenes.flatMap((scene) => scene.clips.map((clip) => clip.id));
}
