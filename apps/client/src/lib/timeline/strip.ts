import type { Asset } from '$lib/types';

/** How a video's filmstrip is cut: `tiles` frames side by side, frame `i` taken at `i * step` seconds. */
export interface Strip {
	url: string;
	tiles: number;
	step: number;
	/** The width over the height of one frame. */
	aspect: number;
}

/** One frame drawn in a clip: which tile of the strip, where it sits in the clip, and the media time it starts at. */
export interface Tile {
	index: number;
	left: number;
	width: number;
	at: number;
}

/** The most frames one clip draws, however long it is at the zoom. */
export const MAX_TILES = 240;

/** The filmstrip of `asset`, when the server has made one. */
export function stripOf(asset: Asset | undefined): Strip | null {
	const { strip_url: url, strip_tiles: tiles, strip_step: step } = asset ?? {};
	if (!url || !tiles || !step || tiles < 1 || step <= 0) return null;
	const aspect = asset?.width && asset.height ? asset.width / asset.height : 16 / 9;
	return { url, tiles, step, aspect };
}

/**
 * The frames a clip `duration` seconds long that starts `mediaIn` seconds into its video draws at `zoom` pixels per
 * second, in a block `height` pixels high: one slot per frame width, each showing the tile of the media time at its
 * left edge, so a trimmed clip starts on the frame it plays first.
 */
export function stripTiles(strip: Strip, mediaIn: number, duration: number, zoom: number, height: number): Tile[] {
	const width = Math.max(8, height * strip.aspect);
	const count = Math.min(MAX_TILES, Math.ceil((Math.max(0, duration) * zoom) / width));
	return Array.from({ length: count }, (_, slot) => {
		const at = Math.max(0, mediaIn) + (slot * width) / zoom;
		const index = Math.min(strip.tiles - 1, Math.max(0, Math.floor(at / strip.step + 1e-9)));
		return { index, left: slot * width, width, at: Math.round(at * 1000) / 1000 };
	});
}

/** The CSS that shows tile `index` of `strip` and nothing else, at any drawn size. */
export function tileStyle(strip: Strip, tile: Tile): string {
	const x = strip.tiles > 1 ? (tile.index / (strip.tiles - 1)) * 100 : 0;
	const url = strip.url.replace(/'/g, '%27');
	return `left: ${tile.left}px; width: ${tile.width}px; background-image: url('${url}'); background-size: ${strip.tiles * 100}% 100%; background-position: ${x}% 0`;
}

/**
 * The SVG path of the part of a waveform a clip plays, in a box `count` wide and 2 high, centred on y = 1: one bar per
 * value from `mediaIn` to `mediaIn + duration` of a sound `length` seconds long, scaled to the loudest value of the
 * whole sound.
 */
export function wavePath(peaks: readonly number[], length: number, mediaIn: number, duration: number): { d: string; count: number } {
	if (peaks.length === 0 || length <= 0) return { d: '', count: 0 };
	const from = Math.max(0, Math.floor((mediaIn / length) * peaks.length));
	const to = Math.min(peaks.length, Math.ceil(((mediaIn + duration) / length) * peaks.length));
	const loudest = Math.max(...peaks, 1e-6);
	const bars = peaks.slice(from, Math.max(from, to)).map((value, x) => {
		const half = Math.max(0.02, Math.min(1, value / loudest));
		return `M${x} ${round(1 - half)}h1V${round(1 + half)}h-1Z`;
	});
	return { d: bars.join(''), count: bars.length };
}

const round = (value: number): number => Math.round(value * 1000) / 1000;
