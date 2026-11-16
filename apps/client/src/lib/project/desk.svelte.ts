import { ApiError, backend } from '$lib/backend';
import { t } from '$lib/i18n';
import type { AppKey } from '$lib/i18n/messages';
import { Engine, type Refusal } from '$lib/timeline/engine.svelte';
import type { DropTarget, Ghost } from '$lib/timeline/ghost';
import { clipAtLine, spanOf } from '$lib/timeline/link';
import { locate } from '$lib/timeline/model';
import type { Asset, Project } from '$lib/types';
import { Drops } from './drops.svelte';
import { watchProject } from './events';
import { Saver } from './saver.svelte';
import type { Starter } from './starters';
import { Transport } from './transport.svelte';
import { Uploads, type Stored } from './uploads.svelte';

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
	readonly drops: Drops;
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
		this.uploads = new Uploads(this.id, (stored) => this.stored(stored));
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
		this.drops = new Drops(this.engine);
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
		this.drops.heard(this.assets);
	}

	async removeAsset(asset: Asset): Promise<void> {
		await backend.removeAsset(this.id, asset.name);
		this.assets = this.assets.filter((known) => known.name !== asset.name);
	}

	/** Places `asset` at the playhead, on the lowest track that is free there. */
	place(asset: Asset): void {
		this.drop(asset, { time: this.transport.time, track: 0 });
	}

	/** Places `asset` at `target`, with the length the browser read when the server has not probed it yet. */
	drop(asset: Asset, target: DropTarget): void {
		this.drops.place(asset, target, this.uploads.measured[asset.name] ?? null);
	}

	/** Sends files dropped on the timeline; each is drawn as a ghost at `target` and placed once stored. */
	dropFiles(files: File[], target: DropTarget): void {
		this.uploads.add(files, target);
	}

	/** The clips drawn before they exist: dropped files still uploading or waiting for the probe. */
	get ghosts(): Ghost[] {
		return [...this.uploads.ghosts, ...this.drops.ghosts];
	}

	/** Adds the starter scene after the one under the playhead. */
	insert(starter: Starter): void {
		const timeline = this.engine.timeline;
		const at = timeline.scenes.length === 0 ? 0 : locate(timeline, this.transport.time).scene + 1;
		void this.engine.commit([{ op: 'add_scene', at, duration: starter.duration, body: starter.body }]);
	}

	private stored({ id, asset, local, target }: Stored): void {
		if (target) this.drops.landed(asset, local, target, id);
		void this.loadAssets();
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

const REASONS: Record<string, AppKey> = { overlap: 'timeline.refused.overlap', out_of_bounds: 'timeline.refused.bounds' };

function refusal(why: Refusal, error: unknown): string {
	const reason = error instanceof ApiError ? REASONS[error.reason] : undefined;
	if (reason) return t(reason);
	const detail = why === 'failed' && error instanceof Error && error.message ? `: ${error.message}` : '';
	return `${t(`timeline.${why}`)}${detail}`;
}
