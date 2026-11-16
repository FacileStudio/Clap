const MAX_PEAKS = 4000;

const cache = new Map<string, Promise<number[]>>();

/** The values of a `{"peaks": [...]}` answer, each clamped to 0..1; anything else reads as no waveform. */
export function readPeaks(body: unknown): number[] {
	const peaks = (body as { peaks?: unknown } | null)?.peaks;
	if (!Array.isArray(peaks)) return [];
	return peaks.slice(0, MAX_PEAKS).map((value) => (typeof value === 'number' && Number.isFinite(value) ? Math.min(1, Math.max(0, value)) : 0));
}

/** The waveform at `url`, fetched once per page and shared by every clip of that sound. A failed fetch is retried next time. */
export function peaksOf(url: string): Promise<number[]> {
	const known = cache.get(url);
	if (known) return known;
	const loaded = fetch(url, { credentials: 'same-origin' })
		.then((answer) => (answer.ok ? answer.json() : Promise.reject(new Error(String(answer.status)))))
		.then(readPeaks)
		.catch(() => {
			cache.delete(url);
			return [];
		});
	cache.set(url, loaded);
	return loaded;
}
