import { mkdirSync, mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import type { Asset, Render } from '../src/lib/types';

type Tell = (event: string, data: unknown) => void;

/** The scratch folder uploaded files are written to. */
export const scratch = mkdtempSync(join(tmpdir(), 'clap-mock-'));

const assets = new Map<string, Asset[]>();
const renders = new Map<string, Render & { project: string }>();
const json = (body: unknown, status = 200) => Response.json(body, { status });

function probe(path: string): number | null {
	const args = ['ffprobe', '-v', 'error', '-show_entries', 'format=duration', '-of', 'csv=p=0', path];
	try {
		const seconds = Number(Bun.spawnSync(args).stdout.toString().trim());
		return Number.isFinite(seconds) && seconds > 0 ? Math.round(seconds * 1000) : null;
	} catch {
		return null;
	}
}

async function store(project: string, file: File, tell: Tell): Promise<Asset> {
	const name = `assets/${file.name}`;
	const path = join(scratch, `${project}-${file.name}`);
	mkdirSync(scratch, { recursive: true });
	writeFileSync(path, new Uint8Array(await file.arrayBuffer()));
	const url = `/api/projects/${project}/assets/${encodeURI(name)}`;
	const kind = file.type.split('/')[0];
	const poster = kind === 'image' ? url : null;
	const asset: Asset = { name, kind, size: file.size, duration_ms: null, width: null, height: null, state: 'probing', poster_url: poster, url };
	assets.set(project, [...(assets.get(project) ?? [])
		.filter((known) => known.name !== name), asset]);
	setTimeout(() => {
		Object.assign(asset, { state: 'ready', duration_ms: kind === 'image' ? null : (probe(path) ?? 2000) });
		tell('asset', { name: asset.name, state: asset.state });
	}, 150);
	return asset;
}

/** `/projects/{id}/assets[/{name}]`: list, upload, read and delete, with the probing state turning ready shortly after. */
export async function assetRoute(request: Request, project: string, name: string | undefined, tell: Tell): Promise<Response> {
	const list = assets.get(project) ?? [];
	if (request.method === 'POST') return json(await store(project, (await request.formData()).get('file') as File, tell), 201);
	if (request.method === 'GET' && !name) return json({ assets: list });
	if (request.method === 'GET') return new Response(Bun.file(join(scratch, `${project}-${name.split('/').pop()}`)));
	assets.set(
		project,
		list.filter((asset) => asset.name !== name)
	);
	return new Response(null, { status: 204 });
}

function advance(job: Render, tell: Tell): void {
	if (job.state === 'cancelled') return;
	job.state = 'running';
	job.frame = Math.min(job.frames, job.frame + Math.ceil(job.frames / 20));
	if (job.frame >= job.frames) job.state = 'done';
	else setTimeout(() => advance(job, tell), 250);
	tell('progress', job);
}

function start(project: string, body: { fps?: number; range?: [number, number] }, seconds: number, tell: Tell): Response {
	const span = body.range ? body.range[1] - body.range[0] : seconds;
	const id = crypto.randomUUID();
	const created_at = new Date().toISOString();
	const job = { id, project, state: 'queued' as const, frame: 0, frames: Math.round(span * (body.fps ?? 30)), error: null, download_url: null, created_at };
	renders.set(id, job);
	setTimeout(() => advance(job, tell), 400);
	return json(job, 202);
}

/** `/projects/{id}/renders`: start a job that counts its frames up by itself, or list the project's jobs. */
export async function projectRenders(request: Request, project: string, seconds: number, tell: Tell): Promise<Response> {
	if (request.method === 'POST') return start(project, await request.json(), seconds, tell);
	return json({ renders: [...renders.values()].filter((job) => job.project === project).reverse() });
}

/** `/renders/{id}`: read or cancel. */
export function renderRoute(request: Request, id: string): Response {
	const job = renders.get(id);
	if (!job) return json({ error: { code: 'not_found', message: 'render not found' } }, 404);
	if (request.method === 'DELETE') job.state = job.state === 'done' ? 'done' : 'cancelled';
	return request.method === 'DELETE' ? new Response(null, { status: 204 }) : json(job);
}
