<script lang="ts">
	import { onDestroy, tick, untrack } from 'svelte';
	import { page } from '$app/state';
	import { Alert, Tabs, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { assistant as askAssistant, DEFAULT_ASSISTANT, rememberSidebar, sidebarOpen } from '$lib/bulle';
	import { t } from '$lib/i18n';
	import { Desk } from '$lib/project/desk.svelte';
	import { loadPanes, savePanes } from '$lib/project/panes';
	import { STARTERS } from '$lib/project/starters';
	import { session } from '$lib/session.svelte';
	import { darkWhileOpen } from '$lib/theme';
	import type { MenuHost } from '$lib/timeline/clip-menu';
	import { runCommand } from '$lib/timeline/commands';
	import { claimed, commandOf, type Command } from '$lib/timeline/keys';
	import { total } from '$lib/timeline/model';
	import type { StageBoxes } from '$lib/project/stage';
	import type { Point } from '$lib/stage/geometry';
	import type { Project } from '$lib/types';
	import BulleSidebar from '../BulleSidebar.svelte';
	import ProjectProblem from '../ProjectProblem.svelte';
	import Timeline from '../timeline/Timeline.svelte';
	import ClipInspector from './ClipInspector.svelte';
	import EditorHeader from './EditorHeader.svelte';
	import MediaBin from './MediaBin.svelte';
	import MediaPicker from './MediaPicker.svelte';
	import PreviewPane from './PreviewPane.svelte';
	import RenderDialog from './RenderDialog.svelte';
	import ShareDialog from './ShareDialog.svelte';
	import ShortcutsDialog from './ShortcutsDialog.svelte';
	import Splitter from './Splitter.svelte';

	let { project }: { project: Project } = $props();

	const EDITS = new Set<Command>(['split', 'delete', 'ripple', 'paste', 'duplicate', 'undo', 'redo']);
	const REVEAL_MS = 2500;
	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));
	const desk = new Desk(
		untrack(() => project),
		(message) => toast.danger(message)
	);
	const { revision, engine, transport } = desk;
	const owner = session.user?.email ?? '';
	const wide = window.matchMedia('(min-width: 1024px)');
	const restoreTheme = darkWhileOpen();

	let tab = $state('media');
	let panes = $state(loadPanes());
	let compact = $state(!wide.matches);
	let renderOpen = $state(false);
	let shareOpen = $state(false);
	let helpOpen = $state(false);
	let revealed = $state<string | null>(null);
	let picker = $state<ReturnType<typeof MediaPicker>>();
	let assistant = $state(DEFAULT_ASSISTANT.name);
	let bulleOpen = $state(sidebarOpen(owner));
	let boxes = $state<StageBoxes>({ t: 0, clips: [] });

	const size = $derived(engine.spans);
	const tabs = $derived([
		{ id: 'media', label: t('editor.media'), icon: icons.folder, badge: desk.assets.length || undefined },
		{ id: 'clip', label: t('editor.clip'), icon: icons.settings }
	]);
	const inserts = $derived(STARTERS.map((starter) => ({ label: t(starter.label), onselect: () => desk.insert(starter) })));

	$effect(() => desk.watch());
	$effect(() => rememberSidebar(owner, bulleOpen));
	$effect(() => savePanes($state.snapshot(panes)));
	$effect(() => {
		if (!engine.busy) untrack(() => desk.settled());
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

	const onStage = (at: Point) => ({ time: transport.time, track: 0, at });
	const run = (command: Command) => (command === 'help' ? (helpOpen = true) : runCommand(command, desk));
	const actions: MenuHost = {
		bench: desk,
		run,
		reveal: (src) => void reveal(src),
		has: (src) => desk.assets.some((asset) => asset.name === src),
		addMedia: (target) => picker?.pick(target)
	};

	void askAssistant().then((found) => (assistant = found.name));

	onDestroy(() => {
		transport.pause();
		restoreTheme();
	});

	function onkeydown(event: KeyboardEvent) {
		const command = claimed(event) || renderOpen || shareOpen || helpOpen ? null : commandOf(event);
		if (!command || (compact && EDITS.has(command))) return;
		event.preventDefault();
		run(command);
	}

	async function reveal(src: string) {
		tab = 'media';
		revealed = src;
		await tick();
		document.querySelector(`[data-testid="media-item"][data-name="${CSS.escape(src)}"]`)?.scrollIntoView({ block: 'nearest' });
		setTimeout(() => revealed === src && (revealed = null), REVEAL_MS);
	}
</script>

<svelte:head><title>{desk.title} · Clap</title></svelte:head>

<svelte:window
	{onkeydown}
	onbeforeunload={(event) => {
		if (engine.busy) event.preventDefault();
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
				{revealed}
				onadd={(asset) => desk.place(asset)}
				onaddabove={(asset) => desk.placeAbove(asset)}
				onremove={(asset) => void desk.removeAsset(asset).catch(failed)}
			/>
		</div>
		<div class="min-h-0 flex-1 {tab === 'clip' ? '' : 'hidden'}">
			<ClipInspector {engine} boxes={boxes.clips} readonly={compact} />
		</div>
	</div>
{/snippet}

{#snippet preview()}
	<div class="size-full min-h-0 overflow-hidden rounded-fc-lg bg-fc-component">
		<PreviewPane
			src={backend.frameUrl(desk.id, { stamp: revision.rev })}
			width={size.width}
			height={size.height}
			{transport}
			{engine}
			assets={desk.assets}
			measured={desk.uploads.measured}
			readonly={compact}
			{actions}
			bind:boxes
			ondropasset={(asset, at) => desk.drop(asset, onStage(at))}
			ondropfiles={(files, at) => desk.dropFiles(files, onStage(at))}
		/>
	</div>
{/snippet}

{#snippet timeline()}
	<div class="size-full min-h-0 overflow-hidden rounded-fc-lg bg-fc-component">
		<Timeline
			{engine}
			{transport}
			assets={desk.assets}
			readonly={compact}
			ghosts={desk.ghosts}
			measured={desk.uploads.measured}
			{actions}
			ondropasset={(asset, target) => desk.drop(asset, target)}
			ondropfiles={(files, target) => desk.dropFiles(files, target)}
		/>
	</div>
{/snippet}

<div class="flex h-full min-h-0 flex-col gap-2 p-3 {bulleOpen ? 'lg:pr-[412px]' : ''}" data-testid="workbench">
	<EditorHeader
		title={desk.title}
		saving={engine.busy}
		{inserts}
		{assistant}
		{bulleOpen}
		onrename={(next) => void desk.rename(next).catch(failed)}
		onrender={() => (renderOpen = true)}
		onshare={() => (shareOpen = true)}
		ontogglebulle={() => (bulleOpen = !bulleOpen)}
	/>
	{#if engine.problem}
		<ProjectProblem message={engine.problem} />
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

<MediaPicker bind:this={picker} onpick={(files, target) => desk.dropFiles(files, target)} />
<ShortcutsDialog bind:open={helpOpen} />
<ShareDialog
	bind:open={shareOpen}
	id={desk.id}
	title={desk.title}
	width={size.width}
	height={size.height}
	onrender={() => (renderOpen = true)}
/>
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
