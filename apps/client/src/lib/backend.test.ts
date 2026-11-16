import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend, request } from './backend';
import type { Op } from './types';

function stub(response: Response | Error) {
	const fetch = vi.fn(async () => {
		if (response instanceof Error) throw response;
		return response;
	});
	vi.stubGlobal('fetch', fetch);
	return fetch;
}

const sent = (fetch: ReturnType<typeof stub>) => fetch.mock.calls[0] as unknown as [string, RequestInit];

afterEach(() => vi.unstubAllGlobals());

describe('request', () => {
	it('sends cookies on every call and the CSRF header only on mutations', async () => {
		const fetch = stub(Response.json({ user: null }));
		await backend.session();
		const [url, init] = sent(fetch);
		expect(url).toBe('/api/auth/session');
		expect(init.credentials).toBe('include');
		expect(new Headers(init.headers).has('X-Facile-CSRF')).toBe(false);

		fetch.mockResolvedValueOnce(Response.json({}));
		await backend.logout();
		const [path, logout] = fetch.mock.calls[1] as unknown as [string, RequestInit];
		expect(path).toBe('/api/auth/logout');
		expect(new Headers(logout.headers).get('X-Facile-CSRF')).toBe('1');
	});

	it('throws the suite envelope as a typed ApiError', async () => {
		stub(Response.json({ error: { code: 'not_found', message: 'route not found' } }, { status: 404 }));
		const error = await request('/nope').catch((e: unknown) => e);
		expect(error).toBeInstanceOf(ApiError);
		expect(error).toMatchObject({ status: 404, code: 'not_found', message: 'route not found' });
	});

	it('maps the status to a code when the body is not the envelope', async () => {
		stub(new Response('<html>', { status: 429, statusText: 'Too Many Requests' }));
		await expect(request('/auth/session')).rejects.toMatchObject({ status: 429, code: 'rate_limited' });
	});

	it('reports a network failure as code network', async () => {
		stub(new TypeError('Failed to fetch'));
		await expect(request('/auth/session')).rejects.toMatchObject({ status: 0, code: 'network' });
	});

	it('answers undefined for 204', async () => {
		stub(new Response(null, { status: 204 }));
		await expect(request('/auth/logout', { method: 'POST' })).resolves.toBeUndefined();
	});
});

describe('backend', () => {
	it('sends timeline ops with the revision they were made at', async () => {
		const fetch = stub(Response.json({ source: 'x', rev: 4 }));
		const ops: Op[] = [{ op: 'move_clip', clip: 'c1', start: 2 }];
		expect(await backend.edit('p 1', 3, ops)).toEqual({ source: 'x', rev: 4 });
		const [url, init] = sent(fetch);
		expect([url, init.method, init.body]).toEqual(['/api/projects/p%201/edits', 'POST', JSON.stringify({ rev: 3, ops })]);
		expect(new Headers(init.headers).get('X-Facile-CSRF')).toBe('1');
	});

	it('keys a stale write on its status and carries the scene errors of a 422', async () => {
		stub(Response.json({ error: { code: 'already_exists', message: 'moved' }, rev: 7 }, { status: 409 }));
		await expect(backend.save('p', 's', 1)).rejects.toMatchObject({ status: 409, code: 'already_exists' });
		stub(Response.json({ error: { code: 'invalid_argument', message: 'bad' }, errors: [{ line: 4, message: 'unknown key' }] }, { status: 422 }));
		await expect(backend.save('p', 's', 1)).rejects.toMatchObject({ status: 422, errors: [{ line: 4, message: 'unknown key' }] });
	});

	it('carries why a timeline edit was refused', async () => {
		stub(Response.json({ error: { code: 'invalid_argument', message: 'overlap' }, timeline_error: 'overlap' }, { status: 422 }));
		await expect(backend.edit('p', 1, [])).rejects.toMatchObject({ status: 422, reason: 'overlap' });
	});

	it('builds the frame, events and render addresses', () => {
		expect(backend.frameUrl('p', { scene: 0, stamp: 3 })).toBe('/api/projects/p/frame?scene=0&v=3');
		expect(backend.frameUrl('p')).toBe('/api/projects/p/frame');
		expect([backend.eventsUrl('p'), backend.renderEventsUrl('r')]).toEqual(['/api/projects/p/events', '/api/renders/r/events']);
	});

	it('lists assets by their source path and shows an image as its own poster', async () => {
		const video = { name: 'assets/a.mp4', kind: 'video', url: '/v', poster_url: '/v?variant=poster' };
		const image = { name: 'assets/logo.png', kind: 'image', url: '/i', poster_url: null };
		const fetch = stub(Response.json({ assets: [video, image] }));
		expect((await backend.assets('p')).map((asset) => asset.poster_url)).toEqual(['/v?variant=poster', '/i']);
		fetch.mockResolvedValueOnce(new Response(null, { status: 204 }));
		await backend.removeAsset('p', 'assets/my clip.mp4');
		expect((fetch.mock.calls[1] as unknown as [string])[0]).toBe('/api/projects/p/assets/assets/my%20clip.mp4');
	});

	it('starts a render with its size, rate and range', async () => {
		const fetch = stub(Response.json({ id: 'r1' }, { status: 201 }));
		await backend.startRender('p', { width: 1280, height: 720, fps: 30, range: [1, 4] });
		const [url, init] = sent(fetch);
		expect([url, init.body]).toEqual(['/api/projects/p/renders', '{"width":1280,"height":720,"fps":30,"range":[1,4]}']);
	});
});

