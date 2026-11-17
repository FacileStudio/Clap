import { findClips } from '../src/lib/timeline/clips';
import { copyName } from '../src/lib/timeline/copies';
import { outline, type ClipSpan } from '../src/lib/timeline/outline';
import type { Op } from '../src/lib/types';
import { restage } from './mock-scenes';
import { clipOf, retag } from './mock-tags';

type Of<K extends Op['op']> = Extract<Op, { op: K }>;

function split(source: string, op: Of<'split_clip'>): string {
	const clip = clipOf(source, op.clip);
	const head = op.at - clip.start;
	const media = clip.kind === 'video' || clip.kind === 'audio' ? clip.media_in + head : undefined;
	const element = source.slice(clip.open[0], clip.end);
	const tail = retag(element, { ...clip, open: [0, clip.open[1] - clip.open[0]] }, {
		'data-start': op.at,
		'data-duration': clip.duration - head,
		'data-in': media
	});
	const first = retag(source, clip, { 'data-duration': head });
	const at = clip.end + (first.length - source.length);
	return `${first.slice(0, at)}\n${tail}${first.slice(at)}`;
}

function element(source: string, op: Of<'add_clip'>): string {
	const plan = outline(source);
	const taken = new Set(plan.scenes.flatMap((scene) => scene.clips.map((clip) => clip.id)));
	let n = 1;
	while (taken.has(`${op.kind}-${n}`)) n += 1;
	const id = `data-clip="${op.kind}-${n}"`;
	const time = `${id} data-start="${op.start}" data-duration="${op.duration}" data-track-index="${op.track}"`;
	const frame = `data-x="0" data-y="0" data-w="${plan.width}" data-h="${plan.height}" data-rot="0" data-fit="cover"`;
	if (op.kind === 'video') return `<video class="clap-fill" src="${op.src}" ${time} ${frame}></video>`;
	if (op.kind === 'audio') return `<audio src="${op.src}" ${time}></audio>`;
	if (op.kind === 'image') return `<img class="clap-fill" src="${op.src}" alt="" ${time} ${frame}>`;
	if (op.kind === 'html') return `<div class="clap-layer" ${time}>${op.text ?? ''}</div>`;
	return `<p class="clap-title" ${time}>${op.text ?? ''}</p>`;
}

const taken = (source: string) => new Set(outline(source).scenes.flatMap((scene) => scene.clips.map((clip) => clip.id)));

const duplicateClip = (source: string, op: Of<'duplicate_clip'>): string => {
	const clip = clipOf(source, op.clip);
	const scene = op.scene ?? outline(source).scenes.findIndex((one) => one.clips.some((known) => known.id === op.clip));
	const tag = { ...clip, open: [0, clip.open[1] - clip.open[0]] as [number, number] };
	const attrs = { 'data-clip': copyName(taken(source), op.clip), 'data-start': op.start, 'data-track-index': op.track };
	const copy = retag(source.slice(clip.open[0], clip.end), tag, attrs);
	return restage(source, (scenes) => void scenes[scene].body.push(copy, ''));
};

const duplicateScene = (source: string, op: Of<'duplicate_scene'>): string => {
	const names = taken(source);
	const renamed = outline(source).scenes[op.scene].clips.map((clip) => {
		const name = copyName(names, clip.id);
		names.add(name);
		const tag = source.slice(clip.open[0], clip.open[1]);
		return [tag, retag(tag, { ...clip, open: [0, tag.length] }, { 'data-clip': name })];
	});
	return restage(source, (scenes) => {
		const body = renamed.reduce((text, [tag, next]) => text.replace(tag, next), scenes[op.scene].body.join('\n'));
		scenes.splice(op.scene + 1, 0, { meta: { ...scenes[op.scene].meta }, body: body.split('\n') });
	});
};

const pasteClip = (source: string, op: Of<'paste_clip'>): string => {
	const element = op.element.trim();
	const [clip] = findClips(element);
	const attrs = { 'data-clip': copyName(taken(source), clip.named ?? 'clip'), 'data-start': op.start, 'data-track-index': op.track };
	return restage(source, (scenes) => void scenes[op.scene].body.push(retag(element, { open: clip.open } as ClipSpan, attrs), ''));
};

const setText = (source: string, op: Of<'set_clip_text'>): string => {
	const clip = clipOf(source, op.clip);
	const close = source.lastIndexOf('</', clip.end);
	const words = op.text.trim().replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
	return `${source.slice(0, clip.open[1])}${words}${source.slice(close)}`;
};

const HANDLERS: { [K in Op['op']]: (source: string, op: Of<K>) => string } = {
	move_clip: (source, op) => retag(source, clipOf(source, op.clip), { 'data-start': op.start, 'data-track-index': op.track }),
	trim_clip: (source, op) =>
		retag(source, clipOf(source, op.clip), { 'data-start': op.start, 'data-duration': op.duration, 'data-in': op.media_in }),
	split_clip: split,
	duplicate_clip: duplicateClip,
	duplicate_scene: duplicateScene,
	paste_clip: pasteClip,
	set_clip_text: setText,
	delete_clip: (source, op) => {
		const clip = clipOf(source, op.clip);
		return source.slice(0, clip.open[0]) + source.slice(clip.end).replace(/^\n/, '');
	},
	update_clip: (source, { op: _, clip, ...update }) =>
		retag(source, clipOf(source, clip), Object.fromEntries(Object.entries(update).map(([key, value]) => [`data-${key.replace('_', '-')}`, value]))),
	set_scene_transition: (source, op) =>
		restage(source, (scenes) => {
			const meta = scenes[op.scene].meta;
			if (op.transition === null) delete meta.transition;
			else meta.transition = op.transition;
			if (op.transition === null || op.duration === null) delete meta.transition_duration;
			else if (op.duration !== undefined) meta.transition_duration = String(op.duration);
		}),
	add_clip: (source, op) => restage(source, (scenes) => void scenes[op.scene].body.push(element(source, op), '')),
	set_scene_duration: (source, op) => restage(source, (scenes) => void (scenes[op.scene].meta.duration = String(op.duration))),
	reorder_scene: (source, op) => restage(source, (scenes) => void scenes.splice(op.to, 0, ...scenes.splice(op.scene, 1))),
	delete_scene: (source, op) => restage(source, (scenes) => void scenes.splice(op.scene, 1)),
	add_scene: (source, op) =>
		restage(source, (scenes) => {
			const meta = { layout: 'raw', duration: String(op.duration) };
			scenes.splice(op.at, 0, { meta, body: ['', ...op.body.split('\n'), ''] });
		})
};

/** `source` after `ops`, the way the mock API patches the text. Throws when an op names a clip that is not there. */
export function applyEdits(source: string, ops: Op[]): string {
	return ops.reduce((text, op) => (HANDLERS[op.op] as (source: string, op: Op) => string)(text, op), source);
}
