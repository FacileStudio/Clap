import { clamp } from './model';

/** The zoom range, in pixels per second. */
export const MIN_ZOOM = 4;
export const MAX_ZOOM = 600;

/** The x of project time `seconds` at `zoom` pixels per second. */
export const toPx = (seconds: number, zoom: number): number => seconds * zoom;

/** The project time under x `px` at `zoom` pixels per second. */
export const toTime = (px: number, zoom: number): number => px / zoom;

const STEPS = [1 / 30, 0.1, 0.25, 0.5, 1, 2, 5, 10, 15, 30, 60, 120, 300, 600];

/** The spacing in seconds between two labelled ruler ticks: the smallest step at least `gap` pixels wide. */
export function tickStep(zoom: number, gap = 80): number {
	return STEPS.find((step) => step * zoom >= gap) ?? STEPS[STEPS.length - 1];
}

/** The labelled ruler ticks between the times `from` and `to`. */
export function ticks(zoom: number, from: number, to: number): number[] {
	const step = tickStep(zoom);
	const found: number[] = [];
	for (let index = Math.floor(from / step); index * step <= to; index += 1) found.push(index * step);
	return found;
}

/** `zoom` scaled by `factor`, kept inside the zoom range. */
export function zoomBy(zoom: number, factor: number): number {
	return clamp(zoom * factor, MIN_ZOOM, MAX_ZOOM);
}

/** The scroll offset that keeps the time under `anchor` (pixels from the viewport's left) still when zoom changes. */
export function scrollFor(anchor: number, scroll: number, before: number, after: number): number {
	return Math.max(0, toPx(toTime(scroll + anchor, before), after) - anchor);
}

/** The zoom that shows `duration` seconds in `width` pixels. */
export function fitZoom(duration: number, width: number): number {
	return clamp(width / Math.max(duration, 0.001), MIN_ZOOM, MAX_ZOOM);
}
