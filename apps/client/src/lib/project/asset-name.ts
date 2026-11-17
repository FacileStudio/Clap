/** The asset name `typed` asks for in place of `name`: in the same folder, unless `typed` names a path itself. Empty when nothing was typed. */
export function renamedAsset(name: string, typed: string): string {
	const next = typed.trim();
	if (!next || next.includes('/')) return next;
	return `${name.slice(0, name.lastIndexOf('/') + 1)}${next}`;
}

/** The file name shown for an asset: the last part of its path. */
export const fileName = (name: string): string => name.split('/').pop() ?? name;
