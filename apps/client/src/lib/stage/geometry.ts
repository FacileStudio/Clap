/** A point or a vector in frame pixels. */
export interface Point {
	x: number;
	y: number;
}

/** A box on the stage in frame pixels from the top left, turned `rot` degrees clockwise around its centre. */
export interface Box {
	x: number;
	y: number;
	w: number;
	h: number;
	rot: number;
}

/** An unturned rectangle in frame pixels. */
export type Rect = Omit<Box, 'rot'>;

/** The stage size in frame pixels. */
export interface Stage {
	width: number;
	height: number;
}

/** The smallest side a box may shrink to, in frame pixels. */
export const MIN_SIDE = 8;

const RADIANS = Math.PI / 180;

/** The centre of `box`. */
export function centre(box: Rect): Point {
	return { x: box.x + box.w / 2, y: box.y + box.h / 2 };
}

/** `vector` turned `degrees` clockwise (screen axes, y down). */
export function turn(vector: Point, degrees: number): Point {
	const [cos, sin] = [Math.cos(degrees * RADIANS), Math.sin(degrees * RADIANS)];
	return { x: vector.x * cos - vector.y * sin, y: vector.x * sin + vector.y * cos };
}

/** The unturned rectangle that holds `box` once turned. */
export function bounds(box: Box): Rect {
	const middle = centre(box);
	const corners = [
		{ x: -box.w / 2, y: -box.h / 2 },
		{ x: box.w / 2, y: -box.h / 2 },
		{ x: box.w / 2, y: box.h / 2 },
		{ x: -box.w / 2, y: box.h / 2 }
	].map((corner) => turn(corner, box.rot));
	const xs = corners.map((corner) => corner.x + middle.x);
	const ys = corners.map((corner) => corner.y + middle.y);
	return { x: Math.min(...xs), y: Math.min(...ys), w: Math.max(...xs) - Math.min(...xs), h: Math.max(...ys) - Math.min(...ys) };
}

/** An angle in degrees brought into (-180, 180]. */
export function normalAngle(degrees: number): number {
	const turned = ((((degrees + 180) % 360) + 360) % 360) - 180;
	return turned === -180 ? 180 : turned;
}

/** `box` with whole pixels and a tenth of a degree, as the source keeps it. Sides stay at least one pixel. */
export function tidy(box: Box): Box {
	const whole = (value: number) => Math.round(value) || 0;
	return {
		x: whole(box.x),
		y: whole(box.y),
		w: Math.max(1, whole(box.w)),
		h: Math.max(1, whole(box.h)),
		rot: Math.round(normalAngle(box.rot) * 10) / 10 || 0
	};
}

/** Whether two boxes are the same once tidied. */
export function sameBox(a: Box, b: Box): boolean {
	const [one, two] = [tidy(a), tidy(b)];
	return one.x === two.x && one.y === two.y && one.w === two.w && one.h === two.h && one.rot === two.rot;
}
