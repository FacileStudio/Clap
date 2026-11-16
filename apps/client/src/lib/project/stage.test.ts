import { describe, expect, it } from 'vitest';
import { stageBoxes, stageMessage } from './stage';

const box = { id: 'image-1', kind: 'image', x: 100, y: 100, w: 200, h: 200, rot: 0 };

describe('stageBoxes', () => {
	it('reads the boxes a frame posts after a seek', () => {
		expect(stageBoxes({ type: 'clap:boxes', t: 1.5, clips: [box] })).toEqual({ t: 1.5, clips: [box] });
		expect(stageBoxes({ type: 'clap:boxes', t: 0, clips: [] })).toEqual({ t: 0, clips: [] });
	});

	it('drops anything a hostile frame could send', () => {
		const bad = [
			null,
			'clap:boxes',
			{ type: 'clap:boxes', t: Number.NaN, clips: [] },
			{ type: 'clap:boxes', t: 0, clips: {} },
			{ type: 'clap:boxes', t: 0, clips: [{ ...box, x: '1' }] },
			{ type: 'clap:boxes', t: 0, clips: [{ ...box, kind: 'audio' }] },
			{ type: 'clap:boxes', t: 0, clips: [{ ...box, w: -1 }] },
			{ type: 'clap:boxes', t: 0, clips: [{ ...box, id: '' }] },
			{ type: 'clap:boxes', t: 0, clips: [{ ...box, id: 'x'.repeat(201) }] },
			{ type: 'clap:boxes', t: 0, clips: Array.from({ length: 513 }, () => box) },
			{ type: 'scene:state', time: 0, step: 0, scene: 0 }
		];
		for (const data of bad) expect(stageBoxes(data)).toBeNull();
	});

	it('keeps scene messages apart from boxes', () => {
		expect(stageMessage({ type: 'clap:boxes', t: 0, clips: [] })).toBeNull();
	});
});
