import { backend } from '$lib/backend';
import type { Project } from '$lib/types';

/** Loads one project for a page: the project, or `missing` when the API refuses it. A late answer for another id is dropped. */
export class ProjectLoader {
	project = $state<Project | null>(null);
	missing = $state(false);
	private wanted = '';

	async load(id: string): Promise<void> {
		this.wanted = id;
		this.project = null;
		this.missing = false;
		try {
			const found = await backend.project(id);
			if (this.wanted === id) this.project = found;
		} catch {
			if (this.wanted === id) this.missing = true;
		}
	}
}
