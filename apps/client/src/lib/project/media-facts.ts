const PLACEABLE = ['video', 'audio', 'image'] as const;

/** The clip kinds a media file can make. */
export type MediaKind = (typeof PLACEABLE)[number];

const POSTER_HEIGHT = 90;

/** The clip kind a MIME type makes, or null when the timeline cannot place it. */
export function kindOfType(type: string): MediaKind | null {
	return PLACEABLE.find((kind) => type.startsWith(`${kind}/`)) ?? null;
}

/** The poster size for a picture of `width` by `height`: `high` pixels tall, ratio kept, at least one pixel wide. */
export function posterSize(width: number, height: number, high = POSTER_HEIGHT): { width: number; height: number } {
	if (width <= 0 || height <= 0) return { width: 0, height: 0 };
	const scale = Math.min(1, high / height);
	return { width: Math.max(1, Math.round(width * scale)), height: Math.max(1, Math.round(height * scale)) };
}

/** A finite positive length in seconds, or null. */
export function seconds(value: number): number | null {
	return Number.isFinite(value) && value > 0 ? Math.round(value * 1000) / 1000 : null;
}
