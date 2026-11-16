import { describe, expect, it } from 'vitest';
import { kindOfType, posterSize, seconds } from '$lib/project/media-facts';
import type { Asset, Timeline } from '$lib/types';
import { dropOps } from './adds';
import { ghostBox, ghostRow, type Ghost } from './ghost';

const TIMELINE: Timeline = {
	scenes: [
		{ index: 0, duration: 6, layout: 'raw', clips: [] },
		{ index: 1, duration: 4, layout: 'raw', clips: [{ id: 'c', kind: 'video', src: 'a.mp4', start: 1, duration: 2, track: 0, media_in: 0 }] }
	]
};

const probing: Asset = {
	name: 'assets/b.mp4',
	kind: 'video',
	size: 1,
	duration_ms: null,
	width: null,
	height: null,
	state: 'probing',
	poster_url: null,
	url: '/b'
};

const ghost = (time: number, track: number, duration: number | null): Ghost => ({
	id: 'g',
	kind: 'video',
	label: 'b.mp4',
	duration,
	poster: null,
	progress: 0.5,
	target: { time, track }
});

describe('local media', () => {
	it('reads the clip kind from a MIME type', () => {
		expect(['video/webm', 'audio/mpeg', 'image/png', 'application/pdf', 'videos/x'].map(kindOfType)).toEqual([
			'video',
			'audio',
			'image',
			null,
			null
		]);
	});

	it('keeps the ratio of a poster and never enlarges it', () => {
		expect([posterSize(1920, 1080), posterSize(40, 30), posterSize(0, 10)]).toEqual([
			{ width: 160, height: 90 },
			{ width: 40, height: 30 },
			{ width: 0, height: 0 }
		]);
		expect([seconds(2.00049), seconds(Number.POSITIVE_INFINITY), seconds(Number.NaN), seconds(0)]).toEqual([2, null, null, null]);
	});
});

describe('drops', () => {
	it('places a probing file with the length the browser measured, and a still for 5 s', () => {
		expect(dropOps(TIMELINE, probing, 7, 0, 2.5).at(-1)).toMatchObject({ scene: 1, start: 1, duration: 2.5, track: 1 });
		expect(dropOps(TIMELINE, { ...probing, duration_ms: 1000 }, 0, 0, 2.5).at(-1)).toMatchObject({ duration: 1 });
		expect(dropOps(TIMELINE, { ...probing, kind: 'image' }, 0, 0).at(-1)).toMatchObject({ duration: 5 });
	});

	it('draws a ghost where its clip will land, in project time and on the free track', () => {
		expect(ghostBox(TIMELINE, ghost(7, 0, 2.5))).toEqual({ start: 7, duration: 2.5, track: 1 });
		expect(ghostBox(TIMELINE, ghost(2, 0, null))).toEqual({ start: 2, duration: 5, track: 0 });
		expect(ghostBox({ scenes: [] }, ghost(0, 0, 1))).toBeNull();
	});

	it('finds the row of a track, or borrows the next one down', () => {
		expect([ghostRow([4, 3, 0], 3), ghostRow([4, 3, 0], 1), ghostRow([4, 3, 0], -1)]).toEqual([1, 2, 2]);
	});
});
