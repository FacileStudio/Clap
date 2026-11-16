import { afterAll, beforeAll, expect, test } from 'bun:test';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium, type Browser, type Page } from 'playwright-core';
import { api, appendSource, dragBy, openPage, settled, startOf, waitData, type Stack } from './pages';
import { startStack } from './stack';

const scratch = mkdtempSync(join(tmpdir(), 'clap-e2e-'));
const clip = join(scratch, 'clip.mp4');
const SRC = 'assets/clip.mp4';
const ffmpeg = Bun.which('ffmpeg');

let stack: Stack;
let browser: Browser;
let editor: Page;
let id = '';

const stored = async () => (await api<{ source: string; rev: number }>(stack, `/projects/${id}`)).source;
const withClip = ffmpeg ? test : test.skip;

function makeClip(): void {
	const video = ['-f', 'lavfi', '-i', 'testsrc=duration=2:size=320x180:rate=15'];
	const sound = ['-f', 'lavfi', '-i', 'sine=frequency=440:duration=2'];
	const encode = ['-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-c:a', 'aac', '-shortest', clip];
	const run = Bun.spawnSync([ffmpeg ?? 'ffmpeg', '-v', 'error', '-y', ...video, ...sound, ...encode]);
	if (run.exitCode !== 0) throw new Error(`ffmpeg could not make the test clip: ${run.stderr.toString()}`);
}

beforeAll(async () => {
	if (ffmpeg) makeClip();
	else console.warn('e2e: SKIPPED the upload, drop, drag and render tests: ffmpeg is not installed');
	stack = await startStack();
	browser = await chromium.launch({ headless: true });
}, 180_000);

afterAll(async () => {
	await browser?.close();
	await stack?.stop();
	rmSync(scratch, { recursive: true, force: true });
});

test('a project is created from the list and opens in the editor', async () => {
	editor = await openPage(browser, stack, '/');
	await editor.getByTestId('new-project').click();
	await editor.getByLabel('Titre').fill('Démo e2e');
	await editor.getByRole('button', { name: 'Créer' }).click();
	await editor.waitForURL(/\/projects\/[^/]+$/);
	id = new URL(editor.url()).pathname.split('/').pop() ?? '';
	await waitData(editor, 'save-status', 'status', 'clean');
	await editor.getByTestId('scene').first().waitFor({ timeout: 15_000 });
	expect(await editor.getByTestId('timecode').textContent()).toContain('00:00:00:00');
});

withClip('an uploaded clip is probed and shown in the media bin', async () => {
	await editor.locator('[data-testid="media-bin"] input[type="file"]').setInputFiles(clip);
	await waitData(editor, 'media-item', 'state', 'ready');
	expect(await editor.getByTestId('media-item').getAttribute('data-name')).toBe('assets/clip.mp4');
});

withClip('dropping it on the timeline writes a clip into the source', async () => {
	const scene = editor.getByTestId('scene').first();
	await editor.getByTestId('media-item').dragTo(scene, { targetPosition: { x: 40, y: 50 } });
	await editor.locator('[data-testid="clip"][data-kind="video"]').waitFor({ timeout: 15_000 });
	await settled(editor);
	expect(await stored()).toContain(SRC);
});

withClip('dragging the clip changes its data-start in the stored source, and undo puts it back', async () => {
	const before = startOf(await stored(), SRC);
	await dragBy(editor, editor.locator('[data-testid="clip"][data-kind="video"]'), 140);
	await settled(editor);
	await waitData(editor, 'save-status', 'status', 'clean');
	const after = startOf(await stored(), SRC);
	expect(after).toBeGreaterThan(before);
	expect(Number(await editor.locator('[data-testid="clip"][data-kind="video"]').getAttribute('data-start'))).toBe(after);

	await editor.getByTestId('undo').click();
	await settled(editor);
	expect(startOf(await stored(), SRC)).toBe(before);
});

withClip('the selected clip is marked in the source, and S splits it at the playhead', async () => {
	const video = editor.locator('[data-testid="clip"][data-kind="video"]');
	await video.click();
	await editor.getByRole('tab', { name: 'Source' }).click();
	await editor.locator('.cm-clip-line').first().waitFor({ timeout: 5_000 });
	expect(await editor.locator('.cm-clip-line').first().textContent()).toContain(SRC);

	const start = Number(await video.getAttribute('data-start'));
	const box = await video.boundingBox();
	const ruler = await editor.getByTestId('ruler').boundingBox();
	if (!box || !ruler) throw new Error('the timeline is not drawn');
	await editor.keyboard.down('Alt');
	await editor.mouse.click(box.x + box.width / 2, ruler.y + ruler.height / 2);
	await editor.keyboard.up('Alt');
	await editor.keyboard.press('s');
	await settled(editor);
	expect(await editor.locator('[data-testid="clip"][data-kind="video"]').count()).toBe(2);
	expect((await stored()).split(SRC).length - 1).toBe(2);
	expect(startOf(await stored(), SRC)).toBe(start);
});

test('typed source is saved, and a source the parser refuses shows its line', async () => {
	await appendSource(editor, '\n<!-- note -->\n');
	await waitData(editor, 'save-status', 'status', 'clean');
	expect(await stored()).toContain('<!-- note -->');
	await appendSource(editor, '\n---\ncolour: red\n---\n\nFin\n');
	await waitData(editor, 'save-status', 'status', 'invalid');
	expect(await editor.getByTestId('scene-errors').textContent()).toMatch(/Ligne \d+/);
	await editor.keyboard.press('Control+z');
	await waitData(editor, 'save-status', 'status', 'clean');
});

test('an outside write reloads a clean editor', async () => {
	const theirs = await api<{ source: string; rev: number }>(stack, `/projects/${id}`);
	const body = { source: theirs.source.replace('<!-- note -->', '<!-- theirs -->'), rev: theirs.rev };
	await api(stack, `/projects/${id}/source`, { method: 'PUT', body });
	await editor.locator('.cm-content').getByText('<!-- theirs -->').waitFor({ timeout: 10_000 });
	await waitData(editor, 'save-status', 'status', 'clean');
});

test('a render job is created from the dialog', async () => {
	await editor.getByTestId('render-open').click();
	await editor.getByTestId('render-start').click();
	await editor.locator('[data-testid="render-dialog"]:not([data-job=""])').waitFor({ timeout: 15_000 });
	const job = (await editor.getByTestId('render-dialog').getAttribute('data-job')) ?? '';
	const status = await api<{ id: string; state: string }>(stack, `/renders/${job}`);
	expect(status.id).toBe(job);
	expect(['queued', 'running', 'done', 'failed', 'cancelled']).toContain(status.state);
	const listed = await api<{ renders: { id: string }[] }>(stack, `/projects/${id}/renders`);
	expect(listed.renders.map((one) => one.id)).toContain(job);
});
