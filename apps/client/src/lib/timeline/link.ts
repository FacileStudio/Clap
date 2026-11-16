import type { Timeline } from '$lib/types';
import type { ClipSpan, Outline } from './outline';

function position(timeline: Timeline, id: string): [number, number] | null {
	for (const [index, scene] of timeline.scenes.entries()) {
		const order = scene.clips.findIndex((clip) => clip.id === id);
		if (order >= 0) return [index, order];
	}
	return null;
}

const aligned = (timeline: Timeline, spans: Outline, scene: number): boolean =>
	timeline.scenes[scene]?.clips.length === spans.scenes[scene]?.clips.length;

/**
 * Where the clip `id` of `timeline` sits in the source that `spans` was read from. The two list a scene's clips
 * in the same order; a scene where their counts differ answers null.
 */
export function spanOf(timeline: Timeline, spans: Outline, id: string): ClipSpan | null {
	const found = position(timeline, id);
	if (!found || !aligned(timeline, spans, found[0])) return null;
	return spans.scenes[found[0]].clips[found[1]] ?? null;
}

/** The id of the innermost clip whose lines hold `line` (from 1), or null. */
export function clipAtLine(timeline: Timeline, spans: Outline, line: number): string | null {
	let best: { id: string; size: number } | null = null;
	spans.scenes.forEach((scene, index) => {
		if (!aligned(timeline, spans, index)) return;
		scene.clips.forEach((clip, order) => {
			const size = clip.to - clip.from;
			if (line >= clip.from && line <= clip.to && (!best || size < best.size)) {
				best = { id: timeline.scenes[index].clips[order].id, size };
			}
		});
	});
	return (best as { id: string } | null)?.id ?? null;
}
