import { kindOfType, posterSize, seconds, type MediaKind } from './media-facts';

/** What the browser can tell about a media file before the server has probed it. */
export interface LocalMedia {
	kind: MediaKind | null;
	/** Seconds, or null when the file has no length (an image) or the browser cannot read it. */
	duration: number | null;
	width: number | null;
	height: number | null;
	/** A small JPEG of the first frame as a data URL, so it outlives the file's blob URL. */
	poster: string | null;
}

const PATIENCE_MS = 8000;

function picture(source: CanvasImageSource, width: number, height: number): string | null {
	const size = posterSize(width, height);
	const canvas = document.createElement('canvas');
	Object.assign(canvas, size);
	const context = canvas.getContext('2d');
	if (!context || size.width === 0) return null;
	try {
		context.drawImage(source, 0, 0, size.width, size.height);
		return canvas.toDataURL('image/jpeg', 0.7);
	} catch {
		return null;
	}
}

function settle(element: HTMLElement, ready: string): Promise<boolean> {
	return new Promise((resolve) => {
		const timer = setTimeout(() => resolve(false), PATIENCE_MS);
		const end = (ok: boolean) => () => (clearTimeout(timer), resolve(ok));
		element.addEventListener(ready, end(true), { once: true });
		element.addEventListener('error', end(false), { once: true });
	});
}

async function readImage(url: string): Promise<LocalMedia> {
	const image = new Image();
	const ready = settle(image, 'load');
	image.src = url;
	if (!(await ready)) return { kind: 'image', duration: null, width: null, height: null, poster: null };
	const [width, height] = [image.naturalWidth, image.naturalHeight];
	return { kind: 'image', duration: null, width, height, poster: picture(image, width, height) };
}

async function readMedia(url: string, kind: 'video' | 'audio'): Promise<LocalMedia> {
	const media = document.createElement(kind === 'video' ? 'video' : 'audio');
	Object.assign(media, { muted: true, preload: 'auto', playsInline: true });
	const ready = settle(media, kind === 'video' ? 'loadeddata' : 'loadedmetadata');
	media.src = url;
	const ok = await ready;
	const duration = ok ? seconds(media.duration) : null;
	if (!(media instanceof HTMLVideoElement) || !ok) return { kind, duration, width: null, height: null, poster: null };
	const [width, height] = [media.videoWidth || null, media.videoHeight || null];
	return { kind, duration, width, height, poster: width && height ? picture(media, width, height) : null };
}

/**
 * Reads `file` in the browser: its kind, length, natural size and a first-frame poster. The blob URL it opens is
 * revoked before it answers; a file the browser cannot decode answers with nulls rather than failing.
 */
export async function measureFile(file: File): Promise<LocalMedia> {
	const kind = kindOfType(file.type);
	if (!kind) return { kind, duration: null, width: null, height: null, poster: null };
	const url = URL.createObjectURL(file);
	try {
		return await (kind === 'image' ? readImage(url) : readMedia(url, kind));
	} finally {
		URL.revokeObjectURL(url);
	}
}
