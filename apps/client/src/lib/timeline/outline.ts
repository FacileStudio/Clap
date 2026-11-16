import type { Clip, Scene, Timeline } from '$lib/types';
import { findClips, type FoundClip } from './clips';
import { round } from './model';
import { blocks } from './scenes';

/** A clip with where it sits in the source: its lines (from 1) and its character offsets. */
export interface ClipSpan extends Clip {
	from: number;
	to: number;
	label: string;
	open: [number, number];
	end: number;
}

/** A scene with where it sits in the source. `from` is its separator line (or line 1), `to` its last line. */
export interface SceneSpan extends Scene {
	from: number;
	to: number;
	/** Whether the frontmatter sets `duration`. */
	explicit: boolean;
	clips: ClipSpan[];
}

/** What the client reads from a source on its own: the stage size and every scene with its clips. */
export interface Outline extends Timeline {
	width: number;
	height: number;
	scenes: SceneSpan[];
}

function steps(meta: Record<string, string>, text: string): number {
	if (meta.steps !== undefined) return Number(meta.steps) || 0;
	const marks = [...text.matchAll(/\{step=(\d+)\}|data-step="(\d+)"/g)].map((found) => Number(found[1] ?? found[2]));
	return Math.max(0, ...marks);
}

function length(meta: Record<string, string>, text: string, clips: FoundClip[], stepDuration: number): number {
	const set = Number(meta.duration);
	if (set > 0) return set;
	const ends = clips.map((clip) => clip.start + (clip.duration ?? 0));
	if (ends.length > 0 && Math.max(...ends) > 0) return Math.max(...ends);
	return (steps(meta, text) + 1) * stepDuration;
}

/** Reads `source` the way `docs/format.md` splits it: scenes, their length, and the clips with their lines. */
export function outline(source: string): Outline {
	const lines = source.replace(/^﻿/, '').split(/\r?\n/);
	const offsets: number[] = [];
	lines.reduce((at, line, index) => ((offsets[index] = at), at + line.length + 1), 0);
	const offsetOf = (line: number) => offsets[line] ?? source.length;
	const lineOf = (offset: number) => Math.max(1, offsets.findLastIndex((start) => start <= offset) + 1);
	const found = blocks(lines);
	const head = found[0].meta;
	const size = /^(\d+)x(\d+)$/.exec(head.size ?? '');
	const taken = new Set<string>();
	const idOf = (named: string | null, derived: string) => {
		const id = named && !taken.has(named) ? named : derived;
		taken.add(id);
		return id;
	};
	const scenes = found.map((block, index): SceneSpan => {
		const base = offsetOf(block.body);
		const text = source.slice(base, offsetOf(block.to));
		const raw = findClips(text);
		const duration = round(length(block.meta, text, raw, Number(head.step_duration) || 4));
		const clips = raw.map((clip, order): ClipSpan => ({
			...clip,
			id: idOf(clip.named, `clip-${index}-${order}`),
			duration: round(clip.duration ?? Math.max(0, duration - clip.start)),
			from: lineOf(base + clip.open[0]),
			to: lineOf(base + Math.max(clip.open[0], clip.end - 1)),
			open: [base + clip.open[0], base + clip.open[1]],
			end: base + clip.end
		}));
		const explicit = Number(block.meta.duration) > 0;
		return { index, duration, layout: block.meta.layout ?? 'default', clips, from: block.from + 1, to: block.to, explicit };
	});
	return { width: Number(size?.[1]) || 1920, height: Number(size?.[2]) || 1080, scenes };
}
