/** One scene as the separators cut it: line indexes from 0, `to` excluded, and its frontmatter. */
export type Block = { from: number; body: number; to: number; meta: Record<string, string> };

const SEP = /^---\s*$/;
const KEY = /^([a-z0-9_]+):(?:\s+(.*))?$/;
const FENCE = /^\s{0,3}(`{3,}|~{3,})/;

function value(raw: string): string {
	const quoted = /^(["'])(.*)\1\s*$/.exec(raw);
	return quoted ? quoted[2] : raw.split(' #')[0].trim();
}

function front(lines: string[], at: number): { meta: Record<string, string>; next: number } {
	const meta: Record<string, string> = {};
	if (at < lines.length && SEP.test(lines[at])) return { meta, next: at + 1 };
	if (at >= lines.length || !KEY.test(lines[at])) return { meta, next: at };
	let line = at;
	for (; line < lines.length && !SEP.test(lines[line]); line += 1) {
		const found = KEY.exec(lines[line]);
		if (found) meta[found[1]] = value(found[2] ?? '');
	}
	return { meta, next: Math.min(line + 1, lines.length) };
}

function bodyEnd(lines: string[], at: number): number {
	let fence = '';
	for (let line = at; line < lines.length; line += 1) {
		const mark = FENCE.exec(lines[line])?.[1][0] ?? '';
		if (mark && (!fence || fence === mark)) fence = fence ? '' : mark;
		else if (!fence && SEP.test(lines[line])) return line;
	}
	return lines.length;
}

/** Cuts the lines of a source into scenes, following the file structure rules of `docs/format.md`. */
export function blocks(lines: string[]): Block[] {
	const found: Block[] = [];
	let from = 0;
	let head = SEP.test(lines[0] ?? '');
	let at = head ? 1 : 0;
	for (;;) {
		const { meta, next } = head ? front(lines, at) : { meta: {}, next: at };
		const to = bodyEnd(lines, next);
		found.push({ from, body: next, to, meta });
		if (to >= lines.length) break;
		[from, at, head] = [to, to + 1, true];
	}
	const last = found[found.length - 1];
	const empty = lines.slice(last.body, last.to).every((line) => !line.trim()) && !Object.keys(last.meta).length;
	return empty && found.length > 1 ? found.slice(0, -1) : found;
}
