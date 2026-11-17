import type { MenuEntry } from '@facile/muse';
import { icons } from '$lib/icons';
import { t } from '$lib/i18n';
import type { Op } from '$lib/types';
import { textOps } from './adds';
import { GAP, clipMenu, commandEntry, editEntry, type MenuHost } from './clip-menu';
import { pasteOps } from './copies';
import type { DropTarget } from './ghost';

/** The body of a blank scene: a raw scene with nothing on it but an empty note. */
export const BLANK_SCENE = '<!-- -->';
const BLANK_SECONDS = 4;

function seek(host: MenuHost, time: number): void {
	host.bench.transport.pause();
	host.bench.transport.seek(time);
}

/** The menu of the empty timeline at `at`: paste, add a text or media there, move the playhead there. */
export function laneMenu(host: MenuHost, at: DropTarget): MenuEntry[] {
	const { engine, copied } = host.bench;
	const pasted = copied ? pasteOps(engine.timeline, copied, at.time) : null;
	return [
		editEntry(host, t('menu.paste.here'), pasted),
		editEntry(host, t('menu.text.here'), textOps(engine.timeline, at.time, t('timeline.text.default')), icons.text),
		{ label: t('menu.media.here'), icon: icons.upload, disabled: engine.busy, onselect: () => host.addMedia(at) },
		GAP,
		{ label: t('menu.playhead.here'), onselect: () => seek(host, at.time) }
	];
}

/** The menu of scene `index`'s head: insert, duplicate, transition, delete. */
export function sceneMenu(host: MenuHost, index: number): MenuEntry[] {
	const timeline = host.bench.engine.timeline;
	const scene = timeline.scenes[index];
	if (!scene) return [];
	const blank = (at: number): Op[] => [{ op: 'add_scene', at, duration: BLANK_SECONDS, body: BLANK_SCENE }];
	const transition: Op[] = [{ op: 'set_scene_transition', scene: index, transition: scene.transition ? null : 'fade' }];
	const alone = timeline.scenes.length === 1;
	return [
		editEntry(host, t('menu.scene.before'), blank(index), icons.plus),
		editEntry(host, t('menu.scene.after'), blank(index + 1)),
		editEntry(host, t('menu.scene.duplicate'), [{ op: 'duplicate_scene', scene: index }], icons.layers),
		...(index > 0 ? [editEntry(host, t(scene.transition ? 'menu.scene.cut' : 'menu.scene.fade'), transition, icons.transition)] : []),
		GAP,
		{ ...editEntry(host, t('menu.scene.delete'), alone ? null : [{ op: 'delete_scene', scene: index }], icons.remove), tone: 'danger' }
	];
}

/** The menu of the ruler at project time `time`; `fit` zooms out to the whole project. */
export function rulerMenu(host: MenuHost, time: number, fit: () => void): MenuEntry[] {
	return [
		commandEntry(host, t('menu.marker.add'), 'marker', { onselect: () => void host.markers.add(time) }),
		{ label: t('menu.playhead.here'), onselect: () => seek(host, time) },
		commandEntry(host, t('menu.start'), 'home'),
		commandEntry(host, t('menu.end'), 'end'),
		GAP,
		{ label: t('timeline.zoom.fit'), icon: icons.fit, onselect: fit }
	];
}

/** The menu of the marker `id` on the ruler: move the playhead to it, rename it, delete it. */
export function markerMenu(host: MenuHost, id: string): MenuEntry[] {
	const marker = host.markers.list.find((known) => known.id === id);
	if (!marker) return [];
	return [
		{ label: t('menu.playhead.here'), onselect: () => seek(host, marker.t) },
		{ label: t('menu.marker.rename'), icon: icons.edit, onselect: () => host.renameMarker(id) },
		GAP,
		{ label: t('menu.marker.delete'), icon: icons.remove, tone: 'danger', onselect: () => void host.markers.remove(id) }
	];
}

/** The menu for a right-click on `target` in the timeline, `at` being the time and track under the pointer. */
export function timelineMenu(host: MenuHost, target: Element, at: DropTarget, fit: () => void): MenuEntry[] {
	const clip = target.closest<HTMLElement>('[data-testid="clip"]')?.dataset.clip;
	if (clip) return clipMenu(host, clip);
	const marker = target.closest<HTMLElement>('[data-testid="marker"]')?.dataset.marker;
	if (marker) return markerMenu(host, marker);
	if (target.closest('[data-testid="ruler"]')) return rulerMenu(host, at.time, fit);
	const scene = target.closest('[data-testid="scene-head"]')?.closest<HTMLElement>('[data-testid="scene"]');
	if (scene) return sceneMenu(host, Number(scene.dataset.scene));
	return laneMenu(host, at);
}
