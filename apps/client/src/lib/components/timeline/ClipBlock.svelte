<script lang="ts">
	import { Icon } from '@facile/muse';
	import { KIND_ICON } from '$lib/icons';
	import { t } from '$lib/i18n';
	import { shortTime } from '$lib/project/timecode';
	import type { Grip } from '$lib/timeline/clip-gesture';
	import { ROW } from '$lib/timeline/layout';
	import { peaksOf } from '$lib/timeline/peaks';
	import { stripOf, stripTiles, tileStyle, wavePath } from '$lib/timeline/strip';
	import type { Asset, Clip } from '$lib/types';

	let {
		clip,
		row,
		zoom,
		label,
		asset,
		selected,
		readonly,
		register,
		ongrab,
		onkeydown
	}: {
		clip: Clip;
		/** The row the clip's track is drawn on, from the top. */
		row: number;
		zoom: number;
		label: string;
		/** The asset the clip loads, when the project has it. */
		asset: Asset | undefined;
		selected: boolean;
		readonly: boolean;
		/** Hands the drawn element to the gestures. */
		register: (node: HTMLElement, id: string) => { destroy: () => void };
		ongrab: (event: PointerEvent, grip: Grip) => void;
		onkeydown: (event: KeyboardEvent) => void;
	} = $props();

	const TONE = {
		video: 'bg-fc-info/35',
		audio: 'bg-fc-success/35',
		image: 'bg-fc-warning/35',
		text: 'bg-fc-admin/35',
		html: 'bg-fc-field'
	} as const;

	const name = $derived(
		t('timeline.clip', {
			kind: t(`kind.${clip.kind}`),
			label,
			track: clip.track,
			from: shortTime(clip.start),
			to: shortTime(clip.start + clip.duration)
		})
	);
	const ramp = (preset: unknown, seconds: number | null | undefined) =>
		preset ? Math.min(seconds ?? 0.5, clip.duration / 2) * zoom : 0;
	const ramps = $derived({ enter: ramp(clip.enter, clip.enter_duration), exit: ramp(clip.exit, clip.exit_duration) });
	const HEIGHT = 34;
	const film = $derived(clip.kind === 'video' ? stripOf(asset) : null);
	const tiles = $derived(film ? stripTiles(film, clip.media_in, clip.duration, zoom, HEIGHT) : []);
	const picture = $derived(clip.kind === 'image' ? (asset?.url ?? null) : clip.kind === 'video' && !film ? (asset?.poster_url ?? null) : null);
	const cover = $derived(picture ? `background-image: url('${picture.replace(/'/g, '%27')}')` : '');
	const peaksUrl = $derived(clip.kind === 'audio' ? (asset?.peaks_url ?? null) : null);
	let peaks = $state<number[]>([]);
	const wave = $derived(wavePath(peaks, (asset?.duration_ms ?? 0) / 1000, clip.media_in, clip.duration));

	$effect(() => {
		const url = peaksUrl;
		peaks = [];
		if (!url) return;
		let live = true;
		peaksOf(url).then((values) => live && (peaks = values));
		return () => void (live = false);
	});

	function grab(event: PointerEvent, grip: Grip) {
		if (event.button === 2) event.stopPropagation();
		if (event.button !== 0) return;
		event.stopPropagation();
		ongrab(event, grip);
	}
</script>

<div
	use:register={clip.id}
	class="clip group absolute flex touch-none items-center rounded-fc-md text-fc-xs text-fc-fg select-none
		focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-fc-ring {TONE[clip.kind]}
		{selected ? 'outline-2 outline-offset-[-2px] outline-fc-info' : 'hover:outline-[1.5px] hover:outline-offset-[-1.5px] hover:outline-fc-info/60'}
		{readonly ? '' : 'cursor-grab active:cursor-grabbing'}"
	style="--x: {clip.start * zoom}px; --w: {Math.max(clip.duration * zoom, 6)}px; --y: {row * ROW}px"
	role="button"
	tabindex="0"
	aria-label={name}
	aria-pressed={selected}
	title={label}
	data-testid="clip"
	data-clip={clip.id}
	data-kind={clip.kind}
	data-start={clip.start}
	data-duration={clip.duration}
	data-track={clip.track}
	onpointerdown={(event) => grab(event, 'move')}
	{onkeydown}
>
	<span class="absolute inset-0 flex items-center overflow-hidden rounded-fc-md">
		{#if film}
			<span class="absolute inset-0 opacity-70" aria-hidden="true" data-testid="clip-strip" data-from={clip.media_in}>
				{#each tiles as tile (tile.left)}
					<span class="absolute inset-y-0 bg-no-repeat" style={tileStyle(film, tile)} data-testid="strip-tile" data-tile={tile.index} data-at={tile.at}></span>
				{/each}
			</span>
		{:else if cover}
			<span class="absolute inset-0 bg-[length:auto_100%] bg-repeat-x opacity-60" style={cover} aria-hidden="true"></span>
		{/if}
		{#if wave.count > 0}
			<svg
				class="absolute inset-0 size-full text-fc-success opacity-70"
				viewBox="0 0 {wave.count} 2"
				preserveAspectRatio="none"
				aria-hidden="true"
				data-testid="clip-wave"
			>
				<path d={wave.d} fill="currentColor" />
			</svg>
		{/if}
		{#if film || cover}
			<span class="absolute inset-0 bg-linear-to-r from-fc-bg/80 to-fc-bg/10" aria-hidden="true"></span>
		{/if}
		{#if ramps.enter > 0}
			<span class="ramp absolute inset-y-0 left-0 bg-fc-fg/25" style="width: {ramps.enter}px" aria-hidden="true" data-testid="ramp-enter"></span>
		{/if}
		{#if ramps.exit > 0}
			<span class="ramp out absolute inset-y-0 right-0 bg-fc-fg/25" style="width: {ramps.exit}px" aria-hidden="true" data-testid="ramp-exit"></span>
		{/if}
		<span class="relative flex min-w-0 items-center gap-1.5 px-2.5">
			<Icon icon={KIND_ICON[clip.kind]} size={14} class="shrink-0" />
			<span class="truncate font-medium">{label}</span>
		</span>
	</span>
	{#if !readonly}
		<span class="trim start" aria-hidden="true" data-testid="trim-start" onpointerdown={(event) => grab(event, 'start')}>
			<span class="handle rounded-l-fc-md"></span>
		</span>
		<span class="trim end" aria-hidden="true" data-testid="trim-end" onpointerdown={(event) => grab(event, 'end')}>
			<span class="handle rounded-r-fc-md"></span>
		</span>
	{/if}
</div>

<style>
	.clip {
		top: calc(var(--y) + var(--dy, 0px) + 3px);
		left: calc(var(--x) + var(--dx, 0px));
		width: calc(var(--w) + var(--dw, 0px));
		height: 34px;
	}

	.ramp {
		clip-path: polygon(0 0, 100% 0, 0 100%);
	}

	.ramp.out {
		clip-path: polygon(0 0, 100% 0, 100% 100%);
	}

	.trim {
		position: absolute;
		top: -5px;
		bottom: -5px;
		z-index: 1;
		display: flex;
		width: 44px;
		padding-block: 5px;
		cursor: ew-resize;
	}

	.trim.start {
		left: -22px;
		padding-left: 22px;
	}

	.trim.end {
		right: -22px;
		justify-content: flex-end;
		padding-right: 22px;
	}

	.handle {
		width: 8px;
		background: transparent;
	}

	.clip:hover .handle,
	.clip:focus-visible .handle {
		background: color-mix(in oklab, var(--color-fc-fg) 45%, transparent);
	}

	@media (pointer: coarse) {
		.handle {
			background: color-mix(in oklab, var(--color-fc-fg) 45%, transparent);
		}
	}
</style>
