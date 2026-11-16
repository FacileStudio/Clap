import { describe, expect, it } from 'vitest';
import type { Timeline } from '$lib/types';
import { applyOps } from './apply';
import { nextClipId, stageDropOps } from './stage-drop';
import { invertOps } from './invert';

const STAGE: Timeline = {
	width: 1920,
	height: 1080,
	scenes: [
		{
			index: 0,
			duration: 6,
			layout: 'raw',
			clips: [
				{ id: 'image-1', kind: 'image', src: 'assets/a.png', start: 0, duration: 6, track: 0, media_in: 0 },
				{ id: 'clip-0-2', kind: 'text', src: null, start: 0, duration: 2, track: 2, media_in: 0 }
			]
		}
	]
};

const picture = { kind: 'image' as const, src: 'assets/b.png', duration: null, width: 800, height: 400 };

describe('stageDropOps', () => {
	it('names the next clip as the server will', () => {
		expect(nextClipId(STAGE, 'image')).toBe('image-2');
		expect(nextClipId(STAGE, 'video')).toBe('video-1');
	});

	it('adds above what plays at the playhead, boxed around the drop point and fitted whole', () => {
		const ops = stageDropOps(STAGE, picture, 1, { x: 960, y: 540 });
		expect(ops).toEqual([
			{ op: 'add_clip', scene: 0, kind: 'image', src: 'assets/b.png', start: 1, duration: 5, track: 3 },
			{ op: 'update_clip', clip: 'image-2', x: 560, y: 340, w: 800, h: 400, rot: 0, fit: 'contain' }
		]);
		expect(stageDropOps(STAGE, picture, 3, { x: 0, y: 0 })[0]).toMatchObject({ start: 3, duration: 3, track: 1 });
	});

	it('gives a sound no box, and undoes the drop with one delete', () => {
		expect(stageDropOps(STAGE, { ...picture, kind: 'audio', duration: 2 }, 0, { x: 1, y: 1 })).toHaveLength(1);
		const ops = stageDropOps(STAGE, picture, 1, { x: 960, y: 540 });
		const after = applyOps(STAGE, ops);
		expect(after.scenes[0].clips.at(-1)).toMatchObject({ id: 'image-2', fit: 'contain', frame: { x: 560, w: 800 } });
		expect(invertOps(STAGE, ops)?.at(-1)).toEqual({ op: 'delete_clip', clip: '\u0000added' });
	});
});
