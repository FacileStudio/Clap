<script lang="ts">
	import { Caret, Stage, Terminal, Tile, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/landing/i18n';

	type Line = { kind: 'cmd' | 'ok'; text: string };

	const name = $derived(t('tile.cli.project'));
	const story = $derived<Line[]>([
		{ kind: 'cmd', text: `clap new ${name}` },
		{ kind: 'ok', text: `${name}/clap.md` },
		{ kind: 'cmd', text: `clap serve ${name}` },
		{ kind: 'cmd', text: `clap render ${name}` },
		{ kind: 'ok', text: `${name}.mp4` }
	]);

	let lines = $state<Line[]>([]);

	function reset() {
		lines = [];
	}

	async function script(act: Act) {
		await act.wait(600);
		for (const line of story) {
			if (line.kind === 'ok') lines.push(line);
			else {
				lines.push({ kind: 'cmd', text: '' });
				const at = lines.length - 1;
				await act.type(line.text, (value) => (lines[at].text = value), 60);
			}
			await act.wait(line.kind === 'ok' ? 800 : 400);
		}
		await act.wait(2400);
		reset();
		await act.wait(400);
	}
</script>

<Tile class="cli" title={t('tile.cli.title')} body={t('tile.cli.body')}>
	<Stage width={310} height={240} {script} {reset} poster={() => (lines = [...story])}>
		{#snippet children()}
			<Terminal style="top:24px;left:20px;right:20px;bottom:16px">
				{#each lines as line, i (i)}
					<div class="line">
						{#if line.kind === 'cmd'}
							<span class="dim">$</span>
							{line.text}{#if i === lines.length - 1}<Caret />{/if}
						{:else}
							<span class="ok">✓</span> <span class="dim">{line.text}</span>
						{/if}
					</div>
				{/each}
				{#if lines.at(-1)?.kind !== 'cmd'}
					<div class="line"><span class="dim">$</span> <Caret /></div>
				{/if}
			</Terminal>
		{/snippet}
	</Stage>
</Tile>

<style>
	:global(.cli .mes-tile-head) {
		flex-wrap: wrap;
	}
	.line {
		white-space: nowrap;
	}
	.dim {
		color: var(--color-fc-on-inverse-muted);
	}
	.ok {
		color: color-mix(in oklab, var(--color-fc-success), var(--color-fc-on-inverse) 35%);
	}
</style>
