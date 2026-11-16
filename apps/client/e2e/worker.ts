import { copyFileSync, mkdirSync, renameSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

/**
 * Writes the render worker's heartbeat. No worker runs here; without a fresh heartbeat the API refuses to queue
 * a render. `finish` then plays the worker's last step.
 */
export function heartbeat(dataDir: string): void {
	mkdirSync(join(dataDir, 'renders'), { recursive: true });
	writeFileSync(join(dataDir, 'renders', '.worker'), new Date().toISOString());
}

/** Does what the worker does at the end of render `id`: writes `out.mp4`, then a `done` status by rename. */
export function finish(dataDir: string, id: string, mp4: string): void {
	const job = join(dataDir, 'renders', id);
	copyFileSync(mp4, join(job, 'out.mp4'));
	const status = { state: 'done', frame: 1, frames: 1, error: null, updated_at: new Date().toISOString() };
	writeFileSync(join(job, 'status.json.tmp'), JSON.stringify(status));
	renameSync(join(job, 'status.json.tmp'), join(job, 'status.json'));
}
