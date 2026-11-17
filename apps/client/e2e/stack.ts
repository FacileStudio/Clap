import { SQL, type Subprocess } from 'bun';
import { existsSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { startMock } from './mock';
import { finish, heartbeat } from './worker';
import type { Stack } from './pages';

const root = resolve(import.meta.dir, '../../..');
const target = process.env.CARGO_TARGET_DIR ?? resolve(root, 'target');
const api = resolve(target, 'debug/api');
const seeder = resolve(target, 'debug/examples/e2e_seed');
const worker = resolve(target, 'debug/worker');
const clientDir = resolve(root, 'apps/client/build');
const admin = process.env.E2E_ADMIN_URL ?? 'postgres://postgres:postgres@127.0.0.1:5672/postgres?sslmode=disable';
const database = process.env.E2E_DATABASE ?? 'clap_e2e';
if (!/^[a-z_][a-z0-9_]*$/.test(database)) throw new Error(`E2E_DATABASE must be a plain Postgres name, not ${database}`);

function freePort(): number {
	const probe = Bun.serve({ port: 0, fetch: () => new Response() });
	const port = probe.port ?? 0;
	probe.stop(true);
	return port;
}

async function freshDatabase(): Promise<string> {
	const sql = new SQL(admin);
	await sql.unsafe(`DROP DATABASE IF EXISTS ${database} WITH (FORCE)`);
	await sql.unsafe(`CREATE DATABASE ${database}`);
	await sql.close();
	const url = new URL(admin);
	url.pathname = `/${database}`;
	return url.toString();
}

async function ready(base: string, server: Subprocess): Promise<void> {
	for (let attempt = 0; attempt < 300; attempt += 1) {
		if (server.exitCode !== null) throw new Error(`the API exited with ${server.exitCode}`);
		const answer = await fetch(`${base}/ready`).catch(() => null);
		if (answer?.ok) return;
		await Bun.sleep(100);
	}
	throw new Error('the API did not become ready in 30 seconds');
}

async function seed(env: Record<string, string>): Promise<string> {
	const run = Bun.spawn([seeder], { env: { ...process.env, ...env }, stdout: 'pipe', stderr: 'inherit' });
	const out = await new Response(run.stdout).text();
	if ((await run.exited) !== 0) throw new Error('e2e_seed failed');
	return (JSON.parse(out) as { cookie: string }).cookie;
}

/**
 * Starts the real worker binary on media jobs only, so uploads are probed and given their posters, filmstrips and
 * waveforms as in production; renders stay for `finish` to play. `undefined` when the worker is not built.
 */
function startMedia(dataDir: string): Subprocess | undefined {
	if (!existsSync(worker)) {
		console.warn('e2e: no worker binary, uploads stay probing (cargo build -p worker)');
		return undefined;
	}
	const env = {
		CLAP_RENDERS_DIR: join(dataDir, 'renders'),
		CLAP_WORKER_JOBS: 'media',
		CLAP_RENDER_POLL_MS: '200',
		CLAP_CHROME_SANDBOX: 'off'
	};
	return Bun.spawn([worker], { env: { ...process.env, ...env }, stdout: 'inherit', stderr: 'inherit' });
}

async function startApi(): Promise<Stack> {
	const port = freePort();
	const base = `http://127.0.0.1:${port}`;
	const dataDir = mkdtempSync(join(tmpdir(), 'clap-e2e-data-'));
	const env = {
		DATABASE_URL: await freshDatabase(),
		PORT: String(port),
		CLIENT_DIR: clientDir,
		DATA_DIR: dataDir,
		PUBLIC_URL: base,
		OIDC_ISSUER: '',
		RUST_LOG: process.env.RUST_LOG ?? 'warn'
	};
	const server = Bun.spawn([api], { env: { ...process.env, ...env }, stdout: 'inherit', stderr: 'inherit' });
	await ready(base, server);
	const beat = setInterval(() => heartbeat(dataDir), 5000);
	heartbeat(dataDir);
	const media = startMedia(dataDir);
	const cookie = await seed(env);
	const stop = async () => {
		clearInterval(beat);
		media?.kill('SIGTERM');
		server.kill('SIGTERM');
		await Promise.all([server.exited, media?.exited]);
		rmSync(dataDir, { recursive: true, force: true });
	};
	return { base, cookie, stop, finish: (id, mp4) => finish(dataDir, id, mp4) };
}

/**
 * Starts the stack the browser tests run against. With the API and its seeder built (`e2e/run.sh` builds them)
 * that is the real API on a fresh database; `E2E_STACK=mock`, or a missing binary, gives the in-memory API of
 * `mock.ts` instead, and says so.
 */
export async function startStack(): Promise<Stack> {
	const built = existsSync(api) && existsSync(seeder);
	if (process.env.E2E_STACK !== 'mock' && built) return startApi();
	console.warn('e2e: running against the in-memory MOCK API, not the real one');
	const mock = startMock();
	return { base: mock.base, cookie: null, stop: mock.stop };
}
