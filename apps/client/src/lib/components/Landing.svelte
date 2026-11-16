<script lang="ts">
	import {
		DownloadSection,
		FeatureSplit,
		ForDevelopers,
		LANDING_HEADING,
		LandingCta,
		PricingPlans,
		WorksWith,
		type DeveloperRow,
		type FeatureItem,
		type WorksWithApp
	} from '@facile/muse';
	import { backend } from '$lib/backend';
	import { locale, t } from '$lib/landing/i18n';
	import { icons } from '$lib/landing/icons';
	import { CHECKSUMS, INSTALL, RELEASES } from '$lib/landing/install';
	import { marks } from '$lib/landing/marks';
	import EditorDemo from './landing/EditorDemo.svelte';
	import LandingShell from './landing/LandingShell.svelte';
	import LandingTiles from './landing/LandingTiles.svelte';

	const signInHref = backend.loginUrl();
	const FEATURES = [
		['media', icons.upload],
		['starters', icons.layers],
		['keys', icons.bolt],
		['undo', icons.history]
	] as const;

	const features = $derived<FeatureItem[]>(
		FEATURES.map(([key, icon]) => ({
			icon,
			title: t(`landing.feature.${key}.title`),
			text: t(`landing.feature.${key}.text`)
		}))
	);

	const apps = $derived<WorksWithApp[]>([
		{ name: 'Bulle', icon: marks.bulle, line: t('landing.with.bulle'), href: 'https://bulle.facile.studio' },
		{ name: 'Diapo', icon: marks.diapo, line: t('landing.with.diapo'), href: 'https://diapo.facile.studio' },
		{ name: 'Prisme', icon: marks.prisme, line: t('landing.with.prisme'), href: 'https://prisme.facile.studio' }
	]);

	const rows = $derived<DeveloperRow[]>([
		{ kind: 'mcp', text: t('landing.dev.mcp') },
		{ kind: 'cli', text: t('landing.dev.cli'), href: '/downloads' },
		{ kind: 'source', text: t('landing.dev.source'), href: 'https://github.com/FacileStudio/Clap' }
	]);

	const selfHostHref = $derived(`https://suite.facile.studio/${locale() === 'fr' ? '' : `${locale()}/`}auto-hebergement`);

	$effect(() => {
		document.querySelector('meta[name="description"]')?.setAttribute('content', t('landing.meta'));
	});

	const width = 'mx-auto w-full max-w-5xl px-4 sm:px-6';
</script>

<svelte:head><title>{t('landing.title')}</title></svelte:head>

<LandingShell>
	<div class="flex flex-col gap-20 pb-24 md:gap-28">
		<section class="{width} flex flex-col items-center pt-16 text-center sm:pt-24">
			<h1 class="max-w-3xl font-fc-title text-fc-3xl leading-tight font-bold tracking-tight text-balance sm:text-5xl">
				{t('landing.slogan')}
			</h1>
			<p class="mt-5 max-w-2xl text-fc-lg text-balance text-fc-fg-muted">{t('landing.subtitle')}</p>
			<LandingCta {signInHref} align="center" class="mt-8 w-full" />
			<div class="mt-12 w-full sm:mt-16"><EditorDemo /></div>
		</section>

		<LandingTiles class={width} />

		<FeatureSplit class={width} title={t('landing.features')} description={t('landing.featuresLine')} {features} />

		<WorksWith class={width} {apps} description={t('landing.with.line')} />

		<section class="{width} flex flex-col items-center gap-5 text-center">
			<h2 class={LANDING_HEADING}>{t('landing.cta.title')}</h2>
			<p class="text-fc-md text-fc-fg-muted">{t('landing.cta.line')}</p>
			<LandingCta {signInHref} align="center" class="w-full" />
			<p class="text-fc-sm text-fc-fg-muted">{t('landing.trust')}</p>
		</section>

		<PricingPlans
			id="pricing"
			class="{width} scroll-mt-24"
			title={t('landing.nav.pricing')}
			headingClass={LANDING_HEADING}
			highlight={['ai.access']}
			highlightOnly
			notes={{ '*': t('landing.plan.paid'), solo: t('landing.plan.included') }}
			recommended="pro"
			suiteHref="https://suite.facile.studio/#tarifs"
		/>

		<div class="{width} flex flex-col gap-10">
			<ForDevelopers {rows} description={t('landing.dev.line')} {selfHostHref} />
			<DownloadSection
				mode="device"
				allHref="/downloads"
				commands={INSTALL}
				releasesHref={RELEASES}
				checksumsHref={CHECKSUMS}
				headingLevel={3}
			/>
		</div>
	</div>
</LandingShell>
