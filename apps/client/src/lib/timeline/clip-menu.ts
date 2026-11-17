import type { MenuAction, MenuEntry } from '@facile/muse';
import { icons } from '$lib/icons';
import { t } from '$lib/i18n';
import type { Markers } from '$lib/project/markers.svelte';
import type { Op } from '$lib/types';
import { fadeOps, laneOps, trimToOps } from './clip-ops';
import type { Bench } from './commands';
import type { DropTarget } from './ghost';
import { SHORTCUTS, type Command } from './keys';
import { findClip } from './model';

/** What the timeline's menus act through: the bench, its commands, and the page around the timeline. */
export interface MenuHost {
	readonly bench: Bench;
	run(command: Command): void;
	/** Shows the file `src` in the media bin. */
	reveal(src: string): void;
	/** Whether the project has the file `src`. */
	has(src: string): boolean;
	/** Asks for files to place at `target`. */
	addMedia(target: DropTarget): void;
	/** The ruler markers. */
	readonly markers: Markers;
	/** Asks for a new name for marker `id`. */
	renameMarker(id: string): void;
}

/** A separator between groups of menu entries. */
export const GAP: MenuEntry = { separator: true };

/** The entry that commits `ops`, disabled when there are none or while an edit runs. */
export function editEntry(host: MenuHost, label: string, ops: Op[] | null, icon?: string): MenuAction {
	const engine = host.bench.engine;
	const disabled = !ops || ops.length === 0 || engine.busy;
	return { label, icon, disabled, onselect: () => void (ops && engine.commit(ops)) };
}

/** The entry that runs `command`, its shortcut shown at the end of the row. */
export function commandEntry(host: MenuHost, label: string, command: Command, extra: Partial<MenuAction> = {}): MenuAction {
	return { label, shortcut: SHORTCUTS[command], onselect: () => host.run(command), ...extra };
}

/** The menu of clip `id`, which becomes the selection unless it already is part of it. */
export function clipMenu(host: MenuHost, id: string): MenuEntry[] {
	const { engine, transport, copied } = host.bench;
	const timeline = engine.timeline;
	const clip = findClip(timeline, id)?.clip;
	if (!clip) return [];
	if (!engine.selection.has(id)) engine.select(id);
	const start = trimToOps(timeline, id, transport.time, 'start');
	const faded = Boolean(clip.enter || clip.exit);
	const src = clip.src;
	const busy = engine.busy;
	return [
		commandEntry(host, t('menu.split'), 'split', { icon: icons.split, disabled: !start || busy }),
		editEntry(host, t('menu.trim.start'), start),
		editEntry(host, t('menu.trim.end'), trimToOps(timeline, id, transport.time, 'end')),
		GAP,
		commandEntry(host, t('menu.cut'), 'cut', { disabled: busy }),
		commandEntry(host, t('menu.copy'), 'copy', { icon: icons.copy }),
		commandEntry(host, t('menu.paste'), 'paste', { disabled: !copied || busy }),
		commandEntry(host, t('menu.duplicate'), 'duplicate', { icon: icons.layers, disabled: busy }),
		GAP,
		editEntry(host, t('menu.track.up'), laneOps(timeline, id, 1), icons.chevronUp),
		editEntry(host, t('menu.track.down'), laneOps(timeline, id, -1), icons.chevronDown),
		editEntry(host, t(faded ? 'menu.fade.clear' : 'menu.fade'), fadeOps(id, faded), icons.transition),
		...(src && host.has(src) ? [{ label: t('menu.reveal'), icon: icons.folder, onselect: () => host.reveal(src) }] : []),
		GAP,
		commandEntry(host, t('menu.delete'), 'delete', { icon: icons.remove, tone: 'danger', disabled: busy }),
		commandEntry(host, t('menu.ripple'), 'ripple', { tone: 'danger', disabled: busy })
	];
}
