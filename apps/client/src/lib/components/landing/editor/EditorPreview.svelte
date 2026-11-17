<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Icon } from '@facile/muse';
	import { icons } from '$lib/landing/icons';
	import type { EditorScene } from '$lib/landing/scene.svelte';
	import Footage from '../Footage.svelte';

	let {
		scene,
		compact = false,
		children
	}: {
		scene: EditorScene;
		compact?: boolean;
		/** What floats over the picture: the render card. */
		children?: Snippet;
	} = $props();
</script>

<div class="pane" class:compact>
	<div class="screen">
		<div class="picture">
			<span class="shot" class:shown={scene.placed}><Footage /></span>
			<span class="veil" class:shown={scene.typed !== ''}></span>
			<span class="title" class:shown={scene.typed !== ''}>
				<span class="words">{scene.typed}</span>
				<span class="rule"></span>
			</span>
		</div>
		{@render children?.()}
	</div>
	<div class="transport">
		{#if !compact}<Icon icon={icons.toStart} size={15} />{/if}
		<span class="play"><Icon icon={icons.play} size={13} /></span>
		{#if !compact}<Icon icon={icons.toEnd} size={15} />{/if}
		<span class="code clap-mono">00:03.07 <span class="total">/ 00:08.00</span></span>
		<span class="slider"><span class="fill"></span><span class="knob"></span></span>
		{#if !compact}<Icon icon={icons.fit} size={15} />{/if}
	</div>
</div>

<style>
	.pane {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-width: 0;
		padding: 10px;
		border-radius: var(--radius-fc-lg);
		background: var(--ce-panel);
	}
	.compact {
		gap: 6px;
		padding: 8px;
	}
	.screen {
		position: relative;
		display: flex;
		flex: 1;
		align-items: center;
		justify-content: center;
		min-height: 0;
	}
	.picture {
		position: relative;
		height: 100%;
		aspect-ratio: 16 / 9;
		overflow: hidden;
		border-radius: var(--radius-fc-md);
		background: var(--ce-bg);
	}
	.shot,
	.veil {
		position: absolute;
		inset: 0;
		opacity: 0;
		transition: opacity 0.45s ease;
	}
	.veil {
		background: color-mix(in oklab, var(--color-fc-inverse) 28%, transparent);
	}
	.title {
		position: absolute;
		inset: 0;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		gap: 10px;
		padding: 0 8% 11%;
		color: var(--color-fc-on-inverse);
		opacity: 0;
		transform: translateY(14px);
		transition:
			opacity 0.5s ease,
			transform 0.8s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.words {
		font-family: var(--font-fc-title);
		font-size: 30px;
		font-weight: 700;
		line-height: 1.05;
		letter-spacing: -0.02em;
	}
	.compact .words {
		font-size: 22px;
	}
	.rule {
		width: 54px;
		height: 5px;
		border-radius: 3px;
		background: currentColor;
		transform: scaleX(0);
		transform-origin: left center;
		transition: transform 0.7s cubic-bezier(0.25, 0.8, 0.25, 1) 0.35s;
	}
	.shown {
		opacity: 1;
	}
	.title.shown {
		transform: none;
	}
	.title.shown .rule {
		transform: none;
	}
	.transport {
		display: flex;
		flex: none;
		align-items: center;
		gap: 10px;
		height: 26px;
		color: var(--ce-muted);
	}
	.play {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: 999px;
		background: var(--ce-fg);
		color: var(--ce-bg);
	}
	.code {
		color: var(--ce-fg);
		font-size: 11px;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.total {
		color: var(--ce-muted);
	}
	.slider {
		position: relative;
		flex: 1;
		height: 4px;
		border-radius: 2px;
		background: var(--ce-field);
	}
	.fill {
		position: absolute;
		inset: 0 60% 0 0;
		border-radius: 2px;
		background: var(--ce-fg);
	}
	.knob {
		position: absolute;
		top: -4px;
		left: calc(40% - 6px);
		width: 12px;
		height: 12px;
		border-radius: 999px;
		background: var(--ce-fg);
	}
	@media (prefers-reduced-motion: reduce) {
		.shot,
		.veil,
		.title,
		.rule {
			transition: none;
		}
	}
</style>
