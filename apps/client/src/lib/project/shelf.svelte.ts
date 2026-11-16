import { backend } from '$lib/backend';
import type { ProjectSummary } from '$lib/types';

/** The projects of the signed-in person, newest first, or their trash, and the writes the list pages make. */
export class Shelf {
	projects = $state<ProjectSummary[]>([]);
	ready = $state(false);
	failed = $state(false);

	constructor(private readonly trashed = false) {}

	async load(): Promise<void> {
		try {
			this.projects = await (this.trashed ? backend.trashed() : backend.projects());
			this.failed = false;
		} catch {
			this.failed = true;
		}
		this.ready = true;
	}

	create(title: string): Promise<ProjectSummary> {
		return backend.create(title);
	}

	async rename(id: string, title: string): Promise<void> {
		const renamed = await backend.rename(id, title);
		this.projects = this.projects.map((project) => (project.id === id ? { ...project, title: renamed.title } : project));
	}

	async trash(id: string): Promise<void> {
		await backend.trash(id, true);
		this.projects = this.projects.filter((project) => project.id !== id);
	}

	async restore(id: string): Promise<void> {
		await backend.trash(id, false);
		await this.load();
	}

	/** Deletes a trashed project for good. */
	async destroy(id: string): Promise<void> {
		await backend.destroy(id);
		this.projects = this.projects.filter((project) => project.id !== id);
	}
}
