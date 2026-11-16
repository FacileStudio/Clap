import { backend } from '$lib/backend';
import { t } from '$lib/i18n';
import type { DropTarget, Ghost } from '$lib/timeline/ghost';
import type { Asset } from '$lib/types';
import { measureFile, type LocalMedia } from './local-media';
import { kindOfType } from './media-facts';

/** One upload as muse's `UploadProgress` shows it, plus where it was dropped and what the browser read of it. */
export interface UploadItem {
	id: string;
	name: string;
	size: number;
	progress: number;
	status: 'uploading' | 'done' | 'error';
	error?: string;
	type: string;
	target: DropTarget | null;
	local: LocalMedia | null;
}

/** A finished upload: the stored asset, what the browser read of the file, and the drop it came from. */
export interface Stored {
	id: string;
	asset: Asset;
	local: LocalMedia;
	target: DropTarget | null;
}

/**
 * The uploads of one project: each file is sent on its own, with its progress, and can be stopped. While it is
 * sent the browser reads its length and first frame, so a dropped file is drawn and placed before the probe ends.
 */
export class Uploads {
	items = $state<UploadItem[]>([]);
	/** What the browser read of each stored file, by asset name. */
	measured = $state<Record<string, LocalMedia>>({});
	private readonly running = new Map<string, () => void>();
	private serial = 0;

	constructor(
		private readonly project: string,
		private readonly onstored: (stored: Stored) => void
	) {}

	/** Sends `files`; with a `target`, each one is aimed there, the next ones a track higher, and drawn as a ghost. */
	add(files: File[], target: DropTarget | null = null): void {
		files.forEach((file, index) => this.send(file, target && { ...target, track: target.track + index }));
	}

	/** Stops the upload `id`, or forgets it when it already ended. */
	cancel(id: string): void {
		this.running.get(id)?.();
		this.items = this.items.filter((item) => item.id !== id);
	}

	/** The dropped files still being sent, as ghosts. */
	get ghosts(): Ghost[] {
		return this.items.flatMap((item) => {
			const kind = item.local?.kind ?? kindOfType(item.type);
			if (!item.target || !kind || item.status !== 'uploading') return [];
			const { duration = null, poster = null } = item.local ?? {};
			return [{ id: item.id, kind, label: item.name, duration, poster, progress: item.progress / 100, target: item.target }];
		});
	}

	private send(file: File, target: DropTarget | null): void {
		const id = `upload-${(this.serial += 1)}`;
		const item = { id, name: file.name, size: file.size, type: file.type, target, local: null };
		this.items = [...this.items, { ...item, progress: 0, status: 'uploading' }];
		const local = measureFile(file);
		void local.then((read) => this.patch(id, { local: read }));
		const upload = backend.upload(this.project, file, (fraction) => this.patch(id, { progress: Math.round(fraction * 100) }));
		this.running.set(id, upload.abort);
		upload.done
			.then(async (asset) => {
				const read = await local;
				this.measured[asset.name] = read;
				this.items = this.items.filter((one) => one.id !== id);
				this.onstored({ id, asset, local: read, target });
			})
			.catch((error: unknown) => this.patch(id, { status: 'error', error: (error instanceof Error && error.message) || t('media.failed') }))
			.finally(() => this.running.delete(id));
	}

	private patch(id: string, change: Partial<UploadItem>): void {
		this.items = this.items.map((item) => (item.id === id ? { ...item, ...change } : item));
	}
}
