import { join, normalize } from 'node:path';

const BUILD = join(import.meta.dir, '..', 'build');

/** A running copy of the built client for a signed-out visitor. */
export type Site = { base: string; stop: () => void };

function asset(pathname: string): string | null {
	const path = normalize(join(BUILD, decodeURIComponent(pathname)));
	return path.startsWith(BUILD) && !pathname.endsWith('/') ? path : null;
}

async function answer(request: Request): Promise<Response> {
	const { pathname } = new URL(request.url);
	if (pathname === '/api/auth/session') return Response.json({ user: null });
	if (pathname.startsWith('/api/')) return Response.json({ error: { code: 'unauthenticated' } }, { status: 401 });
	const path = asset(pathname);
	const file = path ? Bun.file(path) : null;
	if (file && (await file.exists())) return new Response(file);
	return new Response(Bun.file(join(BUILD, 'index.html')));
}

/** Serves the built client with the session route answering "nobody is signed in". */
export function serveBuild(port = 0): Site {
	const server = Bun.serve({ port, hostname: '127.0.0.1', fetch: answer });
	return { base: `http://127.0.0.1:${server.port}`, stop: () => void server.stop(true) };
}
