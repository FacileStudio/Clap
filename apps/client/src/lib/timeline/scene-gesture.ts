import type { DragHandlers } from './drag';
import { reorderTarget } from './edits';
import { offset, type GestureHost } from './host';
import { MIN_CLIP } from './box';
import { round, sceneStarts } from './model';
import { SNAP_PX, snap, snapTargets } from './snap';

function shift(host: GestureHost, from: number, px: number): void {
	for (let index = from; index < host.timeline().scenes.length; index += 1) offset(host.lane(index), { dx: px });
}

/** The drag of the right edge of scene `index`: it changes the scene's length, and the scenes after it follow. */
export function resizeGesture(host: GestureHost, index: number): DragHandlers {
	const scene = host.timeline().scenes[index];
	const end = (sceneStarts(host.timeline())[index] ?? 0) + scene.duration;
	const targets = snapTargets(host.timeline(), host.playhead(), new Set()).filter((target) => target !== end);
	let duration = scene.duration;
	return {
		move(dx, _, event) {
			const snapped = snap(end + dx / host.zoom(), targets, host.snaps(event) ? SNAP_PX / host.zoom() : 0);
			duration = round(Math.max(MIN_CLIP, scene.duration + snapped.value - end));
			const px = (duration - scene.duration) * host.zoom();
			offset(host.lane(index), { dw: px });
			shift(host, index + 1, px);
		},
		end() {
			offset(host.lane(index), { dw: 0 });
			shift(host, index + 1, 0);
			if (duration !== scene.duration) host.commit([{ op: 'set_scene_duration', scene: index, duration }]);
		}
	};
}

/** The drag of the header of scene `index`: it moves the scene among the others. */
export function reorderGesture(host: GestureHost, index: number): DragHandlers {
	const timeline = host.timeline();
	const starts = sceneStarts(timeline);
	const middle = starts[index] + timeline.scenes[index].duration / 2;
	let to = index;
	return {
		move(dx) {
			to = reorderTarget(timeline, index, middle + dx / host.zoom());
			const landing = to > index ? starts[to] + timeline.scenes[to].duration : starts[to];
			offset(host.lane(index), { dx });
			host.lane(index)?.style.setProperty('z-index', '5');
			host.guide(to === index ? null : landing);
		},
		end() {
			offset(host.lane(index), { dx: 0 });
			host.lane(index)?.style.removeProperty('z-index');
			host.guide(null);
			if (to !== index) host.commit([{ op: 'reorder_scene', scene: index, to }]);
		}
	};
}
