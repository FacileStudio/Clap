import { dropOps, kindOfAsset } from '$lib/timeline/adds';
import type { Engine } from '$lib/timeline/engine.svelte';
import type { DropTarget, Ghost } from '$lib/timeline/ghost';
import type { Asset } from '$lib/types';
import type { LocalMedia } from './local-media';

const IDLE_POLL_MS = 50;

/**
 * Places dropped media on the timeline, one drop after the other. A file whose length neither the browser nor the
 * server knows yet waits, drawn as a ghost, until the server's probe answers.
 */
export class Drops {
	waiting = $state<{ name: string; ghost: Ghost }[]>([]);
	private queue: Promise<unknown> = Promise.resolve();

	constructor(private readonly engine: Engine) {}

	/** The ghosts of the files that wait for the server's probe. */
	get ghosts(): Ghost[] {
		return this.waiting.map((one) => one.ghost);
	}

	/** Places `asset` at `target`, with `duration` (seconds, measured locally) standing in for the probed one. */
	place(asset: Asset, target: DropTarget, duration: number | null = null): void {
		this.queue = this.queue.then(async () => {
			while (this.engine.busy) await new Promise((wake) => setTimeout(wake, IDLE_POLL_MS));
			const ops = dropOps(this.engine.timeline, asset, target.time, target.track, duration);
			if (ops.length > 0) await this.engine.commit(ops);
		});
	}

	/** An upload carrying `target` was stored: placed now when its length is known, else when the probe ends. */
	landed(asset: Asset, local: LocalMedia, target: DropTarget, id: string): void {
		const kind = kindOfAsset(asset);
		if (!kind) return;
		if (kind === 'image' || local.duration !== null || asset.duration_ms) return this.place(asset, target, local.duration);
		const ghost = { id, kind, label: asset.name, duration: null, poster: local.poster, progress: null, target };
		this.waiting = [...this.waiting, { name: asset.name, ghost }];
	}

	/** The asset list changed: a waiting file whose probe ended is placed, one that failed is forgotten. */
	heard(assets: Asset[]): void {
		const ended = (name: string) => assets.find((asset) => asset.name === name && asset.state !== 'probing');
		for (const one of this.waiting) {
			const asset = ended(one.name);
			if (asset?.state === 'ready') this.place(asset, one.ghost.target);
		}
		this.waiting = this.waiting.filter((one) => !ended(one.name));
	}
}
