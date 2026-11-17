import { outline, type ClipSpan } from '../src/lib/timeline/outline';

/** An attribute value; `null` removes the attribute, `undefined` leaves it. */
export type Value = number | string | null | undefined;

function setAttr(tag: string, name: string, value: Value): string {
	if (value === undefined) return tag;
	if (value === null) return tag.replace(new RegExp(`\\s${name}\\s*=\\s*"[^"]*"`), '');
	const pattern = new RegExp(`(\\s${name}\\s*=\\s*")[^"]*(")`);
	if (pattern.test(tag)) return tag.replace(pattern, `$1${value}$2`);
	return tag.replace(/(\s*\/?>)$/, ` ${name}="${value}"$1`);
}

/** The clip `id` of `source` with where it sits. Throws when it is not there. */
export function clipOf(source: string, id: string): ClipSpan {
	const found = outline(source).scenes.flatMap((scene) => scene.clips).find((clip) => clip.id === id);
	if (!found) throw new Error(`no clip ${id}`);
	return found;
}

/** `source` with the start tag of `clip` given `attrs`. */
export function retag(source: string, clip: ClipSpan, attrs: Record<string, Value>): string {
	let tag = source.slice(clip.open[0], clip.open[1]);
	for (const [name, value] of Object.entries(attrs)) tag = setAttr(tag, name, value);
	return source.slice(0, clip.open[0]) + tag + source.slice(clip.open[1]);
}
