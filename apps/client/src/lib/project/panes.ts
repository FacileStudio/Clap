const KEY = 'clap.editor.panes';

/** How the editor's three zones share the screen: `x` is the left panel's share, `y` the top row's. */
export interface Panes {
	x: number;
	y: number;
}

const DEFAULT: Panes = { x: 0.42, y: 0.56 };

const share = (value: unknown, fallback: number): number =>
	typeof value === 'number' && value >= 0.15 && value <= 0.85 ? value : fallback;

/** The shares this browser saved, or the defaults. */
export function loadPanes(): Panes {
	try {
		const saved = JSON.parse(localStorage.getItem(KEY) ?? '{}') as Partial<Panes>;
		return { x: share(saved.x, DEFAULT.x), y: share(saved.y, DEFAULT.y) };
	} catch {
		return { ...DEFAULT };
	}
}

/** Saves the shares for this browser. */
export function savePanes(panes: Panes): void {
	try {
		localStorage.setItem(KEY, JSON.stringify(panes));
	} catch {
		return;
	}
}
