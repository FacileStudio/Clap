<script lang="ts">
	import { goto } from '$app/navigation';
	import { Alert, Button, EmptyState, Spinner, icons, toast } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
	import { Shelf } from '$lib/project/shelf.svelte';
	import { t } from '$lib/i18n';
	import type { ProjectSummary } from '$lib/types';
	import ProjectCard from '$lib/components/ProjectCard.svelte';
	import TitleDialog from '$lib/components/TitleDialog.svelte';

	const shelf = new Shelf();

	let dialog = $state<{ project: ProjectSummary | null } | null>(null);

	void shelf.load();

	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));

	async function create(title: string) {
		const project = await shelf.create(title);
		await goto(`/projects/${project.id}`);
	}

	async function submit(title: string) {
		const target = dialog?.project;
		try {
			if (target) await shelf.rename(target.id, title);
			else await create(title);
			dialog = null;
		} catch (error) {
			failed(error);
		}
	}

	async function trash(project: ProjectSummary) {
		try {
			await shelf.trash(project.id);
			const undo = { label: t('action.undo'), onClick: () => void shelf.restore(project.id).catch(failed) };
			toast.success(t('project.trashed', { title: project.title }), { action: undo });
		} catch (error) {
			failed(error);
		}
	}
</script>

<svelte:head><title>{t('projects.title')} · Clap</title></svelte:head>

<div class="mx-auto flex w-full max-w-fc-xl flex-col gap-8 px-4 py-8 pb-24 sm:px-6 md:px-10 md:py-10 md:pb-32">
	<header class="flex flex-wrap items-center gap-4">
		<div class="flex min-w-full flex-1 flex-col gap-1 sm:min-w-0">
			<h1 class="text-fc-2xl font-semibold whitespace-nowrap text-fc-fg">{t('projects.title')}</h1>
			<p class="text-fc-sm text-fc-fg-muted">{t('projects.subtitle')}</p>
		</div>
		<Button icon={icons.plus} onclick={() => (dialog = { project: null })} data-testid="new-project">{t('projects.new')}</Button>
	</header>

	{#if !shelf.ready}
		<div class="flex justify-center py-16"><Spinner /></div>
	{:else if shelf.failed}
		<Alert tone="danger">
			{t('projects.failed')}
			{#snippet action()}
				<Button size="sm" variant="secondary" onclick={() => shelf.load()}>{t('action.retry')}</Button>
			{/snippet}
		</Alert>
	{:else if shelf.projects.length === 0}
		<EmptyState icon={logoIcon} title={t('projects.empty')} description={t('projects.empty.hint')}>
			<Button icon={icons.plus} onclick={() => (dialog = { project: null })}>{t('projects.new')}</Button>
		</EmptyState>
	{:else}
		<ul class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3" data-testid="project-grid">
			{#each shelf.projects as project (project.id)}
				<li><ProjectCard {project} onrename={() => (dialog = { project })} ontrash={() => trash(project)} /></li>
			{/each}
		</ul>
	{/if}
</div>

<TitleDialog
	open={dialog !== null}
	heading={dialog?.project ? t('project.rename') : t('projects.new')}
	value={dialog?.project?.title}
	confirm={dialog?.project ? t('action.save') : t('action.create')}
	onsubmit={submit}
	onclose={() => (dialog = null)}
/>
