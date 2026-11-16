import { outline } from '../src/lib/timeline/outline';
import { blocks } from '../src/lib/timeline/scenes';
import type { Clip, Scene, Timeline } from '../src/lib/types';

const LOOK = ['fit', 'opacity', 'volume', 'enter', 'exit', 'enter_duration', 'exit_duration'] as const;

function attr(tag: string, name: string): string | null {
	return new RegExp(`\\s${name}="([^"]*)"`).exec(tag)?.[1] ?? null;
}

function look(tag: string): Partial<Clip> {
	const number = (name: string) => (attr(tag, `data-${name}`) === null ? null : Number(attr(tag, `data-${name}`)));
	const box = ['x', 'y', 'w', 'h', 'rot'].map(number);
	const [x, y, w, h, rot] = box.map((value) => value ?? 0);
	const found: Record<string, unknown> = { frame: box.some((value) => value !== null) ? { x, y, w, h, rot } : null };
	for (const key of LOOK) {
		const text = attr(tag, `data-${key.replace('_', '-')}`);
		found[key] = text === null ? null : /opacity|volume|duration/.test(key) ? Number(text) : text;
	}
	return found as Partial<Clip>;
}

/** The timeline the real API serves for `source`: the outline plus each clip's box and look and each scene's transition. */
export function servedTimeline(source: string): Timeline {
	const plan = outline(source);
	const metas = blocks(source.split('\n')).map((block) => block.meta);
	const scenes: Scene[] = plan.scenes.map((scene, index) => ({
		index: scene.index,
		duration: scene.duration,
		layout: scene.layout,
		transition: (metas[index]?.transition ?? null) as Scene['transition'],
		transition_duration: metas[index]?.transition_duration ? Number(metas[index].transition_duration) : null,
		clips: scene.clips.map((clip) => ({
			...{ id: clip.id, kind: clip.kind, src: clip.src, start: clip.start, duration: clip.duration },
			...{ track: clip.track, media_in: clip.media_in },
			...look(source.slice(clip.open[0], clip.open[1]))
		}))
	}));
	return { width: plan.width, height: plan.height, scenes };
}
