import { describe, expect, it } from 'vitest';
import type { Engine } from '$lib/timeline/engine.svelte';
import type { Asset, Op, Timeline } from '$lib/types';
import { Drops } from './drops.svelte';

const TIMELINE: Timeline = { scenes: [{ index: 0, duration: 10, layout: 'raw', clips: [] }] };
const asset: Asset = {
	name: 'assets/a.mov',
	kind: 'video',
	size: 1,
	duration_ms: null,
	width: null,
	height: null,
	state: 'probing',
	poster_url: null,
	url: '/a'
};
const unread = { kind: 'video' as const, duration: null, width: null, height: null, poster: 'data:,' };

function engine(): { fake: Engine; sent: Op[][] } {
	const sent: Op[][] = [];
	const fake = { busy: false, timeline: TIMELINE, commit: async (ops: Op[]) => (sent.push(ops), true) };
	return { fake: fake as unknown as Engine, sent };
}

const flush = () => new Promise((wake) => setTimeout(wake, 0));

describe('Drops', () => {
	it('places a file with a known length at once, at the drop time', async () => {
		const { fake, sent } = engine();
		new Drops(fake).landed(asset, { ...unread, duration: 3 }, { time: 2, track: 0 }, 'u1');
		await flush();
		expect(sent).toEqual([[{ op: 'add_clip', scene: 0, kind: 'video', src: asset.name, start: 2, duration: 3, track: 0 }]]);
	});

	it('keeps a ghost for a file nobody could measure until the probe ends', async () => {
		const { fake, sent } = engine();
		const drops = new Drops(fake);
		drops.landed(asset, unread, { time: 1, track: 0 }, 'u2');
		expect(drops.ghosts).toMatchObject([{ id: 'u2', duration: null, poster: 'data:,' }]);
		drops.heard([asset]);
		expect(drops.ghosts).toHaveLength(1);
		drops.heard([{ ...asset, state: 'ready', duration_ms: 4000 }]);
		await flush();
		expect(drops.ghosts).toEqual([]);
		expect(sent[0]?.at(-1)).toMatchObject({ start: 1, duration: 4 });
	});
});
