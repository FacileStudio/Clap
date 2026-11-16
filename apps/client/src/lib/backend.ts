import { API, request, send } from './http';
import type {
	Asset,
	AssistantProfile,
	Op,
	PendingAction,
	Project,
	ProjectSummary,
	Render,
	RenderRequest,
	SessionUser,
	Timeline
} from './types';
import { uploadAsset } from './upload';

export { API, ApiError, request, toApiError, type ErrorCode } from './http';

const project = (id: string, rest = '') => `/projects/${encodeURIComponent(id)}${rest}`;
const render = (id: string, rest = '') => `/renders/${encodeURIComponent(id)}${rest}`;

const action = (id: string, verb: string) => `/mcp/pending/${encodeURIComponent(id)}/${verb}`;

/** Which compiled document a frame shows: the whole project, or one scene (from 0). A new `stamp` loads it again. */
export type FrameQuery = { scene?: number; stamp?: string | number };

function frameUrl(id: string, query: FrameQuery = {}): string {
	const params = new URLSearchParams();
	if (query.scene !== undefined) params.set('scene', String(query.scene));
	if (query.stamp !== undefined) params.set('v', String(query.stamp));
	const text = params.toString();
	return `${API}${project(id, '/frame')}${text ? `?${text}` : ''}`;
}

/** Every call the client makes, one method per route. */
export const backend = {
	session: () => request<{ user: SessionUser | null; bulle_url?: string }>('/auth/session'),
	loginUrl: () => `${API}/auth/oidc`,
	logout: () => request<unknown>('/auth/logout', send('POST')),
	projects: () => request<ProjectSummary[]>('/projects'),
	trashed: () => request<ProjectSummary[]>('/projects?trashed=true'),
	create: (title: string) => request<Project>('/projects', send('POST', { title })),
	project: (id: string) => request<Project>(project(id)),
	save: (id: string, source: string, rev: number) =>
		request<{ rev: number }>(project(id, '/source'), send('PUT', { source, rev })),
	rename: (id: string, title: string) => request<ProjectSummary>(project(id), send('PATCH', { title })),
	trash: (id: string, trashed: boolean) => request<ProjectSummary>(project(id), send('PATCH', { trashed })),
	destroy: (id: string) => request<void>(project(id), send('DELETE')),
	edit: (id: string, rev: number, ops: Op[]) =>
		request<{ source: string; rev: number }>(project(id, '/edits'), send('POST', { rev, ops })),
	timeline: (id: string) => request<Timeline>(project(id, '/timeline')),
	eventsUrl: (id: string) => `${API}${project(id, '/events')}`,
	frameUrl,
	assets: (id: string) => request<{ assets: Asset[] }>(project(id, '/assets')).then((found) => found.assets),
	upload: (id: string, file: File, onprogress: (fraction: number) => void) =>
		uploadAsset(`${API}${project(id, '/assets')}`, file, onprogress),
	removeAsset: (id: string, name: string) =>
		request<void>(project(id, `/assets/${encodeURI(name)}`), send('DELETE')),
	renders: (id: string) => request<{ renders: Render[] }>(project(id, '/renders')).then((found) => found.renders),
	startRender: (id: string, body: RenderRequest) => request<Render>(project(id, '/renders'), send('POST', body)),
	render: (id: string) => request<Render>(render(id)),
	cancelRender: (id: string) => request<Render | undefined>(render(id), send('DELETE')),
	renderEventsUrl: (id: string) => `${API}${render(id, '/events')}`,
	assistant: () => request<AssistantProfile>('/assistant'),
	actions: () => request<{ actions: PendingAction[]; truncated?: boolean }>('/mcp/pending'),
	confirmAction: (id: string) => request<unknown>(action(id, 'confirm'), send('POST')),
	dismissAction: (id: string) => request<unknown>(action(id, 'dismiss'), send('POST'))
};
