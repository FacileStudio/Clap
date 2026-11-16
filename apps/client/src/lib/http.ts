import type { SceneError } from './types';

/** The suite error codes, from tronc's `statusByCode`. */
export type ErrorCode =
	| 'invalid_argument'
	| 'unauthenticated'
	| 'permission_denied'
	| 'not_found'
	| 'already_exists'
	| 'conflict'
	| 'failed_precondition'
	| 'resource_exhausted'
	| 'rate_limited'
	| 'method_not_allowed'
	| 'unavailable'
	| 'internal'
	| 'network';

const CODES: Record<number, ErrorCode> = {
	400: 'invalid_argument',
	401: 'unauthenticated',
	403: 'permission_denied',
	404: 'not_found',
	405: 'method_not_allowed',
	409: 'conflict',
	422: 'invalid_argument',
	412: 'failed_precondition',
	413: 'resource_exhausted',
	429: 'rate_limited',
	503: 'unavailable'
};

const SAFE = new Set(['GET', 'HEAD', 'OPTIONS']);

/** The API base. Same origin: the Vite proxy in development, the Rust binary in production. */
export const API = '/api';

/** A failed API call, carrying the suite envelope `{"error":{"code","message"}}`. */
export class ApiError extends Error {
	constructor(
		readonly status: number,
		readonly code: ErrorCode,
		message: string,
		readonly errors: SceneError[] = [],
		/** Why a timeline edit was refused, such as `overlap`; empty for any other failure. */
		readonly reason = ''
	) {
		super(message);
		this.name = 'ApiError';
	}
}

type Envelope = { error?: { code?: string; message?: string; errors?: unknown }; errors?: unknown; timeline_error?: unknown };

function sceneErrors(body: Envelope): SceneError[] {
	const found = body.errors ?? body.error?.errors;
	if (!Array.isArray(found)) return [];
	return found
		.filter((row): row is SceneError => typeof row?.line === 'number' && typeof row?.message === 'string')
		.map((row) => ({ line: row.line, message: row.message }));
}

/** Reads the suite error envelope of a failed response into an `ApiError`, with the scene errors of a 422. */
export async function toApiError(response: Response): Promise<ApiError> {
	const fallback = CODES[response.status] ?? 'internal';
	try {
		const body = (await response.json()) as Envelope;
		const code = (body.error?.code as ErrorCode | undefined) ?? fallback;
		return new ApiError(response.status, code, body.error?.message ?? response.statusText, sceneErrors(body), String(body.timeline_error ?? ''));
	} catch {
		return new ApiError(response.status, fallback, response.statusText || `HTTP ${response.status}`);
	}
}

/** Calls the API: cookies included, the CSRF header on every mutation, the error envelope thrown typed. */
export async function request<T>(path: string, init: RequestInit = {}): Promise<T> {
	const method = (init.method ?? 'GET').toUpperCase();
	const headers = new Headers(init.headers);
	if (typeof init.body === 'string' && !headers.has('Content-Type')) headers.set('Content-Type', 'application/json');
	if (!SAFE.has(method)) headers.set('X-Facile-CSRF', '1');
	let response: Response;
	try {
		response = await fetch(`${API}${path}`, {
			credentials: 'include',
			...init,
			method,
			headers
		});
	} catch {
		throw new ApiError(0, 'network', 'Network error');
	}
	if (!response.ok) throw await toApiError(response);
	if (response.status === 204) return undefined as T;
	return (await response.json()) as T;
}

/** The init of a JSON call with `method` and an optional body. */
export const send = (method: string, body?: unknown): RequestInit => ({
	method,
	body: body === undefined ? undefined : JSON.stringify(body)
});
