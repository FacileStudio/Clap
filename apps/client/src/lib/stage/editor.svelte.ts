import type { StageBox } from '$lib/project/stage';
import { drag } from '$lib/timeline/drag';
import type { Engine } from '$lib/timeline/engine.svelte';
import { findClip } from '$lib/timeline/model';
import { sameBox, tidy, type Box, type Point, type Stage } from './geometry';
import { moveBox, nudgeBy, resizeBox, rotateBox, type Handle } from './gesture';
import { SNAP_PX, snapLines, snapMove, snapResize, type Lines } from './snap';

/** What the stage editor reads from the preview and tells it back. */
export interface StageHost {
	engine: Engine;
	stage(): Stage;
	/** The clip boxes the preview drew last. */
	boxes(): StageBox[];
	/** The frame point under a pointer. */
	point(event: { clientX: number; clientY: number }): Point;
	/** Screen pixels per frame pixel. */
	scale(): number;
	locked(): boolean;
	/** Draws `ghost` in the preview while a gesture runs; null puts every clip back. */
	ghost(ghost: (Box & { id: string }) | null): void;
	/** A gesture ended: `sent` is true when its edit landed, so the ghost stays until the new preview shows. */
	landed(sent: boolean): void;
}

const NO_GUIDES: Lines = { x: [], y: [] };
const IDLE_POLL_MS = 30;
const MEDIA = new Set(['video', 'image']);

/**
 * Moves, resizes, turns and nudges clips on the stage. A gesture shows live through the preview's ghost and ends
 * as one `update_clip`, so one undo puts the clip back.
 */
export class StageEditor {
	live = $state.raw<{ id: string; box: Box } | null>(null);
	guides = $state.raw<Lines>(NO_GUIDES);
	private nudging: { start: StageBox; box: Box } | null = null;

	constructor(private readonly host: StageHost) {}

	/** The boxes to draw, the one under a gesture at its live place. */
	get drawn(): StageBox[] {
		const live = this.live;
		return this.host.boxes().map((one) => (live && one.id === live.id ? { ...one, ...live.box } : one));
	}

	/** The pointer went down on `box`: selects it, then drags it. */
	grab(event: PointerEvent, box: StageBox): void {
		if (event.button !== 0) return;
		event.stopPropagation();
		const { engine } = this.host;
		if (event.shiftKey) engine.select(box.id, true);
		else if (!engine.selection.has(box.id)) engine.select(box.id);
		if (this.host.locked() || event.shiftKey) return;
		const lines = this.lines(box.id);
		this.follow(event, box, (dx, dy, next) => {
			const moved = moveBox(box, dx, dy);
			return this.snaps(next) ? snapMove(moved, lines, SNAP_PX / this.host.scale()) : { box: moved, guides: NO_GUIDES };
		});
	}

	/** The pointer went down on `handle` of `box`: resizes it. Media keeps its ratio unless Shift is held. */
	resize(event: PointerEvent, box: StageBox, handle: Handle): void {
		if (event.button !== 0 || this.host.locked()) return;
		event.stopPropagation();
		const lines = this.lines(box.id);
		this.follow(event, box, (dx, dy, next) => {
			const keep = MEDIA.has(box.kind) !== next.shiftKey;
			const sized = resizeBox(box, handle, dx, dy, keep);
			const reach = SNAP_PX / this.host.scale();
			return this.snaps(next) ? snapResize({ start: box, handle, keep }, sized, lines, reach) : { box: sized, guides: NO_GUIDES };
		});
	}

	/** The pointer went down on the turn handle of `box`: turns it around its centre. Shift steps by 15 degrees. */
	rotate(event: PointerEvent, box: StageBox): void {
		if (event.button !== 0 || this.host.locked()) return;
		event.stopPropagation();
		const from = this.host.point(event);
		this.follow(event, box, (_, __, next) => {
			const options = { free: !this.snaps(next), steps: next.shiftKey };
			return { box: rotateBox(box, from, this.host.point(next), options), guides: NO_GUIDES };
		});
	}

	/** An arrow key nudges the one selected clip; answers whether the key was used. */
	key(event: KeyboardEvent): boolean {
		const step = nudgeBy(event);
		const [id, more] = [...this.host.engine.selection];
		const box = this.host.boxes().find((one) => one.id === id);
		if (!step || !box || more || this.host.locked()) return false;
		const from = this.nudging?.start.id === id ? this.nudging : { start: box, box };
		this.nudging = { start: from.start, box: moveBox(from.box, step.x, step.y) };
		this.show(id, this.nudging.box, NO_GUIDES);
		return true;
	}

	/** An arrow key went up: the nudges made so far become one edit. */
	keyup(event: KeyboardEvent): void {
		const start = nudgeBy(event) ? this.nudging?.start : null;
		if (!start) return;
		this.nudging = null;
		void this.land(start);
	}

	private follow(event: PointerEvent, box: StageBox, shape: (dx: number, dy: number, next: PointerEvent) => { box: Box; guides: Lines }) {
		drag(event, {
			move: (dx, dy, next) => {
				const scale = this.host.scale();
				const found = shape(dx / scale, dy / scale, next);
				this.show(box.id, found.box, found.guides);
			},
			end: (moved) => void (moved && this.land(box))
		});
	}

	private show(id: string, box: Box, guides: Lines): void {
		this.live = { id, box };
		this.guides = guides;
		this.host.ghost({ id, ...box });
	}

	private async land(start: StageBox): Promise<void> {
		const box = this.live?.id === start.id ? tidy(this.live.box) : null;
		this.guides = NO_GUIDES;
		if (!box || sameBox(start, box) || !findClip(this.host.engine.timeline, start.id)) return this.drop(false);
		while (this.host.engine.busy) await new Promise((wake) => setTimeout(wake, IDLE_POLL_MS));
		const sent = await this.host.engine.commit([{ op: 'update_clip', clip: start.id, ...box }]);
		this.drop(sent);
	}

	private drop(sent: boolean): void {
		this.live = null;
		this.host.landed(sent);
	}

	private lines(id: string): Lines {
		return snapLines(this.host.stage(), this.host.boxes().filter((one) => one.id !== id));
	}

	private snaps(event: PointerEvent): boolean {
		return !event.altKey;
	}
}
