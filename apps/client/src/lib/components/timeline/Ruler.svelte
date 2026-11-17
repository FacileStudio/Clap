<script lang="ts">
	import { timecode } from '$lib/project/timecode';
	import { drag } from '$lib/timeline/drag';
	import { RULER } from '$lib/timeline/layout';
	import { ticks } from '$lib/timeline/scale';
	import { t } from '$lib/i18n';
	import type { Marker } from '$lib/types';

	let {
		zoom,
		from,
		to,
		length,
		markers = [],
		onpointerdown,
		onseek,
		onmove
	}: {
		/** Pixels per second. */
		zoom: number;
		/** The first and last project times in view. */
		from: number;
		to: number;
		/** The project's length in seconds: a marker is dragged no further. */
		length: number;
		markers?: Marker[];
		onpointerdown: (event: PointerEvent) => void;
		/** A marker was clicked: the playhead goes to `time`. */
		onseek: (time: number) => void;
		/** A marker was dragged to `time`. */
		onmove: (id: string, time: number) => void;
	} = $props();

	const marks = $derived(ticks(zoom, Math.max(0, from), to));

	let moving = $state<{ id: string; t: number } | null>(null);

	function grab(event: PointerEvent, marker: Marker) {
		if (event.button !== 0) return;
		event.stopPropagation();
		drag(event, {
			move: (dx) => (moving = { id: marker.id, t: Math.min(Math.max(0, marker.t + dx / zoom), Math.max(length, marker.t)) }),
			end: (moved) => {
				const at = moving?.t ?? marker.t;
				moving = null;
				if (moved) onmove(marker.id, at);
				else onseek(marker.t);
			}
		});
	}
</script>

<div
	class="sticky top-0 z-10 w-full cursor-ew-resize touch-none bg-fc-component select-none"
	style="height: {RULER}px"
	role="presentation"
	aria-label={t('timeline.ruler')}
	data-testid="ruler"
	{onpointerdown}
>
	{#each marks as mark (mark)}
		<span
			class="absolute top-0 flex h-full items-end pb-1 pl-1.5 text-[11px] leading-none text-fc-fg-muted tabular-nums"
			style="left: {mark * zoom}px"
		>
			<span class="absolute top-0 left-0 h-2 w-px bg-fc-fg-muted/50"></span>
			{timecode(mark).slice(3)}
		</span>
	{/each}
	{#each markers as marker (marker.id)}
		{@const at = moving?.id === marker.id ? moving.t : marker.t}
		<button
			type="button"
			class="group absolute top-0 z-10 flex h-3/5 max-w-40 -translate-x-[5px] cursor-grab items-start gap-1 rounded-fc-xs
				focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-fc-ring active:cursor-grabbing"
			style="left: {at * zoom}px"
			title={marker.label || t('marker.untitled')}
			aria-label={t('marker.pin', { label: marker.label || t('marker.untitled'), time: timecode(marker.t) })}
			data-testid="marker"
			data-marker={marker.id}
			data-time={marker.t.toFixed(3)}
			onpointerdown={(event) => grab(event, marker)}
			onclick={(event) => event.detail === 0 && onseek(marker.t)}
		>
			<span
				class="mt-0.5 h-3 w-2.5 shrink-0 rounded-t-[2px] rounded-b-full bg-fc-warning shadow-sm transition-transform group-hover:scale-110"
			></span>
			{#if marker.label}
				<span class="truncate rounded-fc-xs bg-fc-component/90 px-1 text-[10px] leading-4 text-fc-fg">{marker.label}</span>
			{/if}
		</button>
	{/each}
</div>
