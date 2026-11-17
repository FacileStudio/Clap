import { ApiError, backend } from '$lib/backend';
import { t } from '$lib/i18n';
import type { AppKey } from '$lib/i18n/messages';
import { Engine, type Refusal } from '$lib/timeline/engine.svelte';
import type { DropTarget, Ghost } from '$lib/timeline/ghost';
import { locate, trackRows } from '$lib/timeline/model';
import type { Asset, Project } from '$lib/types';
import { renamedAsset } from './asset-name';
import { Drops } from './drops.svelte';
import { watchProject } from './events';
import { Markers } from './markers.svelte';
import { Revision } from './revision.svelte';
import type { Starter } from './starters';
import { Transport } from './transport.svelte';
import { Uploads, type Stored } from './uploads.svelte';

/**
 * One open project: its revision, its timeline, its clock and its media, kept in step. Every edit goes through the
 * timeline engine as ops; the text itself stays with the CLI and the agents.
 */
export class Desk {
	readonly id: string;
	readonly transport = new Transport();
	readonly revision: Revision;
	readonly engine: Engine;
	readonly uploads: Uploads;
	readonly drops: Drops;
	readonly markers: Markers;
	title = $state('');
	assets = $state<Asset[]>([]);
	/** The clip the person copied last, pasted by id while it exists. */
	copied = $state<string | null>(null);
	private late = 0;

	constructor(
		project: Project,
		private readonly fail: (message: string) => void
	) {
		this.id = project.id;
		this.title = project.title;
		this.uploads = new Uploads(this.id, (stored) => this.stored(stored));
		this.revision = new Revision({
			get: () => backend.project(this.id),
			onreplace: (source) => void this.engine.sync(source)
		});
		this.revision.open(project.rev);
		this.engine = new Engine({
			rev: () => this.revision.rev,
			edit: (rev, ops) => backend.edit(this.id, rev, ops),
			put: (source, rev) => backend.save(this.id, source, rev),
			timeline: () => backend.timeline(this.id),
			adopt: (_, rev) => this.revision.open(rev),
			reload: () => this.revision.reload(),
			refused: (why, error) => this.fail(refusal(why, error))
		});
		this.drops = new Drops(this.engine);
		this.markers = new Markers(this.id, (error) => this.fail(error instanceof Error ? error.message : t('error.generic')));
		void this.engine.sync(project.source);
		void this.loadAssets();
		void this.markers.load();
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

	/**
	 * Renames `asset` to the name `typed` asks for, its references in the source with it, then shows the stored
	 * project. A stale revision is read again and the rename tried once more. Answers whether a rename was sent.
	 */
	async renameAsset(asset: Asset, typed: string): Promise<boolean> {
		const next = renamedAsset(asset.name, typed);
		if (!next || next === asset.name) return false;
		const send = () => backend.renameAsset(this.id, asset.name, next, this.revision.rev);
		await send().catch(async (error: unknown) => {
			if (!(error instanceof ApiError) || error.status !== 409) throw error;
			await this.revision.reload();
			return send();
		});
		await Promise.all([this.revision.reload(), this.loadAssets()]);
		return true;
	}

	/** Places `asset` at the playhead, on the lowest track that is free there. */
	place(asset: Asset): void {
		this.drop(asset, { time: this.transport.time, track: 0 });
	}

	/** Places `asset` at the playhead, on a new track above every other. */
	placeAbove(asset: Asset): void {
		this.drop(asset, { time: this.transport.time, track: trackRows(this.engine.timeline)[0] ?? 0 });
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

	private heard(rev: number): void {
		void this.revision.remote(rev).catch((error: unknown) => this.fail(error instanceof Error ? error.message : t('error.generic')));
	}
}

const REASONS: Record<string, AppKey> = { overlap: 'timeline.refused.overlap', out_of_bounds: 'timeline.refused.bounds' };

function refusal(why: Refusal, error: unknown): string {
	const reason = error instanceof ApiError ? REASONS[error.reason] : undefined;
	if (reason) return t(reason);
	const detail = why === 'failed' && error instanceof Error && error.message ? `: ${error.message}` : '';
	return `${t(`timeline.${why}`)}${detail}`;
}
