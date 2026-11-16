/** What a drag reports: each move past the dead zone, then its end. */
export interface DragHandlers {
	/** The pointer is `dx`, `dy` pixels from where it went down. */
	move(dx: number, dy: number, event: PointerEvent): void;
	/** The pointer went up. `moved` is false for a plain click. */
	end(moved: boolean, event: PointerEvent): void;
}

const DEAD_ZONE = 3;

/** Follows the pointer from `event` with pointer capture on the element that took it, until it goes up. */
export function drag(event: PointerEvent, handlers: DragHandlers): void {
	const target = event.currentTarget as HTMLElement;
	const origin = { x: event.clientX, y: event.clientY };
	let moved = false;
	const onmove = (next: PointerEvent) => {
		const [dx, dy] = [next.clientX - origin.x, next.clientY - origin.y];
		if (!moved && Math.hypot(dx, dy) < DEAD_ZONE) return;
		moved = true;
		handlers.move(dx, dy, next);
	};
	const onup = (last: PointerEvent) => {
		target.removeEventListener('pointermove', onmove);
		target.removeEventListener('pointerup', onup);
		target.removeEventListener('pointercancel', onup);
		if (target.hasPointerCapture(last.pointerId)) target.releasePointerCapture(last.pointerId);
		handlers.end(moved, last);
	};
	target.setPointerCapture(event.pointerId);
	target.addEventListener('pointermove', onmove);
	target.addEventListener('pointerup', onup);
	target.addEventListener('pointercancel', onup);
}
