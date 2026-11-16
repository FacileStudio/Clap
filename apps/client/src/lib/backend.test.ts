import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend, request } from './backend';

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
