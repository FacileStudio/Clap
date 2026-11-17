import { FPS } from '$lib/project/timecode';
import type { Transport } from '$lib/project/transport.svelte';
import type { Op, Timeline } from '$lib/types';
import { applyOps } from './apply';
import { rippleOps } from './clip-ops';
import { duplicateOps, pasteOps } from './copies';
import { splitOps } from './edits';
import type { Engine } from './engine.svelte';
import type { Command } from './keys';

/** What a command acts on: the timeline engine, the clock, and the clip copied last. */
export interface Bench {
	readonly engine: Engine;
	readonly transport: Transport;
	copied: string | null;
}

function commit(engine: Engine, ops: Op[]): void {
	if (ops.length > 0) void engine.commit(ops);
}

function jump(transport: Transport, time: number): void {
	transport.pause();
	transport.seek(time);
}

/** The ops `make` gives for each of `ids` in turn, each one made on the timeline the ones before it leave. */
export function chain(timeline: Timeline, ids: Iterable<string>, make: (timeline: Timeline, id: string) => Op[]): Op[] {
	const ops: Op[] = [];
	let state = timeline;
	for (const id of ids) {
		const next = make(state, id);
		ops.push(...next);
		state = applyOps(state, next);
	}
	return ops;
}

/** Does what `command` asks on `bench`. `help` is the page's to answer. */
export function runCommand(command: Command, bench: Bench): void {
	const { engine, transport } = bench;
	const picked = () => [...engine.selection];
	const actions: Record<Command, () => unknown> = {
		toggle: () => transport.toggle(),
		back: () => transport.shuttle(-1),
		pause: () => transport.pause(),
		forward: () => transport.shuttle(1),
		prevFrame: () => transport.step(-1),
		nextFrame: () => transport.step(1),
		prevSecond: () => transport.step(-FPS),
		nextSecond: () => transport.step(FPS),
		home: () => jump(transport, 0),
		end: () => jump(transport, transport.duration),
		split: () => commit(engine, splitOps(engine.timeline, engine.selection, transport.time)),
		delete: () => commit(engine, picked().map((clip) => ({ op: 'delete_clip', clip }))),
		ripple: () => commit(engine, chain(engine.timeline, picked(), rippleOps)),
		copy: () => (bench.copied = picked()[0] ?? bench.copied),
		paste: () => bench.copied && commit(engine, pasteOps(engine.timeline, bench.copied, transport.time)),
		duplicate: () => commit(engine, chain(engine.timeline, picked(), duplicateOps)),
		undo: () => engine.undo(),
		redo: () => engine.redo(),
		help: () => undefined
	};
	void actions[command]();
}
