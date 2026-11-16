<script lang="ts">
	import { LANDING_HEADING } from '@facile/muse';
	import { t } from '$lib/landing/i18n';
	import BulleTile from './tiles/BulleTile.svelte';
	import CliTile from './tiles/CliTile.svelte';
	import FileTile from './tiles/FileTile.svelte';
	import RenderTile from './tiles/RenderTile.svelte';
	import TimelineTile from './tiles/TimelineTile.svelte';

	let { class: className = '' }: { class?: string } = $props();
</script>

<section id="features" aria-labelledby="tiles-title" class="{className} scroll-mt-24">
	<h2 id="tiles-title" class="mb-10 text-center {LANDING_HEADING}">{t('landing.tiles')}</h2>
	<div class="grid gap-4 md:grid-cols-6 md:gap-6">
		<div class="grid md:col-span-3"><FileTile /></div>
		<div class="grid md:col-span-3"><BulleTile /></div>
		<div class="grid md:col-span-2"><TimelineTile /></div>
		<div class="grid md:col-span-2"><RenderTile /></div>
		<div class="grid md:col-span-2"><CliTile /></div>
	</div>
</section>
