import { dropOps, kindOfAsset } from '$lib/timeline/adds';
import { stageDropOps } from '$lib/timeline/stage-drop';
import type { Engine } from '$lib/timeline/engine.svelte';
import type { DropTarget, Ghost } from '$lib/timeline/ghost';
import type { Asset, Op } from '$lib/types';
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

	/** Places `asset` at `target`, with what the browser read of it standing in for what the server has not probed. */
	place(asset: Asset, target: DropTarget, local: LocalMedia | null = null): void {
		this.queue = this.queue.then(async () => {
			while (this.engine.busy) await new Promise((wake) => setTimeout(wake, IDLE_POLL_MS));
			const ops = this.opsFor(asset, target, local);
			if (ops.length > 0) await this.engine.commit(ops);
		});
	}

	/** An upload carrying `target` was stored: placed now when its length is known, else when the probe ends. */
	landed(asset: Asset, local: LocalMedia, target: DropTarget, id: string): void {
		const kind = kindOfAsset(asset);
		if (!kind) return;
		if (kind === 'image' || local.duration !== null || asset.duration_ms) return this.place(asset, target, local);
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

	private opsFor(asset: Asset, target: DropTarget, local: LocalMedia | null): Op[] {
		const kind = kindOfAsset(asset);
		if (!target.at) return dropOps(this.engine.timeline, asset, target.time, target.track, local?.duration);
		if (!kind) return [];
		const duration = asset.duration_ms ? asset.duration_ms / 1000 : (local?.duration ?? null);
		const [width, height] = [asset.width ?? local?.width ?? null, asset.height ?? local?.height ?? null];
		const media = { kind, src: asset.name, duration, width, height };
		return stageDropOps(this.engine.timeline, media, target.time, target.at);
	}
}
