<script lang="ts">
	import { Icon } from '@facile/muse';
	import { icons } from '$lib/landing/icons';
	import { t } from '$lib/landing/i18n';
	import { SCENE, type EditorScene } from '$lib/landing/scene.svelte';
	import Footage from '../Footage.svelte';

	let {
		scene,
		compact = false,
		media = $bindable(),
		sourceTab = $bindable()
	}: {
		scene: EditorScene;
		compact?: boolean;
		/** The video item of the media bin, which the cursor picks up. */
		media?: HTMLElement;
		/** The Source tab, which the cursor clicks. */
		sourceTab?: HTMLElement;
	} = $props();

	const head = $derived(['---', `title: ${t('demo.project')}`, `duration: ${SCENE.duration}`, '---']);
	const audio = $derived([`<audio src="assets/${t('demo.audio')}"`, `  data-start="0" data-duration="${SCENE.audio}">`]);
	const video = $derived(`<video src="assets/${t('demo.video')}"`);
</script>

<div class="panel" class:compact>
	<div class="tabs">
		<span class="tab" class:on={scene.tab === 'media'}>{t('demo.media')}</span>
		<span class="tab" class:on={scene.tab === 'source'} bind:this={sourceTab}>{t('demo.source')}</span>
	</div>
	<div class="body">
		<ul class="bin" class:shown={scene.tab === 'media'}>
			<li bind:this={media} class:lifted={scene.dragging}>
				<span class="thumb"><Footage /><span class="time">0:06</span></span>
				<span class="name">{t('demo.video')}</span>
			</li>
			<li>
				<span class="thumb"><Icon icon={icons.audio} size={compact ? 16 : 22} /></span>
				<span class="name">{t('demo.audio')}</span>
			</li>
			<li>
				<span class="thumb"><Icon icon={icons.image} size={compact ? 16 : 22} /></span>
				<span class="name">logo.png</span>
			</li>
		</ul>
		<pre class="code clap-mono" class:shown={scene.tab === 'source'}>{#if !compact}{#each [...head, ...audio] as line, i (i)}<span
						class="dim">{line}</span
					>{'\n'}{/each}{/if}{#if scene.placed}<span class="dim">{video}</span>{'\n'}<span class="dim"
					>{'  '}data-start="{SCENE.videoStart}" data-duration="</span
				><span class="num" class:cut={scene.length !== SCENE.videoLong}>{scene.length}</span><span class="dim">"&gt;</span
				>{'\n'}{/if}<span class="typed">{scene.typed}</span>{#if scene.tab === 'source' && !scene.titled}<span class="caret"
				></span>{/if}</pre>
	</div>
</div>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		min-width: 0;
		overflow: hidden;
		border-radius: var(--radius-fc-lg);
		background: var(--ce-panel);
	}
	.tabs {
		display: flex;
		flex: none;
		gap: 4px;
		padding: 8px 8px 6px;
	}
	.tab {
		padding: 4px 10px;
		border-radius: var(--radius-fc-md);
		color: var(--ce-muted);
		font-size: 12px;
		font-weight: 500;
		transition:
			background 0.25s ease,
			color 0.25s ease;
	}
	.tab.on {
		background: var(--ce-raise);
		color: var(--ce-fg);
	}
	.body {
		position: relative;
		flex: 1;
		min-height: 0;
	}
	.bin,
	.code {
		position: absolute;
		inset: 0;
		margin: 0;
		opacity: 0;
		transition: opacity 0.25s ease;
	}
	.shown {
		opacity: 1;
	}
	.bin {
		display: grid;
		grid-template-columns: 1fr 1fr;
		align-content: start;
		gap: 8px;
		padding: 2px 10px 10px;
		list-style: none;
	}
	.compact .bin {
		grid-template-columns: repeat(3, 1fr);
		gap: 6px;
		padding: 0 8px 8px;
	}
	.bin li {
		display: flex;
		flex-direction: column;
		gap: 5px;
		min-width: 0;
		padding: 5px;
		border-radius: var(--radius-fc-md);
		background: var(--ce-raise);
		transition: opacity 0.2s ease;
	}
	.bin li.lifted {
		opacity: 0.45;
	}
	.thumb {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: center;
		aspect-ratio: 16 / 9;
		overflow: hidden;
		border-radius: var(--radius-fc-sm);
		background: var(--ce-field);
		color: var(--ce-muted);
	}
	.compact .thumb {
		aspect-ratio: auto;
		height: 30px;
	}
	.time {
		position: absolute;
		right: 4px;
		bottom: 4px;
		padding: 0 4px;
		border-radius: var(--radius-fc-xs);
		background: color-mix(in oklab, var(--ce-bg) 80%, transparent);
		color: var(--ce-fg);
		font-size: 10px;
		font-variant-numeric: tabular-nums;
	}
	.compact .time {
		display: none;
	}
	.name {
		overflow: hidden;
		padding: 0 2px;
		font-size: 11px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.code {
		overflow: hidden;
		padding: 2px 12px 10px;
		font-size: 10.5px;
		line-height: 17px;
		white-space: pre;
	}
	.compact .code {
		padding: 0 10px 8px;
		font-size: 10px;
		line-height: 14px;
	}
	.dim {
		color: var(--ce-muted);
	}
	.num {
		color: var(--ce-muted);
		transition: color 0.4s ease;
	}
	.num.cut {
		color: var(--color-fc-warning);
	}
	.typed {
		color: var(--ce-fg);
	}
	.caret {
		display: inline-block;
		width: 1.5px;
		height: 1.1em;
		margin-left: 1px;
		vertical-align: text-bottom;
		background: var(--ce-fg);
	}
</style>
