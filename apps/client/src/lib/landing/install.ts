import type { CliCommand } from '@facile/muse';

/** The clap releases: one archive per platform beside checksums.txt. */
export const RELEASES = 'https://github.com/FacileStudio/Clap/releases';

/** The checksums of the newest release, which the installer verifies every archive against. */
export const CHECKSUMS = `${RELEASES}/latest/download/checksums.txt`;

/** The two install routes: kit, and the repository shim that bootstraps it. */
export const INSTALL: CliCommand[] = [
	{ label: 'kit', command: 'kit add clap' },
	{ label: 'curl', command: 'curl -fsSL https://raw.githubusercontent.com/FacileStudio/Clap/main/install.sh | bash' }
];
