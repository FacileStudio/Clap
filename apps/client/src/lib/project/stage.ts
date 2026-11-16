/** One scene of a compiled document, as its `scene:ready` message lists it. `first` is its first global step. */
export interface StageScene {
	start: number;
	duration: number;
	steps: number;
	first: number;
}

/** What a compiled document says once it is ready: its length, its last step and its scenes. */
export interface StageInfo {
	duration: number;
	steps: number;
	scenes: StageScene[];
}

/** What a compiled document says after every draw. `scene` is the index of the shown scene. */
export interface StageState {
	time: number;
	step: number;
	scene: number;
}

/** A message from a compiled document to its host, per `docs/format.md`. */
export type StageMessage = ({ type: 'scene:ready' } & StageInfo) | ({ type: 'scene:state' } & StageState);

const finite = (value: unknown): value is number => typeof value === 'number' && Number.isFinite(value);

function scenes(raw: unknown): StageScene[] | null {
	if (!Array.isArray(raw)) return null;
	const found = raw.map((row) => ({
		start: row?.start,
		duration: row?.duration,
		steps: row?.steps,
		first: row?.first
	}));
	return found.every((row) => Object.values(row).every(finite)) ? (found as StageScene[]) : null;
}

/** The scene message in `data`, or null when it is not one. A frame is untrusted: every field is checked. */
export function stageMessage(data: unknown): StageMessage | null {
	if (typeof data !== 'object' || data === null) return null;
	const raw = data as Record<string, unknown>;
	if (raw.type === 'scene:state' && finite(raw.time) && finite(raw.step) && finite(raw.scene)) {
		return { type: 'scene:state', time: raw.time, step: raw.step, scene: raw.scene };
	}
	const list = raw.type === 'scene:ready' ? scenes(raw.scenes) : null;
	if (!list || !finite(raw.duration) || !finite(raw.steps)) return null;
	return { type: 'scene:ready', duration: raw.duration, steps: raw.steps, scenes: list };
}

/** What a clip shows on the stage, as `clap:boxes` names it. Sounds have no box. */
export type StageBoxKind = 'video' | 'image' | 'text' | 'html';

/** One clip shown at the drawn time, its box in stage pixels turned `rot` degrees around its centre. */
export interface StageBox {
	id: string;
	kind: StageBoxKind;
	x: number;
	y: number;
	w: number;
	h: number;
	rot: number;
}

/** What a compiled document says after every seek: the boxes of the clips shown at `t`. */
export interface StageBoxes {
	t: number;
	clips: StageBox[];
}

const BOX_KINDS: readonly string[] = ['video', 'image', 'text', 'html'];
const MOST_BOXES = 512;

function stageBox(raw: unknown): StageBox | null {
	if (typeof raw !== 'object' || raw === null) return null;
	const row = raw as Record<string, unknown>;
	const { id, kind, x, y, w, h, rot } = row;
	if (typeof id !== 'string' || id.length === 0 || id.length > 200) return null;
	if (typeof kind !== 'string' || !BOX_KINDS.includes(kind)) return null;
	if (![x, y, w, h, rot].every(finite) || (w as number) < 0 || (h as number) < 0) return null;
	return { id, kind: kind as StageBoxKind, x: x as number, y: y as number, w: w as number, h: h as number, rot: rot as number };
}

/** The `clap:boxes` message in `data`, or null when it is not one. A frame is untrusted: a bad row drops the message. */
export function stageBoxes(data: unknown): StageBoxes | null {
	if (typeof data !== 'object' || data === null) return null;
	const raw = data as Record<string, unknown>;
	if (raw.type !== 'clap:boxes' || !finite(raw.t) || !Array.isArray(raw.clips) || raw.clips.length > MOST_BOXES) return null;
	const clips = raw.clips.map(stageBox);
	return clips.every((clip) => clip !== null) ? { t: raw.t, clips: clips as StageBox[] } : null;
}
