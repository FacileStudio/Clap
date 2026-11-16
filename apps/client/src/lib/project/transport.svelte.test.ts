import { describe, expect, it } from 'vitest';
import { MAX_RATE, Transport } from './transport.svelte';

function clock() {
	let pending: ((now: number) => void) | null = null;
	const transport = new Transport(
		(tick) => ((pending = tick), 1),
		() => (pending = null)
	);
	transport.duration = 10;
	const frame = (now: number) => {
		const tick = pending;
		pending = null;
		tick?.(now);
	};
	return { transport, frame, waiting: () => pending !== null };
}

describe('Transport', () => {
	it('advances with the frames while playing and stops at the end', () => {
		const { transport, frame, waiting } = clock();
		transport.play();
		frame(1000);
		frame(1500);
		expect([transport.time, transport.playing]).toEqual([0.5, true]);
		frame(20_000);
		expect([transport.time, transport.playing, waiting()]).toEqual([10, false, false]);
		transport.play();
		expect(transport.time).toBe(0);
	});

	it('shuttles faster each time, backwards too, up to the cap', () => {
		const { transport, frame } = clock();
		transport.seek(5);
		transport.shuttle(1);
		transport.shuttle(1);
		expect(transport.rate).toBe(2);
		transport.shuttle(-1);
		expect(transport.rate).toBe(-1);
		for (let press = 0; press < 6; press += 1) transport.shuttle(-1);
		expect(transport.rate).toBe(-MAX_RATE);
		frame(0);
		frame(1);
		frame(251);
		expect(transport.time).toBe(3);
	});

	it('steps by frames, pausing, and clamps a seek', () => {
		const { transport } = clock();
		transport.play();
		transport.step(3);
		expect([transport.playing, transport.time]).toEqual([false, 0.1]);
		transport.seek(99);
		expect(transport.time).toBe(10);
		transport.toggle();
		expect(transport.playing).toBe(true);
	});
});
