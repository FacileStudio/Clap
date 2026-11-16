<script lang="ts">
	import { Badge, Button, icons } from '@facile/muse';
	import LandingShell from '$lib/components/landing/LandingShell.svelte';
	import { t } from '$lib/landing/i18n';
	import { marks } from '$lib/landing/marks';
</script>

<svelte:head><title>{t('downloads.title')}</title></svelte:head>

<LandingShell>
	<section
		class="mx-auto flex w-full max-w-2xl flex-col items-start gap-5 px-4 py-16 sm:px-6 sm:py-24"
		aria-labelledby="downloads-title"
	>
		<Badge tone="neutral" class="whitespace-nowrap">{t('downloads.soon')}</Badge>
		<h1 id="downloads-title" class="font-fc-title text-fc-3xl font-bold tracking-tight sm:text-5xl">
			{t('downloads.heading')}
		</h1>
		<p class="text-fc-md text-fc-fg-muted">{t('downloads.line')}</p>
		<div class="mt-2 flex w-full flex-col gap-3 sm:w-auto sm:flex-row">
			<Button href="/" size="lg" icon={marks.clap}>{t('downloads.back')}</Button>
			<Button href="https://github.com/FacileStudio/Clap" variant="secondary" size="lg" icon={icons.github}>
				{t('downloads.source')}
			</Button>
		</div>
	</section>
</LandingShell>
