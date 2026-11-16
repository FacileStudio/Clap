import { describe, expect, it } from 'vitest';
import { bindLocale, pickLocale, t } from './index';
import { catalogs } from './messages';

describe('i18n', () => {
	it('carries every message in the four locales, with the same placeholders', () => {
		const names = (text: unknown) => (JSON.stringify(text).match(/\{\w+\}/g) ?? []).sort().join();
		for (const key of Object.keys(catalogs.en) as (keyof typeof catalogs.en)[]) {
			for (const locale of ['fr', 'es', 'de'] as const) {
				expect(catalogs[locale][key], `${locale} ${key}`).toBeTruthy();
				expect(names(catalogs[locale][key]), `${locale} ${key}`).toBe(names(catalogs.en[key]));
			}
		}
	});

	it('defaults to French and follows the bound locale, plurals included', () => {
		expect(t('projects.title')).toBe('Projets');
		expect([pickLocale(['pt-BR', 'de-AT']), pickLocale(['ja', null])]).toEqual(['de', 'fr']);
		bindLocale(() => 'en');
		expect([t('bulle.ask', { name: 'Bulle' }), t('timeline.scene', { scene: 3 })]).toEqual(['Ask Bulle', 'Scene 3']);
		bindLocale(() => 'fr');
	});
});
