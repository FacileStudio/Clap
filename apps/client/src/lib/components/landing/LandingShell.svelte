<script lang="ts">
	import type { Snippet } from 'svelte';
	import { LandingFooter, LandingHeader, SignInWithFacile, getI18n, signUpUrl } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { bindLanding, locale, rememberLocale, t, visitorLocale } from '$lib/landing/i18n';
	import { marks } from '$lib/landing/marks';
	import './landing.css';

	let { children }: { children: Snippet } = $props();

	const i18n = getI18n();
	const visitor = visitorLocale();
	if (visitor) i18n.set(visitor);
	bindLanding(() => i18n.locale);

	const links = $derived([
		{ href: '/#features', label: t('landing.nav.features') },
		{ href: '/#pricing', label: t('landing.nav.pricing') }
	]);
	const columns = $derived([
		{ title: 'Clap', links: [...links, { href: '/downloads', label: t('landing.nav.cli') }] },
		{
			title: 'Facile',
			links: [
				{ href: 'https://suite.facile.studio', label: t('landing.nav.suite'), external: true },
				{ href: signUpUrl(locale()), label: t('landing.nav.signup'), external: true }
			]
		}
	]);
</script>

<!--
@component
The public shell of Clap: the landing header and footer around a signed-out page.
-->

<div class="flex min-h-dvh flex-col overflow-x-clip bg-fc-page text-fc-fg" data-sveltekit-reload>
	<LandingHeader appName="Clap" icon={marks.clap} {links} skipHref="#main">
		{#snippet actions()}
			<SignInWithFacile href={backend.loginUrl()} size="sm" />
		{/snippet}
	</LandingHeader>

	<main id="main" class="flex flex-1 flex-col">{@render children()}</main>

	<LandingFooter
		appName="Clap"
		icon={marks.clap}
		tagline={t('landing.line')}
		{columns}
		legal={{ onLocaleChange: rememberLocale }}
	/>
</div>
