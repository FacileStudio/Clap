import { kindOfType } from '$lib/project/media-facts';
import type { Asset, ClipKind } from '$lib/types';
import { HEAD, ROW, RULER } from './layout';
import { clamp } from './model';

let carried: Asset | null = null;

/**
 * Remembers the media bin item being dragged. A dragover cannot read the drag's data, only its types, so the
 * timeline asks here which asset it is aiming before the drop.
 */
export function carry(asset: Asset | null): void {
	carried = asset;
}

/** The media bin item being dragged, or null. */
export function carriedAsset(): Asset | null {
	return carried;
}

/** Whether a drag brings files from outside the page. */
export function bringsFiles(data: DataTransfer | null | undefined): boolean {
	return data?.types.includes('Files') ?? false;
}

/** The clip kind of the first file a drag brings, read from its type during dragover; null when none is media. */
export function broughtKind(data: DataTransfer | null | undefined): ClipKind | null {
	const items = [...(data?.items ?? [])].filter((item) => item.kind === 'file');
	return items.map((item) => kindOfType(item.type)).find((kind) => kind !== null) ?? null;
}

/** The dropped files the timeline can place, in their dropped order. */
export function placeableFiles(data: DataTransfer | null | undefined): File[] {
	return [...(data?.files ?? [])].filter((file) => kindOfType(file.type) !== null);
}

/** The track under the pointer at viewport height `y` over the timeline's `area`, whose rows hold `rows`. */
export function trackAt(y: number, area: HTMLElement | undefined, rows: readonly number[]): number {
	const inside = y - (area?.getBoundingClientRect().top ?? 0) - RULER - HEAD;
	return rows[clamp(Math.floor(inside / ROW), 0, rows.length - 1)] ?? 0;
}
