<script lang="ts">
	import { page } from '$app/state';
	import { Button, EmptyState, Spinner, icons } from '@facile/muse';
	import { ProjectLoader } from '$lib/project/loader.svelte';
	import { t } from '$lib/i18n';
	import Workbench from '$lib/components/editor/Workbench.svelte';

	const loader = new ProjectLoader();

	$effect(() => void loader.load(page.params.id ?? ''));
</script>

{#if loader.missing}
	<div class="mx-auto flex w-full max-w-3xl flex-col px-4 py-16 sm:px-6">
		<EmptyState icon={icons.warning} title={t('project.missing')} description={t('project.missing.hint')}>
			<Button href="/">{t('project.back')}</Button>
		</EmptyState>
	</div>
{:else if !loader.project}
	<div class="flex h-full items-center justify-center"><Spinner /></div>
{:else}
	{#key loader.project.id}
		<Workbench project={loader.project} />
	{/key}
{/if}
