<script lang="ts">
	import { DownloadSection } from '@facile/muse';
	import LandingShell from '$lib/components/landing/LandingShell.svelte';
	import { t } from '$lib/landing/i18n';
	import { CHECKSUMS, INSTALL, RELEASES } from '$lib/landing/install';
</script>

<svelte:head><title>{t('downloads.title')}</title></svelte:head>

<LandingShell>
	<section
		class="mx-auto flex w-full max-w-2xl flex-col items-start gap-8 px-4 py-16 sm:px-6 sm:py-24"
		aria-labelledby="downloads-title"
	>
		<div class="flex flex-col gap-3">
			<h1 id="downloads-title" class="font-fc-title text-fc-3xl font-bold tracking-tight sm:text-5xl">
				{t('downloads.heading')}
			</h1>
			<p class="text-fc-md text-fc-fg-muted">{t('downloads.line')}</p>
		</div>
		<DownloadSection class="w-full" commands={INSTALL} releasesHref={RELEASES} checksumsHref={CHECKSUMS} />
	</section>
</LandingShell>
