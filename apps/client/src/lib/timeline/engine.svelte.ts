import { ApiError } from '$lib/http';
import type { Op, Timeline } from '$lib/types';
import { applyOps } from './apply';
import { History, type Entry } from './history';
import { invertOps, nameAdded } from './invert';
import { carry } from './link';
import { outline, type Outline } from './outline';

/** Why a timeline edit did not land. */
export type Refusal = 'blocked' | 'stale' | 'failed';

/** A write the server took. */
export type Written = { source: string; rev: number; timeline?: Timeline };

/** What the engine needs from the page: the API calls and the source editor's state. */
export interface EngineIo {
	/** Saves pending text, then answers the revision to edit at; null when the source cannot be saved now. */
	settle(): Promise<number | null>;
	/** Answers the new source and revision, and the timeline read from it when the server sends one. */
	edit(rev: number, ops: Op[]): Promise<Written>;
	put(source: string, rev: number): Promise<{ rev: number }>;
	timeline(): Promise<Timeline>;
	/** The engine wrote the project: the page must show `source` at `rev`. */
	adopt(source: string, rev: number): void;
	/** The project moved under the engine: the page must load the stored one. */
	reload(): Promise<void>;
	refused(why: Refusal, error: unknown): void;
}

class Blocked extends Error {}

const EMPTY: Outline = { width: 1920, height: 1080, scenes: [] };

/**
 * The timeline the editor draws and the edits made on it. An edit shows at once, goes to the server as ops with
 * the current revision, and is replaced by what the server answers; a refused edit is rolled back. One edit runs
 * at a time: while `busy`, a new one is dropped.
 */
export class Engine {
	timeline = $state.raw<Timeline>(EMPTY);
	spans = $state.raw<Outline>(EMPTY);
	selection = $state.raw<ReadonlySet<string>>(new Set());
	busy = $state(false);
	canUndo = $state(false);
	canRedo = $state(false);
	private source: string | null = null;
	private served = true;
	private readonly history = new History();

	constructor(private readonly io: EngineIo) {}

	/** The stored source is now `source`. When the engine did not write it, the undo history no longer applies. */
	async sync(source: string): Promise<void> {
		if (source === this.source) return;
		this.history.clear();
		this.mark();
		await this.read(source);
	}

	/** Runs `ops`, undoable. Answers whether they landed. */
	commit(ops: Op[]): Promise<boolean> {
		return this.alone(() => this.run(ops, (entry) => this.history.push(entry)));
	}

	undo(): Promise<boolean> {
		return this.alone(async () => {
			const entry = this.history.takeUndo();
			if (!entry) return false;
			const done = entry.inverse ? await this.send(entry.inverse) : await this.restore(entry.before);
			if (done) this.history.undone(entry);
			else this.history.redone(entry);
			return done;
		});
	}

	redo(): Promise<boolean> {
		return this.alone(async () => {
			const entry = this.history.takeRedo();
			if (!entry) return false;
			const done = await this.run(entry.ops, (again) => this.history.redone(again));
			if (!done) this.history.undone(entry);
			return done;
		});
	}

	/** Selects `id`; with `add`, toggles it in the selection. Null clears. */
	select(id: string | null, add = false): void {
		if (id === null) this.selection = new Set();
		else if (!add) this.selection = new Set([id]);
		else if (this.selection.has(id)) this.selection = new Set([...this.selection].filter((known) => known !== id));
		else this.selection = new Set([...this.selection, id]);
	}

	private async alone(job: () => Promise<boolean>): Promise<boolean> {
		if (this.busy) return false;
		this.busy = true;
		try {
			return await job();
		} finally {
			this.busy = false;
			this.mark();
		}
	}

	private async run(ops: Op[], record: (entry: Entry) => void): Promise<boolean> {
		const before = { timeline: this.timeline, source: this.source ?? '' };
		const inverse = invertOps(before.timeline, ops);
		if (!(await this.send(ops))) return false;
		record({ ops, inverse: inverse && nameAdded(inverse, before.timeline, this.timeline), before: before.source });
		return true;
	}

	private async send(ops: Op[]): Promise<boolean> {
		const kept = this.timeline;
		this.timeline = applyOps(kept, ops);
		try {
			await this.took(await this.io.edit(await this.rev(), ops));
			return true;
		} catch (error) {
			this.timeline = kept;
			await this.refuse(error);
			return false;
		}
	}

	private async restore(source: string): Promise<boolean> {
		try {
			await this.took({ source, rev: (await this.io.put(source, await this.rev())).rev });
			return true;
		} catch (error) {
			await this.refuse(error);
			return false;
		}
	}

	private async took(answer: Written): Promise<void> {
		const shown = this.timeline;
		this.io.adopt(answer.source, answer.rev);
		await this.read(answer.source, answer.timeline, shown);
	}

	private async rev(): Promise<number> {
		const rev = await this.io.settle();
		if (rev === null) throw new Blocked();
		return rev;
	}

	private async refuse(error: unknown): Promise<void> {
		if (error instanceof Blocked) return this.io.refused('blocked', error);
		if (!(error instanceof ApiError) || error.status !== 409) return this.io.refused('failed', error);
		this.io.refused('stale', error);
		await this.io.reload().catch(() => undefined);
	}

	private async read(source: string, sent?: Timeline, shown: Timeline = this.timeline): Promise<void> {
		this.source = source;
		const spans = outline(source);
		const served = sent ?? (this.served ? await this.io.timeline().catch(() => null) : null);
		if (this.source !== source) return;
		this.served = served !== null;
		this.spans = spans;
		this.timeline = served ?? spans;
		this.selection = carry(this.selection, shown, this.timeline);
	}

	private mark(): void {
		this.canUndo = this.history.canUndo;
		this.canRedo = this.history.canRedo;
	}
}
