import { round } from '$lib/timeline/model';
import type { Clip, ClipTransition, ClipUpdate, Fit, Op, SceneTransition } from '$lib/types';
import { tidy, type Box } from './geometry';

/** The numbers the clip inspector edits. Opacity and volume are shown in percent. */
export type Field = 'x' | 'y' | 'w' | 'h' | 'rot' | 'opacity' | 'volume' | 'start' | 'duration' | 'enter_duration' | 'exit_duration';

/** The clip enter and exit presets, in the order the picker lists them. */
export const CLIP_TRANSITIONS: readonly ClipTransition[] = [
	'fade',
	'slide-left',
	'slide-right',
	'slide-up',
	'slide-down',
	'zoom-in',
	'zoom-out',
	'wipe-left',
	'wipe-right',
	'blur',
	'pop'
];

/** The scene transitions, in the order the picker lists them. */
export const SCENE_TRANSITIONS: readonly SceneTransition[] = ['fade', 'dip', 'slide-left', 'slide-right', 'wipe-left', 'wipe-right', 'zoom'];

/** How a video or a picture fills its box. */
export const FITS: readonly Fit[] = ['cover', 'contain', 'fill'];

/** The length a preset runs when the clip names none, in seconds. */
export const CLIP_TRANSITION_SECONDS = 0.5;

/** The length a scene transition runs when the scene names none, in seconds. */
export const SCENE_TRANSITION_SECONDS = 0.6;

const BOX_FIELDS = new Set<Field>(['x', 'y', 'w', 'h', 'rot']);
const update = (clip: Clip, change: ClipUpdate): Op[] => [{ op: 'update_clip', clip: clip.id, ...change }];

/** The `update_clip` that writes `box` whole, as the source keeps it. */
export function boxOps(clip: Clip, box: Box): Op[] {
	return update(clip, tidy(box));
}

/**
 * The ops that set `field` of `clip` to the typed `text`, its box read from `box`; null when the text is not a
 * value the field takes. An empty preset length removes it.
 */
export function fieldOps(clip: Clip, box: Box, field: Field, text: string): Op[] | null {
	const value = Number(text.trim().replace(',', '.'));
	const typed = text.trim() !== '' && Number.isFinite(value);
	if ((field === 'enter_duration' || field === 'exit_duration') && text.trim() === '') return update(clip, { [field]: null });
	if (!typed) return null;
	if (BOX_FIELDS.has(field)) return (field === 'w' || field === 'h') && value <= 0 ? null : boxOps(clip, { ...box, [field]: value });
	if (field === 'opacity' || field === 'volume') return update(clip, { [field]: Math.min(1, Math.max(0, value / 100)) });
	if (field === 'start') return value < 0 ? null : [{ op: 'move_clip', clip: clip.id, start: round(value) }];
	if (value <= 0) return null;
	if (field === 'duration') return [{ op: 'trim_clip', clip: clip.id, duration: round(value) }];
	return update(clip, { [field]: round(value) });
}

/** The value `field` shows for `clip` whose box is `box`, as the inspector writes it. */
export function fieldValue(clip: Clip, box: Box, field: Field): string {
	if (BOX_FIELDS.has(field)) return String(tidy(box)[field as keyof Box]);
	if (field === 'opacity' || field === 'volume') return String(Math.round((clip[field] ?? 1) * 100));
	const value = clip[field as 'start' | 'duration' | 'enter_duration' | 'exit_duration'];
	return value === null || value === undefined ? '' : String(value);
}
