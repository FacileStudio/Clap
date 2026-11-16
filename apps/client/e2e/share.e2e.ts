import { afterAll, beforeAll, expect, test } from 'bun:test';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium, type Browser, type Page } from 'playwright-core';
import { api, openPage, waitData, type Stack } from './pages';
import { startStack } from './stack';

const scratch = mkdtempSync(join(tmpdir(), 'clap-share-'));
const clip = join(scratch, 'out.mp4');
const ffmpeg = Bun.which('ffmpeg');
const TITLE = 'Partage e2e';
const SECONDS = 4;

let stack: Stack;
let browser: Browser;
let editor: Page;
let id = '';
let url = '';

function makeClip(): boolean {
	const video = ['-f', 'lavfi', '-i', `testsrc=duration=${SECONDS}:size=320x180:rate=15`, '-c:v', 'libx264', '-pix_fmt', 'yuv420p'];
	return Bun.spawnSync([ffmpeg ?? 'ffmpeg', '-v', 'error', '-y', ...video, clip]).exitCode === 0;
}

async function finished(render: string): Promise<void> {
	for (let attempt = 0; attempt < 60; attempt += 1) {
		const status = await api<{ state: string }>(stack, `/renders/${render}`);
		if (status.state === 'done') return;
		await Bun.sleep(250);
	}
	throw new Error(`render ${render} never finished`);
}

async function pictured(token: string): Promise<void> {
	for (let attempt = 0; attempt < 40; attempt += 1) {
		const answer = await fetch(`${stack.base}/api/public/renders/${token}`);
		if (answer.ok && ((await answer.json()) as { poster: string | null }).poster) return;
		await Bun.sleep(250);
	}
	throw new Error('the shared render never got a poster');
}

const confirmIn = (page: Page, title: string, button: string) =>
	page.getByRole('dialog', { name: title }).getByRole('button', { name: button }).click();

beforeAll(async () => {
	stack = await startStack();
	browser = await chromium.launch({ headless: true, executablePath: process.env.CLAP_CHROME || undefined });
}, 180_000);

afterAll(async () => {
	await browser?.close();
	await stack?.stop();
	rmSync(scratch, { recursive: true, force: true });
});

const ready = () => Boolean(stack?.finish) && ffmpeg !== null;

test('a project with no finished render says so in the share dialog', async () => {
	if (!stack.finish) console.warn('e2e: SKIPPED the share tests: they need the real API, not the mock');
	if (!ffmpeg) console.warn('e2e: SKIPPED the share tests: ffmpeg is not installed');
	if (!ready()) return;
	id = (await api<{ id: string }>(stack, '/projects', { method: 'POST', body: { title: TITLE } })).id;
	editor = await openPage(browser, stack, `/projects/${id}`);
	await editor.getByTestId('share-open').click();
	await waitData(editor, 'share-dialog', 'rendered', 'false');
	await editor.getByTestId('share-norender').waitFor();
	await editor.keyboard.press('Escape');
});

test('once a render is done, the link is shown once with its embed code', async () => {
	if (!ready() || !makeClip()) return console.warn('e2e: SKIPPED: no finished render could be made');
	const render = await api<{ id: string }>(stack, `/projects/${id}/renders`, {
		method: 'POST',
		body: { width: 320, height: 180, fps: 15 }
	});
	stack.finish?.(render.id, clip);
	await finished(render.id);
	await editor.getByTestId('share-open').click();
	await waitData(editor, 'share-dialog', 'rendered', 'true');
	await editor.getByTestId('share-create').click();
	url = await editor.getByTestId('share-url').inputValue();
	const token = url.split('/v/')[1];
	expect(url).toBe(`${stack.base}/v/${token}`);
	expect(await editor.getByTestId('share-embed').inputValue()).toContain(`src="${stack.base}/embed/${token}"`);
});

test('a visitor without a session watches it, as a page and as an embed, through the public API only', async () => {
	if (!url) return;
	const token = url.split('/v/')[1];
	await pictured(token);
	const context = await browser.newContext({ locale: 'fr-FR', viewport: { width: 1280, height: 720 } });
	const visitor = await context.newPage();
	const calls: string[] = [];
	visitor.on('request', (request) => calls.push(new URL(request.url()).pathname));
	await visitor.goto(url);
	expect(await visitor.getByTestId('player-title').textContent()).toBe(TITLE);
	expect(await visitor.getByTestId('made-with').getAttribute('href')).toBe('/');
	expect(await visitor.getByTestId('player-video').getAttribute('poster')).toMatch(/thumbnail|poster/);
	await visitor.waitForFunction(() => (document.querySelector('video')?.readyState ?? 0) >= 1, null, { timeout: 15_000 });
	await visitor.keyboard.press('Space');
	await waitData(visitor, 'player', 'playing', 'true');

	await visitor.goto(`${stack.base}/embed/${token}`);
	await visitor.getByTestId('player-video').waitFor();
	expect(await visitor.getByTestId('player-open').getAttribute('href')).toBe(`/v/${token}`);
	expect(await visitor.getByTestId('player-open').getAttribute('target')).toBe('_blank');
	expect(await visitor.getByTestId('made-with').count()).toBe(0);
	expect(calls.filter((path) => path.startsWith('/api/') && !path.startsWith('/api/public/'))).toEqual([]);
	expect(calls.some((path) => path === `/api/public/renders/${token}`)).toBe(true);
	expect(await context.cookies()).toEqual([]);
	await context.close();
});

test('turning the link off shows the dead page', async () => {
	if (!url) return;
	await editor.getByTestId('share-revoke').click();
	await confirmIn(editor, 'Désactiver le lien ?', 'Désactiver le lien');
	await waitData(editor, 'share-dialog', 'status', 'off');
	const visitor = await (await browser.newContext({ locale: 'fr-FR' })).newPage();
	await visitor.goto(url);
	await visitor.getByTestId('link-gone').getByText('Cette vidéo n’est plus disponible').waitFor({ timeout: 10_000 });
	await visitor.context().close();
});
