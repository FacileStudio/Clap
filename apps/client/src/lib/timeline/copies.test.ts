import { describe, expect, it } from 'vitest';
import type { Op, Timeline } from '$lib/types';
import { applyOps } from './apply';
import { fadeOps, laneOps, rippleOps, trimToOps } from './clip-ops';
import { chain } from './commands';
import { copyName, duplicateOps, pasteOps } from './copies';
import { ADDED, invertOps, nameAdded } from './invert';

const clip = (id: string, start: number, duration: number, track = 0) =>
	({ id, kind: 'video', src: 'assets/a.mp4', start, duration, track, media_in: 0 }) as const;

const BASE: Timeline = {
	scenes: [
		{ index: 0, duration: 6, layout: 'raw', clips: [clip('a', 0, 2), clip('b', 2, 1), clip('c', 4, 1), clip('d', 0, 6, 1)] },
		{ index: 1, duration: 4, layout: 'raw', clips: [clip('e', 0, 4)] }
	]
};

const shape = (timeline: Timeline) =>
	timeline.scenes.map((scene) => scene.clips.map((one) => `${one.id}@${one.start}+${one.duration}/${one.track}`));

describe('copies', () => {
	it('names a copy the way the server does', () => {
		expect(copyName(new Set(['a']), 'a')).toBe('a-copy');
		expect(copyName(new Set(['a', 'a-copy', 'a-copy-2']), 'a')).toBe('a-copy-3');
	});

	it('duplicates a clip right after itself on the first free track, the scene growing when needed', () => {
		const ops = duplicateOps(BASE, 'c');
		expect(ops).toEqual([{ op: 'duplicate_clip', clip: 'c', scene: 0, start: 5, track: 0 }]);
		expect(shape(applyOps(BASE, ops))[0].at(-1)).toBe('c-copy@5+1/0');
		expect(duplicateOps(BASE, 'e')[0]).toEqual({ op: 'set_scene_duration', scene: 1, duration: 8 });
		expect(duplicateOps(BASE, 'a')[0]).toEqual({ op: 'duplicate_clip', clip: 'a', scene: 0, start: 2, track: 2 });
	});

	it('pastes into the scene under the playhead and grows it', () => {
		const ops = pasteOps(BASE, 'a', 8.5);
		expect(ops).toEqual([
			{ op: 'set_scene_duration', scene: 1, duration: 4.5 },
			{ op: 'duplicate_clip', clip: 'a', scene: 1, start: 2.5, track: 1 }
		]);
		expect(pasteOps(BASE, 'gone', 1)).toEqual([]);
	});

	it('copies a scene with its clips renamed, and undoes both copies', () => {
		const ops: Op[] = [{ op: 'duplicate_scene', scene: 1 }];
		expect(shape(applyOps(BASE, ops))[2]).toEqual(['e-copy@0+4/0']);
		expect(invertOps(BASE, ops)).toEqual([{ op: 'delete_scene', scene: 2 }]);
		const copy: Op[] = [{ op: 'duplicate_clip', clip: 'e', scene: 0, start: 1, track: 3 }];
		const after = applyOps(BASE, copy);
		expect(nameAdded(invertOps(BASE, copy) ?? [], BASE, after)).toEqual([{ op: 'delete_clip', clip: 'e-copy' }]);
		expect(invertOps(BASE, copy)).toEqual([{ op: 'delete_clip', clip: ADDED }]);
	});
});

describe('clip ops', () => {
	it('ripples a delete back along its track only', () => {
		expect(rippleOps(BASE, 'a')).toEqual([
			{ op: 'delete_clip', clip: 'a' },
			{ op: 'move_clip', clip: 'b', start: 0 },
			{ op: 'move_clip', clip: 'c', start: 2 }
		]);
		expect(shape(applyOps(BASE, chain(BASE, ['a', 'b'], rippleOps)))[0]).toEqual(['c@1+1/0', 'd@0+6/1']);
	});

	it('moves a clip to the nearest free track up or down, never below 0', () => {
		expect(laneOps(BASE, 'a', 1)).toEqual([{ op: 'move_clip', clip: 'a', start: 0, track: 2 }]);
		expect(laneOps(BASE, 'd', -1)).toBeNull();
		expect(laneOps(BASE, 'a', -1)).toBeNull();
	});

	it('trims to the playhead only when it is inside the clip', () => {
		expect(trimToOps(BASE, 'd', 1.5, 'start')).toEqual([{ op: 'trim_clip', clip: 'd', duration: 4.5, start: 1.5, media_in: 1.5 }]);
		expect(trimToOps(BASE, 'd', 1.5, 'end')).toEqual([{ op: 'trim_clip', clip: 'd', duration: 1.5 }]);
		expect(trimToOps(BASE, 'a', 3, 'end')).toBeNull();
		expect(trimToOps(BASE, 'e', 1, 'end')).toBeNull();
	});

	it('sets or clears both clip transitions', () => {
		expect(fadeOps('a', false)).toEqual([{ op: 'update_clip', clip: 'a', enter: 'fade', exit: 'fade' }]);
		expect(fadeOps('a', true)).toEqual([{ op: 'update_clip', clip: 'a', enter: null, exit: null }]);
	});
});
