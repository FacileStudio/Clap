<script lang="ts">
	import { Icon } from '@facile/muse';
	import { KIND_ICON } from '$lib/icons';
	import { t } from '$lib/i18n';
	import { shortTime } from '$lib/project/timecode';
	import type { Grip } from '$lib/timeline/clip-gesture';
	import { ROW } from '$lib/timeline/layout';
	import type { Clip } from '$lib/types';

	let {
		clip,
		row,
		zoom,
		label,
		poster,
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
		poster: string | null;
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
	const strip = $derived(poster ? `background-image: url('${poster.replace(/'/g, '%27')}')` : '');

	function grab(event: PointerEvent, grip: Grip) {
		if (event.button !== 0) return;
		event.stopPropagation();
		ongrab(event, grip);
	}
</script>

<div
	use:register={clip.id}
	class="clip group absolute flex touch-none items-center overflow-hidden rounded-fc-md text-fc-xs text-fc-fg select-none
		focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-fc-ring {TONE[clip.kind]}
		{selected ? 'outline-2 outline-offset-[-2px] outline-fc-fg' : ''} {readonly ? '' : 'cursor-grab active:cursor-grabbing'}"
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
	{#if poster}
		<span class="absolute inset-0 bg-[length:auto_100%] bg-repeat-x opacity-60" style={strip} aria-hidden="true"></span>
		<span class="absolute inset-0 bg-linear-to-r from-fc-bg/80 to-fc-bg/10" aria-hidden="true"></span>
	{/if}
	<span class="relative flex min-w-0 items-center gap-1.5 px-2.5">
		<Icon icon={KIND_ICON[clip.kind]} size={14} class="shrink-0" />
		<span class="truncate font-medium">{label}</span>
	</span>
	{#if !readonly}
		<span
			class="absolute inset-y-0 left-0 w-2 cursor-ew-resize rounded-l-fc-md bg-fc-fg/0 group-hover:bg-fc-fg/40"
			aria-hidden="true"
			data-testid="trim-start"
			onpointerdown={(event) => grab(event, 'start')}
		></span>
		<span
			class="absolute inset-y-0 right-0 w-2 cursor-ew-resize rounded-r-fc-md bg-fc-fg/0 group-hover:bg-fc-fg/40"
			aria-hidden="true"
			data-testid="trim-end"
			onpointerdown={(event) => grab(event, 'end')}
		></span>
	{/if}
</div>

<style>
	.clip {
		top: calc(var(--y) + var(--dy, 0px) + 3px);
		left: calc(var(--x) + var(--dx, 0px));
		width: calc(var(--w) + var(--dw, 0px));
		height: 34px;
	}
</style>
