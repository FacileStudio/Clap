import type { LocalMedia } from '$lib/project/local-media';
import type { Asset, ClipKind, Timeline } from '$lib/types';
import { kindOfAsset, placeOps } from './adds';
import { sceneStarts } from './model';

/** Where a drop asked to land: a project time and a track. */
export interface DropTarget {
	time: number;
	track: number;
}

/** A clip drawn on the timeline before it exists: a drop being aimed, or a file still uploading. */
export interface Ghost {
	id: string;
	kind: ClipKind;
	label: string;
	/** Seconds, or null when nobody knows the length yet. */
	duration: number | null;
	poster: string | null;
	/** The sent fraction (0 to 1) while uploading, or null. */
	progress: number | null;
	target: DropTarget;
}

/** Where a ghost's clip would really land, as the drop's ops place it: project start, length and track. */
export interface GhostBox {
	start: number;
	duration: number;
	track: number;
}

/** The box `ghost` takes on `timeline`: the first free track at or above the aimed one, as `placeOps` picks it. */
export function ghostBox(timeline: Timeline, ghost: Ghost): GhostBox | null {
	const media = { kind: ghost.kind, src: '', duration: ghost.duration };
	const add = placeOps(timeline, media, ghost.target.time, ghost.target.track).at(-1);
	if (add?.op !== 'add_clip') return null;
	return { start: (sceneStarts(timeline)[add.scene] ?? 0) + add.start, duration: add.duration, track: add.track };
}

/** The row a track is drawn on; a track with no row yet borrows the row of the next track down. */
export function ghostRow(rows: readonly number[], track: number): number {
	const row = rows.findIndex((one) => one <= track);
	return row >= 0 ? row : Math.max(0, rows.length - 1);
}

/**
 * The ghost of a drag being aimed at `target`: the media bin's `asset` when one is carried, else a desktop file of
 * `brought` kind, whose length and poster stay unknown until it is dropped.
 */
export function aimedGhost(
	asset: Asset | null,
	brought: ClipKind | null,
	measured: Readonly<Record<string, LocalMedia>>,
	target: DropTarget
): Ghost | null {
	const kind = asset ? kindOfAsset(asset) : brought;
	if (!kind) return null;
	const local = asset ? measured[asset.name] : undefined;
	const duration = asset?.duration_ms ? asset.duration_ms / 1000 : (local?.duration ?? null);
	const label = asset?.name.split('/').pop() ?? '';
	return { id: 'aimed', kind, label, duration, poster: asset?.poster_url ?? local?.poster ?? null, progress: null, target };
}
