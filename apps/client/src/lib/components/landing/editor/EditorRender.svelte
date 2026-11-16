<script lang="ts">
	import { Icon } from '@facile/muse';
	import { icons } from '$lib/landing/icons';
	import { t } from '$lib/landing/i18n';
	import { FRAMES, type EditorScene } from '$lib/landing/scene.svelte';

	let {
		scene,
		compact = false,
		centered = false
	}: {
		scene: EditorScene;
		compact?: boolean;
		/** Centers the card in its frame; otherwise it hangs in the top right corner, under the Render button. */
		centered?: boolean;
	} = $props();

	const done = $derived(scene.job === 'done');
</script>

<div class="card" class:compact class:centered class:shown={scene.job !== 'idle'}>
	<div class="row">
		<b>{t(done ? 'demo.render.done' : 'demo.render.running')}</b>
		<span class="count">{t('demo.render.progress', { frame: scene.frame, frames: FRAMES })}</span>
	</div>
	<div class="swap">
		<span class="bar" class:shown={!done}><span style:width="{(scene.frame / FRAMES) * 100}%"></span></span>
		<span class="file" class:shown={done}>
			<span class="mark"><Icon icon={icons.video} size={14} /></span>
			<span class="name">{t('demo.render.file')}</span>
			<span class="chip">1080p</span>
			<span class="get"><Icon icon={icons.download} size={14} /></span>
		</span>
	</div>
</div>

<style>
	.card {
		position: absolute;
		top: 0;
		right: 0;
		z-index: 5;
		display: flex;
		flex-direction: column;
		gap: 12px;
		width: 290px;
		padding: 14px 16px 16px;
		border-radius: var(--radius-fc-lg);
		background: var(--ce-raise);
		box-shadow: 0 18px 40px -12px rgb(0 0 0 / 0.55);
		opacity: 0;
		transform: translateY(-8px) scale(0.97);
		transform-origin: top right;
		transition:
			opacity 0.3s ease,
			transform 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.card.compact {
		gap: 8px;
		width: 236px;
		padding: 10px 12px 12px;
	}
	.card.centered {
		top: 50%;
		right: 50%;
		translate: 50% -50%;
	}
	.card.shown {
		opacity: 1;
		transform: none;
	}
	.row {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		font-size: 13px;
		white-space: nowrap;
	}
	.row b {
		font-weight: 600;
	}
	.count {
		color: var(--ce-muted);
		font-size: 12px;
		font-variant-numeric: tabular-nums;
	}
	.swap {
		position: relative;
		height: 32px;
	}
	.bar,
	.file {
		position: absolute;
		inset: 0;
		opacity: 0;
		transition: opacity 0.3s ease;
	}
	.swap .shown {
		opacity: 1;
	}
	.bar {
		top: 13px;
		bottom: 13px;
		overflow: hidden;
		border-radius: 3px;
		background: var(--ce-field);
	}
	.bar span {
		display: block;
		height: 100%;
		border-radius: 3px;
		background: var(--ce-fg);
		transition: width 0.12s linear;
	}
	.file {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 0 4px 0 8px;
		border-radius: var(--radius-fc-md);
		background: var(--ce-field);
		font-size: 12px;
	}
	.mark {
		display: flex;
		color: var(--ce-muted);
	}
	.name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		font-family: var(--font-fc-mono);
		font-size: 11px;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.chip {
		color: var(--ce-muted);
		font-size: 11px;
	}
	.get {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 999px;
		background: var(--ce-fg);
		color: var(--ce-bg);
	}
	@media (prefers-reduced-motion: reduce) {
		.card,
		.bar,
		.file,
		.bar span {
			transition: none;
		}
	}
</style>
