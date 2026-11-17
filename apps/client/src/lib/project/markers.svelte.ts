import { backend } from '$lib/backend';
import type { Marker } from '$lib/types';

const byTime = (list: Marker[]) => [...list].sort((a, b) => a.t - b.t);

/** The ruler markers of one project, kept in time order. Each change goes to the API; a refusal is reported and leaves the list as the server holds it. */
export class Markers {
	list = $state<Marker[]>([]);

	constructor(
		private readonly project: string,
		private readonly fail: (error: unknown) => void
	) {}

	async load(): Promise<void> {
		this.list = byTime(await backend.markers(this.project).catch(() => this.list));
	}

	/** Adds a marker at project time `t`, rounded to the millisecond. */
	add(t: number): Promise<void> {
		return this.write(async () => [...this.list, await backend.addMarker(this.project, Math.max(0, Math.round(t * 1000) / 1000))]);
	}

	/** Moves marker `id` to project time `t`. */
	move(id: string, t: number): Promise<void> {
		return this.change(id, { t: Math.max(0, Math.round(t * 1000) / 1000) });
	}

	rename(id: string, label: string): Promise<void> {
		return this.change(id, { label });
	}

	remove(id: string): Promise<void> {
		return this.write(async () => {
			await backend.removeMarker(this.project, id);
			return this.list.filter((marker) => marker.id !== id);
		});
	}

	private change(id: string, change: { t?: number; label?: string }): Promise<void> {
		return this.write(async () => {
			const changed = await backend.changeMarker(this.project, id, change);
			return this.list.map((marker) => (marker.id === id ? changed : marker));
		});
	}

	private async write(job: () => Promise<Marker[]>): Promise<void> {
		try {
			this.list = byTime(await job());
		} catch (error) {
			this.fail(error);
			await this.load();
		}
	}
}
