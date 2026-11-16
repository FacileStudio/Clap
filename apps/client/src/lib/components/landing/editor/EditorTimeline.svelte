<script lang="ts">
	import { Icon } from '@facile/muse';
	import { icons } from '$lib/landing/icons';
	import { t } from '$lib/landing/i18n';
	import { SCENE, type EditorScene } from '$lib/landing/scene.svelte';

	let {
		scene,
		compact = false,
		rows = $bindable(),
		clip = $bindable()
	}: {
		scene: EditorScene;
		compact?: boolean;
		/** The clip area of the scene, which the cursor drops onto and trims along. */
		rows?: HTMLElement;
		/** The video clip, once placed. */
		clip?: HTMLElement;
	} = $props();

	const TOOLS = [icons.undo, icons.redo, icons.split, icons.remove, icons.text];
	const MARKS = [0, 2, 4, 6];
	const top = (row: number) => `calc(var(--row) * ${row} + 3px)`;
	const share = (seconds: number) => `${(seconds / SCENE.duration) * 100}%`;
</script>

<div class="timeline" class:compact>
	{#if !compact}
		<div class="tools">
			{#each TOOLS as icon (icon)}<span class="tool"><Icon {icon} size={15} /></span>{/each}
			<span class="tool on"><Icon icon={icons.snap} size={15} /></span>
			<span class="gap"></span>
			<span class="tool"><Icon icon={icons.minus} size={15} /></span>
			<span class="tool"><Icon icon={icons.plus} size={15} /></span>
		</div>
	{/if}
	<div class="tracks">
		<div class="gutter">
			{#each [2, 1, 0] as track (track)}<span>{track}</span>{/each}
		</div>
		<div class="lane">
			<div class="ruler">
				{#each MARKS as mark (mark)}<span style:left={share(mark)}>0:0{mark}</span>{/each}
			</div>
			<div class="head"><b>{t('demo.scene')}</b><span>0:0{SCENE.duration}</span></div>
			<div class="rows" bind:this={rows}>
				<span class="stripe"></span>
				<span
					class="clap-clip clip"
					class:shown={scene.titled}
					data-kind="text"
					style:top={top(0)}
					style:left={share(SCENE.titleStart)}
					style:width={share(SCENE.title)}
				>
					<Icon icon={icons.text} size={13} /><span>{t('demo.clip.title')}</span>
				</span>
				<span
					bind:this={clip}
					class="clap-clip clip"
					class:shown={scene.placed}
					data-kind="video"
					style:top={top(1)}
					style:left={share(SCENE.videoStart)}
					style:width={share(scene.length)}
				>
					<Icon icon={icons.video} size={13} /><span>{t('demo.video')}</span>
				</span>
				<span class="clap-clip clip shown" data-kind="audio" style:top={top(2)} style:left="0" style:width={share(SCENE.audio)}>
					<Icon icon={icons.audio} size={13} /><span>{t('demo.audio')}</span>
				</span>
				<span class="guide" class:shown={scene.guide} style:left={share(SCENE.audio)}></span>
			</div>
			<span class="playhead"><span></span></span>
		</div>
	</div>
</div>

<style>
	.timeline {
		--row: 36px;
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow: hidden;
		border-radius: var(--radius-fc-lg);
		background: var(--ce-panel);
	}
	.compact {
		--row: 30px;
	}
	.tools {
		display: flex;
		flex: none;
		align-items: center;
		gap: 4px;
		padding: 6px 10px 0;
		color: var(--ce-muted);
	}
	.tool {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border-radius: var(--radius-fc-md);
	}
	.tool.on {
		background: var(--ce-raise);
		color: var(--ce-fg);
	}
	.gap {
		flex: 1;
	}
	.tracks {
		display: flex;
		flex: 1;
		min-height: 0;
		padding: 0 10px 10px 0;
	}
	.gutter {
		display: flex;
		flex: none;
		flex-direction: column;
		width: 34px;
		padding-top: 46px;
		color: var(--ce-muted);
		font-size: 10px;
		font-variant-numeric: tabular-nums;
	}
	.compact .gutter {
		width: 24px;
	}
	.gutter span {
		display: flex;
		align-items: center;
		justify-content: center;
		height: var(--row);
	}
	.lane {
		position: relative;
		flex: 1;
		min-width: 0;
	}
	.ruler {
		position: relative;
		height: 22px;
		color: var(--ce-muted);
		font-size: 10px;
		font-variant-numeric: tabular-nums;
	}
	.ruler span {
		position: absolute;
		bottom: 4px;
		padding-left: 5px;
		line-height: 1;
	}
	.ruler span::before {
		position: absolute;
		top: -8px;
		left: 0;
		width: 1px;
		height: 7px;
		background: color-mix(in oklab, var(--ce-muted) 50%, transparent);
		content: '';
	}
	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		height: 24px;
		padding: 0 10px;
		border-radius: var(--radius-fc-md) var(--radius-fc-md) 0 0;
		background: var(--ce-raise);
		color: var(--ce-muted);
		font-size: 11px;
		white-space: nowrap;
	}
	.head b {
		color: var(--ce-fg);
		font-weight: 600;
	}
	.rows {
		position: relative;
		height: calc(var(--row) * 3);
		border-radius: 0 0 var(--radius-fc-md) var(--radius-fc-md);
		background: color-mix(in oklab, var(--ce-raise) 35%, var(--ce-panel));
	}
	.stripe {
		position: absolute;
		inset: var(--row) 0 var(--row);
		background: var(--ce-panel);
	}
	.clip {
		position: absolute;
		height: calc(var(--row) - 6px);
		opacity: 0;
		transform: scale(0.96);
		transition:
			opacity 0.3s ease,
			transform 0.35s cubic-bezier(0.25, 0.8, 0.25, 1),
			width 0.9s cubic-bezier(0.45, 0, 0.1, 1);
	}
	.clip.shown {
		opacity: 1;
		transform: none;
	}
	.guide {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		margin-left: -1px;
		background: var(--color-fc-warning);
		opacity: 0;
		transition: opacity 0.2s ease;
	}
	.guide.shown {
		opacity: 1;
	}
	.playhead {
		position: absolute;
		top: 4px;
		bottom: 0;
		left: 40.6%;
		width: 2px;
		background: var(--color-fc-danger);
	}
	.playhead span {
		position: absolute;
		top: 0;
		left: -4px;
		width: 10px;
		height: 10px;
		border-radius: var(--radius-fc-sm);
		background: var(--color-fc-danger);
	}
	@media (prefers-reduced-motion: reduce) {
		.clip,
		.guide {
			transition: none;
		}
	}
</style>
