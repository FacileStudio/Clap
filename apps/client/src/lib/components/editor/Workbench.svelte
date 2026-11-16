<script lang="ts">
	import { onDestroy, untrack } from 'svelte';
	import { page } from '$app/state';
	import { Alert, Tabs, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { assistant as askAssistant, DEFAULT_ASSISTANT, rememberSidebar, sidebarOpen } from '$lib/bulle';
	import { t } from '$lib/i18n';
	import { Desk } from '$lib/project/desk.svelte';
	import { loadPanes, savePanes } from '$lib/project/panes';
	import { STARTERS } from '$lib/project/starters';
	import { session } from '$lib/session.svelte';
	import { darkWhileOpen } from '$lib/theme';
	import { runCommand } from '$lib/timeline/commands';
	import { claimed, commandOf } from '$lib/timeline/keys';
	import { total } from '$lib/timeline/model';
	import type { Project } from '$lib/types';
	import BulleSidebar from '../BulleSidebar.svelte';
	import ConflictBanner from '../ConflictBanner.svelte';
	import ErrorList from '../ErrorList.svelte';
	import Timeline from '../timeline/Timeline.svelte';
	import EditorHeader from './EditorHeader.svelte';
	import MediaBin from './MediaBin.svelte';
	import PreviewPane from './PreviewPane.svelte';
	import RenderDialog from './RenderDialog.svelte';
	import SourceEditor from './SourceEditor.svelte';
	import Splitter from './Splitter.svelte';

	let { project }: { project: Project } = $props();

	const EDITS = new Set(['split', 'delete', 'undo', 'redo']);
	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));
	const desk = new Desk(
		untrack(() => project),
		(message) => toast.danger(message)
	);
	const { saver, engine, transport } = desk;
	const owner = session.user?.email ?? '';
	const wide = window.matchMedia('(min-width: 1024px)');
	const restoreTheme = darkWhileOpen();

	let editor = $state<ReturnType<typeof SourceEditor>>();
	let tab = $state('media');
	let panes = $state(loadPanes());
	let compact = $state(!wide.matches);
	let renderOpen = $state(false);
	let assistant = $state(DEFAULT_ASSISTANT.name);
	let bulleOpen = $state(sidebarOpen(owner));

	const size = $derived(engine.spans);
	const tabs = $derived([
		{ id: 'media', label: t('editor.media') },
		{ id: 'source', label: t('editor.source') }
	]);
	const inserts = $derived(STARTERS.map((starter) => ({ label: t(starter.label), onselect: () => desk.insert(starter) })));

	$effect(() => {
		desk.view = editor;
	});
	$effect(() => editor?.mark(desk.marked));
	$effect(() => desk.watch());
	$effect(() => rememberSidebar(owner, bulleOpen));
	$effect(() => savePanes($state.snapshot(panes)));
	$effect(() => {
		if (!engine.busy) untrack(() => desk.settled());
	});
	$effect(() => {
		void saver.rev;
		if (saver.status === 'clean') untrack(() => void engine.sync(saver.buffer));
	});
	$effect(() => {
		transport.duration = total(engine.timeline);
		untrack(() => transport.seek(transport.time));
	});
	$effect(() => {
		const onchange = () => (compact = !wide.matches);
		wide.addEventListener('change', onchange);
		return () => wide.removeEventListener('change', onchange);
	});

	void askAssistant().then((found) => (assistant = found.name));

	onDestroy(() => {
		transport.pause();
		void saver.flush();
		saver.dispose();
		restoreTheme();
	});

	function onkeydown(event: KeyboardEvent) {
		const command = claimed(event) || renderOpen ? null : commandOf(event);
		if (!command || (compact && EDITS.has(command))) return;
		event.preventDefault();
		runCommand(command, engine, transport);
	}
</script>

<svelte:head><title>{desk.title} · Clap</title></svelte:head>

<svelte:window
	{onkeydown}
	onbeforeunload={(event) => {
		if (saver.status !== 'clean') event.preventDefault();
	}}
/>

{#snippet bin()}
	<div class="flex size-full min-h-0 flex-col overflow-hidden rounded-fc-lg bg-fc-component">
		<Tabs items={tabs} value={tab} onChange={(next) => (tab = next)} label={t('editor.panels')} class="shrink-0 p-2" />
		<div class="min-h-0 flex-1 {tab === 'media' ? '' : 'hidden'}">
			<MediaBin
				assets={desk.assets}
				uploads={desk.uploads}
				readonly={compact}
				onadd={(asset) => desk.place(asset)}
				onremove={(asset) => void desk.removeAsset(asset).catch(failed)}
			/>
		</div>
		<div class="min-h-0 flex-1 flex-col {tab === 'source' ? 'flex' : 'hidden'}">
			<div class="min-h-0 flex-1">
				<SourceEditor
					bind:this={editor}
					value={untrack(() => desk.text)}
					errors={saver.errors}
					label={t('editor.source')}
					onchange={(source) => desk.typed(source)}
					oncursor={(line) => desk.caret(line)}
				/>
			</div>
			{#if saver.errors.length > 0}
				<ErrorList errors={saver.errors} onpick={(line) => editor?.reveal(line)} />
			{/if}
		</div>
	</div>
{/snippet}

{#snippet preview()}
	<div class="size-full min-h-0 overflow-hidden rounded-fc-lg bg-fc-component">
		<PreviewPane src={backend.frameUrl(desk.id, { stamp: saver.rev })} width={size.width} height={size.height} {transport} />
	</div>
{/snippet}

{#snippet timeline()}
	<div class="size-full min-h-0 overflow-hidden rounded-fc-lg bg-fc-component">
		<Timeline {engine} {transport} assets={desk.assets} readonly={compact} />
	</div>
{/snippet}

<div class="flex h-full min-h-0 flex-col gap-2 p-3 {bulleOpen ? 'lg:pr-[412px]' : ''}" data-testid="workbench">
	<EditorHeader
		title={desk.title}
		status={saver.status}
		{inserts}
		{assistant}
		{bulleOpen}
		onrename={(next) => void desk.rename(next).catch(failed)}
		onrender={() => (renderOpen = true)}
		ontogglebulle={() => (bulleOpen = !bulleOpen)}
	/>
	{#if saver.status === 'conflict'}
		<ConflictBanner
			onreload={() => void saver.reload().catch(failed)}
			onoverwrite={() => void saver.overwrite().catch(failed)}
		/>
	{/if}
	{#if compact}
		<div class="flex min-h-0 flex-1 flex-col gap-3 overflow-auto">
			<div class="h-[calc(56.25vw+11rem)] max-h-[70dvh] shrink-0">{@render preview()}</div>
			<Alert tone="info" data-testid="compact-note">{t('editor.compact')}</Alert>
			<div class="h-64 shrink-0">{@render timeline()}</div>
			<div class="h-[70dvh] shrink-0">{@render bin()}</div>
		</div>
	{:else}
		<div class="flex min-h-0 flex-1 flex-col">
			<div class="flex min-h-0" style="flex: {panes.y} 1 0">
				<div class="min-w-0" style="flex: {panes.x} 1 0">{@render bin()}</div>
				<Splitter axis="x" bind:value={panes.x} />
				<div class="min-w-0" style="flex: {1 - panes.x} 1 0">{@render preview()}</div>
			</div>
			<Splitter axis="y" bind:value={panes.y} />
			<div class="min-h-0" style="flex: {1 - panes.y} 1 0">{@render timeline()}</div>
		</div>
	{/if}
</div>

<RenderDialog
	bind:open={renderOpen}
	id={desk.id}
	ratio={size.width / size.height}
	duration={transport.duration}
	playhead={transport.time}
/>
<BulleSidebar
	bind:open={bulleOpen}
	{assistant}
	page={{ app: 'clap', id: desk.id, title: desk.title, url: `${page.url.origin}/projects/${desk.id}` }}
	onready={(name) => (assistant = name)}
/>
