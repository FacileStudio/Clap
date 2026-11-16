import { rmSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { outline } from '../src/lib/timeline/outline';
import { servedTimeline } from './mock-look';
import { events, tell } from './mock-events';
import { frameResponse } from './mock-frame';
import { edit, json, refuse, save, type Project } from './mock-writes';
import { assetRoute, projectRenders, renderRoute, scratch } from './mock-media';


const clientDir = resolve(import.meta.dir, '../build');
const projects = new Map<string, Project>();

const starter = (title: string) => `---
title: ${title}
size: 1920x1080
layout: raw
duration: 6
---

<div class="clip" data-start="0" data-duration="6" data-track-index="0" style="position:absolute;inset:0;display:flex;align-items:center;justify-content:center">
  <h1 style="margin:0;font-size:140px;letter-spacing:-3px">${title}</h1>
</div>
`;

async function one(request: Request, project: Project, rest: string[], url: URL): Promise<Response> {
	const [part, name] = rest;
	const seconds = outline(project.source).scenes.reduce((sum, scene) => sum + scene.duration, 0);
	if (part === 'source') return save(project, await request.json());
	if (part === 'edits') return edit(project, await request.json());
	if (part === 'timeline') return json({ rev: project.rev, timeline: servedTimeline(project.source) });
	if (part === 'events') return events(project.id, project.rev);
	if (part === 'frame') return frameResponse(project, url.searchParams.get('scene'));
	if (part === 'assets') return assetRoute(request, project.id, name && decodeURIComponent(rest.slice(1).join('/')), tell(project.id));
	if (part === 'renders') return projectRenders(request, project.id, seconds, tell(`render:${project.id}`));
	if (request.method === 'DELETE') projects.delete(project.id);
	if (request.method === 'PATCH') Object.assign(project, await request.json());
	return json(project);
}

async function collection(request: Request, url: URL): Promise<Response> {
	if (request.method === 'GET') {
		const trashed = url.searchParams.get('trashed') === 'true';
		const shown = [...projects.values()].filter((project) => project.trashed === trashed);
		return json(shown.map(({ id, title, rev, updated_at }) => ({ id, title, rev, updated_at })).reverse());
	}
	const { title } = (await request.json()) as { title: string };
	const project = { id: crypto.randomUUID(), title, source: starter(title), rev: 1, updated_at: new Date().toISOString(), trashed: false };
	projects.set(project.id, project);
	return json(project, 201);
}

async function api(request: Request, url: URL): Promise<Response> {
	const [head, id, ...rest] = url.pathname.slice('/api/'.length).split('/');
	if (head === 'auth') return json({ user: { email: 'alice@example.test', name: 'Alice Martin' }, bulle_url: 'http://127.0.0.1:9' });
	if (head === 'assistant') return json({ name: 'Bulle', accent: null });
	if (head === 'mcp') return json({ actions: [] });
	if (head === 'renders' && rest[0] === 'events') return new Response(': open\n\n', { headers: { 'content-type': 'text/event-stream' } });
	if (head === 'renders') return renderRoute(request, id);
	if (head !== 'projects') return refuse(404, 'not_found', 'route not found');
	if (!id) return collection(request, url);
	const project = projects.get(id);
	return project ? one(request, project, rest, url) : refuse(404, 'not_found', 'project not found');
}

/** Serves the built client and an in-memory API that follows the Clap contract. It compiles nothing and renders nothing. */
export function startMock(port = 0): { base: string; stop: () => void } {
	const server = Bun.serve({
		port,
		hostname: '127.0.0.1',
		idleTimeout: 0,
		async fetch(request) {
			const url = new URL(request.url);
			if (url.pathname.startsWith('/api/')) return api(request, url);
			const file = Bun.file(join(clientDir, url.pathname));
			return new Response((await file.exists()) ? file : Bun.file(join(clientDir, 'index.html')));
		}
	});
	const stop = () => {
		server.stop(true);
		rmSync(scratch, { recursive: true, force: true });
	};
	return { base: `http://127.0.0.1:${server.port}`, stop };
}

if (import.meta.main) console.log(startMock(Number(process.env.PORT ?? 4010)).base);
