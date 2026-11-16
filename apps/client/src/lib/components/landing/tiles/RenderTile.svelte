<script lang="ts">
	import { Stage, Tile, type Act } from '@facile/mise-en-scene';
	import { t } from '$lib/landing/i18n';
	import { EditorScene, FRAMES } from '$lib/landing/scene.svelte';
	import EditorRender from '../editor/EditorRender.svelte';
	import Footage from '../Footage.svelte';

	const scene = new EditorScene();

	function reset() {
		scene.reset();
		scene.job = 'running';
	}

	async function script(act: Act) {
		await act.wait(700);
		for (let frame = 6; frame <= FRAMES; frame += 6) {
			scene.frame = frame;
			await act.wait(75);
		}
		scene.job = 'done';
		await act.wait(3000);
		reset();
		await act.wait(300);
	}
</script>

<Tile title={t('tile.render.title')} body={t('tile.render.body')}>
	<Stage width={310} height={240} {script} {reset} poster={() => scene.poster('')}>
		{#snippet children()}
			<div class="clap-dark panel">
				<Footage /><span class="veil"></span>
				<EditorRender {scene} compact centered />
			</div>
		{/snippet}
	</Stage>
</Tile>

<style>
	.panel {
		position: absolute;
		inset: 24px 20px 16px;
		overflow: hidden;
		border-radius: var(--radius-fc-lg);
		box-shadow: 0 14px 30px -14px rgb(0 0 0 / 0.4);
	}
	.veil {
		position: absolute;
		inset: 0;
		background: color-mix(in oklab, var(--color-fc-inverse) 40%, transparent);
	}
	.panel :global(.card.compact) {
		width: 238px;
	}
</style>
