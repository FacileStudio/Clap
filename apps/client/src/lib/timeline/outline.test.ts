import { describe, expect, it } from 'vitest';
import { outline } from './outline';

const SOURCE = `---
title: Demo
size: 1280x720
layout: raw
duration: 6
---

<video src="assets/a.mp4" data-start="1" data-duration="4" data-track-index="0" data-in="0.5"></video>
<h1 id="t" data-start="0.5" data-duration="2" data-track-index="2">
  Launch day
</h1>
<script>const not = '<p data-start="9">no</p>';</script>

---
layout: raw
---

<img src="assets/logo.png" data-start="0" data-duration="3">
<div data-start="0" data-duration="3"></div>
<audio src="assets/music.mp3" data-start="1"></audio>

\`\`\`md
---
\`\`\`

---

## Plain

- One {step=1}
`;

describe('outline', () => {
	const found = outline(SOURCE);

	it('splits scenes on separators, not inside a fence, and reads the stage size', () => {
		expect([found.width, found.height, found.scenes.length]).toEqual([1280, 720, 3]);
		expect(found.scenes.map((scene) => [scene.from, scene.to, scene.layout])).toEqual([
			[1, 13, 'raw'],
			[14, 25, 'raw'],
			[26, 31, 'default']
		]);
	});

	it('gives each scene its length: duration, then the last clip end, then the steps', () => {
		expect(found.scenes.map((scene) => [scene.duration, scene.explicit])).toEqual([
			[6, true],
			[3, false],
			[8, false]
		]);
	});

	it('finds the clips with their kind, track, lines and label', () => {
		const [video, title] = found.scenes[0].clips;
		expect(video).toMatchObject({ id: 'clip-0-0', kind: 'video', src: 'assets/a.mp4', start: 1, duration: 4, from: 8, to: 8 });
		expect(video.media_in).toBe(0.5);
		expect(title).toMatchObject({ id: 't', kind: 'text', track: 2, from: 9, to: 11, label: 'Launch day' });
		expect(SOURCE.slice(title.open[0], title.end)).toMatch(/^<h1 id="t"[\s\S]*<\/h1>$/);
		expect(found.scenes[0].clips).toHaveLength(2);
	});

	it('runs a clip without data-duration to the end of its scene', () => {
		const [image, fill, audio] = found.scenes[1].clips;
		expect([image.kind, fill.kind, audio.kind, audio.start, audio.duration]).toEqual(['image', 'html', 'audio', 1, 2]);
	});

	it('reads a file with no headmatter as one default scene', () => {
		expect(outline('# Hello\n')).toMatchObject({ width: 1920, scenes: [{ from: 1, layout: 'default', duration: 4 }] });
	});
});
