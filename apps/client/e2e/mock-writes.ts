import { renameSync } from 'node:fs';
import { join } from 'node:path';
import { servedTimeline } from './mock-look';
import type { Asset, Op } from '../src/lib/types';
import { applyEdits } from './mock-edits';
import { tell } from './mock-events';
import { assets, scratch } from './mock-media';

/** One project the mock keeps in memory. */
export type Project = { id: string; title: string; source: string; rev: number; updated_at: string; trashed: boolean };

/** A JSON answer. */
export const json = (body: unknown, status = 200) => Response.json(body, { status });

/** A refusal in the suite envelope, with `extra` beside it. */
export const refuse = (status: number, code: string, message: string, extra: object = {}) =>
	json({ error: { code, message }, ...extra }, status);

function write(project: Project, source: string): void {
	Object.assign(project, { source, rev: project.rev + 1, updated_at: new Date().toISOString() });
	tell(project.id)('changed', { rev: project.rev });
}

/** `PUT /projects/{id}/source`: 409 on a stale revision, 422 on a `colour:` key, the next revision otherwise. */
export function save(project: Project, body: { source: string; rev: number }): Response {
	if (body.rev !== project.rev) return refuse(409, 'already_exists', 'the project moved', { rev: project.rev });
	const bad = body.source.split('\n').findIndex((line) => /^colour:/.test(line));
	if (bad >= 0) return refuse(422, 'invalid_argument', 'invalid', { errors: [{ line: bad + 1, message: 'unknown key `colour`' }] });
	write(project, body.source);
	return json({ rev: project.rev });
}

/** `POST /projects/{id}/edits`: the ops patched into the text, answered with the new source. */
export function edit(project: Project, body: { rev: number; ops: Op[] }): Response {
	if (body.rev !== project.rev) return refuse(409, 'already_exists', 'the project moved', { rev: project.rev });
	try {
		write(project, applyEdits(project.source, body.ops));
	} catch (error) {
		return refuse(400, 'invalid_argument', (error as Error).message);
	}
	return json({ source: project.source, rev: project.rev, timeline: servedTimeline(project.source) });
}

/** Renames the asset `from` of `project` to `to`, its file with it. Null when there is none, or `to` is taken. */
function renameAsset(project: string, from: string, to: string): Asset | null {
	const list = assets.get(project) ?? [];
	const asset = list.find((known) => known.name === from);
	if (!asset || list.some((known) => known.name === to)) return null;
	renameSync(join(scratch, `${project}-${from.split('/').pop()}`), join(scratch, `${project}-${to.split('/').pop()}`));
	const url = `/api/projects/${project}/assets/${encodeURI(to)}`;
	return Object.assign(asset, { name: to, url, poster_url: asset.poster_url && url });
}

/** `PATCH /projects/{id}/assets/{name}`: the asset renamed and every quoted reference to it in the source, as one revision. */
export function renamed(project: Project, from: string, body: { name: string; rev: number }): Response {
	if (body.rev !== project.rev) return refuse(409, 'already_exists', 'the project moved', { rev: project.rev });
	const asset = renameAsset(project.id, from, body.name);
	if (!asset) return refuse(409, 'already_exists', 'the asset is missing or the name is taken');
	write(project, project.source.replaceAll(`"${from}`, `"${body.name}`));
	tell(project.id)('asset', { name: body.name, state: asset.state });
	return json({ asset, rev: project.rev });
}
