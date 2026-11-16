import { tidy, type Box, type Point, type Stage } from './geometry';

/** The whole stage as a box. */
export function fullBox(stage: Stage): Box {
	return { x: 0, y: 0, w: stage.width, h: stage.height, rot: 0 };
}

/** `box` moved so its centre is the stage's. */
export function centredBox(box: Box, stage: Stage): Box {
	return { ...box, x: (stage.width - box.w) / 2, y: (stage.height - box.h) / 2 };
}

/** The largest `aspect` (width over height) rectangle that fits in `room`, scaled by `share`. */
export function containSize(aspect: number, room: Stage, share = 1): { w: number; h: number } {
	const ratio = aspect > 0 && Number.isFinite(aspect) ? aspect : room.width / room.height;
	const w = Math.min(room.width, room.height * ratio) * share;
	return { w, h: w / ratio };
}

const DROP_SHARE = 0.5;

/**
 * The box a picture of `natural` size takes when dropped at `at`: its own size, but no more than half the stage
 * fitted at its ratio, centred on the drop point. An unknown size takes the stage ratio.
 */
export function dropBox(stage: Stage, at: Point, natural: { width: number | null; height: number | null } | null): Box {
	const [width, height] = [natural?.width ?? 0, natural?.height ?? 0];
	const aspect = width > 0 && height > 0 ? width / height : stage.width / stage.height;
	const room = containSize(aspect, stage, DROP_SHARE);
	const w = width > 0 && height > 0 ? Math.min(width, room.w) : room.w;
	const h = w / aspect;
	return tidy({ x: at.x - w / 2, y: at.y - h / 2, w, h, rot: 0 });
}
