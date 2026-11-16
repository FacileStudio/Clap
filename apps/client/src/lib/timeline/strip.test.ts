import { describe, expect, it } from 'vitest';
import type { Asset } from '$lib/types';
import { readPeaks } from './peaks';
import { MAX_TILES, stripOf, stripTiles, tileStyle, wavePath } from './strip';

const video: Asset = {
	name: 'assets/a.mp4',
	kind: 'video',
	size: 1,
	duration_ms: 8000,
	width: 320,
	height: 180,
	state: 'ready',
	poster_url: null,
	url: '/api/projects/p/assets/assets/a.mp4',
	strip_url: '/api/projects/p/assets/assets/a.mp4?variant=strip',
	strip_tiles: 4,
	strip_step: 2
};

describe('filmstrip', () => {
	it('reads the strip of an asset only when the server made one', () => {
		expect(stripOf(video)).toEqual({ url: video.strip_url, tiles: 4, step: 2, aspect: 16 / 9 });
		expect(stripOf({ ...video, strip_url: null })).toBeNull();
		expect(stripOf({ ...video, strip_tiles: 0 })).toBeNull();
		expect(stripOf(undefined)).toBeNull();
	});

	it('starts a trimmed clip on the tile of its media_in and follows the media time slot by slot', () => {
		const strip = { url: 'u', tiles: 4, step: 2, aspect: 2 };
		const tiles = stripTiles(strip, 4.5, 4, 40, 30);
		expect(tiles.map((tile) => [tile.index, tile.left, tile.at])).toEqual([
			[2, 0, 4.5],
			[3, 60, 6],
			[3, 120, 7.5]
		]);
		expect(stripTiles(strip, 0, 2, 40, 30)[0]).toEqual({ index: 0, left: 0, width: 60, at: 0 });
		expect(stripTiles(strip, 30, 1, 40, 30).map((tile) => tile.index)).toEqual([3]);
	});

	it('draws more tiles as the zoom grows, never past the cap', () => {
		const strip = { url: 'u', tiles: 60, step: 2, aspect: 16 / 9 };
		expect(stripTiles(strip, 0, 10, 20, 34).length).toBeLessThan(stripTiles(strip, 0, 10, 200, 34).length);
		expect(stripTiles(strip, 0, 3600, 400, 34).length).toBe(MAX_TILES);
		expect(stripTiles(strip, 0, 0, 60, 34)).toEqual([]);
	});

	it('shows exactly one tile of the sprite', () => {
		const style = tileStyle({ url: "a'b", tiles: 5, step: 2, aspect: 1 }, { index: 2, left: 10, width: 34, at: 4 });
		expect(style).toContain("url('a%27b')");
		expect(style).toContain('background-size: 500% 100%');
		expect(style).toContain('background-position: 50% 0');
	});
});

describe('waveform', () => {
	it('draws the part of the sound a trimmed clip plays, scaled to the loudest value', () => {
		const peaks = [0.1, 0.5, 0.25, 0.5, 0];
		expect(wavePath(peaks, 5, 1, 2)).toEqual({ d: 'M0 0h1V2h-1ZM1 0.5h1V1.5h-1Z', count: 2 });
		expect(wavePath(peaks, 5, 4, 3).d).toBe('M0 0.98h1V1.02h-1Z');
		expect(wavePath([], 5, 0, 1)).toEqual({ d: '', count: 0 });
	});

	it('takes only numbers from 0 to 1 out of an answer', () => {
		expect(readPeaks({ peaks: [0.5, 2, -1, 'x', null] })).toEqual([0.5, 1, 0, 0, 0]);
		expect(readPeaks({ peaks: 'no' })).toEqual([]);
		expect(readPeaks(null)).toEqual([]);
	});
});
