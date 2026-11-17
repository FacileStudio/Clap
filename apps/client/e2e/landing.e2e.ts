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

test('the downloads page offers the install commands and the release files', async () => {
	const page = await visit('/downloads', 'fr', 390);
	const links = await page.locator('main a').evaluateAll((all) => all.map((a) => a.getAttribute('href')));
	expect(links).toContain('https://github.com/FacileStudio/Clap/releases');
	expect(links).toContain('https://github.com/FacileStudio/Clap/releases/latest/download/checksums.txt');
	expect(await page.locator('main').innerText()).toContain('kit add clap');
	await page.context().close();
});

test('the hero editor shows the clip settings, never a Source tab, in every locale', async () => {
	const words: Record<string, string> = { fr: 'Texte', en: 'Text', es: 'Texto', de: 'Text' };
	for (const locale of ['fr', 'en', 'es', 'de']) {
		const page = await visit('/', locale, 1440);
		const demo = page.locator('[role="img"]:has(.clap-clip)').first();
		await demo.waitFor();
		const label = (await demo.getAttribute('aria-label')) ?? '';
		const text = (await demo.textContent()) ?? '';
		expect(label, locale).not.toMatch(/source|fuente|quelltext/i);
		expect(text, locale).not.toMatch(/Source|Fuente|Quelltext/);
		expect(text, locale).toContain(words[locale]);
		await page.context().close();
	}
});
