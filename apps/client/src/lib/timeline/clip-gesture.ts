import type { Clip, Scene } from '$lib/types';
import { moveBox, startFloor, trimEnd, trimStart, type Box } from './box';
import type { DragHandlers } from './drag';
import { moveOps, trimOps } from './edits';
import { offset, type GestureHost } from './host';
import { ROW } from './layout';
import { clamp, findClip, sceneStarts, trackRows } from './model';
import { SNAP_PX, snap, snapSpan, snapTargets } from './snap';

/** Which part of a clip the pointer took. */
export type Grip = 'move' | 'start' | 'end';

type Held = { clip: Clip; scene: Scene; origin: number };

function held(host: GestureHost, ids: Iterable<string>): Held[] {
	const timeline = host.timeline();
	const starts = sceneStarts(timeline);
	return [...ids].flatMap((id) => {
		const found = findClip(timeline, id);
		return found ? [{ ...found, origin: starts[found.scene.index] ?? 0 }] : [];
	});
}

function lifted(rows: number[], track: number, by: number): number {
	const row = clamp(rows.indexOf(track) + by, 0, rows.length - 1);
	return rows[row] ?? track;
}

function preview(host: GestureHost, rows: number[], clip: Clip, box: Box): void {
	const zoom = host.zoom();
	offset(host.clip(clip.id), {
		dx: (box.start - clip.start) * zoom,
		dw: (box.duration - clip.duration) * zoom,
		dy: (rows.indexOf(box.track) - rows.indexOf(clip.track)) * ROW
	});
}

function mover(host: GestureHost, grabbed: Held, all: Held[]): DragHandlers {
	const rows = trackRows(host.timeline());
	const targets = snapTargets(host.timeline(), host.playhead(), new Set(all.map((one) => one.clip.id)));
	const boxes = new Map<string, Box>();
	return {
		move(dx, dy, event) {
			const at = grabbed.origin + grabbed.clip.start + dx / host.zoom();
			const tolerance = host.snaps(event) ? SNAP_PX / host.zoom() : 0;
			const snapped = snapSpan(at, grabbed.clip.duration, targets, tolerance);
			host.guide(snapped.target);
			for (const one of all) {
				const moved = moveBox(one.clip, one.scene.duration, snapped.value - grabbed.origin - grabbed.clip.start);
				const box = { ...moved, track: lifted(rows, one.clip.track, Math.round(dy / ROW)) };
				boxes.set(one.clip.id, box);
				preview(host, rows, one.clip, box);
			}
		},
		end() {
			for (const one of all) offset(host.clip(one.clip.id), { dx: 0, dw: 0, dy: 0 });
			host.guide(null);
			host.commit(moveOps(host.timeline(), boxes));
		}
	};
}

function trimmer(host: GestureHost, grip: 'start' | 'end', one: Held): DragHandlers {
	const { clip, scene, origin } = one;
	const targets = snapTargets(host.timeline(), host.playhead(), new Set([clip.id]));
	const edge = origin + clip.start + (grip === 'end' ? clip.duration : 0);
	let box: Box = clip;
	return {
		move(dx, _, event) {
			const snapped = snap(edge + dx / host.zoom(), targets, host.snaps(event) ? SNAP_PX / host.zoom() : 0);
			const dt = snapped.value - edge;
			box = grip === 'start' ? trimStart(clip, dt, startFloor(clip)) : trimEnd(clip, scene.duration, dt);
			host.guide(snapped.target);
			preview(host, [clip.track], clip, box);
		},
		end() {
			offset(host.clip(clip.id), { dx: 0, dw: 0, dy: 0 });
			host.guide(null);
			host.commit(trimOps(clip, box));
		}
	};
}

/**
 * The drag that starts on clip `id` by `grip`. Moving takes every selected clip along when `id` is one of them;
 * trimming changes `id` alone. Null when the timeline has no such clip.
 */
export function clipGesture(host: GestureHost, id: string, grip: Grip): DragHandlers | null {
	const [grabbed] = held(host, [id]);
	if (!grabbed) return null;
	if (grip !== 'move') return trimmer(host, grip, grabbed);
	return mover(host, grabbed, host.selection().has(id) ? held(host, host.selection()) : [grabbed]);
}
