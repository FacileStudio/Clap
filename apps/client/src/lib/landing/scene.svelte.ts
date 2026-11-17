/** The frames of the demo render: eight seconds at thirty frames a second. */
export const FRAMES = 240;

/** The seconds the demo scene lasts, and where its clips sit on it. */
export const SCENE = { duration: 8, audio: 6, videoStart: 1, videoLong: 6, videoCut: 5, titleStart: 1.5, title: 3 };

/** What the faux editor shows at one moment of its story. The scripts move it; the parts only draw it. */
export class EditorScene {
	tab = $state<'media' | 'clip'>('media');
	dragging = $state(false);
	placed = $state(false);
	length = $state(SCENE.videoLong);
	guide = $state(false);
	typed = $state('');
	titled = $state(false);
	job = $state<'idle' | 'running' | 'done'>('idle');
	frame = $state(0);

	/** The opening frame: an empty track, the media bin open, nothing rendering. */
	reset(): void {
		this.tab = 'media';
		this.dragging = false;
		this.placed = false;
		this.length = SCENE.videoLong;
		this.guide = false;
		this.typed = '';
		this.titled = false;
		this.job = 'idle';
		this.frame = 0;
	}

	/** The frame that tells the story alone: clip trimmed, title typed in the clip settings, MP4 ready. */
	poster(headline: string): void {
		this.reset();
		this.tab = 'clip';
		this.placed = true;
		this.length = SCENE.videoCut;
		this.typed = headline;
		this.titled = true;
		this.job = 'done';
		this.frame = FRAMES;
	}
}
