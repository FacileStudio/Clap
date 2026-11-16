import { centre, MIN_SIDE, normalAngle, turn, type Box, type Point } from './geometry';

/** A resize handle: the sides it moves. */
export type Handle = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w';

/** The corner handles, which keep a picture's ratio. */
export const CORNERS: readonly Handle[] = ['nw', 'ne', 'se', 'sw'];

/** The side handles, which stretch one side. */
export const SIDES: readonly Handle[] = ['n', 'e', 's', 'w'];

/** Whether `handle` is a corner. */
export const isCorner = (handle: Handle): boolean => handle.length === 2;

const side = (handle: Handle, near: string, far: string): number => (handle.includes(near) ? -1 : handle.includes(far) ? 1 : 0);

/** `start` moved by `dx`, `dy` frame pixels. */
export function moveBox(start: Box, dx: number, dy: number): Box {
	return { ...start, x: start.x + dx, y: start.y + dy };
}

/**
 * `start` resized to `w` by `h` with the side or corner opposite `handle` kept in place on the stage, whatever
 * the box's turn.
 */
export function anchored(start: Box, handle: Handle, w: number, h: number): Box {
	const [sx, sy] = [side(handle, 'w', 'e'), side(handle, 'n', 's')];
	const middle = centre(start);
	const before = turn({ x: (-sx * start.w) / 2, y: (-sy * start.h) / 2 }, start.rot);
	const anchor = { x: middle.x + before.x, y: middle.y + before.y };
	const after = turn({ x: (-sx * w) / 2, y: (-sy * h) / 2 }, start.rot);
	const next = { x: anchor.x - after.x, y: anchor.y - after.y };
	return { x: next.x - w / 2, y: next.y - h / 2, w, h, rot: start.rot };
}

/**
 * `start` with `handle` dragged by `dx`, `dy` frame pixels. The drag is read in the box's own turned axes; a
 * corner keeps the ratio when `keep` is set, and no side drops under `MIN_SIDE`.
 */
export function resizeBox(start: Box, handle: Handle, dx: number, dy: number, keep: boolean): Box {
	const local = turn({ x: dx, y: dy }, -start.rot);
	let w = Math.max(MIN_SIDE, start.w + side(handle, 'w', 'e') * local.x);
	let h = Math.max(MIN_SIDE, start.h + side(handle, 'n', 's') * local.y);
	if (keep && isCorner(handle) && start.w > 0 && start.h > 0) {
		const scale = Math.max(w / start.w, h / start.h, MIN_SIDE / Math.min(start.w, start.h));
		[w, h] = [start.w * scale, start.h * scale];
	}
	return anchored(start, handle, w, h);
}

const STEP = 15;
const MAGNET = 4;

/**
 * `start` turned by the angle the pointer swept around its centre, from `from` to `to` (frame pixels). With
 * `free` unset the turn sticks to the right angles; `steps` locks it to 15 degree steps.
 */
export function rotateBox(start: Box, from: Point, to: Point, options: { free: boolean; steps: boolean }): Box {
	const middle = centre(start);
	const angle = (point: Point) => (Math.atan2(point.y - middle.y, point.x - middle.x) * 180) / Math.PI;
	let rot = normalAngle(start.rot + angle(to) - angle(from));
	if (options.steps) rot = Math.round(rot / STEP) * STEP;
	else if (!options.free && Math.abs(rot - Math.round(rot / 90) * 90) <= MAGNET) rot = Math.round(rot / 90) * 90;
	return { ...start, rot: normalAngle(rot) };
}

const ARROWS: Record<string, Point> = {
	ArrowLeft: { x: -1, y: 0 },
	ArrowRight: { x: 1, y: 0 },
	ArrowUp: { x: 0, y: -1 },
	ArrowDown: { x: 0, y: 1 }
};

/** The move an arrow key asks for: one pixel, ten with Shift; null for any other key. */
export function nudgeBy(event: Pick<KeyboardEvent, 'key' | 'shiftKey'>): Point | null {
	const arrow = ARROWS[event.key];
	if (!arrow) return null;
	const step = event.shiftKey ? 10 : 1;
	return { x: arrow.x * step, y: arrow.y * step };
}
