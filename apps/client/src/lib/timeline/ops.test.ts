import { describe, expect, it } from 'vitest';
import type { Asset, Op, Timeline } from '$lib/types';
import { applyOps } from './apply';
import { dropOps } from './adds';
import { moveOps, reorderTarget, splitOps, trimOps } from './edits';
import { ADDED, invertOps, nameAdded } from './invert';
import { chord, commandOf } from './keys';
import { nudgeOps } from './nudge';

const clip = (id: string, start: number, duration: number, track = 0) =>
	({ id, kind: 'video', src: 'assets/a.mp4', start, duration, track, media_in: 0 }) as const;

const BASE: Timeline = {
	scenes: [
		{ index: 0, duration: 6, layout: 'raw', clips: [clip('a', 1, 2), clip('b', 4, 1, 2)] },
		{ index: 1, duration: 4, layout: 'raw', clips: [clip('c', 0, 4)] }
	]
};

const shape = (timeline: Timeline) =>
	timeline.scenes.map((scene) => [scene.duration, ...scene.clips.map((one) => `${one.id}@${one.start}+${one.duration}/${one.track}`)]);

describe('applyOps', () => {
	it('moves, trims and deletes without touching its argument', () => {
		const ops: Op[] = [
			{ op: 'move_clip', clip: 'a', start: 2, track: 1 },
			{ op: 'trim_clip', clip: 'b', duration: 0.5 },
			{ op: 'delete_clip', clip: 'c' }
		];
		expect(shape(applyOps(BASE, ops))).toEqual([[6, 'a@2+2/1', 'b@4+0.5/2'], [4]]);
		expect(shape(BASE)[0][1]).toBe('a@1+2/0');
	});

	it('splits a clip in two and moves the media offset of the tail', () => {
		const [head, tail] = applyOps(BASE, [{ op: 'split_clip', clip: 'a', at: 1.5 }]).scenes[0].clips;
		expect([head.duration, tail.start, tail.duration, tail.media_in]).toEqual([0.5, 1.5, 1.5, 0.5]);
	});

	it('adds, reorders and resizes scenes, keeping their indexes in order', () => {
		const next = applyOps(BASE, [
			{ op: 'add_scene', at: 1, duration: 3, body: '' },
			{ op: 'reorder_scene', scene: 0, to: 2 },
			{ op: 'set_scene_duration', scene: 0, duration: 5 }
		]);
		expect(next.scenes.map((scene) => [scene.index, scene.duration])).toEqual([[0, 5], [1, 4], [2, 6]]);
	});
});

describe('invertOps', () => {
	const exact: Op[][] = [
		[{ op: 'move_clip', clip: 'a', start: 3, track: 4 }],
		[{ op: 'trim_clip', clip: 'a', start: 1.5, duration: 1, media_in: 0.5 }],
		[{ op: 'set_scene_duration', scene: 1, duration: 9 }],
		[{ op: 'reorder_scene', scene: 0, to: 1 }],
		[{ op: 'add_scene', at: 1, duration: 2, body: '' }],
		[{ op: 'move_clip', clip: 'a', start: 3 }, { op: 'move_clip', clip: 'a', start: 0 }, { op: 'reorder_scene', scene: 1, to: 0 }],
		[{ op: 'update_clip', clip: 'a', x: 100, y: 50, w: 200, h: 100, rot: 15, fit: 'contain', opacity: 0.5 }],
		[{ op: 'set_scene_transition', scene: 1, transition: 'dip', duration: 1 }]
	];

	it.each(exact)('undoes %j back to the same timeline', (...ops) => {
		const inverse = invertOps(BASE, ops);
		expect(inverse).not.toBeNull();
		expect(shape(applyOps(applyOps(BASE, ops), inverse ?? []))).toEqual(shape(BASE));
	});

	it('undoes a stage edit with the values the clip had, null where it had none', () => {
		const framed = applyOps(BASE, [{ op: 'update_clip', clip: 'a', x: 0, y: 0, w: 1280, h: 720, rot: 0, enter: 'fade' }]);
		const edit: Op = { op: 'update_clip', clip: 'a', x: 40, opacity: 0.25, enter: null };
		expect(invertOps(framed, [edit])).toEqual([{ op: 'update_clip', clip: 'a', x: 0, opacity: null, enter: 'fade' }]);
		const moved = applyOps(framed, [edit]).scenes[0].clips[0];
		expect([moved.frame?.x, moved.frame?.w, moved.opacity, moved.enter]).toEqual([40, 1280, 0.25, null]);
		const cut = applyOps(BASE, [{ op: 'set_scene_transition', scene: 0, transition: 'fade', duration: 0.8 }]);
		expect(invertOps(cut, [{ op: 'set_scene_transition', scene: 0, transition: null }])).toEqual([
			{ op: 'set_scene_transition', scene: 0, transition: 'fade', duration: 0.8 }
		]);
	});

	it('answers null for an edit that destroys text', () => {
		expect(invertOps(BASE, [{ op: 'delete_clip', clip: 'a' }])).toBeNull();
		expect(invertOps(BASE, [{ op: 'split_clip', clip: 'a', at: 2 }])).toBeNull();
		expect(invertOps(BASE, [{ op: 'delete_scene', scene: 0 }])).toBeNull();
	});

	it('names the clip an add created once the server has', () => {
		const add: Op = { op: 'add_clip', scene: 0, kind: 'text', start: 0, duration: 1, track: 3, text: 'x' };
		const inverse = invertOps(BASE, [add]);
		expect(inverse).toEqual([{ op: 'delete_clip', clip: ADDED }]);
		const after = structuredClone(BASE);
		after.scenes[0].clips.push(clip('fresh', 0, 1, 3));
		expect(nameAdded(inverse ?? [], BASE, after)).toEqual([{ op: 'delete_clip', clip: 'fresh' }]);
	});
});

describe('edits', () => {
	it('emits a move only for a clip that moved, with the track only when it changed', () => {
		const moves = new Map([['a', { start: 2, duration: 2, track: 0 }], ['b', { start: 4, duration: 1, track: 2 }]]);
		expect(moveOps(BASE, moves)).toEqual([{ op: 'move_clip', clip: 'a', start: 2 }]);
	});

	it('moves media_in with a trimmed start', () => {
		const ops = trimOps({ ...clip('a', 1, 2), media_in: 0.5 }, { start: 1.4, duration: 1.6, track: 0 });
		expect(ops).toEqual([{ op: 'trim_clip', clip: 'a', duration: 1.6, start: 1.4, media_in: 0.9 }]);
	});

	it('splits what is under the playhead, in scene time', () => {
		expect(splitOps(BASE, new Set(), 8)).toEqual([{ op: 'split_clip', clip: 'c', at: 2 }]);
		expect(splitOps(BASE, new Set(), 3.5)).toEqual([]);
	});

	it('drops a file at a project time and grows the scene when the clip runs past its end', () => {
		const asset = { name: 'assets/b.mp4', kind: 'video', duration_ms: 3000, state: 'ready' } as Asset;
		expect(dropOps(BASE, asset, 8, 1)).toEqual([
			{ op: 'set_scene_duration', scene: 1, duration: 5 },
			{ op: 'add_clip', scene: 1, kind: 'video', src: 'assets/b.mp4', start: 2, duration: 3, track: 1 }
		]);
		expect(dropOps(BASE, asset, 8, 0).at(-1)).toMatchObject({ track: 1 });
		expect(dropOps(BASE, { ...asset, kind: 'application/pdf' }, 0, 0)).toEqual([]);
	});

	it('finds where a dragged scene lands', () => {
		expect([reorderTarget(BASE, 0, 9), reorderTarget(BASE, 1, 1), reorderTarget(BASE, 0, 3)]).toEqual([1, 0, 0]);
	});
});

describe('keys', () => {
	const key = (name: string, mods: Partial<KeyboardEvent> = {}) =>
		({ key: name, shiftKey: false, ctrlKey: false, metaKey: false, altKey: false, ...mods }) as KeyboardEvent;

	it('maps the transport and edit shortcuts', () => {
		expect([' ', 'J', 'k', 'l', 's', 'Delete', 'Home'].map((name) => commandOf(key(name)))).toEqual([
			'toggle', 'back', 'pause', 'forward', 'split', 'delete', 'home'
		]);
		expect(commandOf(key('ArrowRight', { shiftKey: true }))).toBe('nextSecond');
		expect([chord(key('z', { metaKey: true })), commandOf(key('Z', { ctrlKey: true, shiftKey: true }))]).toEqual(['Mod+z', 'redo']);
		expect(commandOf(key('ArrowLeft', { altKey: true }))).toBeNull();
	});

	it('nudges a focused clip by a frame, a second, an edge or a track', () => {
		const [scene] = BASE.scenes;
		const a = scene.clips[0];
		expect(nudgeOps(a, scene, key('ArrowRight', { shiftKey: true }))).toEqual([{ op: 'move_clip', clip: 'a', start: 2 }]);
		expect(nudgeOps(a, scene, key('ArrowUp'))).toEqual([{ op: 'move_clip', clip: 'a', start: 1, track: 1 }]);
		expect(nudgeOps(a, scene, key('ArrowLeft', { altKey: true }), 10)).toEqual([{ op: 'trim_clip', clip: 'a', duration: 1.9 }]);
		expect(nudgeOps(a, scene, key('x'))).toBeNull();
	});
});
