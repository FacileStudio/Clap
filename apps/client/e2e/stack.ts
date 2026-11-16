import { startMock } from './mock';
import type { Stack } from './pages';

/** Starts the stack the browser tests run against: the built client on the in-memory API of `mock.ts`. */
export async function startStack(): Promise<Stack> {
	const mock = startMock();
	return { base: mock.base, cookie: null, stop: mock.stop };
}
