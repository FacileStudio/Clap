import { describe, expect, it } from 'vitest';
import { outline } from '$lib/timeline/outline';
import { readStarter, STARTERS } from './starters';

describe('starters', () => {
	it('reads the length and the body of a starter file', () => {
		const starter = readStarter('x', 'starter.title', '---\nlayout: raw\nduration: 2.5\n---\n<h1 data-start="0">Hi</h1>\n');
		expect([starter.duration, starter.body]).toEqual([2.5, '<h1 data-start="0">Hi</h1>']);
	});

	it.each(STARTERS)('$id is a raw body with clips that fit its length and no scene separator', (starter) => {
		expect(starter.body.startsWith('<')).toBe(true);
		expect(starter.body).not.toMatch(/^---\s*$/m);
		const [scene] = outline(`---\nlayout: raw\nduration: ${starter.duration}\n---\n${starter.body}\n`).scenes;
		expect(scene.clips.length).toBeGreaterThan(0);
		for (const clip of scene.clips) expect(clip.start + clip.duration).toBeLessThanOrEqual(starter.duration);
	});
});
