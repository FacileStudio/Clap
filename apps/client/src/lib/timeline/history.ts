import type { Op } from '$lib/types';

/** One edit the person made: what it did, how to undo it, and the source before it. */
export interface Entry {
	ops: Op[];
	/** The ops that undo it; null when undo must restore `before`. */
	inverse: Op[] | null;
	before: string;
}

/** The undo and redo stacks of the timeline. A new edit drops what could be redone. */
export class History {
	private past: Entry[] = [];
	private future: Entry[] = [];

	constructor(private readonly limit = 100) {}

	get canUndo(): boolean {
		return this.past.length > 0;
	}

	get canRedo(): boolean {
		return this.future.length > 0;
	}

	/** Records a new edit. */
	push(entry: Entry): void {
		this.past.push(entry);
		if (this.past.length > this.limit) this.past.shift();
		this.future = [];
	}

	/** Takes the edit to undo, or undefined. Hand it back with `undone` once it is undone. */
	takeUndo(): Entry | undefined {
		return this.past.pop();
	}

	/** Takes the edit to redo, or undefined. Hand it back with `redone` once it is done again. */
	takeRedo(): Entry | undefined {
		return this.future.pop();
	}

	undone(entry: Entry): void {
		this.future.push(entry);
	}

	redone(entry: Entry): void {
		this.past.push(entry);
	}

	clear(): void {
		this.past = [];
		this.future = [];
	}
}
