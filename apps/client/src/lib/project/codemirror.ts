import { HighlightStyle, syntaxHighlighting } from '@codemirror/language';
import { setDiagnostics, type Diagnostic } from '@codemirror/lint';
import { StateEffect, StateField, type EditorState, type Extension, type TransactionSpec } from '@codemirror/state';
import { Decoration, EditorView, type DecorationSet } from '@codemirror/view';
import { tags } from '@lezer/highlight';
import type { SceneError } from '$lib/types';

const fg = 'var(--color-fc-fg)';
const muted = 'var(--color-fc-fg-muted)';

const chrome = EditorView.theme({
	'&': { height: '100%', color: fg, backgroundColor: 'transparent', fontSize: '13px' },
	'&.cm-focused': { outline: 'none' },
	'.cm-scroller': { fontFamily: 'var(--font-fc-mono)', lineHeight: '1.6' },
	'.cm-content': { padding: '16px 0', caretColor: fg },
	'.cm-line': { padding: '0 16px 0 8px' },
	'.cm-cursor': { borderLeftColor: fg },
	'.cm-gutters': { border: 'none', backgroundColor: 'transparent', color: muted },
	'.cm-activeLine': { backgroundColor: 'color-mix(in oklab, var(--color-fc-surface) 60%, transparent)' },
	'.cm-activeLineGutter': { backgroundColor: 'transparent', color: fg },
	'.cm-selectionBackground, &.cm-focused .cm-selectionBackground': { backgroundColor: 'var(--color-fc-field)' },
	'.cm-panels': { border: 'none', backgroundColor: 'var(--color-fc-surface)', color: fg },
	'.cm-textfield': { border: 'none', borderRadius: '6px', backgroundColor: 'var(--color-fc-field)' },
	'.cm-button': {
		border: 'none',
		borderRadius: '6px',
		backgroundImage: 'none',
		backgroundColor: 'var(--color-fc-field)'
	},
	'.cm-tooltip': {
		border: 'none',
		borderRadius: '8px',
		overflow: 'hidden',
		backgroundColor: 'var(--color-fc-bg)',
		color: fg,
		boxShadow: 'var(--shadow-lg)'
	},
	'.cm-diagnostic': { border: 'none', padding: '8px 12px', fontFamily: 'var(--font-fc-body)' },
	'.cm-lintRange-error': { backgroundColor: 'color-mix(in oklab, var(--color-fc-danger) 16%, transparent)' },
	'.cm-clip-line': { backgroundColor: 'color-mix(in oklab, var(--color-fc-info) 18%, transparent)' },
	'.cm-foldPlaceholder': { border: 'none', backgroundColor: 'var(--color-fc-field)', color: muted }
});

const syntax = HighlightStyle.define([
	{ tag: tags.heading, fontWeight: '600', color: fg },
	{ tag: tags.strong, fontWeight: '700' },
	{ tag: tags.emphasis, fontStyle: 'italic' },
	{ tag: [tags.processingInstruction, tags.meta, tags.comment, tags.contentSeparator], color: muted },
	{ tag: [tags.link, tags.url], color: 'var(--color-fc-info)' },
	{ tag: [tags.monospace, tags.string, tags.attributeValue], color: 'var(--color-fc-success)' },
	{ tag: [tags.tagName, tags.keyword, tags.angleBracket], color: 'var(--color-fc-info)' },
	{ tag: [tags.attributeName, tags.propertyName, tags.number], color: 'var(--color-fc-warning)' }
]);

/** The look of the source editor: muse tokens, no borders, the mono stack. */
export const look: Extension = [chrome, syntaxHighlighting(syntax)];

/** The transaction that marks the lines of `errors` in `state`. A line past the end marks the last line. */
export function markErrors(state: EditorState, errors: SceneError[]): TransactionSpec {
	const marks: Diagnostic[] = errors.map((error) => {
		const line = state.doc.line(Math.min(Math.max(error.line, 1), state.doc.lines));
		return { from: line.from, to: line.to, severity: 'error', message: error.message };
	});
	return setDiagnostics(state, marks);
}

const setLines = StateEffect.define<{ from: number; to: number } | null>();
const lineMark = Decoration.line({ class: 'cm-clip-line' });

/** Marks the lines of the clip selected in the timeline. */
export const clipLines = StateField.define<DecorationSet>({
	create: () => Decoration.none,
	update(marks, transaction) {
		let next = marks.map(transaction.changes);
		for (const effect of transaction.effects) {
			if (!effect.is(setLines)) continue;
			const { doc } = transaction.state;
			const span = effect.value;
			const lines = span ? Array.from({ length: span.to - span.from + 1 }, (_, index) => span.from + index) : [];
			next = Decoration.set(lines.filter((line) => line <= doc.lines).map((line) => lineMark.range(doc.line(line).from)));
		}
		return next;
	},
	provide: (field) => EditorView.decorations.from(field)
});

/** The transaction that marks the lines `from` to `to` (from 1), or clears the mark with null. */
export function markLines(span: { from: number; to: number } | null): TransactionSpec {
	return { effects: setLines.of(span) };
}
