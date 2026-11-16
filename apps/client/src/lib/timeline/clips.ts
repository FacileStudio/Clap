import type { ClipKind } from '$lib/types';

/** One element with `data-start` found in a scene body. Offsets count characters from the start of that body. */
export interface FoundClip {
	kind: ClipKind;
	src: string | null;
	start: number;
	/** Null when the element has no `data-duration`: it then stays to the end of its scene. */
	duration: number | null;
	track: number;
	media_in: number;
	/** The id the source gives it with `data-clip` or `id`, or null. */
	named: string | null;
	label: string;
	/** Where the opening tag starts and ends. */
	open: [number, number];
	/** Where the whole element ends. */
	end: number;
}

const OPEN = /<([a-zA-Z][\w-]*)\b([^<>]*?\sdata-start\s*=\s*"[^"]*"[^<>]*?)(\/?)>/g;
const VOID = new Set(['img', 'source', 'br', 'hr', 'input', 'embed', 'track']);
const MEDIA: Record<string, ClipKind> = { video: 'video', audio: 'audio', img: 'image' };

/** The value of attribute `name` in the attribute text `attrs`, or null. */
export function attribute(attrs: string, name: string): string | null {
	return new RegExp(`(?:^|\\s)${name}\\s*=\\s*"([^"]*)"`).exec(attrs)?.[1] ?? null;
}

function closeOf(text: string, tag: string, from: number): number {
	const pattern = new RegExp(`<(/?)${tag}\\b[^>]*>`, 'gi');
	pattern.lastIndex = from;
	let depth = 1;
	for (let found = pattern.exec(text); found; found = pattern.exec(text)) {
		if (found[0].endsWith('/>')) continue;
		depth += found[1] ? -1 : 1;
		if (depth === 0) return found.index + found[0].length;
	}
	return from;
}

function kindOf(tag: string, inner: string): ClipKind {
	const own = MEDIA[tag];
	if (own) return own;
	return !/<[a-zA-Z]/.test(inner) && inner.replace(/<[^>]*>/g, '').trim() ? 'text' : 'html';
}

function labelOf(src: string | null, inner: string): string {
	if (src) return src.split('/').pop() ?? src;
	return inner.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim().slice(0, 48);
}

const seconds = (raw: string | null): number => Math.max(0, Number(raw) || 0);

function describe(text: string, found: RegExpExecArray): FoundClip {
	const tag = found[1].toLowerCase();
	const openEnd = found.index + found[0].length;
	const end = VOID.has(tag) || found[3] === '/' ? openEnd : closeOf(text, tag, openEnd);
	const inner = text.slice(openEnd, end);
	const src = attribute(found[2], 'src');
	const duration = attribute(found[2], 'data-duration');
	return {
		kind: kindOf(tag, inner),
		src,
		start: seconds(attribute(found[2], 'data-start')),
		duration: duration === null ? null : seconds(duration),
		track: Math.round(Number(attribute(found[2], 'data-track-index')) || 0),
		media_in: seconds(attribute(found[2], 'data-in')),
		named: attribute(found[2], 'data-clip') ?? attribute(found[2], 'id'),
		label: labelOf(src, inner),
		open: [found.index, openEnd],
		end
	};
}

const HIDDEN = /^(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1[^\n]*$|<!--[\s\S]*?-->|<(script|style)\b[\s\S]*?<\/\2\s*>/gm;

/**
 * Every element of the scene body `text` that carries `data-start`, in source order. An element inside a fenced
 * code block, a comment, a `<script>` or a `<style>` is not a clip.
 */
export function findClips(text: string): FoundClip[] {
	const seen = text.replace(HIDDEN, (hidden) => hidden.replace(/[^\n]/g, ' '));
	return [...seen.matchAll(OPEN)].map((found) => describe(text, found));
}
