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
		clipTab = $bindable(),
		textField = $bindable()
	}: {
		scene: EditorScene;
		compact?: boolean;
		/** The video item of the media bin, which the cursor picks up. */
		media?: HTMLElement;
		/** The Clip tab, which the cursor clicks. */
		clipTab?: HTMLElement;
		/** The text field of the clip settings, which the cursor types the title into. */
		textField?: HTMLElement;
	} = $props();
</script>

<div class="panel" class:compact>
	<div class="tabs">
		<span class="tab" class:on={scene.tab === 'media'}><Icon icon={icons.folder} size={13} />{t('demo.media')}</span>
		<span class="tab" class:on={scene.tab === 'clip'} bind:this={clipTab}><Icon icon={icons.settings} size={13} />{t('demo.clip')}</span>
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
		<div class="inspector" class:shown={scene.tab === 'clip'}>
			<p class="head"><Icon icon={icons.text} size={13} /><span>{t('demo.clip.title')}</span><span class="id clap-mono">text-1</span></p>
			<span class="label">{t('demo.text')}</span>
			<span class="field" class:on={scene.tab === 'clip' && !scene.titled} bind:this={textField}
				>{scene.typed}{#if scene.tab === 'clip' && !scene.titled}<span class="caret"></span>{/if}</span
			>
			{#if !compact}
				<div class="pair">
					<span class="label">{t('demo.start')}</span><span class="label">{t('demo.length')}</span>
					<span class="field">{SCENE.titleStart} s</span><span class="field">{SCENE.title} s</span>
				</div>
			{/if}
		</div>
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
		display: flex;
		align-items: center;
		gap: 5px;
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
	.inspector {
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
	.inspector {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 2px 12px 10px;
	}
	.compact .inspector {
		gap: 4px;
		padding: 0 10px 8px;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 6px;
		margin: 0 0 4px;
		font-size: 12px;
		font-weight: 600;
	}
	.id {
		margin-left: auto;
		color: var(--ce-muted);
		font-size: 10px;
		font-weight: 400;
	}
	.label {
		color: var(--ce-muted);
		font-size: 10.5px;
	}
	.field {
		display: flex;
		align-items: center;
		min-height: 26px;
		padding: 0 8px;
		overflow: hidden;
		border-radius: var(--radius-fc-md);
		background: var(--ce-field);
		font-size: 11.5px;
		white-space: nowrap;
		transition: background 0.25s ease;
	}
	.field.on {
		background: var(--ce-raise);
	}
	.pair {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 4px 8px;
		margin-top: 4px;
		font-variant-numeric: tabular-nums;
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
