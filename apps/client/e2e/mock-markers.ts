import type { Marker } from '../src/lib/types';

const markers = new Map<string, Marker[]>();
const json = (body: unknown, status = 200) => Response.json(body, { status });

/** `/projects/{id}/markers[/{marker}]`: list by time, add, move or rename, delete. */
export async function markerRoute(request: Request, project: string, id: string | undefined): Promise<Response> {
	const list = markers.get(project) ?? [];
	if (request.method === 'GET') return json({ markers: [...list].sort((a, b) => a.t - b.t) });
	if (request.method === 'POST') {
		const body = (await request.json()) as { t: number; label?: string };
		const marker = { id: crypto.randomUUID(), t: body.t, label: body.label ?? '' };
		markers.set(project, [...list, marker]);
		return json(marker, 201);
	}
	const marker = list.find((known) => known.id === id);
	if (!marker) return json({ error: { code: 'not_found', message: 'marker not found' } }, 404);
	if (request.method === 'DELETE') {
		markers.set(project, list.filter((known) => known !== marker));
		return new Response(null, { status: 204 });
	}
	return json(Object.assign(marker, await request.json()));
}
