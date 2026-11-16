import { registerIcons } from '@facile/muse';

/** The brand mark of Clap: a clapperboard, drawn in the linear style of the suite's icons. */
export const logoIcon = 'clap:logo';

const stroke = 'fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5"';
const slate = 'M3.5 10.5h17v6c0 2.400-1.100 3.500-3.500 3.500H7c-2.400 0-3.500-1.100-3.500-3.500Z';
const arm = 'm3.500 10.500-.8-3.400c-.3-1.300.4-2.500 1.700-2.800l12.200-2.800c1.300-.3 2.500.4 2.800 1.700l.4 1.700Z';
const stripes = 'm8.200 4.900 2.900 3.800M13.200 3.800l2.900 3.800';

registerIcons({
	[logoIcon]: {
		body: `<g ${stroke}><path d="${slate}"/><path d="${arm}"/><path d="${stripes}"/></g>`,
		width: 24,
		height: 24
	}
});
