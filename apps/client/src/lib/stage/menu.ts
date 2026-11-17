import type { MenuEntry } from '@facile/muse';
import { icons } from '$lib/icons';
import { t } from '$lib/i18n';
import { GAP, commandEntry, editEntry, type MenuHost } from '$lib/timeline/clip-menu';
import { laneOps } from '$lib/timeline/clip-ops';
import { findClip } from '$lib/timeline/model';
import type { Fit, Op } from '$lib/types';
import type { StageBox } from '$lib/project/stage';
import type { Box, Stage } from './geometry';
import { boxOps } from './inspect';
import { centredBox, fullBox } from './place';

/** The ops that put clip `id` on the whole frame, a picture or a video drawn the `fit` way. */
function frameOps(id: string, stage: Stage, fit: Fit, media: boolean): Op[] {
	return [{ op: 'update_clip', clip: id, ...fullBox(stage), ...(media ? { fit } : {}) }];
}

/**
 * The menu of clip `id` on the stage, drawn in `box`: its stacking (the track the runtime stacks by), its place
 * in the frame, and the clip commands. The clip becomes the selection unless it is already part of it.
 */
export function stageMenu(host: MenuHost, id: string, box: Box, stage: Stage): MenuEntry[] {
	const engine = host.bench.engine;
	const clip = findClip(engine.timeline, id)?.clip;
	if (!clip) return [];
	if (!engine.selection.has(id)) engine.select(id);
	const media = clip.kind === 'video' || clip.kind === 'image';
	const reset: Op[] | null = clip.frame ? [{ op: 'update_clip', clip: id, x: null, y: null, w: null, h: null, rot: null }] : null;
	return [
		editEntry(host, t('menu.forward'), laneOps(engine.timeline, id, 1), icons.chevronUp),
		editEntry(host, t('menu.backward'), laneOps(engine.timeline, id, -1), icons.chevronDown),
		GAP,
		editEntry(host, t('menu.fill'), frameOps(id, stage, 'cover', media), icons.fit),
		...(media ? [editEntry(host, t('menu.fit'), frameOps(id, stage, 'contain', media))] : []),
		editEntry(host, t('menu.centre'), boxOps(clip, centredBox(box, stage)), icons.centre),
		editEntry(host, t('menu.reset'), reset, icons.refresh),
		GAP,
		commandEntry(host, t('menu.copy'), 'copy', { icon: icons.copy }),
		commandEntry(host, t('menu.duplicate'), 'duplicate', { icon: icons.layers, disabled: engine.busy }),
		commandEntry(host, t('menu.delete'), 'delete', { icon: icons.remove, tone: 'danger', disabled: engine.busy })
	];
}

/** The menu for a right-click on `target` over the stage: the clip box under it, else the selected one. */
export function stageMenuAt(host: MenuHost, target: Element, drawn: StageBox[], chosen: string | null, stage: Stage): MenuEntry[] {
	const id = target.closest<HTMLElement>('[data-testid="stage-box"]')?.dataset.clip ?? chosen;
	const box = drawn.find((one) => one.id === id);
	return box ? stageMenu(host, box.id, box, stage) : [];
}
