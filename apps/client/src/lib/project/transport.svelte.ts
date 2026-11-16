import { clamp } from '$lib/timeline/model';
import { FPS, stepFrames } from './timecode';

type Schedule = (tick: (now: number) => void) => number;

/** The fastest shuttle speed J and L reach. */
export const MAX_RATE = 8;

/**
 * The editor's clock. The compiled document has none: while playing, this one advances `time` on every
 * animation frame and the preview is asked to draw it.
 */
export class Transport {
	time = $state(0);
	duration = $state(0);
	playing = $state(false);
	rate = $state(1);
	private handle = 0;
	private last = 0;

	constructor(
		private readonly schedule: Schedule = (tick) => requestAnimationFrame(tick),
		private readonly cancel: (handle: number) => void = (handle) => cancelAnimationFrame(handle)
	) {}

	seek(time: number): void {
		this.time = clamp(time, 0, this.duration);
	}

	/** Plays at `rate` times real time; a negative rate plays backwards. */
	play(rate = 1): void {
		this.rate = rate;
		if (this.playing) return;
		if (rate > 0 && this.time >= this.duration) this.time = 0;
		this.playing = true;
		this.last = 0;
		this.handle = this.schedule(this.tick);
	}

	pause(): void {
		this.playing = false;
		this.cancel(this.handle);
	}

	toggle(): void {
		if (this.playing) this.pause();
		else this.play(1);
	}

	/** J and L: plays in `direction`, twice as fast each time it is asked again in the same direction. */
	shuttle(direction: 1 | -1): void {
		const same = this.playing && Math.sign(this.rate) === direction;
		this.play(same ? direction * Math.min(Math.abs(this.rate) * 2, MAX_RATE) : direction);
	}

	/** Stops and moves by whole frames. */
	step(frames: number, fps = FPS): void {
		this.pause();
		this.seek(stepFrames(this.time, frames, fps));
	}

	private tick = (now: number): void => {
		if (!this.playing) return;
		const elapsed = this.last ? (now - this.last) / 1000 : 0;
		this.last = now;
		const next = this.time + elapsed * this.rate;
		this.time = clamp(next, 0, this.duration);
		const ended = this.rate < 0 ? next <= 0 : next >= this.duration;
		if (ended) return this.pause();
		this.handle = this.schedule(this.tick);
	};
}
