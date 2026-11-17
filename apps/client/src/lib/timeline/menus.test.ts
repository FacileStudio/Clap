import { describe, expect, it, vi } from 'vitest';
import type { MenuAction, MenuEntry } from '@facile/muse';
import { ApiError } from '$lib/http';
import { Markers } from '$lib/project/markers.svelte';
import { Transport } from '$lib/project/transport.svelte';
import { stageMenu } from '$lib/stage/menu';
import type { MenuHost } from './clip-menu';
import { runCommand, type Bench } from './commands';
import { Engine } from './engine.svelte';
import { commandOf, withKeys } from './keys';
import { BLANK_SCENE, timelineMenu } from './menus';

const SOURCE = `---
layout: raw
duration: 6
---

<video src="assets/a.mp4" data-start="1" data-duration="2" data-track-index="0"></video>
<p data-start="0" data-duration="6" data-track-index="1">Hi</p>
`;

async function setup() {
	const engine = new Engine({
		rev: () => 1,
		edit: async () => ({ source: SOURCE, rev: 2 }),
		put: async () => ({ rev: 2 }),
		timeline: async () => {
			throw new ApiError(404, 'not_found', 'none');
		},
		adopt: () => undefined,
		reload: async () => undefined,
		refused: () => undefined
	});
	await engine.sync(SOURCE);
	const transport = new Transport();
	transport.duration = 6;
	transport.seek(2);
	const bench: Bench = { engine, transport, copied: null };
	const markers = new Markers('p', vi.fn());
	markers.list = [{ id: 'm1', t: 3, label: 'Drop' }];
	const host: MenuHost = { bench, run: vi.fn(), reveal: vi.fn(), has: () => true, addMedia: vi.fn(), markers, renameMarker: vi.fn() };
	const commit = vi.spyOn(engine, 'commit').mockResolvedValue(true);
	return { engine, transport, bench, host, commit, markers };
}

function node(html: string): Element {
	const root = document.createElement('div');
	root.innerHTML = html;
	return root.querySelector('[data-pick]') ?? root;
}

const actions = (entries: MenuEntry[]) => entries.filter((entry): entry is MenuAction => !('separator' in entry));
const pick = (entries: MenuEntry[], label: string) => actions(entries).find((entry) => entry.label.startsWith(label));
const AT = { time: 2, track: 0 };

describe('timeline menus', () => {
	it('opens the clip menu on a clip, selects it, and shows the shortcuts', async () => {
		const { engine, host } = await setup();
		const menu = timelineMenu(host, node('<div data-testid="clip" data-clip="clip-0-0"><span data-pick></span></div>'), AT, vi.fn());
		expect([...engine.selection]).toEqual(['clip-0-0']);
		expect([pick(menu, 'Couper')?.label, pick(menu, 'Couper')?.shortcut]).toEqual(['Couper à la tête de lecture', 's']);
		expect([pick(menu, 'Coller')?.disabled, pick(menu, 'Rogner le début')?.disabled]).toEqual([true, false]);
		pick(menu, 'Dupliquer')?.onselect?.();
		expect(host.run).toHaveBeenCalledWith('duplicate');
	});

	it('commits the clip edits it offers', async () => {
		const { host, commit } = await setup();
		const menu = timelineMenu(host, node('<div data-testid="clip" data-clip="clip-0-0" data-pick></div>'), AT, vi.fn());
		pick(menu, 'Rogner la fin')?.onselect?.();
		pick(menu, 'Fondu')?.onselect?.();
		expect(commit.mock.calls).toEqual([
			[[{ op: 'trim_clip', clip: 'clip-0-0', duration: 1 }]],
			[[{ op: 'update_clip', clip: 'clip-0-0', enter: 'fade', exit: 'fade' }]]
		]);
		pick(menu, 'Afficher dans les médias')?.onselect?.();
		expect(host.reveal).toHaveBeenCalledWith('assets/a.mp4');
	});

	it('opens the scene, ruler and lane menus where they belong', async () => {
		const { host, commit, transport } = await setup();
		const head = '<div data-testid="scene" data-scene="0"><div data-testid="scene-head" data-pick></div></div>';
		const scene = timelineMenu(host, node(head), AT, vi.fn());
		expect(pick(scene, 'Supprimer la scène')?.disabled).toBe(true);
		pick(scene, 'Insérer une scène vide après')?.onselect?.();
		pick(scene, 'Dupliquer la scène')?.onselect?.();
		expect(commit.mock.calls).toEqual([
			[[{ op: 'add_scene', at: 1, duration: 4, body: BLANK_SCENE }]],
			[[{ op: 'duplicate_scene', scene: 0 }]]
		]);
		const fit = vi.fn();
		const ruler = timelineMenu(host, node('<div data-testid="ruler" data-pick></div>'), { time: 4.5, track: 0 }, fit);
		pick(ruler, 'Placer la tête de lecture')?.onselect?.();
		pick(ruler, 'Afficher tout le projet')?.onselect?.();
		expect([transport.time, fit.mock.calls.length]).toEqual([4.5, 1]);
		const lane = timelineMenu(host, node('<div data-pick></div>'), AT, fit);
		pick(lane, 'Ajouter un média ici')?.onselect?.();
		expect(host.addMedia).toHaveBeenCalledWith(AT);
	});

	it('adds a marker where the ruler was clicked and renames, deletes or jumps to one', async () => {
		const { host, markers, transport } = await setup();
		const add = vi.spyOn(markers, 'add').mockResolvedValue();
		const remove = vi.spyOn(markers, 'remove').mockResolvedValue();
		const ruler = timelineMenu(host, node('<div data-testid="ruler" data-pick></div>'), { time: 4.5, track: 0 }, vi.fn());
		expect(pick(ruler, 'Ajouter un repère ici')?.shortcut).toBe('m');
		pick(ruler, 'Ajouter un repère ici')?.onselect?.();
		expect(add).toHaveBeenCalledWith(4.5);
		const pin = '<div data-testid="ruler"><button data-testid="marker" data-marker="m1" data-pick></button></div>';
		const menu = timelineMenu(host, node(pin), AT, vi.fn());
		pick(menu, 'Renommer le repère')?.onselect?.();
		pick(menu, 'Supprimer le repère')?.onselect?.();
		pick(menu, 'Placer la tête de lecture')?.onselect?.();
		expect([host.renameMarker, remove].map((spy) => vi.mocked(spy).mock.calls)).toEqual([[['m1']], [['m1']]]);
		expect(transport.time).toBe(3);
	});

	it('acts on the stage selection: stacking, frame, reset', async () => {
		const { host, commit } = await setup();
		const box = { id: 'clip-0-0', kind: 'video' as const, x: 10, y: 10, w: 100, h: 50, rot: 0 };
		const menu = stageMenu(host, 'clip-0-0', box, { width: 1280, height: 720 });
		expect([pick(menu, 'Reculer')?.disabled, pick(menu, 'Réinitialiser')?.disabled]).toEqual([true, true]);
		pick(menu, 'Avancer')?.onselect?.();
		pick(menu, 'Ajuster au cadre')?.onselect?.();
		expect(commit.mock.calls).toEqual([
			[[{ op: 'move_clip', clip: 'clip-0-0', start: 1, track: 2 }]],
			[[{ op: 'update_clip', clip: 'clip-0-0', x: 0, y: 0, w: 1280, h: 720, rot: 0, fit: 'contain' }]]
		]);
	});
});

describe('clip commands', () => {
	it('copies the selection, pastes it at the playhead and duplicates it', async () => {
		const { engine, bench, commit } = await setup();
		engine.select('clip-0-0');
		runCommand('copy', bench);
		runCommand('paste', bench);
		runCommand('duplicate', bench);
		expect(bench.copied).toBe('clip-0-0');
		expect(commit.mock.calls).toEqual([
			[[{ op: 'duplicate_clip', clip: 'clip-0-0', scene: 0, start: 2, track: 2 }]],
			[[{ op: 'duplicate_clip', clip: 'clip-0-0', scene: 0, start: 3, track: 0 }]]
		]);
	});

	it('cuts the selection: copied, then deleted in one undoable edit', async () => {
		const { engine, bench, commit, host } = await setup();
		engine.select('clip-0-0');
		engine.select('clip-0-1', true);
		runCommand('cut', bench);
		expect(bench.copied).toBe('clip-0-0');
		expect(commit.mock.calls).toEqual([
			[
				[
					{ op: 'delete_clip', clip: 'clip-0-0' },
					{ op: 'delete_clip', clip: 'clip-0-1' }
				]
			]
		]);
		const menu = timelineMenu(host, node('<div data-testid="clip" data-clip="clip-0-0" data-pick></div>'), AT, vi.fn());
		expect(pick(menu, 'Couper')?.label).toBe('Couper à la tête de lecture');
		expect(actions(menu).find((entry) => entry.label === 'Couper')?.shortcut).toBe('mod+x');
	});

	it('maps the new shortcuts and spells them for menus', () => {
		const key = (key: string, more: Partial<KeyboardEvent> = {}) =>
			commandOf({ key, shiftKey: false, ctrlKey: false, metaKey: false, altKey: false, ...more });
		expect([key('d', { ctrlKey: true }), key('Backspace', { shiftKey: true }), key('?', { shiftKey: true })]).toEqual([
			'duplicate',
			'ripple',
			'help'
		]);
		expect([key('x', { ctrlKey: true }), key('x', { metaKey: true }), key('m'), key('M', { shiftKey: true })]).toEqual([
			'cut',
			'cut',
			'marker',
			null
		]);
		expect(withKeys('Annuler', 'undo')).toBe('Annuler (Ctrl+Z)');
	});
});
