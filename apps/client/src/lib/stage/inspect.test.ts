import { describe, expect, it } from 'vitest';
import type { Clip } from '$lib/types';
import { fieldOps, fieldValue } from './inspect';

const clip: Clip = { id: 'image-1', kind: 'image', src: 'a.png', start: 1, duration: 4, track: 0, media_in: 0, opacity: null };
const box = { x: 10, y: 20, w: 300, h: 200, rot: 0 };

describe('fieldOps', () => {
	it('writes the whole box when one side changes', () => {
		expect(fieldOps(clip, box, 'x', '110.4')).toEqual([{ op: 'update_clip', clip: 'image-1', x: 110, y: 20, w: 300, h: 200, rot: 0 }]);
		expect(fieldOps(clip, box, 'w', '0')).toBeNull();
	});

	it('reads opacity and volume in percent, clamped', () => {
		expect(fieldOps(clip, box, 'opacity', '50')).toEqual([{ op: 'update_clip', clip: 'image-1', opacity: 0.5 }]);
		expect(fieldOps(clip, box, 'volume', '150')).toEqual([{ op: 'update_clip', clip: 'image-1', volume: 1 }]);
		expect(fieldValue(clip, box, 'opacity')).toBe('100');
	});

	it('moves and trims through the timeline ops, and clears an empty preset length', () => {
		expect(fieldOps(clip, box, 'start', '2,5')).toEqual([{ op: 'move_clip', clip: 'image-1', start: 2.5 }]);
		expect(fieldOps(clip, box, 'duration', '3')).toEqual([{ op: 'trim_clip', clip: 'image-1', duration: 3 }]);
		expect(fieldOps(clip, box, 'enter_duration', '')).toEqual([{ op: 'update_clip', clip: 'image-1', enter_duration: null }]);
		expect(fieldOps(clip, box, 'start', 'abc')).toBeNull();
		expect(fieldValue(clip, box, 'enter_duration')).toBe('');
	});
});
