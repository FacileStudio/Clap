<script lang="ts">
	import { Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { Icon, icons } from '@facile/muse';
	import { t } from '$lib/landing/i18n';
	import { marks } from '$lib/landing/marks';
	import Footage from '../Footage.svelte';

	let draft = $state('');
	let sent = $state(false);
	let added = $state(false);

	function reset() {
		draft = '';
		sent = false;
		added = false;
	}

	function poster() {
		draft = '';
		sent = true;
		added = true;
	}

	async function script(act: Act) {
		await act.wait(900);
		await act.type(t('tile.bulle.ask'), (value) => (draft = value), 34);
		await act.wait(450);
		draft = '';
		sent = true;
		await act.wait(1300);
		added = true;
		await act.wait(3600);
		reset();
		await act.wait(500);
	}
</script>

<Tile title={t('tile.bulle.title')} body={t('tile.bulle.body')}>
	<Stage width={476} height={260} compact={{ below: 420, width: 326, height: 262 }} {script} {reset} {poster}>
		{#snippet children({ compact })}
			<div class="chat" class:compact>
				<div class="head">
					<span class="mark"><Icon icon={marks.bulle} size={16} /></span>Bulle
				</div>
				<div class="thread">
					<p class="ask" class:shown={sent}>{t('tile.bulle.ask')}</p>
					<div class="result" class:shown={added}>
						<span class="clap-dark thumb">
							<Footage /><span class="veil"></span>
							<span class="words" class:shown={added}>{t('demo.headline')}</span>
						</span>
						<span class="note">
							<span class="ok"><Icon icon={icons.check} size={13} />{t('tile.bulle.done')}</span>
							<span class="meta">{t('demo.clip.title')} · 5 s</span>
						</span>
					</div>
				</div>
				<div class="composer">
					<span class="draft" class:placeholder={!draft}>{draft || t('tile.bulle.placeholder')}</span>
					<span class="send"><Icon icon={icons.arrow} size={15} /></span>
				</div>
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.chat {
		position: absolute;
		inset: 20px 56px 12px;
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px;
		border-radius: 18px;
		background: var(--color-fc-bg);
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.25);
		color: var(--color-fc-fg);
		font-family: var(--font-fc-body);
		font-size: 12.5px;
		line-height: 1.35;
	}
	.chat.compact {
		inset: 14px 16px 8px;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 8px;
		font-weight: 700;
	}
	.mark {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		border-radius: 999px;
		background: var(--color-fc-accent);
		color: var(--color-fc-accent-fg);
	}
	.thread {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 8px;
		min-height: 0;
	}
	.ask,
	.result {
		opacity: 0;
		transform: translateY(6px);
		transition:
			opacity 0.3s ease,
			transform 0.35s cubic-bezier(0.25, 0.8, 0.25, 1);
	}
	.ask {
		align-self: flex-end;
		max-width: 92%;
		margin: 0;
		padding: 6px 12px;
		border-radius: 14px;
		background: var(--color-fc-surface);
	}
	.result {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.thumb {
		position: relative;
		flex: none;
		width: 124px;
		aspect-ratio: 16 / 9;
		overflow: hidden;
		border-radius: var(--radius-fc-md);
	}
	.veil {
		position: absolute;
		inset: 0;
		background: color-mix(in oklab, var(--color-fc-inverse) 28%, transparent);
	}
	.words {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		padding: 0 9%;
		font-family: var(--font-fc-title);
		font-size: 12px;
		font-weight: 700;
		white-space: nowrap;
		opacity: 0;
		transform: translateY(6px);
		transition:
			opacity 0.5s ease 0.5s,
			transform 0.8s cubic-bezier(0.25, 0.8, 0.25, 1) 0.5s;
	}
	.shown {
		opacity: 1;
		transform: none;
	}
	.note {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		white-space: nowrap;
	}
	.ok {
		display: flex;
		align-items: center;
		gap: 5px;
		font-weight: 600;
	}
	.ok :global(svg) {
		color: var(--color-fc-success);
	}
	.meta {
		color: var(--color-fc-fg-muted);
		font-size: 11.5px;
	}
	.composer {
		display: flex;
		flex: none;
		align-items: center;
		gap: 8px;
		height: 36px;
		padding: 0 5px 0 14px;
		border-radius: 999px;
		background: var(--color-fc-surface);
	}
	.draft {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.placeholder {
		color: var(--color-fc-fg-muted);
	}
	.send {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 26px;
		height: 26px;
		border-radius: 999px;
		background: var(--color-fc-accent);
		color: var(--color-fc-accent-fg);
	}
	@media (prefers-reduced-motion: reduce) {
		.ask,
		.result,
		.words {
			transition: none;
		}
	}
</style>
