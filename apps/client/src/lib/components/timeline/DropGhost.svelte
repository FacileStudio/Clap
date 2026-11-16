<script lang="ts">
	import { Icon } from '@facile/muse';
	import { KIND_ICON } from '$lib/icons';
	import { t } from '$lib/i18n';
	import { shortTime } from '$lib/project/timecode';
	import { ghostBox, ghostRow, type Ghost } from '$lib/timeline/ghost';
	import { HEAD, ROW, RULER } from '$lib/timeline/layout';
	import type { Timeline } from '$lib/types';

	let {
		ghost,
		timeline,
		rows,
		zoom
	}: {
		ghost: Ghost;
		timeline: Timeline;
		/** The track of each row, top first. */
		rows: number[];
		zoom: number;
	} = $props();

	const TONE = {
		video: 'bg-fc-info/25',
		audio: 'bg-fc-success/25',
		image: 'bg-fc-warning/25',
		text: 'bg-fc-admin/25',
		html: 'bg-fc-field'
	} as const;

	const box = $derived(ghostBox(timeline, ghost));
	const length = $derived(ghost.duration === null ? '…' : shortTime(box?.duration ?? 0));
	const status = $derived(
		ghost.progress === null
			? t('timeline.ghost.waiting', { name: ghost.label })
			: t('timeline.ghost.uploading', { name: ghost.label, percent: Math.round(ghost.progress * 100) })
	);
	const backdrop = $derived(ghost.poster ? `background-image: url('${ghost.poster.replace(/'/g, '%27')}')` : '');
</script>

{#if box}
	<div
		class="ghost pointer-events-none absolute z-10 flex items-center overflow-hidden rounded-fc-md text-fc-xs text-fc-fg
			opacity-80 outline-2 outline-offset-[-2px] outline-fc-ring/60 {TONE[ghost.kind]}"
		style="--x: {box.start * zoom}px; --w: {Math.max(box.duration * zoom, 6)}px;
			--y: {RULER + HEAD + ghostRow(rows, box.track) * ROW}px"
		role="status"
		aria-label={ghost.id === 'aimed' ? undefined : status}
		data-testid="drop-ghost"
		data-start={box.start}
		data-duration={box.duration}
		data-track={box.track}
		data-progress={ghost.progress ?? ''}
	>
		{#if ghost.poster}
			<span class="absolute inset-0 bg-[length:auto_100%] bg-repeat-x opacity-60" style={backdrop} aria-hidden="true"></span>
		{/if}
		<span class="relative flex min-w-0 items-center gap-1.5 px-2.5">
			<Icon icon={KIND_ICON[ghost.kind]} size={14} class="shrink-0" />
			{#if ghost.label}<span class="truncate font-medium">{ghost.label}</span>{/if}
			<span class="shrink-0 text-fc-fg-muted tabular-nums">{length}</span>
		</span>
		{#if ghost.progress !== null}
			<span class="absolute bottom-0 left-0 h-1 bg-fc-accent" style="width: {ghost.progress * 100}%" aria-hidden="true"></span>
		{/if}
	</div>
{/if}

<style>
	.ghost {
		top: calc(var(--y) + 3px);
		left: var(--x);
		width: var(--w);
		height: 34px;
	}
</style>
