import { backend } from '$lib/backend';
import type { Asset } from '$lib/types';

/** One upload as muse's `UploadProgress` shows it. */
export interface UploadItem {
	id: string;
	name: string;
	size: number;
	progress: number;
	status: 'uploading' | 'done' | 'error';
	error?: string;
}

/** The uploads of one project: each file is sent on its own, with its progress, and can be stopped. */
export class Uploads {
	items = $state<UploadItem[]>([]);
	private readonly running = new Map<string, () => void>();
	private serial = 0;

	constructor(
		private readonly project: string,
		private readonly onstored: (asset: Asset) => void
	) {}

	add(files: File[]): void {
		for (const file of files) this.send(file);
	}

	/** Stops the upload `id`, or forgets it when it already ended. */
	cancel(id: string): void {
		this.running.get(id)?.();
		this.items = this.items.filter((item) => item.id !== id);
	}

	private send(file: File): void {
		const id = `upload-${(this.serial += 1)}`;
		this.items = [...this.items, { id, name: file.name, size: file.size, progress: 0, status: 'uploading' }];
		const upload = backend.upload(this.project, file, (fraction) => this.patch(id, { progress: Math.round(fraction * 100) }));
		this.running.set(id, upload.abort);
		upload.done
			.then((asset) => {
				this.onstored(asset);
				this.items = this.items.filter((item) => item.id !== id);
			})
			.catch((error: unknown) => this.patch(id, { status: 'error', error: error instanceof Error ? error.message : '' }))
			.finally(() => this.running.delete(id));
	}

	private patch(id: string, change: Partial<UploadItem>): void {
		this.items = this.items.map((item) => (item.id === id ? { ...item, ...change } : item));
	}
}
