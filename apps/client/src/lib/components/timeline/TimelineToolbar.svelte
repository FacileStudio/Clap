<script lang="ts">
	import { Icon, IconButton } from '@facile/muse';
	import { icons } from '$lib/icons';
	import { t } from '$lib/i18n';
	import type { AppKey } from '$lib/i18n/messages';
	import type { Engine } from '$lib/timeline/engine.svelte';
	import type { Command } from '$lib/timeline/keys';

	let {
		engine,
		snapping = $bindable(),
		oncommand,
		ontext,
		onzoom
	}: {
		engine: Engine;
		snapping: boolean;
		oncommand: (command: Command) => void;
		ontext: () => void;
		/** Zooms by `factor`; 0 shows the whole project. */
		onzoom: (factor: number) => void;
	} = $props();

	type Tool = { label: AppKey; icon: string; keys?: string; disabled?: boolean; run: () => void; testid: string };

	const picked = $derived(engine.selection.size > 0);
	const edits = $derived<Tool[]>([
		{ label: 'timeline.undo', icon: icons.undo, keys: 'Ctrl+Z', disabled: !engine.canUndo, run: () => oncommand('undo'), testid: 'undo' },
		{ label: 'timeline.redo', icon: icons.redo, keys: 'Ctrl+Shift+Z', disabled: !engine.canRedo, run: () => oncommand('redo'), testid: 'redo' },
		{ label: 'timeline.split', icon: icons.split, keys: 'S', run: () => oncommand('split'), testid: 'split' },
		{ label: 'timeline.delete', icon: icons.remove, keys: '⌫', disabled: !picked, run: () => oncommand('delete'), testid: 'delete' },
		{ label: 'timeline.text', icon: icons.text, run: ontext, testid: 'add-text' }
	]);
	const zooms = $derived<Tool[]>([
		{ label: 'timeline.zoom.out', icon: icons.minus, run: () => onzoom(0.8), testid: 'zoom-out' },
		{ label: 'timeline.zoom.fit', icon: icons.fit, run: () => onzoom(0), testid: 'zoom-fit' },
		{ label: 'timeline.zoom.in', icon: icons.plus, run: () => onzoom(1.25), testid: 'zoom-in' }
	]);
</script>

{#snippet tool(item: Tool)}
	<IconButton
		variant="ghost"
		aria-label={t(item.label)}
		tooltip={item.keys ? `${t(item.label)} (${item.keys})` : t(item.label)}
		disabled={item.disabled || engine.busy}
		data-testid={item.testid}
		onclick={item.run}
	>
		<Icon icon={item.icon} size={18} />
	</IconButton>
{/snippet}

<div class="flex shrink-0 items-center gap-1 px-2 py-1" role="toolbar" aria-label={t('timeline.tools')}>
	{#each edits as item (item.testid)}{@render tool(item)}{/each}
	<IconButton
		variant="ghost"
		aria-label={t('timeline.snap')}
		aria-pressed={snapping}
		class={snapping ? 'bg-fc-surface' : ''}
		data-testid="snap"
		onclick={() => (snapping = !snapping)}
	>
		<Icon icon={icons.snap} size={18} />
	</IconButton>
	<span class="flex-1"></span>
	{#each zooms as item (item.testid)}{@render tool(item)}{/each}
</div>
