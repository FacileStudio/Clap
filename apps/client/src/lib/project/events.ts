import { backend } from '$lib/backend';
import type { AssetState } from '$lib/types';

/** What a project's event stream reports. */
export interface ProjectEvents {
	/** The stored project reached revision `rev`. */
	onchanged(rev: number): void;
	/** The uploaded file `name` moved to `state`. */
	onasset(name: string, state: AssetState): void;
}

function listen<T>(source: EventSource, name: string, take: (data: T) => void): void {
	source.addEventListener(name, (event) => {
		try {
			take(JSON.parse((event as MessageEvent<string>).data) as T);
		} catch {
			return;
		}
	});
}

/** Follows the events of project `id`; answers the function that stops following. */
export function watchProject(id: string, events: ProjectEvents): () => void {
	const source = new EventSource(backend.eventsUrl(id), { withCredentials: true });
	listen<{ rev: unknown }>(source, 'changed', ({ rev }) => {
		if (Number.isFinite(Number(rev))) events.onchanged(Number(rev));
	});
	listen<{ name: unknown; state: AssetState }>(source, 'asset', ({ name, state }) => {
		if (typeof name === 'string') events.onasset(name, state);
	});
	return () => source.close();
}
