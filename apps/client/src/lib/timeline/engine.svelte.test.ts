import { describe, expect, it, vi } from 'vitest';
import { ApiError } from '$lib/http';
import type { Op } from '$lib/types';
import { Engine, type EngineIo } from './engine.svelte';
import { outline } from './outline';

const SOURCE = `---
layout: raw
duration: 6
---

<video src="assets/a.mp4" data-start="1" data-duration="2" data-track-index="0"></video>
`;

const moved = (start: number) => SOURCE.replace('data-start="1"', `data-start="${start}"`);

function setup(edit: EngineIo['edit'], timeline: EngineIo['timeline'] = async () => {
	throw new ApiError(404, 'not_found', 'none');
}) {
	const io = {
		current: 1,
		rev: () => io.current,
		edit: vi.fn(edit),
		put: vi.fn(async () => ({ rev: (io.current += 1) })),
		timeline: vi.fn(timeline),
		adopt: vi.fn((_: string, rev: number) => void (io.current = rev)),
		reload: vi.fn(async () => undefined),
		refused: vi.fn()
	};
	return { io, engine: new Engine(io) };
}

const start = (engine: Engine) => engine.timeline.scenes[0]?.clips[0]?.start;
const move = (to: number): Op[] => [{ op: 'move_clip', clip: 'clip-0-0', start: to }];

describe('Engine', () => {
	it('falls back to the client outline when the API serves no timeline, and asks only once', async () => {
		const { io, engine } = setup(async () => ({ source: SOURCE, rev: 2 }));
		await engine.sync(SOURCE);
		await engine.sync(moved(3));
		expect([start(engine), io.timeline.mock.calls.length]).toEqual([3, 1]);
	});

	it('shows an edit at once, then adopts the source the server answers', async () => {
		let release: (answer: { source: string; rev: number }) => void = () => undefined;
		const { io, engine } = setup(() => new Promise((resolve) => (release = resolve)));
		await engine.sync(SOURCE);
		const done = engine.commit(move(3));
		expect([start(engine), engine.busy]).toEqual([3, true]);
		expect(await engine.commit(move(5))).toBe(false);
		await vi.waitFor(() => expect(io.edit).toHaveBeenCalledWith(1, move(3)));
		release({ source: moved(3.5), rev: 2 });
		expect(await done).toBe(true);
		expect([start(engine), engine.busy, engine.canUndo]).toEqual([3.5, false, true]);
		expect(io.adopt).toHaveBeenCalledWith(moved(3.5), 2);
	});

	it('rolls a refused edit back and says why', async () => {
		const { io, engine } = setup(async () => {
			throw new ApiError(500, 'internal', 'boom');
		});
		await engine.sync(SOURCE);
		expect(await engine.commit(move(3))).toBe(false);
		expect([start(engine), engine.canUndo, io.refused.mock.calls[0][0]]).toEqual([1, false, 'failed']);
	});

	it('reloads on a stale revision', async () => {
		const { io, engine } = setup(async () => {
			throw new ApiError(409, 'already_exists', 'moved');
		});
		await engine.sync(SOURCE);
		await engine.commit(move(3));
		expect([io.refused.mock.calls[0][0], io.reload.mock.calls.length, start(engine)]).toEqual(['stale', 1, 1]);
	});

	it('reports a project the server cannot read, keeps asking, and clears it once it reads again', async () => {
		const broken = new ApiError(422, 'invalid_argument', 'line 2: unknown key `colour`', [], 'parse');
		const { io, engine } = setup(async () => ({ source: SOURCE, rev: 2 }), async () => {
			throw broken;
		});
		await engine.sync(SOURCE);
		expect([engine.problem, start(engine)]).toEqual(['line 2: unknown key `colour`', 1]);
		io.timeline.mockResolvedValueOnce(outline(moved(2)));
		await engine.sync(moved(2));
		expect([engine.problem, start(engine), io.timeline.mock.calls.length]).toEqual([null, 2, 2]);
	});

	it('undoes with inverse ops and redoes the same edit', async () => {
		const { io, engine } = setup(async (rev, ops) => {
			const op = ops[0] as Extract<Op, { op: 'move_clip' }>;
			return { source: moved(op.start), rev: rev + 1 };
		});
		await engine.sync(SOURCE);
		await engine.commit(move(3));
		expect(await engine.undo()).toBe(true);
		expect([start(engine), engine.canUndo, engine.canRedo]).toEqual([1, false, true]);
		expect(io.edit.mock.calls[1][1]).toEqual([{ op: 'move_clip', clip: 'clip-0-0', start: 1, track: 0 }]);
		expect(await engine.redo()).toBe(true);
		expect([start(engine), engine.canUndo, engine.canRedo]).toEqual([3, true, false]);
	});

	it('undoes a delete by writing the kept source back', async () => {
		const gone = SOURCE.replace(/<video.*\n/, '');
		const { io, engine } = setup(async (rev) => ({ source: gone, rev: rev + 1 }));
		await engine.sync(SOURCE);
		await engine.commit([{ op: 'delete_clip', clip: 'clip-0-0' }]);
		expect(outline(gone).scenes[0].clips).toHaveLength(0);
		expect(await engine.undo()).toBe(true);
		expect(io.put).toHaveBeenCalledWith(SOURCE, 2);
		expect(start(engine)).toBe(1);
	});

	it('forgets the history when someone else writes the source', async () => {
		const { engine } = setup(async (rev) => ({ source: moved(3), rev: rev + 1 }));
		await engine.sync(SOURCE);
		await engine.commit(move(3));
		await engine.sync(moved(3));
		expect(engine.canUndo).toBe(true);
		await engine.sync(moved(4));
		expect([engine.canUndo, start(engine)]).toEqual([false, 4]);
	});

	it('takes the timeline an edit answers and carries the selection by position', async () => {
		const named = { scenes: [{ index: 0, duration: 6, layout: 'raw', clips: [{ ...outline(SOURCE).scenes[0].clips[0], id: 'video-1', start: 3 }] }] };
		const { io, engine } = setup(async (rev) => ({ source: moved(3), rev: rev + 1, timeline: named }));
		await engine.sync(SOURCE);
		engine.select('clip-0-0');
		await engine.commit(move(3));
		expect([...engine.selection, io.timeline.mock.calls.length]).toEqual(['video-1', 1]);
	});

	it('selects one clip, toggles with add, and drops ids that are gone', async () => {
		const { engine } = setup(async () => ({ source: SOURCE, rev: 2 }));
		await engine.sync(SOURCE);
		engine.select('clip-0-0');
		engine.select('ghost', true);
		expect([...engine.selection]).toEqual(['clip-0-0', 'ghost']);
		await engine.sync(moved(2));
		expect([...engine.selection]).toEqual(['clip-0-0']);
		engine.select('clip-0-0', true);
		expect(engine.selection.size).toBe(0);
	});
});
