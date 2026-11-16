<script lang="ts">
	import { Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { Icon } from '@facile/muse';
	import { t } from '$lib/landing/i18n';
	import { icons } from '$lib/landing/icons';

	const OPEN = { start: 50, length: 44, cut: false, guide: false };

	let shot = $state({ ...OPEN });

	function reset() {
		shot = { ...OPEN };
	}

	function poster() {
		shot = { start: 40, length: 34, cut: true, guide: false };
	}

	async function script(act: Act) {
		await act.wait(1000);
		shot.start = 40;
		await act.wait(650);
		shot.guide = true;
		await act.wait(700);
		shot.guide = false;
		shot.length = 34;
		await act.wait(1100);
		shot.cut = true;
		await act.wait(2600);
		reset();
		await act.wait(600);
	}
</script>

<Tile title={t('tile.timeline.title')} body={t('tile.timeline.body')}>
	<Stage width={310} height={240} {script} {reset} {poster}>
		{#snippet children()}
			<div class="clap-dark panel">
				<div class="ruler"><span>0:00</span><span>0:02</span><span>0:04</span><span>0:06</span></div>
				<div class="rows">
					<span class="clap-clip clip" data-kind="text" style:left="12%" style:width="30%" style:top="4px">
						<Icon icon={icons.text} size={12} /><span>{t('demo.clip.title')}</span>
					</span>
					<span class="clap-clip clip" data-kind="video" style:left="0" style:width="40%" style:top="40px">
						<Icon icon={icons.video} size={12} /><span>{t('demo.video')}</span>
					</span>
					<span
						class="clap-clip clip moving"
						class:cut={shot.cut}
						data-kind="video"
						style:left="{shot.start}%"
						style:width="{shot.length}%"
						style:top="40px"
					>
						<Icon icon={icons.video} size={12} />
						<span class="slit"></span>
					</span>
					<span class="clap-clip clip" data-kind="audio" style:left="0" style:width="88%" style:top="76px">
						<Icon icon={icons.audio} size={12} /><span>{t('demo.audio')}</span>
					</span>
					<span class="guide" class:shown={shot.guide}></span>
					<span class="playhead" class:shown={shot.cut}></span>
				</div>
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.panel {
		position: absolute;
		inset: 24px 20px 16px;
		display: flex;
		flex-direction: column;
		padding: 10px 12px 12px;
		border-radius: var(--radius-fc-lg);
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.4);
	}
	.ruler {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		height: 24px;
		color: var(--ce-muted);
		font-size: 10px;
		font-variant-numeric: tabular-nums;
	}
	.rows {
		position: relative;
		flex: 1;
		border-radius: var(--radius-fc-md);
		background: var(--ce-panel);
	}
	.clip {
		position: absolute;
		height: 30px;
		transition:
			left 0.6s cubic-bezier(0.45, 0, 0.1, 1),
			width 0.8s cubic-bezier(0.45, 0, 0.1, 1);
	}
	.slit {
		position: absolute;
		top: 0;
		bottom: 0;
		left: 52%;
		width: 0;
		background: var(--ce-panel);
		transition: width 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.cut .slit {
		width: 4px;
	}
	.guide,
	.playhead {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		opacity: 0;
		transition: opacity 0.2s ease;
	}
	.guide {
		left: 40%;
		margin-left: -1px;
		background: var(--color-fc-warning);
	}
	.playhead {
		top: -8px;
		left: calc(40% + 34% * 0.52 + 1px);
		background: var(--color-fc-danger);
	}
	.shown {
		opacity: 1;
	}
	@media (prefers-reduced-motion: reduce) {
		.clip,
		.slit,
		.guide,
		.playhead {
			transition: none;
		}
	}
</style>
