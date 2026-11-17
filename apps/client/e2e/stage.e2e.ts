import { afterAll, beforeAll, expect, test } from 'bun:test';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { chromium, type Browser, type Page } from 'playwright-core';
import { dragFile } from './drops';
import { api, dragBy, openPage, settled, waitData, type Stack } from './pages';
import { startStack } from './stack';

const scratch = mkdtempSync(join(tmpdir(), 'clap-stage-'));
const still = join(scratch, 'logo.png');
const ffmpeg = Bun.which('ffmpeg');
const SRC = 'assets/logo.png';

let stack: Stack;
let browser: Browser;
let editor: Page;
let id = '';

const stored = async () => (await api<{ source: string; rev: number }>(stack, `/projects/${id}`)).source;
const withPicture = ffmpeg ? test : test.skip;

function attribute(source: string, name: string): number {
	const tag = new RegExp(`<[^>]*${SRC.replace('.', '\\.')}[^>]*>`).exec(source)?.[0] ?? '';
	return Number(new RegExp(`\\s${name}="([^"]*)"`).exec(tag)?.[1] ?? Number.NaN);
}

async function stageSize(): Promise<{ width: number; height: number; scale: number }> {
	const overlay = editor.getByTestId('stage-overlay');
	const box = await overlay.boundingBox();
	const width = Number(await overlay.getAttribute('data-width'));
	const height = Number(await overlay.getAttribute('data-height'));
	if (!box) throw new Error('the stage is not drawn');
	return { width, height, scale: box.width / width };
}

beforeAll(async () => {
	if (ffmpeg) {
		const made = Bun.spawnSync([ffmpeg, '-v', 'error', '-y', '-f', 'lavfi', '-i', 'color=c=blue:s=64x36', '-frames:v', '1', still]);
		if (made.exitCode !== 0) throw new Error(`ffmpeg could not make the test picture: ${made.stderr.toString()}`);
	} else console.warn('e2e: SKIPPED the stage tests: ffmpeg is not installed');
	stack = await startStack();
	browser = await chromium.launch({ headless: true, executablePath: process.env.CLAP_CHROME || undefined });
	id = (await api<{ id: string }>(stack, '/projects', { method: 'POST', body: { title: 'Scène e2e' } })).id;
	editor = await openPage(browser, stack, `/projects/${id}`);
	await editor.getByTestId('scene').first().waitFor({ timeout: 15_000 });
}, 180_000);

afterAll(async () => {
	await browser?.close();
	await stack?.stop();
	rmSync(scratch, { recursive: true, force: true });
});

withPicture('a picture dropped at the stage centre lands centred, fitted whole', async () => {
	const file = { name: 'logo.png', type: 'image/png', bytes: new Uint8Array(await Bun.file(still).arrayBuffer()) };
	const { width, height, scale } = await stageSize();
	await dragFile(editor.getByTestId('stage-overlay'), file, { x: (width / 2) * scale, y: (height / 2) * scale });
	await editor.locator('[data-testid="clip"][data-kind="image"]').waitFor({ timeout: 15_000 });
	await settled(editor);
	const source = await stored();
	expect(Math.abs(attribute(source, 'data-x') + attribute(source, 'data-w') / 2 - width / 2)).toBeLessThanOrEqual(2);
	expect(Math.abs(attribute(source, 'data-y') + attribute(source, 'data-h') / 2 - height / 2)).toBeLessThanOrEqual(2);
	expect(source).toMatch(new RegExp(`${SRC}[^>]*data-fit="contain"`));
});

withPicture('dragging the picture 100 px right moves its data-x by 100, and undo puts it back', async () => {
	const before = attribute(await stored(), 'data-x');
	const { scale } = await stageSize();
	const box = editor.locator('[data-testid="stage-box"][data-clip="image-1"]');
	await box.waitFor({ timeout: 10_000 });
	await editor.keyboard.down('Alt');
	await dragBy(editor, box, 100 * scale);
	await editor.keyboard.up('Alt');
	await settled(editor);
	await waitData(editor, 'save-status', 'status', 'clean');
	expect(Math.abs(attribute(await stored(), 'data-x') - before - 100)).toBeLessThanOrEqual(1);
	await editor.getByTestId('undo').click();
	await settled(editor);
	expect(attribute(await stored(), 'data-x')).toBe(before);
});

withPicture('the clip inspector sets the opacity', async () => {
	await editor.locator('[data-testid="clip"][data-kind="image"]').click();
	await editor.getByRole('tab', { name: 'Clip' }).click();
	const field = editor.getByTestId('inspector-opacity');
	await field.fill('50');
	await field.press('Enter');
	await settled(editor);
	expect(attribute(await stored(), 'data-opacity')).toBe(0.5);
});

withPicture('the stage menu fills the frame with the picture, drawn with Toile\'s knobs', async () => {
	const { width } = await stageSize();
	const box = editor.locator('[data-testid="stage-box"][data-clip="image-1"]');
	await editor.locator('[data-testid="clip"][data-kind="image"]').click();
	await editor.getByTestId('stage-selection').waitFor({ timeout: 5_000 });
	expect(await editor.locator('[data-testid="stage-handle"]').count()).toBe(8);
	await box.click({ button: 'right', force: true });
	await editor.getByRole('menuitem', { name: 'Remplir le cadre' }).click();
	await settled(editor);
	const source = await stored();
	expect([attribute(source, 'data-x'), attribute(source, 'data-w')]).toEqual([0, width]);
	expect(source).toMatch(new RegExp(`${SRC}[^>]*data-fit="cover"`));
});

test('a scene transition picked on the timeline is written into the source', async () => {
	const theirs = await api<{ source: string; rev: number }>(stack, `/projects/${id}`);
	const body = { source: `${theirs.source.trimEnd()}\n\n---\nlayout: raw\nduration: 2\n---\n\n`, rev: theirs.rev };
	await api(stack, `/projects/${id}/source`, { method: 'PUT', body });
	const chip = editor.getByTestId('transition-chip').first();
	await chip.waitFor({ timeout: 15_000 });
	await settled(editor);
	await chip.click();
	await editor.locator('[data-testid="transition-choice"][data-choice="dip"]').click();
	await settled(editor);
	expect(await stored()).toMatch(/^transition: dip$/m);
	await waitData(editor, 'transition-chip', 'transition', 'dip');
});
