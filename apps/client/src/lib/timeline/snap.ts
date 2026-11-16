import type { Timeline } from '$lib/types';
import { sceneStarts } from './model';

/** The snapping distance in pixels. */
export const SNAP_PX = 8;

/** A snapped value, and the target it landed on (null when nothing was close enough). */
export interface Snapped {
	value: number;
	target: number | null;
}

/** `value` moved onto the nearest of `targets` within `tolerance`, or left alone. */
export function snap(value: number, targets: readonly number[], tolerance: number): Snapped {
	let best: number | null = null;
	for (const target of targets) {
		const closer = best === null || Math.abs(target - value) < Math.abs(best - value);
		if (Math.abs(target - value) <= tolerance && closer) best = target;
	}
	return { value: best ?? value, target: best };
}

/** The start of a span of `duration` after snapping whichever of its two edges is closest to a target. */
export function snapSpan(start: number, duration: number, targets: readonly number[], tolerance: number): Snapped {
	const head = snap(start, targets, tolerance);
	const tail = snap(start + duration, targets, tolerance);
	const useTail = tail.target !== null && (head.target === null || Math.abs(tail.value - start - duration) < Math.abs(head.value - start));
	if (useTail) return { value: tail.value - duration, target: tail.target };
	return head;
}

/**
 * The project times an edge may snap to: the playhead, every scene bound and every clip edge, leaving out the
 * clips in `moving`.
 */
export function snapTargets(timeline: Timeline, playhead: number, moving: ReadonlySet<string>): number[] {
	const starts = sceneStarts(timeline);
	const found = new Set<number>([playhead]);
	timeline.scenes.forEach((scene, index) => {
		found.add(starts[index]).add(starts[index] + scene.duration);
		for (const clip of scene.clips) {
			if (!moving.has(clip.id)) found.add(starts[index] + clip.start).add(starts[index] + clip.start + clip.duration);
		}
	});
	return [...found];
}
