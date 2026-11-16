<script lang="ts">
	import { timecode } from '$lib/project/timecode';
	import { RULER } from '$lib/timeline/layout';
	import { ticks } from '$lib/timeline/scale';
	import { t } from '$lib/i18n';

	let {
		zoom,
		from,
		to,
		onpointerdown
	}: {
		/** Pixels per second. */
		zoom: number;
		/** The first and last project times in view. */
		from: number;
		to: number;
		onpointerdown: (event: PointerEvent) => void;
	} = $props();

	const marks = $derived(ticks(zoom, Math.max(0, from), to));
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
</div>
