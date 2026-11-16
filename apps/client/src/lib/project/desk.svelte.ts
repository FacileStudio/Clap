import { backend } from '$lib/backend';
import { t } from '$lib/i18n';
import { dropOps } from '$lib/timeline/adds';
import { Engine, type Refusal } from '$lib/timeline/engine.svelte';
import { clipAtLine, spanOf } from '$lib/timeline/link';
import { locate } from '$lib/timeline/model';
import type { Asset, Project } from '$lib/types';
import { watchProject } from './events';
import { Saver } from './saver.svelte';
import type { Starter } from './starters';
import { Transport } from './transport.svelte';
import { Uploads } from './uploads.svelte';

/** What the desk asks of the mounted source editor. */
export interface SourceView {
	replace(source: string): void;
	mark(span: { from: number; to: number } | null): void;
}

/**
 * One open project: its saved text, its timeline, its clock and its media, kept in step. The source editor and
 * the timeline both write the same file, each with the revision it read.
 */
export class Desk {
	readonly id: string;
	readonly transport = new Transport();
	readonly saver: Saver;
	readonly engine: Engine;
	readonly uploads: Uploads;
	title = $state('');
	text = $state('');
	assets = $state<Asset[]>([]);
	view = $state<SourceView>();
	private late = 0;

	constructor(
		project: Project,
		private readonly fail: (message: string) => void
	) {
		this.id = project.id;
		this.title = project.title;
		this.text = project.source;
		this.uploads = new Uploads(this.id, () => void this.loadAssets());
		this.saver = new Saver({
			put: (source, rev) => backend.save(this.id, source, rev),
			get: () => backend.project(this.id),
			onreplace: (source) => this.show(source)
		});
		this.saver.open(project);
		this.engine = new Engine({
			settle: () => this.settle(),
			edit: (rev, ops) => backend.edit(this.id, rev, ops),
			put: (source, rev) => backend.save(this.id, source, rev),
			timeline: () => backend.timeline(this.id),
			adopt: (source, rev) => (this.saver.open({ source, rev }), this.show(source)),
			reload: () => this.saver.reload(),
			refused: (why, error) => this.fail(refusal(why, error))
		});
		void this.engine.sync(project.source);
		void this.loadAssets();
	}

	/** The person typed in the source editor. */
	typed(source: string): void {
		this.text = source;
		this.saver.edit(source);
	}

	/** The caret moved to `line`: the clip written there becomes the selection. */
	caret(line: number): void {
		const id = clipAtLine(this.engine.timeline, this.engine.spans, line);
		if (id && !this.engine.selection.has(id)) this.engine.select(id);
	}

	/** The source lines of the one selected clip, or null. */
	get marked(): { from: number; to: number } | null {
		const [only, more] = [...this.engine.selection];
		const span = only && !more ? spanOf(this.engine.timeline, this.engine.spans, only) : null;
		return span ? { from: span.from, to: span.to } : null;
	}

	/** Follows the project's events until the answered function is called. */
	watch(): () => void {
		return watchProject(this.id, {
			onchanged: (rev) => (this.engine.busy ? (this.late = Math.max(this.late, rev)) : this.heard(rev)),
			onasset: () => void this.loadAssets()
		});
	}

	/** The engine finished an edit: an event held back meanwhile is handled now. */
	settled(): void {
		const rev = this.late;
		this.late = 0;
		if (rev) this.heard(rev);
	}

	async rename(next: string): Promise<void> {
		this.title = (await backend.rename(this.id, next)).title;
	}

	async loadAssets(): Promise<void> {
		this.assets = await backend.assets(this.id).catch(() => this.assets);
	}

	async removeAsset(asset: Asset): Promise<void> {
		await backend.removeAsset(this.id, asset.name);
		this.assets = this.assets.filter((known) => known.name !== asset.name);
	}

	/** Places `asset` at the playhead, on the lowest track that is free there. */
	place(asset: Asset): void {
		const ops = dropOps(this.engine.timeline, asset, this.transport.time, 0);
		if (ops.length > 0) void this.engine.commit(ops);
	}

	/** Adds the starter scene after the one under the playhead. */
	insert(starter: Starter): void {
		const timeline = this.engine.timeline;
		const at = timeline.scenes.length === 0 ? 0 : locate(timeline, this.transport.time).scene + 1;
		void this.engine.commit([{ op: 'add_scene', at, duration: starter.duration, body: starter.body }]);
	}

	private show(source: string): void {
		this.text = source;
		this.view?.replace(source);
		void this.engine.sync(source);
	}

	private async settle(): Promise<number | null> {
		await this.saver.flush();
		return this.saver.status === 'clean' ? this.saver.rev : null;
	}

	private heard(rev: number): void {
		void this.saver.remote(rev).catch((error: unknown) => this.fail(error instanceof Error ? error.message : t('error.generic')));
	}
}

function refusal(why: Refusal, error: unknown): string {
	const detail = why === 'failed' && error instanceof Error && error.message ? `: ${error.message}` : '';
	return `${t(`timeline.${why}`)}${detail}`;
}
