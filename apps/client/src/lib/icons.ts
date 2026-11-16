import { icons as museIcons, registerIcons } from '@facile/muse';

const line = (body: string) => ({
	body: `<g fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">${body}</g>`,
	width: 24,
	height: 24
});

const solid = (body: string) => ({ body: `<g fill="currentColor">${body}</g>`, width: 24, height: 24 });

registerIcons({
	'clap:play': solid('<path d="M8 5.6v12.8a1 1 0 0 0 1.5.86l10.6-6.4a1 1 0 0 0 0-1.72L9.5 4.74A1 1 0 0 0 8 5.6Z"/>'),
	'clap:pause': solid('<rect x="6" y="5" width="4.5" height="14" rx="1.2"/><rect x="13.5" y="5" width="4.5" height="14" rx="1.2"/>'),
	'clap:start': solid('<rect x="5" y="5" width="2.5" height="14" rx="1"/><path d="M19 6.2v11.6a.8.8 0 0 1-1.24.67L9.4 12.67a.8.8 0 0 1 0-1.34l8.36-5.8A.8.8 0 0 1 19 6.2Z"/>'),
	'clap:end': solid('<rect x="16.5" y="5" width="2.5" height="14" rx="1"/><path d="M5 6.2v11.6a.8.8 0 0 0 1.24.67l8.36-5.8a.8.8 0 0 0 0-1.34L6.24 5.53A.8.8 0 0 0 5 6.2Z"/>'),
	'clap:split': line('<circle cx="6" cy="18" r="3"/><circle cx="6" cy="6" r="3"/><path d="M8.2 8.2 20 20M8.2 15.8 20 4"/>'),
	'clap:undo': line('<path d="M8 5 4 9l4 4"/><path d="M4 9h10a6 6 0 0 1 0 12h-3"/>'),
	'clap:redo': line('<path d="m16 5 4 4-4 4"/><path d="M20 9H10a6 6 0 0 0 0 12h3"/>'),
	'clap:snap': line('<path d="M5 4h4v8a3 3 0 0 0 6 0V4h4v8a7 7 0 0 1-14 0Z"/><path d="M5 8h4M15 8h4"/>'),
	'clap:fit': line('<path d="M4 9V5h4M20 9V5h-4M4 15v4h4M20 15v4h-4"/>'),
	'clap:safe': line('<rect x="3" y="5" width="18" height="14" rx="2"/><rect x="7" y="8.5" width="10" height="7" rx="1" stroke-dasharray="2 2"/>'),
	'clap:video': line('<rect x="3" y="5" width="18" height="14" rx="3"/><path d="M8 5v14M16 5v14M3 10h5M3 14h5M16 10h5M16 14h5"/>'),
	'clap:audio': line('<path d="M9 18V6l10-2v12"/><circle cx="6.5" cy="18" r="2.5"/><circle cx="16.500" cy="16" r="2.5"/>'),
	'clap:image': line('<rect x="3" y="4" width="18" height="16" rx="3"/><circle cx="8.500" cy="9.500" r="1.500"/><path d="m4 17 5-5 4 4 3-3 4 4"/>')
});

/** The icons Clap uses: muse's, plus the transport and timeline glyphs it draws itself. */
export const icons = {
	...museIcons,
	play: 'clap:play',
	pause: 'clap:pause',
	toStart: 'clap:start',
	toEnd: 'clap:end',
	split: 'clap:split',
	undo: 'clap:undo',
	redo: 'clap:redo',
	snap: 'clap:snap',
	fit: 'clap:fit',
	safe: 'clap:safe',
	video: 'clap:video',
	audio: 'clap:audio',
	image: 'clap:image'
} as const;

/** The icon of each clip kind. */
export const KIND_ICON = {
	video: icons.video,
	audio: icons.audio,
	image: icons.image,
	text: icons.text,
	html: icons.code
} as const;
