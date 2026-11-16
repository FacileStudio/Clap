import type { Catalogs, Message, Row } from '@facile/muse';
import { demo } from './demo';
import { page } from './page';
import { tiles } from './tiles';

const rows = { ...page, ...tiles, ...demo };

export type LandingKey = keyof typeof rows;
type LandingCatalog = Record<LandingKey, Message>;

function column(index: number): LandingCatalog {
	const entries = Object.entries(rows as Record<string, Row>).map(([key, row]) => [key, row[index]]);
	return Object.fromEntries(entries) as LandingCatalog;
}

/** The landing's catalogs: English is the typed base, and every row carries all four locales. */
export const catalogs: Catalogs<LandingCatalog> = {
	en: column(0),
	fr: column(1),
	es: column(2),
	de: column(3)
};
