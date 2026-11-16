import caption from '$starters/caption.md?raw';
import chart from '$starters/chart-race.md?raw';
import logo from '$starters/logo-reveal.md?raw';
import lower from '$starters/lower-third.md?raw';
import title from '$starters/title.md?raw';
import type { AppKey } from '$lib/i18n/messages';

/** A ready-made raw scene of the insert menu: what `add_scene` takes. */
export interface Starter {
	id: string;
	label: AppKey;
	duration: number;
	body: string;
}

/** Reads one file of `starters/`: its frontmatter gives the length, everything after it is the body. */
export function readStarter(id: string, label: AppKey, file: string): Starter {
	const found = /^---\r?\n([\s\S]*?)\r?\n---\r?\n([\s\S]*)$/.exec(file);
	const duration = Number(/^duration:\s*([\d.]+)/m.exec(found?.[1] ?? '')?.[1]);
	return { id, label, duration: duration > 0 ? duration : 4, body: (found?.[2] ?? file).trim() };
}

/** The motion starters of the insert menu, in menu order: the files of `starters/` at the repo root. */
export const STARTERS: readonly Starter[] = [
	readStarter('title', 'starter.title', title),
	readStarter('lower-third', 'starter.lower', lower),
	readStarter('caption', 'starter.caption', caption),
	readStarter('chart-race', 'starter.chart', chart),
	readStarter('logo-reveal', 'starter.logo', logo)
];
