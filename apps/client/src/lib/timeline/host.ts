import type { Op, Timeline } from '$lib/types';

/** What a gesture reads from the timeline view and writes back to it, without going through a render. */
export interface GestureHost {
	timeline(): Timeline;
	selection(): ReadonlySet<string>;
	zoom(): number;
	playhead(): number;
	/** Whether edges snap: the toolbar toggle, and Alt not held. */
	snaps(event: PointerEvent): boolean;
	/** The element drawn for clip `id`. */
	clip(id: string): HTMLElement | undefined;
	/** The element drawn for scene `index`. */
	lane(index: number): HTMLElement | undefined;
	/** Shows the snap guide at project time `time`, or hides it with null. */
	guide(time: number | null): void;
	commit(ops: Op[]): void;
}

/** Sets the pixel offsets a gesture previews on `node`; a missing or zero offset is removed. */
export function offset(node: HTMLElement | undefined, offsets: Record<string, number>): void {
	for (const [name, value] of Object.entries(offsets)) {
		if (value) node?.style.setProperty(`--${name}`, `${value}px`);
		else node?.style.removeProperty(`--${name}`);
	}
}
