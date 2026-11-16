import { outline, type ClipSpan } from '../src/lib/timeline/outline';
import type { Op } from '../src/lib/types';
import { restage } from './mock-scenes';

type Of<K extends Op['op']> = Extract<Op, { op: K }>;

type Value = number | string | null | undefined;

function setAttr(tag: string, name: string, value: Value): string {
	if (value === undefined) return tag;
	if (value === null) return tag.replace(new RegExp(`\\s${name}\\s*=\\s*"[^"]*"`), '');
	const pattern = new RegExp(`(\\s${name}\\s*=\\s*")[^"]*(")`);
	if (pattern.test(tag)) return tag.replace(pattern, `$1${value}$2`);
	return tag.replace(/(\s*\/?>)$/, ` ${name}="${value}"$1`);
}

function clipOf(source: string, id: string): ClipSpan {
	const found = outline(source).scenes.flatMap((scene) => scene.clips).find((clip) => clip.id === id);
	if (!found) throw new Error(`no clip ${id}`);
	return found;
}

function retag(source: string, clip: ClipSpan, attrs: Record<string, Value>): string {
	let tag = source.slice(clip.open[0], clip.open[1]);
	for (const [name, value] of Object.entries(attrs)) tag = setAttr(tag, name, value);
	return source.slice(0, clip.open[0]) + tag + source.slice(clip.open[1]);
}

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

const HANDLERS: { [K in Op['op']]: (source: string, op: Of<K>) => string } = {
	move_clip: (source, op) => retag(source, clipOf(source, op.clip), { 'data-start': op.start, 'data-track-index': op.track }),
	trim_clip: (source, op) =>
		retag(source, clipOf(source, op.clip), { 'data-start': op.start, 'data-duration': op.duration, 'data-in': op.media_in }),
	split_clip: split,
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
