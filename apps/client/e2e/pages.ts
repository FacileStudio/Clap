import type { Browser, Locator, Page } from 'playwright-core';

/** What a running stack hands the tests: the app's URL and the session cookie of the signed-in person, if it needs one. */
export type Stack = {
	base: string;
	cookie: string | null;
	stop: () => Promise<void> | void;
	/** Plays the render worker for render `id`: writes `mp4` as its output and reports it done. Real API only. */
	finish?: (id: string, mp4: string) => void;
};

/** A French browser page at `path`, signed in, `width` pixels wide. */
export async function openPage(browser: Browser, stack: Stack, path: string, width = 1440): Promise<Page> {
	const context = await browser.newContext({ locale: 'fr-FR', viewport: { width, height: 900 }, acceptDownloads: true });
	if (stack.cookie) await context.addCookies([{ name: 'session', value: stack.cookie, url: stack.base }]);
	const page = await context.newPage();
	await page.goto(`${stack.base}${path}`);
	return page;
}

/** Calls the API as the signed-in person, with the CSRF header a browser write carries. */
export async function api<T>(stack: Stack, path: string, init: { method?: string; body?: unknown } = {}): Promise<T> {
	const answer = await fetch(`${stack.base}/api${path}`, {
		method: init.method ?? 'GET',
		headers: {
			...(stack.cookie ? { cookie: `session=${stack.cookie}` } : {}),
			'content-type': 'application/json',
			'x-facile-csrf': '1'
		},
		body: init.body === undefined ? undefined : JSON.stringify(init.body)
	});
	if (!answer.ok) throw new Error(`${path}: ${answer.status} ${await answer.text()}`);
	return (await answer.json()) as T;
}

/** Waits until the element `testid` carries `data-<name>="<value>"`. */
export async function waitData(page: Page, testid: string, name: string, value: string | number): Promise<void> {
	await page.locator(`[data-testid="${testid}"][data-${name}="${value}"]`).first().waitFor({ timeout: 15_000 });
}

/** Waits until the timeline has no edit in flight. */
export async function settled(page: Page): Promise<void> {
	await waitData(page, 'timeline', 'busy', 'false');
}

/** Drags `target` by `dx`, `dy` pixels with the mouse, in steps, the way a hand does. */
export async function dragBy(page: Page, target: Locator, dx: number, dy = 0): Promise<void> {
	const box = await target.boundingBox();
	if (!box) throw new Error('nothing to drag');
	const [x, y] = [box.x + box.width / 2, box.y + box.height / 2];
	await page.mouse.move(x, y);
	await page.mouse.down();
	await page.mouse.move(x + dx, y + dy, { steps: 8 });
	await page.mouse.up();
}

/** The `data-start` the stored source gives the first element that loads `src`. */
export function startOf(source: string, src: string): number {
	return attributeOf(source, src, 'data-start');
}

/** The number the attribute `name` holds on the first element of the stored source that loads `src`. */
export function attributeOf(source: string, src: string, name: string): number {
	const tag = new RegExp(`<[^>]*${src.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}[^>]*>`).exec(source)?.[0] ?? '';
	return Number(new RegExp(`${name}="([^"]*)"`).exec(tag)?.[1] ?? Number.NaN);
}
