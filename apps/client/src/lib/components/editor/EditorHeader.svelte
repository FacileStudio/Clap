<script lang="ts">
	import { Blob } from '@facile/blob';
	import { Button, Icon, Menu, icons, type MenuEntry } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
	import type { SaveStatus } from '$lib/project/saver.svelte';
	import { t } from '$lib/i18n';
	import type { AppKey } from '$lib/i18n/messages';

	let {
		title,
		status,
		inserts,
		assistant,
		bulleOpen,
		onrename,
		onrender,
		ontogglebulle
	}: {
		title: string;
		status: SaveStatus;
		/** The ready-made scenes of the insert menu. */
		inserts: MenuEntry[];
		/** The name of the person's Bulle assistant. */
		assistant: string;
		bulleOpen: boolean;
		onrename: (title: string) => void;
		onrender: () => void;
		ontogglebulle: () => void;
	} = $props();

	const STATUS: Record<SaveStatus, AppKey> = {
		clean: 'save.clean',
		dirty: 'save.dirty',
		saving: 'save.saving',
		invalid: 'save.invalid',
		conflict: 'save.conflict',
		failed: 'save.failed'
	};

	const DOT: Record<SaveStatus, string> = {
		clean: 'bg-fc-success',
		dirty: 'bg-fc-fg-muted',
		saving: 'bg-fc-fg-muted motion-safe:animate-pulse',
		invalid: 'bg-fc-danger',
		conflict: 'bg-fc-warning',
		failed: 'bg-fc-danger'
	};

	let draft = $state('');

	$effect(() => {
		draft = title;
	});

	function commit() {
		const next = draft.trim();
		if (next && next !== title) onrename(next);
		else draft = title;
	}

	const ask = $derived(t('bulle.ask', { name: assistant }));
</script>

<header class="flex flex-wrap items-center gap-x-2 gap-y-1">
	<Button variant="ghost" size="sm" icon={icons.chevronLeft} href="/" aria-label={t('projects.title')}>
		<span class="hidden sm:inline">{t('projects.title')}</span>
	</Button>
	<input
		class="h-11 min-w-0 flex-1 rounded-fc-md bg-transparent px-2 text-fc-lg font-semibold text-fc-fg
			placeholder:text-fc-fg-muted hover:bg-fc-surface focus-visible:bg-fc-surface focus-visible:outline-2
			focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
		aria-label={t('project.title')}
		placeholder={t('project.untitled')}
		maxlength="200"
		bind:value={draft}
		onblur={commit}
		onkeydown={(event) => {
			if (event.key === 'Enter') event.currentTarget.blur();
		}}
	/>
	<span
		class="relative flex items-center gap-2 px-2 text-fc-xs whitespace-nowrap text-fc-fg-muted"
		role="status"
		data-testid="save-status"
		data-status={status}
	>
		<span class="size-2 shrink-0 rounded-fc-full {DOT[status]}" aria-hidden="true"></span>
		<span class="max-sm:sr-only">{t(STATUS[status])}</span>
	</span>
	<div class="ml-auto flex shrink-0 items-center gap-1">
		<Menu label={t('insert.label')} items={inserts} align="end">
			{#snippet trigger()}
				<span class="flex items-center gap-2">
					<Icon icon={icons.plus} size={16} /><span class="hidden md:inline">{t('insert.label')}</span>
				</span>
			{/snippet}
		</Menu>
		<button
			type="button"
			class="flex min-h-11 items-center gap-2 rounded-fc-md px-2 text-fc-sm text-fc-fg hover:bg-fc-surface
				focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring
				{bulleOpen ? 'bg-fc-surface' : ''}"
			aria-pressed={bulleOpen}
			aria-label={ask}
			data-testid="bulle-toggle"
			onclick={ontogglebulle}
		>
			<Blob name={assistant} size={24} />
			<span class="hidden xl:inline">{ask}</span>
		</button>
		<Button size="sm" icon={logoIcon} onclick={onrender} data-testid="render-open">
			<span class="hidden sm:inline">{t('render.open')}</span>
		</Button>
	</div>
</header>
