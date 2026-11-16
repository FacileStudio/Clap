import { afterEach, describe, expect, it, vi } from 'vitest';
import { ApiError, backend } from '$lib/backend';
import type { Render } from '$lib/types';
import { Sharing, embedSnippet, shareToken } from './sharing.svelte';
import { Watching } from './watching.svelte';

afterEach(() => vi.restoreAllMocks());

const URL_ONCE = 'https://clap.test/v/tok_EN-1';
const done = { id: 'r1', state: 'done' } as Render;

describe('Sharing', () => {
	it('reads the state, and shows the URL only from the call that created the link', async () => {
		vi.spyOn(backend, 'share').mockResolvedValue({ shared: false, created_at: null });
		vi.spyOn(backend, 'renders').mockResolvedValue([done]);
		const create = vi.spyOn(backend, 'createShare').mockResolvedValue({ shared: true, created_at: 'now', url: URL_ONCE });
		const sharing = new Sharing('p1');
		expect(sharing.status).toBe('loading');
		await sharing.load();
		expect([sharing.status, sharing.url, sharing.rendered]).toEqual(['off', null, true]);
		await sharing.create();
		expect([sharing.status, sharing.url, create.mock.calls[0]]).toEqual(['on', URL_ONCE, ['p1', false]]);
		sharing.forget();
		expect([sharing.status, sharing.url]).toEqual(['on', null]);
	});

	it('knows a project with no finished render has nothing to show, and when it cannot tell', async () => {
		vi.spyOn(backend, 'share').mockResolvedValue({ shared: true, created_at: 'then' });
		const renders = vi.spyOn(backend, 'renders').mockResolvedValue([{ ...done, state: 'running' }]);
		const sharing = new Sharing('p1');
		await sharing.load();
		expect([sharing.status, sharing.rendered]).toEqual(['on', false]);
		renders.mockRejectedValue(new ApiError(0, 'network', 'offline'));
		await sharing.load();
		expect(sharing.rendered).toBeNull();
	});

	it('rotates to a new URL and revokes back to off', async () => {
		const create = vi.spyOn(backend, 'createShare').mockResolvedValue({ shared: true, created_at: 'now', url: URL_ONCE });
		vi.spyOn(backend, 'revokeShare').mockResolvedValue(undefined);
		const sharing = new Sharing('p1');
		await sharing.create(true);
		expect([create.mock.calls[0], sharing.url]).toEqual([['p1', true], URL_ONCE]);
		await sharing.revoke();
		expect([sharing.status, sharing.url, sharing.createdAt]).toEqual(['off', null, null]);
	});

	it('keeps its state and frees the buttons when a call fails, and marks a failed read', async () => {
		vi.spyOn(backend, 'share').mockRejectedValue(new ApiError(0, 'network', 'offline'));
		vi.spyOn(backend, 'renders').mockResolvedValue([]);
		vi.spyOn(backend, 'revokeShare').mockRejectedValue(new ApiError(404, 'not_found', 'not shared'));
		const sharing = new Sharing('p1');
		await sharing.load();
		expect(sharing.status).toBe('failed');
		sharing.status = 'on';
		await expect(sharing.revoke()).rejects.toMatchObject({ status: 404 });
		expect([sharing.status, sharing.busy]).toEqual(['on', false]);
	});
});

describe('embedSnippet', () => {
	it('frames /embed/<token> of the same origin at the video ratio', () => {
		expect(shareToken(URL_ONCE)).toBe('tok_EN-1');
		expect(embedSnippet(URL_ONCE, 'Q3 "teaser" <b>', 1920, 1080)).toBe(
			'<iframe src="https://clap.test/embed/tok_EN-1" width="960" height="540" title="Q3 teaser b" frameborder="0" allow="autoplay; fullscreen; picture-in-picture" allowfullscreen></iframe>'
		);
	});

	it('answers nothing for a URL that is not a share link', () => {
		expect([shareToken('https://clap.test/projects/p1'), shareToken('nope')]).toEqual([null, null]);
		expect(embedSnippet('https://clap.test/p/x', 'x', 16, 9)).toBe('');
	});
});

describe('Watching', () => {
	it('calls only the public API, without cookies', async () => {
		const fetch = vi.fn(async () => Response.json({ title: 'T', duration_ms: 1000, width: 2, height: 2, poster: null, video: '/v' }));
		vi.stubGlobal('fetch', fetch);
		const watching = new Watching();
		await watching.load('tok');
		const [url, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
		expect([url, init.credentials, watching.video?.title]).toEqual(['/api/public/renders/tok', 'omit', 'T']);
		vi.unstubAllGlobals();
	});

	it('calls a 404 dead, and a 429 or a network failure worth a retry', async () => {
		const read = vi.spyOn(backend, 'publicRender').mockRejectedValue(new ApiError(404, 'not_found', 'gone'));
		const watching = new Watching();
		await watching.load('tok');
		expect([watching.gone, watching.failed]).toEqual([true, false]);
		read.mockRejectedValue(new ApiError(429, 'rate_limited', 'slow down'));
		await watching.load('tok');
		expect([watching.gone, watching.failed]).toEqual([false, true]);
		read.mockRejectedValue(new ApiError(0, 'network', 'offline'));
		await watching.load('tok');
		expect([watching.gone, watching.failed]).toEqual([false, true]);
	});
});
