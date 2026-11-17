import type { Timeline } from '$lib/types';
import { clipIds } from './model';
import type { ClipSpan, Outline } from './outline';

/**
 * `ids` as `after` names them, for clips that sit where they sat in `before`. Derived ids move when a clip is
 * added or removed, so a selection is carried by position; when the two do not have the same clips per scene,
 * only the ids `after` still has are kept.
 */
export function carry(ids: ReadonlySet<string>, before: Timeline, after: Timeline): Set<string> {
	const same = before.scenes.length === after.scenes.length && before.scenes.every((scene, index) => aligned(before, after, index));
	const kept = [...ids].flatMap((id) => {
		const found = position(before, id);
		const moved = found && same ? after.scenes[found[0]].clips[found[1]].id : id;
		return clipIds(after).includes(moved) ? [moved] : [];
	});
	return new Set(kept);
}

function position(timeline: Timeline, id: string): [number, number] | null {
	for (const [index, scene] of timeline.scenes.entries()) {
		const order = scene.clips.findIndex((clip) => clip.id === id);
		if (order >= 0) return [index, order];
	}
	return null;
}

const aligned = (timeline: Timeline, spans: Timeline, scene: number): boolean =>
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
