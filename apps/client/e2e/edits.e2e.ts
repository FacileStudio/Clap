import { afterAll, beforeAll, expect, test } from 'bun:test';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium, type Browser, type Page } from 'playwright-core';
import { api, dragBy, openPage, settled, type Stack } from './pages';
import { startStack } from './stack';

const scratch = mkdtempSync(join(tmpdir(), 'clap-edits-'));
const still = join(scratch, 'still.png');
const PNG = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==';
const SOURCE = `---
title: Edits e2e
size: 1280x720
layout: raw
duration: 6
---

<p data-clip="one" data-start="0" data-duration="2" data-track-index="0">One</p>
<p data-clip="two" data-start="2" data-duration="2" data-track-index="0">Two</p>
<img data-clip="still" src="assets/still.png" alt="" data-start="0" data-duration="3" data-track-index="1">
`;

let stack: Stack;
let browser: Browser;
let editor: Page;
let id = '';

type Listed = { markers: { id: string; t: number; label: string }[] };
const stored = async () => (await api<{ source: string; rev: number }>(stack, `/projects/${id}`)).source;
const markers = async () => (await api<Listed>(stack, `/projects/${id}/markers`)).markers;
const pin = (marker: string) => editor.locator(`[data-testid="marker"][data-marker="${marker}"]`);

async function until<T>(read: () => Promise<T>, ok: (value: T) => boolean): Promise<T> {
	for (let attempt = 0; attempt < 50; attempt += 1) {
		const value = await read();
		if (ok(value)) return value;
		await Bun.sleep(200);
	}
	throw new Error('the condition never held');
}

beforeAll(async () => {
	writeFileSync(still, Buffer.from(PNG, 'base64'));
	stack = await startStack();
	browser = await chromium.launch({ headless: true, executablePath: process.env.CLAP_CHROME || undefined });
	const made = await api<{ id: string; rev: number }>(stack, '/projects', { method: 'POST', body: { title: 'Edits e2e' } });
	id = made.id;
	await api(stack, `/projects/${id}/source`, { method: 'PUT', body: { source: SOURCE, rev: made.rev } });
	editor = await openPage(browser, stack, `/projects/${id}`);
	await editor.locator('[data-testid="clip"][data-clip="two"]').waitFor({ timeout: 15_000 });
}, 180_000);

afterAll(async () => {
	await browser?.close();
	await stack?.stop();
	rmSync(scratch, { recursive: true, force: true });
});

test('Ctrl+X and the clip menu cut a clip, and one undo puts it back', async () => {
	const two = editor.locator('[data-testid="clip"][data-clip="two"]');
	const source = await stored();
	await two.click();
	await editor.keyboard.press('Control+x');
	await settled(editor);
	expect([await two.count(), (await stored()).includes('Two')]).toEqual([0, false]);
	await editor.getByTestId('undo').click();
	await settled(editor);
	expect([await two.count(), await stored()]).toEqual([1, source]);

	await two.click({ button: 'right' });
	await editor.getByRole('menuitem', { name: 'Couper', exact: true }).click();
	await settled(editor);
	expect(await two.count()).toBe(0);
	await editor.getByTestId('undo').click();
	await settled(editor);
	expect(await stored()).toBe(source);
});

test('a cut clip pastes back at the playhead and stays after a reload', async () => {
	const two = editor.locator('[data-testid="clip"][data-clip="two"]');
	await two.click();
	await editor.keyboard.press('Control+x');
	await settled(editor);
	expect(await two.count()).toBe(0);
	await editor.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
	await editor.keyboard.press('Home');
	await editor.keyboard.press('Control+v');
	await settled(editor);
	const copy = editor.locator('[data-testid="clip"][data-clip="two-copy"]');
	await copy.waitFor({ timeout: 10_000 });
	const source = await until(stored, (text) => text.includes('data-clip="two-copy"'));
	expect(source).toContain('>Two</p>');
	await editor.reload();
	await copy.waitFor({ timeout: 15_000 });
	expect(await copy.getAttribute('data-start')).toBe('0');
});

test('the words of a text clip are edited in the clip settings and kept after a reload', async () => {
	const one = editor.locator('[data-testid="clip"][data-clip="one"]');
	const field = editor.getByTestId('inspector-text');
	await one.click();
	await editor.getByRole('tab', { name: 'Clip' }).click();
	expect(await field.inputValue()).toBe('One');
	await field.fill('Uno & co');
	await field.press('Enter');
	await settled(editor);
	await until(stored, (text) => text.includes('>Uno &amp; co</p>'));
	await editor.getByTestId('undo').click();
	await settled(editor);
	await until(stored, (text) => text.includes('>One</p>'));
	await editor.getByTestId('redo').click();
	await settled(editor);
	await until(stored, (text) => text.includes('>Uno &amp; co</p>'));
	await editor.reload();
	await one.click();
	await editor.getByRole('tab', { name: 'Clip' }).click();
	expect(await field.inputValue()).toBe('Uno & co');
});

test('a media file renamed in the bin is renamed in the source, and Escape cancels', async () => {
	await editor.getByRole('tab', { name: /^Média/ }).click();
	await editor.locator('[data-testid="media-bin"] input[type="file"]').setInputFiles(still);
	const item = editor.locator('[data-testid="media-item"][data-name="assets/still.png"]');
	await item.waitFor({ timeout: 15_000 });
	const field = editor.getByTestId('media-rename');
	await item.getByTestId('media-name').dblclick();
	await field.fill('ignored.png');
	await field.press('Escape');
	await field.waitFor({ state: 'detached', timeout: 5_000 });
	expect(await item.count()).toBe(1);

	await item.click({ button: 'right' });
	await editor.getByRole('menuitem', { name: 'Renommer' }).click();
	await field.fill('photo.png');
	await field.press('Enter');
	const renamed = editor.locator('[data-testid="media-item"][data-name="assets/photo.png"]');
	await renamed.waitFor({ timeout: 15_000 });
	const source = await until(stored, (text) => text.includes('src="assets/photo.png"'));
	expect(source.includes('assets/still.png')).toBe(false);
	await editor.reload();
	await renamed.waitFor({ timeout: 15_000 });
	await editor.locator('[data-testid="clip"][data-clip="still"]').waitFor({ timeout: 15_000 });
	expect(await stored()).toBe(source);
});

test('markers are added on the ruler and with M, moved, renamed, deleted and kept on reload', async () => {
	await editor.getByTestId('ruler').click({ button: 'right', position: { x: 60, y: 8 } });
	await editor.getByRole('menuitem', { name: /^Ajouter un repère ici/ }).click();
	const [added] = await until(markers, (list) => list.length === 1);
	await dragBy(editor, pin(added.id), 90);
	const [moved] = await until(markers, (list) => list[0]?.t > added.t);
	await editor.locator(`[data-marker="${added.id}"][data-time="${moved.t.toFixed(3)}"]`).waitFor({ timeout: 10_000 });
	await pin(added.id).click();
	expect(await editor.getByTestId('playhead').getAttribute('data-time')).toBe(moved.t.toFixed(3));

	await editor.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
	await editor.keyboard.press('Home');
	await editor.keyboard.press('m');
	const two = await until(markers, (list) => list.length === 2);
	await pin(added.id).click({ button: 'right' });
	await editor.getByRole('menuitem', { name: 'Renommer le repère' }).click();
	await editor.getByLabel('Nom du repère').fill('Refrain');
	await editor.getByRole('dialog').getByRole('button', { name: 'Enregistrer' }).click();
	await until(markers, (list) => list.some((marker) => marker.label === 'Refrain'));

	await editor.reload();
	await pin(added.id).waitFor({ timeout: 15_000 });
	expect(await editor.getByTestId('marker').count()).toBe(2);
	expect(await pin(added.id).getAttribute('data-time')).toBe(moved.t.toFixed(3));
	await pin(two.find((marker) => marker.id !== added.id)?.id ?? '').click({ button: 'right' });
	await editor.getByRole('menuitem', { name: 'Supprimer le repère' }).click();
	await until(markers, (list) => list.length === 1);
	await editor.reload();
	await pin(added.id).waitFor({ timeout: 15_000 });
	expect(await editor.getByTestId('marker').count()).toBe(1);
});
