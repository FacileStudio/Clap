import { tick } from 'svelte';
import type { Act, CursorState } from '@facile/mise-en-scene';
import { FRAMES, SCENE, titleLines, type EditorScene } from './scene.svelte';

/** Everything the hero story moves: the scene's state, the cursor, and the parts the cursor touches. */
export interface Cast {
	scene: EditorScene;
	readonly cursor: CursorState;
	readonly headline: string;
	readonly media?: HTMLElement;
	readonly rows?: HTMLElement;
	readonly clip?: HTMLElement;
	readonly sourceTab?: HTMLElement;
	readonly renderButton?: HTMLElement;
}

const GRIP = 26;
const along = (seconds: number) => seconds / SCENE.duration;

async function drop(act: Act, { scene, cursor, media, rows }: Cast): Promise<void> {
	cursor.visible = true;
	await act.move(cursor, act.point(media, 0.5, 0.4), 1000);
	cursor.down = true;
	scene.dragging = true;
	await act.wait(260);
	const target = act.point(rows, along(SCENE.videoStart), 0.5);
	await act.move(cursor, { x: target.x + GRIP, y: target.y });
	cursor.down = false;
	scene.dragging = false;
	scene.placed = true;
	await act.wait(700);
}

async function trim(act: Act, { scene, cursor, rows, clip }: Cast): Promise<void> {
	await tick();
	const edge = act.point(clip, 1, 0.5);
	await act.move(cursor, { x: edge.x - 3, y: edge.y }, 850);
	cursor.down = true;
	await act.wait(200);
	scene.length = SCENE.videoCut;
	cursor.x = act.point(rows, along(SCENE.videoStart + SCENE.videoCut), 0.5).x - 3;
	await act.wait(700);
	scene.guide = true;
	await act.wait(500);
	cursor.down = false;
	scene.guide = false;
	await act.wait(350);
}

async function write(act: Act, { scene, cursor, sourceTab, headline }: Cast): Promise<void> {
	await act.move(cursor, act.point(sourceTab, 0.6, 0.6));
	await act.click(cursor, () => (scene.tab = 'source'));
	await act.wait(450);
	cursor.visible = false;
	await act.type(titleLines(headline), (value) => (scene.typed = value), 30);
	scene.titled = true;
	await act.wait(1300);
}

async function render(act: Act, { scene, cursor, renderButton }: Cast): Promise<void> {
	cursor.visible = true;
	await act.move(cursor, act.point(renderButton, 0.55, 0.6));
	await act.click(cursor, () => (scene.job = 'running'));
	cursor.visible = false;
	for (let frame = 12; frame <= FRAMES; frame += 12) {
		scene.frame = frame;
		await act.wait(70);
	}
	scene.job = 'done';
	await act.wait(2400);
}

/** The hero story: drop a clip, trim it, write a title in the source, render to an MP4. */
export async function play(act: Act, cast: Cast): Promise<void> {
	await act.wait(700);
	await drop(act, cast);
	await trim(act, cast);
	await write(act, cast);
	await render(act, cast);
}
