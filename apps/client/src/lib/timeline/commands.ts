import { FPS } from '$lib/project/timecode';
import type { Transport } from '$lib/project/transport.svelte';
import type { Op } from '$lib/types';
import { splitOps } from './edits';
import type { Engine } from './engine.svelte';
import type { Command } from './keys';

function commit(engine: Engine, ops: Op[]): void {
	if (ops.length > 0) void engine.commit(ops);
}

function jump(transport: Transport, time: number): void {
	transport.pause();
	transport.seek(time);
}

/** Does what `command` asks, on the timeline `engine` edits and the clock `transport` runs. */
export function runCommand(command: Command, engine: Engine, transport: Transport): void {
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
		delete: () => commit(engine, [...engine.selection].map((clip) => ({ op: 'delete_clip', clip }))),
		undo: () => engine.undo(),
		redo: () => engine.redo()
	};
	void actions[command]();
}
