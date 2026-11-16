import { describe, expect, it } from 'vitest';
import { bounds, centre, normalAngle, sameBox, tidy, turn } from './geometry';
import { anchored, nudgeBy, resizeBox, rotateBox } from './gesture';
import { centredBox, dropBox, fullBox } from './place';
import { snapLines, snapMove, snapResize } from './snap';

const STAGE = { width: 1920, height: 1080 };
const box = { x: 100, y: 100, w: 200, h: 100, rot: 0 };
const near = (value: number, wanted: number) => expect(value).toBeCloseTo(wanted, 6);

describe('geometry', () => {
	it('turns a vector clockwise with y down and bounds a turned box', () => {
		const turned = turn({ x: 1, y: 0 }, 90);
		near(turned.x, 0);
		near(turned.y, 1);
		const outer = bounds({ ...box, rot: 90 });
		near(outer.w, 100);
		near(outer.h, 200);
		expect(centre(outer)).toEqual(centre(box));
	});

	it('normalises angles and tidies a box to whole pixels', () => {
		expect(normalAngle(270)).toBe(-90);
		expect(normalAngle(-180)).toBe(180);
		expect(tidy({ x: 1.4, y: -0.4, w: 0.2, h: 10.6, rot: 12.34 })).toEqual({ x: 1, y: 0, w: 1, h: 11, rot: 12.3 });
		expect(sameBox(box, { ...box, x: 100.2 })).toBe(true);
	});
});

describe('place', () => {
	it('fills and centres on the stage', () => {
		expect(fullBox(STAGE)).toEqual({ x: 0, y: 0, w: 1920, h: 1080, rot: 0 });
		expect(centredBox(box, STAGE)).toMatchObject({ x: 860, y: 490 });
	});

	it('drops a picture centred on the point, its ratio kept, at most half the stage', () => {
		const big = dropBox(STAGE, { x: 960, y: 540 }, { width: 4000, height: 1000 });
		expect(big).toEqual({ x: 480, y: 420, w: 960, h: 240, rot: 0 });
		const small = dropBox(STAGE, { x: 100, y: 100 }, { width: 64, height: 36 });
		expect(small).toEqual({ x: 68, y: 82, w: 64, h: 36, rot: 0 });
		const unknown = dropBox(STAGE, { x: 960, y: 540 }, null);
		expect(unknown).toMatchObject({ w: 960, h: 540 });
	});
});

describe('gesture', () => {
	it('resizes from a corner with the opposite corner pinned, ratio kept on demand', () => {
		expect(resizeBox(box, 'se', 100, 0, false)).toEqual({ ...box, w: 300 });
		expect(resizeBox(box, 'se', 100, 0, true)).toEqual({ ...box, w: 300, h: 150 });
		expect(resizeBox(box, 'nw', 50, 50, false)).toEqual({ x: 150, y: 150, w: 150, h: 50, rot: 0 });
		expect(resizeBox(box, 'e', -500, 0, false).w).toBe(8);
	});

	it('pins the far corner of a turned box on the stage', () => {
		const turned = { ...box, rot: 30 };
		const far = (one: typeof box) => {
			const corner = turn({ x: -one.w / 2, y: -one.h / 2 }, one.rot);
			return { x: centre(one).x + corner.x, y: centre(one).y + corner.y };
		};
		const grown = anchored(turned, 'se', 400, 300);
		near(far(grown).x, far(turned).x);
		near(far(grown).y, far(turned).y);
	});

	it('rotates around the centre, sticking to right angles unless free', () => {
		const middle = centre(box);
		const from = { x: middle.x + 100, y: middle.y };
		expect(rotateBox(box, from, { x: middle.x, y: middle.y + 100 }, { free: false, steps: false }).rot).toBe(90);
		const tilt = { x: middle.x + 100, y: middle.y + 5 };
		expect(rotateBox(box, from, tilt, { free: false, steps: false }).rot).toBe(0);
		expect(rotateBox(box, from, tilt, { free: true, steps: false }).rot).toBeCloseTo(2.86, 1);
		expect(rotateBox(box, from, { x: middle.x + 100, y: middle.y + 40 }, { free: true, steps: true }).rot).toBe(15);
	});

	it('nudges one pixel, ten with Shift', () => {
		expect(nudgeBy({ key: 'ArrowLeft', shiftKey: false })).toEqual({ x: -1, y: 0 });
		expect(nudgeBy({ key: 'ArrowDown', shiftKey: true })).toEqual({ x: 0, y: 10 });
		expect(nudgeBy({ key: 'a', shiftKey: false })).toBeNull();
	});
});

describe('snap', () => {
	const lines = snapLines(STAGE, [{ x: 1000, y: 700, w: 100, h: 100, rot: 0 }]);

	it('lists the stage edges, centre, thirds and the other clips', () => {
		expect(lines.x).toEqual([0, 640, 960, 1280, 1920, 1000, 1050, 1100]);
	});

	it('moves a box onto the nearest line within reach, with its guide', () => {
		const snapped = snapMove({ ...box, x: 857, y: 400 }, lines, 6);
		expect(snapped.box).toMatchObject({ x: 860, y: 400 });
		expect(snapped.guides).toEqual({ x: [960], y: [] });
		expect(snapMove({ ...box, x: 300 }, lines, 6).guides).toEqual({ x: [], y: [] });
	});

	it('sticks a resized edge, and keeps the ratio by the closer edge', () => {
		const free = snapResize({ start: box, handle: 'e', keep: false }, { ...box, w: 537 }, lines, 6);
		expect(free.box).toMatchObject({ x: 100, w: 540 });
		const kept = snapResize({ start: box, handle: 'se', keep: true }, { ...box, w: 538, h: 269 }, lines, 6);
		expect(kept.box).toMatchObject({ w: 540, h: 270 });
		expect(kept.guides.x).toEqual([640]);
	});
});
