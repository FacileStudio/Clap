import { backend } from '$lib/backend';
import type { Render, RenderRequest, RenderState } from '$lib/types';

const ENDED: RenderState[] = ['done', 'failed', 'cancelled'];
const EVENTS = ['status', 'message'];

/** Whether a render in `state` will not move any more. */
export const ended = (state: RenderState): boolean => ENDED.includes(state);

/** The size a render `height` pixels high has for a stage `ratio` wide over high: even numbers, as H.264 needs. */
export function renderSize(height: number, ratio: number): { width: number; height: number } {
	const even = (value: number) => Math.max(2, Math.round(value / 2) * 2);
	return { width: even(height * ratio), height: even(height) };
}

/**
 * One render job as the dialog follows it: started, then kept current from the job's event stream, with a plain
 * read every few seconds in case the stream drops.
 */
export class RenderJob {
	status = $state<Render | null>(null);
	starting = $state(false);
	private stream: EventSource | null = null;
	private timer: ReturnType<typeof setInterval> | null = null;

	constructor(private readonly interval = 3000) {}

	/** Starts a render of project `project`. Throws when the API refuses it. */
	async start(project: string, request: RenderRequest): Promise<void> {
		this.starting = true;
		try {
			const queued = await backend.startRender(project, request);
			this.follow(queued.id, { ...queued, state: queued.state ?? 'queued', frame: queued.frame ?? 0, frames: queued.frames ?? 0 });
		} finally {
			this.starting = false;
		}
	}

	/** Follows the job `id`, showing `known` until the first answer. */
	follow(id: string, known: Render): void {
		this.stop();
		this.status = known;
		if (ended(known.state)) return;
		this.stream = new EventSource(backend.renderEventsUrl(id), { withCredentials: true });
		for (const name of EVENTS) this.stream.addEventListener(name, (event) => this.heard(id, event as MessageEvent<string>));
		this.timer = setInterval(() => void this.read(id), this.interval);
		void this.read(id);
	}

	async cancel(): Promise<void> {
		const id = this.status?.id;
		if (!id) return;
		await backend.cancelRender(id);
		await this.read(id);
	}

	/** Forgets the job and stops following it. */
	reset(): void {
		this.stop();
		this.status = null;
	}

	private heard(id: string, event: MessageEvent<string>): void {
		try {
			this.take(id, { ...this.status, ...(JSON.parse(event.data) as Partial<Render>), id } as Render);
		} catch {
			return;
		}
	}

	private async read(id: string): Promise<void> {
		const found = await backend.render(id).catch(() => null);
		if (found) this.take(id, found);
	}

	private take(id: string, next: Render): void {
		if (this.status?.id !== id) return;
		this.status = next;
		if (!ended(next.state)) return;
		this.stop();
		if (next.state === 'done' && !next.download_url) void this.read(id);
	}

	private stop(): void {
		this.stream?.close();
		this.stream = null;
		if (this.timer) clearInterval(this.timer);
		this.timer = null;
	}
}
