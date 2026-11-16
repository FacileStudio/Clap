import type { Op } from '../src/lib/types';
import { applyEdits } from './mock-edits';
import { tell } from './mock-events';

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
	return json({ source: project.source, rev: project.rev });
}
