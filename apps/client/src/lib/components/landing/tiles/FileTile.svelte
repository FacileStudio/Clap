<script lang="ts">
	import { Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/landing/i18n';
	import Footage from '../Footage.svelte';

	const YEAR = ' 2026';

	let extra = $state('');
	let typing = $state(false);

	function reset() {
		extra = '';
		typing = false;
	}

	function poster() {
		extra = YEAR;
		typing = false;
	}

	async function script(act: Act) {
		await act.wait(1200);
		typing = true;
		await act.wait(700);
		await act.type(YEAR, (value) => (extra = value), 170);
		await act.wait(2800);
		reset();
		await act.wait(500);
	}
</script>

<Tile title={t('tile.file.title')} body={t('tile.file.body')}>
	<Stage width={476} height={260} compact={{ below: 420, width: 326, height: 262 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div class="scene" class:compact>
				<div class="clap-dark picture">
					<Footage />
					<span class="veil"></span>
					<span class="words">{t('tile.file.before')}{extra}</span>
				</div>
				<pre class="clap-dark clap-mono source"><span class="dim">---{'\n'}duration: 5{'\n'}---{'\n'}&lt;h1 data-start="0.5"&gt;</span
					>{'\n'}  {t('tile.file.before')}{extra}{#if typing}<span class="caret"></span>{/if}{'\n'}<span class="dim">&lt;/h1&gt;</span
					></pre>
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.scene {
		position: absolute;
		inset: 0;
	}
	.picture {
		position: absolute;
		top: 22px;
		right: 24px;
		width: 304px;
		aspect-ratio: 16 / 9;
		overflow: hidden;
		border-radius: var(--radius-fc-lg);
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.4);
	}
	.compact .picture {
		top: 14px;
		right: 16px;
		width: 236px;
	}
	.source {
		position: absolute;
		bottom: 12px;
		left: 24px;
		width: 194px;
		margin: 0;
		padding: 12px 14px;
		border-radius: var(--radius-fc-lg);
		box-shadow: 0 14px 30px -10px rgb(0 0 0 / 0.5);
		font-size: 11px;
		line-height: 17px;
		white-space: pre;
	}
	.compact .source {
		bottom: 8px;
		left: 16px;
		width: 186px;
		padding: 9px 12px;
		line-height: 15px;
	}
	.dim {
		color: var(--ce-muted);
	}
	.caret {
		display: inline-block;
		width: 1.5px;
		height: 1.1em;
		margin-left: 1px;
		vertical-align: text-bottom;
		background: var(--ce-fg);
	}
	.veil {
		position: absolute;
		inset: 0;
		background: color-mix(in oklab, var(--color-fc-inverse) 28%, transparent);
	}
	.words {
		position: absolute;
		inset: 0 0 0 38%;
		display: flex;
		align-items: center;
		font-family: var(--font-fc-title);
		font-size: 26px;
		font-weight: 700;
		letter-spacing: -0.02em;
		white-space: nowrap;
	}
	.compact .words {
		inset: 0 0 34% 30%;
		font-size: 22px;
	}
</style>
