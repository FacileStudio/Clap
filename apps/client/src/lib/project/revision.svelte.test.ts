import { describe, expect, it, vi } from 'vitest';
import { Revision } from './revision.svelte';

function opened(rev: number) {
	const io = { get: vi.fn(async () => ({ source: 'theirs', rev: 9 })), onreplace: vi.fn((_source: string) => undefined) };
	const revision = new Revision(io);
	revision.open(rev);
	return { io, revision };
}

describe('Revision', () => {
	it('loads the stored project when an event reports a newer revision', async () => {
		const { io, revision } = opened(1);
		await revision.remote(4);
		expect([revision.rev, io.onreplace.mock.calls]).toEqual([9, [['theirs']]]);
	});

	it('ignores the event that echoes its own write', async () => {
		const { io, revision } = opened(1);
		revision.open(2);
		await revision.remote(2);
		await revision.remote(1);
		expect([revision.rev, io.get.mock.calls.length]).toEqual([2, 0]);
	});
});
