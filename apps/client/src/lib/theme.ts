import { applyTheme, isTheme, readTheme, type Theme } from '@facile/muse';

const KEY = 'clap.theme';

/** Applies the theme this browser saved, if any. */
export function restoreTheme(): void {
	try {
		const saved = localStorage.getItem(KEY);
		if (isTheme(saved)) applyTheme(saved);
	} catch {
		return;
	}
}

/** Applies `theme` and saves it for this browser. */
export function saveTheme(theme: Theme): void {
	document.documentElement.removeAttribute('data-editor-dark');
	applyTheme(theme);
	try {
		localStorage.setItem(KEY, theme);
	} catch {
		return;
	}
}

/**
 * Video tools are dark: while the editor is open and the person picked no theme, the page is dark. A picked
 * light or dark theme is left alone. Answers the function that puts the system theme back.
 */
export function darkWhileOpen(root: Element = document.documentElement): () => void {
	if (readTheme(root) !== 'system') return () => undefined;
	applyTheme('dark', root);
	root.setAttribute('data-editor-dark', '');
	return () => {
		if (!root.hasAttribute('data-editor-dark')) return;
		root.removeAttribute('data-editor-dark');
		applyTheme('system', root);
	};
}
