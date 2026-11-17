<script lang="ts">
	import { Cursor, Stage, cursorAt, type Act } from '@facile/mise-en-scene';
	import { Icon } from '@facile/muse';
	import { t } from '$lib/landing/i18n';
	import { icons } from '$lib/landing/icons';
	import { marks } from '$lib/landing/marks';
	import { EditorScene } from '$lib/landing/scene.svelte';
	import { play, type Cast } from '$lib/landing/story';
	import EditorPanel from './editor/EditorPanel.svelte';
	import EditorPreview from './editor/EditorPreview.svelte';
	import EditorRender from './editor/EditorRender.svelte';
	import EditorTimeline from './editor/EditorTimeline.svelte';

	const scene = new EditorScene();

	let cursor = $state(cursorAt());
	let media = $state<HTMLElement>();
	let rows = $state<HTMLElement>();
	let clip = $state<HTMLElement>();
	let clipTab = $state<HTMLElement>();
	let textField = $state<HTMLElement>();
	let renderButton = $state<HTMLElement>();

	const cast: Cast = {
		scene,
		get cursor() {
			return cursor;
		},
		get headline() {
			return t('demo.headline');
		},
		get media() {
			return media;
		},
		get rows() {
			return rows;
		},
		get clip() {
			return clip;
		},
		get clipTab() {
			return clipTab;
		},
		get textField() {
			return textField;
		},
		get renderButton() {
			return renderButton;
		}
	};

	function reset() {
		scene.reset();
		cursor.visible = false;
		cursor.down = false;
	}

	async function script(act: Act) {
		const middle = act.point(rows, 0.6, -1.2);
		cursor.x = middle.x;
		cursor.y = middle.y;
		await play(act, cast);
		reset();
		await act.wait(700);
	}
</script>

<Stage
	width={960}
	height={600}
	compact={{ below: 640, width: 358, height: 560 }}
	{script}
	{reset}
	poster={() => scene.poster(t('demo.headline'))}
	label={t('demo.label')}
>
	{#snippet children({ compact })}
		<div class="clap-dark win" class:compact>
			<div class="bar">
				<span class="dot"></span><span class="dot"></span><span class="dot"></span>
				<span class="name">{t('demo.project')}</span>
				<span class="saved"><span></span>{#if !compact}{t('demo.saved')}{/if}</span>
				<span class="render" bind:this={renderButton}>
					<Icon icon={marks.clap} size={15} />{t('demo.render')}
				</span>
			</div>
			<div class="top">
				<div class="side"><EditorPanel {scene} {compact} bind:media bind:clipTab bind:textField /></div>
				<div class="main">
					<EditorPreview {scene} {compact}><EditorRender {scene} {compact} /></EditorPreview>
				</div>
			</div>
			<div class="bottom"><EditorTimeline {scene} {compact} bind:rows bind:clip /></div>
		</div>
		<span
			class="clap-dark clap-clip ghost"
			class:shown={scene.dragging}
			data-kind="video"
			style:translate="{cursor.x - 26}px {cursor.y - 15}px"
		>
			<Icon icon={icons.video} size={13} /><span>{t('demo.video')}</span>
		</span>
		<Cursor {cursor} />
	{/snippet}
</Stage>

<style>
	.win {
		position: absolute;
		top: 16px;
		left: 40px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		width: 880px;
		height: 568px;
		padding: 0 10px 10px;
		border-radius: 14px;
		box-shadow:
			0 30px 70px -20px rgb(0 0 0 / 0.35),
			0 8px 24px -8px rgb(0 0 0 / 0.18);
		text-align: left;
	}
	.win.compact {
		top: 12px;
		left: 12px;
		width: 334px;
		height: 536px;
		padding: 0 8px 8px;
	}
	.bar {
		display: flex;
		flex: none;
		align-items: center;
		gap: 8px;
		height: 44px;
		padding: 0 2px 0 4px;
	}
	.compact .bar {
		height: 40px;
	}
	.dot {
		width: 11px;
		height: 11px;
		border-radius: 999px;
		background: var(--ce-field);
	}
	.compact .dot {
		display: none;
	}
	.name {
		margin-left: 10px;
		overflow: hidden;
		font-family: var(--font-fc-title);
		font-size: 14px;
		font-weight: 600;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.compact .name {
		margin-left: 4px;
	}
	.saved {
		display: flex;
		flex: 1;
		align-items: center;
		gap: 6px;
		color: var(--ce-muted);
		font-size: 11px;
		white-space: nowrap;
	}
	.saved span {
		width: 7px;
		height: 7px;
		border-radius: 999px;
		background: var(--color-fc-success);
	}
	.render {
		display: flex;
		align-items: center;
		gap: 6px;
		height: 28px;
		padding: 0 12px 0 10px;
		border-radius: var(--radius-fc-pill);
		background: var(--ce-fg);
		color: var(--ce-bg);
		font-size: 12px;
		font-weight: 600;
		white-space: nowrap;
	}
	.top {
		display: flex;
		flex: none;
		gap: 8px;
		height: 300px;
	}
	.side {
		display: flex;
		flex: none;
		width: 290px;
	}
	.main,
	.bottom {
		display: flex;
		flex: 1;
		min-width: 0;
		min-height: 0;
	}
	.side > :global(*),
	.main > :global(*),
	.bottom > :global(*) {
		flex: 1;
	}
	.compact .top {
		flex-direction: column-reverse;
		height: 324px;
	}
	.compact .side {
		width: auto;
		height: 120px;
	}
	.ghost {
		position: absolute;
		top: 0;
		left: 0;
		z-index: 20;
		width: 150px;
		height: 30px;
		box-shadow: 0 10px 24px -8px rgb(0 0 0 / 0.5);
		opacity: 0;
		transition:
			translate 0.9s cubic-bezier(0.45, 0, 0.1, 1),
			opacity 0.2s ease;
	}
	.ghost.shown {
		opacity: 0.92;
	}
	@media (prefers-reduced-motion: reduce) {
		.ghost {
			transition: none;
		}
	}
</style>
