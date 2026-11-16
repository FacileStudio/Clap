/** The frame rate the editor counts in until a project names its own. */
export const FPS = 30;

/** The whole frame that holds `seconds` at `fps`. */
export const frameOf = (seconds: number, fps = FPS): number => Math.floor(Math.max(0, seconds) * fps + 1e-6);

/** `seconds` as `HH:MM:SS:FF` at `fps`. */
export function timecode(seconds: number, fps = FPS): string {
	const frames = frameOf(seconds, fps);
	const whole = Math.floor(frames / fps);
	const parts = [Math.floor(whole / 3600), Math.floor(whole / 60) % 60, whole % 60, frames % fps];
	return parts.map((part) => String(part).padStart(2, '0')).join(':');
}

/** `seconds` as a short length for a label: `6 s`, `1:05`. */
export function shortTime(seconds: number): string {
	if (seconds < 60) return `${Math.round(seconds * 10) / 10} s`;
	const whole = Math.round(seconds);
	return `${Math.floor(whole / 60)}:${String(whole % 60).padStart(2, '0')}`;
}

/** `seconds` moved by `frames` whole frames at `fps`, landing on a frame boundary. */
export function stepFrames(seconds: number, frames: number, fps = FPS): number {
	return Math.max(0, (Math.round(seconds * fps) + frames) / fps);
}
