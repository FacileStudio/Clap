<script lang="ts">
	import { Icon, Menu, icons } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { formatMoment, t } from '$lib/i18n';
	import type { ProjectSummary } from '$lib/types';
	import SceneFrame from './SceneFrame.svelte';

	let {
		project,
		onrename,
		ontrash
	}: {
		project: ProjectSummary;
		onrename: () => void;
		ontrash: () => void;
	} = $props();

	let still = $state(0);

	const actions = $derived([
		{ label: t('project.rename'), icon: icons.edit, onselect: onrename },
		{ label: t('project.trash'), icon: icons.remove, tone: 'danger' as const, onselect: ontrash }
	]);
</script>

<article class="flex flex-col gap-1 rounded-fc-lg bg-fc-component p-2" data-testid="project-card">
	<a
		href="/projects/{project.id}"
		class="block aspect-video overflow-hidden rounded-fc-md focus-visible:outline-2 focus-visible:outline-offset-2
			focus-visible:outline-fc-ring"
		aria-label={project.title}
	>
		<SceneFrame src={backend.frameUrl(project.id, { scene: 0, stamp: project.rev })} title={project.title} time={still} lazy onready={(info) => (still = Math.min(info.duration / 2, 2))} />
	</a>
	<div class="flex items-center gap-1 pl-2">
		<div class="min-w-0 flex-1 py-1">
			<h2 class="truncate text-fc-sm font-semibold text-fc-fg">{project.title}</h2>
			<p class="truncate text-fc-xs text-fc-fg-muted">{formatMoment(project.updated_at)}</p>
		</div>
		<Menu label={t('project.actions', { title: project.title })} items={actions} align="end">
			{#snippet trigger()}<Icon icon={icons.menu} size={18} />{/snippet}
		</Menu>
	</div>
</article>
