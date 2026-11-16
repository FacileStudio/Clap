<script lang="ts">
	import { Icon, IconButton, Slider } from '@facile/muse';
	import { icons } from '$lib/icons';
	import { t } from '$lib/i18n';
	import type { StageState } from '$lib/project/stage';
	import { FPS, timecode } from '$lib/project/timecode';
	import type { Transport } from '$lib/project/transport.svelte';
	import SceneFrame from '../SceneFrame.svelte';

	let {
		src,
		width,
		height,
		transport
	}: {
		src: string;
		/** The stage size in pixels. */
		width: number;
		height: number;
		transport: Transport;
	} = $props();

	let actual = $state(false);
	let safe = $state(false);
	let drawn = $state<StageState>({ time: 0, step: 0, scene: 0 });

	const ratio = $derived(width / height);
	const box = $derived(
		actual
			? `width: ${width}px; height: ${height}px`
			: `aspect-ratio: ${ratio}; width: 100%; max-width: calc(100cqh * ${ratio}); max-height: 100%`
	);

	function scrubbed(time: number) {
		transport.pause();
		transport.seek(time);
	}
</script>

<section class="flex size-full min-h-0 flex-col gap-2 p-3" aria-label={t('preview.title')}>
	<div
		class="flex min-h-0 flex-1 [container-type:size] {actual ? 'overflow-auto' : 'items-center justify-center'}"
		data-testid="preview"
		data-time={drawn.time.toFixed(3)}
		data-scene={drawn.scene}
	>
		<div class="relative shrink-0 overflow-hidden rounded-fc-md shadow-sm" style={box}>
			<SceneFrame {src} title={t('preview.title')} time={transport.time} playing={transport.playing} onstate={(state) => (drawn = state)} />
			{#if safe}
				<div class="pointer-events-none absolute inset-[5%] outline-1 outline-fc-warning outline-dashed" data-testid="safe"></div>
				<div class="pointer-events-none absolute inset-[10%] outline-1 outline-fc-info outline-dashed"></div>
			{/if}
		</div>
	</div>
	<div class="flex shrink-0 flex-wrap items-center gap-x-1 gap-y-2">
		<IconButton variant="ghost" aria-label={t('preview.start')} onclick={() => scrubbed(0)}>
			<Icon icon={icons.toStart} size={18} />
		</IconButton>
		<IconButton
			aria-label={transport.playing ? t('preview.pause') : t('preview.play')}
			data-testid="play"
			data-playing={transport.playing}
			onclick={() => transport.toggle()}
		>
			<Icon icon={transport.playing ? icons.pause : icons.play} size={18} />
		</IconButton>
		<IconButton variant="ghost" aria-label={t('preview.end')} onclick={() => scrubbed(transport.duration)}>
			<Icon icon={icons.toEnd} size={18} />
		</IconButton>
		<span
			class="px-2 font-fc-mono text-fc-sm whitespace-nowrap text-fc-fg tabular-nums"
			aria-label={t('preview.timecode')}
			data-testid="timecode"
		>
			{timecode(transport.time)}
			<span class="text-fc-fg-muted">/ {timecode(transport.duration)}</span>
		</span>
		<Slider
			class="min-w-24 flex-1"
			aria-label={t('preview.scrub')}
			min={0}
			max={Math.max(transport.duration, 1 / FPS)}
			step={1 / FPS}
			valueText={(value) => timecode(value)}
			bind:value={() => transport.time, scrubbed}
		/>
		<IconButton
			variant="ghost"
			aria-label={t('preview.zoom')}
			tooltip={actual ? t('preview.fit') : t('preview.actual')}
			aria-pressed={actual}
			class={actual ? 'bg-fc-surface' : ''}
			data-testid="preview-zoom"
			onclick={() => (actual = !actual)}
		>
			<Icon icon={icons.fit} size={18} />
		</IconButton>
		<IconButton
			variant="ghost"
			aria-label={t('preview.safe')}
			aria-pressed={safe}
			class={safe ? 'bg-fc-surface' : ''}
			data-testid="safe-toggle"
			onclick={() => (safe = !safe)}
		>
			<Icon icon={icons.safe} size={18} />
		</IconButton>
	</div>
</section>
