import { afterAll, beforeAll, expect, test } from 'bun:test';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium, type Browser, type Page } from 'playwright-core';
import { dragFile } from './drops';
import { api, appendSource, attributeOf, dragBy, openPage, settled, startOf, waitData, type Stack } from './pages';
import { startStack } from './stack';

const scratch = mkdtempSync(join(tmpdir(), 'clap-e2e-'));
const clip = join(scratch, 'clip.mp4');
const still = join(scratch, 'still.png');
const long = join(scratch, 'long.mp4');
const SRC = 'assets/clip.mp4';
const ffmpeg = Bun.which('ffmpeg');

let stack: Stack;
let browser: Browser;
let editor: Page;
let id = '';

const stored = async () => (await api<{ source: string; rev: number }>(stack, `/projects/${id}`)).source;
const withClip = ffmpeg ? test : test.skip;

function makeClip(path: string, seconds: number): void {
	const video = ['-f', 'lavfi', '-i', `testsrc=duration=${seconds}:size=320x180:rate=15`];
	const sound = ['-f', 'lavfi', '-i', `sine=frequency=440:duration=${seconds}`];
	const encode = ['-c:v', 'libx264', '-pix_fmt', 'yuv420p', '-c:a', 'aac', '-shortest', path];
	const run = Bun.spawnSync([ffmpeg ?? 'ffmpeg', '-v', 'error', '-y', ...video, ...sound, ...encode]);
	if (run.exitCode !== 0) throw new Error(`ffmpeg could not make the test clip: ${run.stderr.toString()}`);
}

function makePicture(): void {
	const picture = Bun.spawnSync([ffmpeg ?? 'ffmpeg', '-v', 'error', '-y', '-f', 'lavfi', '-i', 'color=c=red:s=64x36', '-frames:v', '1', still]);
	if (picture.exitCode !== 0) throw new Error(`ffmpeg could not make the test picture: ${picture.stderr.toString()}`);
}

beforeAll(async () => {
	if (ffmpeg) {
		makeClip(clip, 2);
		makeClip(long, 8);
		makePicture();
	} else console.warn('e2e: SKIPPED the upload, drop, drag and render tests: ffmpeg is not installed');
	stack = await startStack();
	browser = await chromium.launch({ headless: true, executablePath: process.env.CLAP_CHROME || undefined });
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

withClip('a picture dropped from the desktop shows a ghost, then lands at the drop time', async () => {
	const file = { name: 'still.png', type: 'image/png', bytes: new Uint8Array(await Bun.file(still).arrayBuffer()) };
	const lane = editor.getByTestId('scene').first();
	const at = { x: 120, y: 70 };
	await dragFile(lane, file, { ...at, hover: true });
	const aimed = editor.getByTestId('drop-ghost').first();
	await aimed.waitFor({ timeout: 5_000 });
	const time = Number(await aimed.getAttribute('data-start'));
	await dragFile(lane, file, at);
	await editor.locator('[data-testid="clip"][data-kind="image"]').waitFor({ timeout: 15_000 });
	await settled(editor);
	expect(startOf(await stored(), 'assets/still.png')).toBe(time);
	expect(await editor.getByTestId('drop-ghost').count()).toBe(0);
});

withClip('a trimmed video starts its filmstrip on the frame of its media_in', async () => {
	await editor.locator('[data-testid="media-bin"] input[type="file"]').setInputFiles(long);
	await editor.locator('[data-testid="media-item"][data-name="assets/long.mp4"][data-state="ready"]').waitFor({ state: 'attached', timeout: 30_000 });
	const theirs = await api<{ source: string; rev: number }>(stack, `/projects/${id}`);
	const scene = '\n---\nduration: 4\n---\n\n<video src="assets/long.mp4" data-start="0" data-duration="3" data-in="4.5" data-track-index="0"></video>\n';
	const scenes = await editor.getByTestId('scene').count();
	await api(stack, `/projects/${id}/source`, { method: 'PUT', body: { source: theirs.source + scene, rev: theirs.rev } });
	await editor.locator(`[data-testid="scene"][data-scene="${scenes}"]`).waitFor({ state: 'attached', timeout: 15_000 });
	const block = editor.locator(`[data-testid="scene"][data-scene="${scenes}"] [data-testid="clip"][data-kind="video"]`);
	const first = block.locator('[data-testid="strip-tile"]').first();
	await first.waitFor({ state: 'attached', timeout: 30_000 });
	expect([await first.getAttribute('data-tile'), await first.getAttribute('data-at')]).toEqual(['2', '4.5']);

	await block.scrollIntoViewIfNeeded();
	const grip = await block.getByTestId('trim-start').boundingBox();
	if (!grip) throw new Error('the clip has no trim handle');
	await editor.mouse.move(grip.x + grip.width * 0.75, grip.y + grip.height / 2);
	await editor.mouse.down();
	await editor.mouse.move(grip.x + grip.width * 0.75 + 50, grip.y + grip.height / 2, { steps: 8 });
	await editor.mouse.up();
	await settled(editor);
	await waitData(editor, 'save-status', 'status', 'clean');
	const media_in = attributeOf(await stored(), 'assets/long.mp4', 'data-in');
	expect(media_in).toBeGreaterThan(4.5);
	await editor.locator(`[data-testid="clip-strip"][data-from="${media_in}"]`).waitFor({ state: 'attached', timeout: 10_000 });
	expect(Number(await first.getAttribute('data-at'))).toBeCloseTo(media_in, 3);
	expect(Number(await first.getAttribute('data-tile'))).toBe(Math.min(3, Math.floor(media_in / 2)));
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
