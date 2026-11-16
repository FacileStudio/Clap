import type { Clip, Op, Timeline } from '$lib/types';
import type { Box } from './box';
import { findClip, locate, round, sceneStarts } from './model';

/** The ops that put each clip of `moves` in its new box; a clip that did not move gives none. */
export function moveOps(timeline: Timeline, moves: ReadonlyMap<string, Box>): Op[] {
	const ops: Op[] = [];
	for (const [id, box] of moves) {
		const clip = findClip(timeline, id)?.clip;
		if (!clip || (clip.start === box.start && clip.track === box.track)) continue;
		ops.push({ op: 'move_clip', clip: id, start: box.start, ...(clip.track === box.track ? {} : { track: box.track }) });
	}
	return ops;
}

/** The op that trims `clip` to `box`. Moving the start of a video or a sound moves its `media_in` as much. */
export function trimOps(clip: Clip, box: Box): Op[] {
	if (clip.start === box.start && clip.duration === box.duration) return [];
	const op: Op = { op: 'trim_clip', clip: clip.id, duration: box.duration };
	if (box.start !== clip.start) op.start = box.start;
	const media = clip.kind === 'video' || clip.kind === 'audio';
	if (media && box.start !== clip.start) op.media_in = round(Math.max(0, clip.media_in + box.start - clip.start));
	return [op];
}

/** The ops that cut at project time `time`: the selected clips under it, or every clip under it when none is selected. */
export function splitOps(timeline: Timeline, selection: ReadonlySet<string>, time: number): Op[] {
	const { scene, local } = locate(timeline, time);
	const under = (timeline.scenes[scene]?.clips ?? []).filter((clip) => local > clip.start && local < clip.start + clip.duration);
	const chosen = under.some((clip) => selection.has(clip.id)) ? under.filter((clip) => selection.has(clip.id)) : under;
	return chosen.map((clip) => ({ op: 'split_clip', clip: clip.id, at: round(local) }));
}

/** Where scene `index` lands when its middle is dragged to project time `center`. */
export function reorderTarget(timeline: Timeline, index: number, center: number): number {
	const starts = sceneStarts(timeline);
	const before = timeline.scenes.filter((scene, other) => other !== index && starts[other] + scene.duration / 2 < center);
	return before.length;
}
