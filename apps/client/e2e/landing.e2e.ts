import { afterAll, beforeAll, expect, test } from 'bun:test';
import { join } from 'node:path';
import { chromium, type Browser, type Page } from 'playwright-core';
import { serveBuild, type Site } from './landing-site';

const WIDTHS = [360, 390, 768, 1024, 1440];
const CATALOG = Bun.file(join(import.meta.dir, 'catalog.json'));

let site: Site;
let browser: Browser;

/** A signed-out page at `path`, in `locale`, `width` pixels wide, with the plan catalog answered from the fixture. */
async function visit(path: string, locale: string, width: number): Promise<Page> {
	const context = await browser.newContext({ locale, viewport: { width, height: 900 }, reducedMotion: 'reduce' });
	await context.addCookies([{ name: 'fc_locale', value: locale, url: site.base }]);
	const page = await context.newPage();
	await page.route(/^https?:\/\/(?!127\.0\.0\.1)/, (route) =>
		route.request().url().endsWith('/api/catalog')
			? route.fulfill({ contentType: 'application/json', path: CATALOG.name })
			: route.abort()
	);
	await page.goto(`${site.base}${path}`);
	await page.locator('h1').waitFor();
	return page;
}

beforeAll(async () => {
	site = serveBuild();
	browser = await chromium.launch({ headless: true, executablePath: process.env.CLAP_CHROME || undefined });
}, 60_000);

afterAll(async () => {
	await browser?.close();
	site?.stop();
});

test('the landing and the downloads page never scroll sideways', async () => {
	for (const path of ['/', '/downloads']) {
		for (const width of WIDTHS) {
			const page = await visit(path, 'fr', width);
			await page.evaluate(() => window.scrollTo(0, document.documentElement.scrollHeight));
			const wide = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
			expect(wide, `${path} at ${width}px`).toBeLessThanOrEqual(0);
			await page.context().close();
		}
	}
}, 120_000);

test('every sign-up and pricing button opens the sign-up page in the visitor locale', async () => {
	for (const locale of ['fr', 'en', 'es', 'de']) {
		const page = await visit('/', locale, 1440);
		const signup = `https://sso.facile.studio/signup?lang=${locale}`;
		await page.locator('#pricing a').first().waitFor();
		for (const interval of ['month', 'year']) {
			await page.locator(`#pricing input[type=radio][value=${interval}]`).check({ force: true });
			const plans = await page.locator('#pricing div.grid a').evaluateAll((links) => links.map((a) => a.getAttribute('href')));
			expect(plans.length, `${locale} ${interval}`).toBeGreaterThanOrEqual(3);
			for (const href of plans) expect(href, `${locale} ${interval}`).toBe(signup);
		}
		const hero = await page.locator('main section').first().locator('a').first().getAttribute('href');
		expect(hero).toBe(signup);
		expect(await page.locator('html').getAttribute('lang')).toBe(locale);
		await page.context().close();
	}
}, 120_000);

test('the downloads page offers no download while the command is unpublished', async () => {
	const page = await visit('/downloads', 'fr', 390);
	const links = await page.locator('main a').evaluateAll((all) => all.map((a) => a.getAttribute('href')));
	expect(links).toEqual(['/', 'https://github.com/FacileStudio/Clap']);
	await page.context().close();
});
