import type { Clip, Op, Scene } from '$lib/types';
import { FPS } from '$lib/project/timecode';
import { moveBox, startFloor, trimEnd, trimStart } from './box';
import { trimOps } from './edits';

type Key = Pick<KeyboardEvent, 'key' | 'shiftKey' | 'altKey'>;

const SIDE: Record<string, number> = { ArrowLeft: -1, ArrowRight: 1 };
const LIFT: Record<string, number> = { ArrowUp: 1, ArrowDown: -1 };

/**
 * The ops a key makes on the focused clip, or null when the key is not one of them. Left and right move it one
 * frame (one second with Shift); with Alt they trim its end, with Alt and Shift its start. Up and down change its track.
 */
export function nudgeOps(clip: Clip, scene: Scene, event: Key, fps = FPS): Op[] | null {
	const rows = LIFT[event.key];
	if (rows && !event.altKey) {
		return [{ op: 'move_clip', clip: clip.id, start: clip.start, track: Math.max(0, clip.track + rows) }];
	}
	const side = SIDE[event.key];
	if (!side) return null;
	if (event.altKey && event.shiftKey) return trimOps(clip, trimStart(clip, side / fps, startFloor(clip)));
	if (event.altKey) return trimOps(clip, trimEnd(clip, scene.duration, side / fps));
	const box = moveBox(clip, scene.duration, event.shiftKey ? side : side / fps);
	return box.start === clip.start ? [] : [{ op: 'move_clip', clip: clip.id, start: box.start }];
}
