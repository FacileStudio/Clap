import { describe, expect, it } from 'vitest';
import { stepFrames, timecode } from '$lib/project/timecode';
import type { Timeline } from '$lib/types';
import { moveBox, trimEnd, trimStart } from './box';
import { locate, sceneStarts, total, trackRows } from './model';
import { fitZoom, MAX_ZOOM, scrollFor, tickStep, ticks, toPx, toTime, zoomBy } from './scale';
import { snap, snapSpan, snapTargets } from './snap';

const clip = (id: string, start: number, duration: number, track = 0) =>
	({ id, kind: 'video', src: 'assets/a.mp4', start, duration, track, media_in: 0 }) as const;

const TIMELINE: Timeline = {
	scenes: [
		{ index: 0, duration: 6, layout: 'raw', clips: [clip('a', 1, 2), clip('b', 4, 1, 2)] },
		{ index: 1, duration: 4, layout: 'raw', clips: [clip('c', 0, 4)] }
	]
};

describe('time and pixels', () => {
	it('converts both ways at a zoom', () => {
		expect([toPx(2.5, 80), toTime(200, 80)]).toEqual([200, 2.5]);
	});

	it('picks the smallest tick step that leaves room for a label', () => {
		expect([tickStep(60), tickStep(600), tickStep(4)]).toEqual([2, 0.25, 30]);
		expect(ticks(60, 0, 6)).toEqual([0, 2, 4, 6]);
	});

	it('keeps the time under the pointer still when zooming', () => {
		const left = scrollFor(300, 100, 50, 100);
		expect(toTime(left + 300, 100)).toBe(toTime(100 + 300, 50));
		expect(zoomBy(MAX_ZOOM, 2)).toBe(MAX_ZOOM);
		expect(fitZoom(10, 1000)).toBe(100);
	});

	it('writes a timecode as HH:MM:SS:FF and steps on frame bounds', () => {
		expect([timecode(0), timecode(3661.5), timecode(1 / 30)]).toEqual(['00:00:00:00', '01:01:01:15', '00:00:00:01']);
		expect([stepFrames(1, 1), stepFrames(0, -1), stepFrames(1.01, 30)]).toEqual([31 / 30, 0, 2]);
	});
});

describe('model', () => {
	it('lays scenes end to end', () => {
		expect([sceneStarts(TIMELINE), total(TIMELINE)]).toEqual([[0, 6], 10]);
		expect([locate(TIMELINE, 5), locate(TIMELINE, 6), locate(TIMELINE, 10)]).toEqual([
			{ scene: 0, local: 5 },
			{ scene: 1, local: 0 },
			{ scene: 1, local: 4 }
		]);
	});

	it('lists the track rows top first, with a free one above', () => {
		expect(trackRows(TIMELINE)).toEqual([3, 2, 0]);
	});
});

describe('snapping', () => {
	it('lands on the nearest target inside the tolerance only', () => {
		expect(snap(1.04, [1, 1.1, 3], 0.05)).toEqual({ value: 1, target: 1 });
		expect(snap(2, [1, 3], 0.05)).toEqual({ value: 2, target: null });
		expect(snap(2, [1, 3], 0)).toEqual({ value: 2, target: null });
	});

	it('snaps a span by whichever edge is closer', () => {
		expect(snapSpan(1.02, 2, [1, 3.1], 0.1)).toEqual({ value: 1, target: 1 });
		expect(snapSpan(1.08, 2, [1, 3.1], 0.1)).toEqual({ value: 1.1, target: 3.1 });
	});

	it('offers the playhead, scene bounds and the edges of the clips that stay', () => {
		expect(snapTargets(TIMELINE, 2.5, new Set(['a'])).sort((a, b) => a - b)).toEqual([0, 2.5, 4, 5, 6, 10]);
	});
});

describe('boxes', () => {
	const box = { start: 1, duration: 2, track: 0 };

	it('moves inside the scene', () => {
		expect(moveBox(box, 6, 10)).toEqual({ start: 4, duration: 2, track: 0 });
		expect(moveBox(box, 6, -5, -1)).toEqual({ start: 0, duration: 2, track: 0 });
	});

	it('trims each edge, never below the shortest clip or past the floor', () => {
		expect(trimStart(box, 0.5)).toEqual({ start: 1.5, duration: 1.5, track: 0 });
		expect(trimStart(box, 5)).toEqual({ start: 2.9, duration: 0.1, track: 0 });
		expect(trimStart(box, -5, 0.4)).toEqual({ start: 0.4, duration: 2.6, track: 0 });
		expect(trimEnd(box, 6, 10)).toEqual({ start: 1, duration: 5, track: 0 });
		expect(trimEnd(box, 6, -10)).toEqual({ start: 1, duration: 0.1, track: 0 });
	});
});
