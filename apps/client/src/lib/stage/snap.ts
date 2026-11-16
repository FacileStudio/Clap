import { bounds, MIN_SIDE, type Box, type Rect, type Stage } from './geometry';
import { anchored, isCorner, type Handle } from './gesture';

/** The lines a box may snap to, in frame pixels: upright ones (`x`) and level ones (`y`). */
export interface Lines {
	x: number[];
	y: number[];
}

/** A snapped box and the guides to draw where it landed. */
export interface Snapped {
	box: Box;
	guides: Lines;
}

/** A resize under way: the box it started from, the handle held, and whether a corner keeps the ratio. */
export interface Grip {
	start: Box;
	handle: Handle;
	keep: boolean;
}

/** How close, in screen pixels, an edge must come to a line to stick to it. */
export const SNAP_PX = 6;

const sides = (rect: Rect, axis: 'x' | 'y'): number[] =>
	axis === 'x' ? [rect.x, rect.x + rect.w / 2, rect.x + rect.w] : [rect.y, rect.y + rect.h / 2, rect.y + rect.h];

/** The stage's edges, centre and thirds, then the edges and centres of `others`. */
export function snapLines(stage: Stage, others: readonly Box[]): Lines {
	const x = [0, stage.width / 3, stage.width / 2, (2 * stage.width) / 3, stage.width];
	const y = [0, stage.height / 3, stage.height / 2, (2 * stage.height) / 3, stage.height];
	for (const other of others.map(bounds)) {
		x.push(...sides(other, 'x'));
		y.push(...sides(other, 'y'));
	}
	return { x, y };
}

/** The shift that brings the closest of `mine` onto the closest of `targets` within `reach`, and the line hit. */
function nearest(mine: readonly number[], targets: readonly number[], reach: number): { shift: number; line: number } | null {
	let best: { shift: number; line: number } | null = null;
	for (const line of targets) {
		for (const own of mine) {
			const shift = line - own;
			if (Math.abs(shift) <= reach && (!best || Math.abs(shift) < Math.abs(best.shift))) best = { shift, line };
		}
	}
	return best;
}

/** `box` moved so its outer edges or centre stick to the closest lines within `reach` frame pixels, per axis. */
export function snapMove(box: Box, lines: Lines, reach: number): Snapped {
	const outer = bounds(box);
	const x = nearest(sides(outer, 'x'), lines.x, reach);
	const y = nearest(sides(outer, 'y'), lines.y, reach);
	const moved = { ...box, x: box.x + (x?.shift ?? 0), y: box.y + (y?.shift ?? 0) };
	return { box: moved, guides: { x: x ? [x.line] : [], y: y ? [y.line] : [] } };
}

/**
 * `box`, resized from `start` by `handle`, with its moving edges stuck to the closest lines within `reach`. A
 * turned box does not snap. A corner that keeps the ratio follows the one edge that came closest.
 */
export function snapResize(grip: Grip, box: Box, lines: Lines, reach: number): Snapped {
	const { start, handle, keep } = grip;
	if (box.rot !== 0) return { box, guides: { x: [], y: [] } };
	const right = handle.includes('e');
	const bottom = handle.includes('s');
	const x = right || handle.includes('w') ? nearest([right ? box.x + box.w : box.x], lines.x, reach) : null;
	const y = bottom || handle.includes('n') ? nearest([bottom ? box.y + box.h : box.y], lines.y, reach) : null;
	let w = box.w + (x ? (right ? x.shift : -x.shift) : 0);
	let h = box.h + (y ? (bottom ? y.shift : -y.shift) : 0);
	const byX = x !== null && (y === null || Math.abs(x.shift) <= Math.abs(y.shift));
	const ratio = keep && isCorner(handle);
	if (ratio && (x || y)) [w, h] = byX ? [w, (w * start.h) / start.w] : [(h * start.w) / start.h, h];
	const guides = { x: x && (!ratio || byX) ? [x.line] : [], y: y && (!ratio || !byX) ? [y.line] : [] };
	return { box: anchored(start, handle, Math.max(MIN_SIDE, w), Math.max(MIN_SIDE, h)), guides };
}
