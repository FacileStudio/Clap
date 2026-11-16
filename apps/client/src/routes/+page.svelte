<script lang="ts">
	import { EmptyState } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
</script>

<svelte:head><title>Projets · Clap</title></svelte:head>

<div class="mx-auto flex w-full max-w-fc-xl flex-col gap-8 px-4 py-8 pb-24 sm:px-6 md:px-10 md:py-10 md:pb-32">
	<header class="flex flex-col gap-1">
		<h1 class="text-fc-2xl font-semibold whitespace-nowrap text-fc-fg">Projets</h1>
		<p class="text-fc-sm text-fc-fg-muted">Chaque projet est un fichier texte : scènes, clips et titres.</p>
	</header>

	<EmptyState
		icon={logoIcon}
		title="Aucun projet"
		description="Les projets arrivent bientôt : l’éditeur, l’aperçu et le rendu MP4 sont en construction."
	/>
</div>
