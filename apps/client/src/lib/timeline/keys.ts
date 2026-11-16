/** What a keyboard shortcut asks the editor to do. */
export type Command =
	| 'toggle'
	| 'back'
	| 'pause'
	| 'forward'
	| 'prevFrame'
	| 'nextFrame'
	| 'prevSecond'
	| 'nextSecond'
	| 'home'
	| 'end'
	| 'split'
	| 'delete'
	| 'undo'
	| 'redo';

type Key = Pick<KeyboardEvent, 'key' | 'shiftKey' | 'ctrlKey' | 'metaKey' | 'altKey'>;

const KEYS: Record<string, Command> = {
	' ': 'toggle',
	j: 'back',
	k: 'pause',
	l: 'forward',
	ArrowLeft: 'prevFrame',
	ArrowRight: 'nextFrame',
	'Shift+ArrowLeft': 'prevSecond',
	'Shift+ArrowRight': 'nextSecond',
	Home: 'home',
	End: 'end',
	s: 'split',
	Backspace: 'delete',
	Delete: 'delete',
	'Mod+z': 'undo',
	'Mod+Shift+z': 'redo',
	'Mod+y': 'redo'
};

/** The chord `event` spells, such as `Mod+Shift+z`. Ctrl and Cmd are both `Mod`. */
export function chord(event: Key): string {
	const key = event.key.length === 1 ? event.key.toLowerCase() : event.key;
	return `${event.ctrlKey || event.metaKey ? 'Mod+' : ''}${event.shiftKey ? 'Shift+' : ''}${key}`;
}

/** The command `event` asks for, or null. Alt chords are left to the browser and to focused clips. */
export function commandOf(event: Key): Command | null {
	return event.altKey ? null : (KEYS[chord(event)] ?? null);
}

const TYPING = 'input, textarea, select, .cm-editor, [role="dialog"], [role="menu"]';
const PRESSING = 'button, a, summary';
const STEPPING = '[role="slider"], [role="separator"], [role="tab"], [role="radio"]';

/** Whether the element that has the focus uses the key of `event` itself, so no shortcut must fire. */
export function claimed(event: Pick<KeyboardEvent, 'key' | 'target'>): boolean {
	const target = event.target;
	if (!(target instanceof HTMLElement)) return false;
	if (target.isContentEditable || target.closest(TYPING)) return true;
	if (event.key === ' ' || event.key === 'Enter') return target.closest(PRESSING) !== null;
	return /^(Arrow|Home|End)/.test(event.key) && target.closest(STEPPING) !== null;
}
