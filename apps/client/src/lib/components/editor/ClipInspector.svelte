<script lang="ts">
	import { Button, EmptyState, Field, Icon, Input, Select } from '@facile/muse';
	import { KIND_ICON, icons } from '$lib/icons';
	import { t } from '$lib/i18n';
	import type { StageBox } from '$lib/project/stage';
	import type { Box } from '$lib/stage/geometry';
	import { CLIP_TRANSITIONS, CLIP_TRANSITION_SECONDS, FITS, boxOps, fieldOps, fieldValue, type Field as Key } from '$lib/stage/inspect';
	import { centredBox, fullBox } from '$lib/stage/place';
	import type { Engine } from '$lib/timeline/engine.svelte';
	import { fullFrame } from '$lib/timeline/look';
	import { findClip } from '$lib/timeline/model';
	import type { ClipTransition, Fit, Op } from '$lib/types';

	let {
		engine,
		boxes,
		readonly = false
	}: {
		engine: Engine;
		/** The clip boxes the preview drew last, for clips whose source gives no box. */
		boxes: StageBox[];
		readonly?: boolean;
	} = $props();

	const id = $derived(engine.selection.size === 1 ? [...engine.selection][0] : null);
	const clip = $derived(id ? (findClip(engine.timeline, id)?.clip ?? null) : null);
	const stage = $derived({ width: fullFrame(engine.timeline).w, height: fullFrame(engine.timeline).h });
	const box = $derived<Box>(clip?.frame ?? boxes.find((one) => one.id === id) ?? fullFrame(engine.timeline));
	const placed = $derived(clip !== null && clip.kind !== 'audio');
	const media = $derived(clip?.kind === 'video' || clip?.kind === 'image');
	const sounds = $derived(clip?.kind === 'video' || clip?.kind === 'audio');
	const locked = $derived(readonly || engine.busy);
	const presets = $derived([
		{ value: '', label: t('inspector.none') },
		...CLIP_TRANSITIONS.map((name) => ({ value: name, label: t(`transition.${name}`) }))
	]);
	const fits = $derived(FITS.map((fit) => ({ value: fit, label: t(`inspector.fit.${fit}`) })));

	const commit = (ops: Op[] | null) => void (ops && !locked && engine.commit(ops));
	const typed = (key: Key) => (event: Event & { currentTarget: HTMLInputElement }) => {
		if (!clip) return;
		const ops = fieldOps(clip, box, key, event.currentTarget.value);
		if (ops) commit(ops);
		else event.currentTarget.value = fieldValue(clip, box, key);
	};
	const retext = (input: HTMLInputElement) => {
		if (!clip) return;
		const text = input.value.trim();
		if (text && text !== clip.text) commit([{ op: 'set_clip_text', clip: clip.id, text }]);
		else input.value = clip.text ?? '';
	};
	const preset = (side: 'enter' | 'exit') => (value: string) =>
		clip && commit([{ op: 'update_clip', clip: clip.id, [side]: (value || null) as ClipTransition | null }]);
</script>

{#snippet number(key: Key, label: string, unit = '')}
	<Field {label}>
		<div class="flex items-center gap-1.5">
			<Input
				type="text"
				inputmode="decimal"
				class="tabular-nums"
				value={clip ? fieldValue(clip, box, key) : ''}
				placeholder={key.endsWith('_duration') ? String(CLIP_TRANSITION_SECONDS) : ''}
				disabled={locked}
				data-testid="inspector-{key}"
				onchange={typed(key)}
			/>
			{#if unit}<span class="text-fc-xs text-fc-fg-muted">{unit}</span>{/if}
		</div>
	</Field>
{/snippet}

{#snippet motion(side: 'enter' | 'exit')}
	<div class="grid grid-cols-[1fr_7rem] items-end gap-2">
		<Field label={t(`inspector.${side}`)}>
			<Select
				options={presets}
				disabled={locked}
				data-testid="inspector-{side}"
				bind:value={() => clip?.[side] ?? '', preset(side)}
			/>
		</Field>
		{@render number(`${side}_duration`, t('inspector.seconds'), 's')}
	</div>
{/snippet}

<section class="flex size-full min-h-0 flex-col gap-4 overflow-auto p-3" aria-label={t('inspector.title')} data-testid="inspector">
	{#if !clip}
		<div data-testid="inspector-empty">
			<EmptyState icon={icons.settings} title={t('inspector.none.title')} description={t('inspector.empty')} />
		</div>
	{:else}
		<header class="flex items-center gap-2 text-fc-sm font-semibold text-fc-fg">
			<Icon icon={KIND_ICON[clip.kind]} size={16} />
			<span class="truncate">{clip.src?.split('/').pop() ?? t(`kind.${clip.kind}`)}</span>
			<span class="ml-auto font-fc-mono text-fc-xs font-normal text-fc-fg-muted">{clip.id}</span>
		</header>
		{#if clip.kind === 'text'}
			<Field label={t('inspector.text')}>
				<Input
					type="text"
					value={clip.text ?? ''}
					disabled={locked}
					data-testid="inspector-text"
					onchange={(event) => retext(event.currentTarget)}
				/>
			</Field>
		{/if}
		{#if placed}
			<div class="grid grid-cols-2 gap-2">
				{@render number('x', 'X', 'px')}
				{@render number('y', 'Y', 'px')}
				{@render number('w', t('inspector.width'), 'px')}
				{@render number('h', t('inspector.height'), 'px')}
				{@render number('rot', t('inspector.rotation'), '°')}
				{@render number('opacity', t('inspector.opacity'), '%')}
			</div>
			<div class="flex flex-wrap gap-2">
				<Button variant="secondary" size="sm" disabled={locked} data-testid="inspector-fill" onclick={() => commit(boxOps(clip, fullBox(stage)))}>
					<Icon icon={icons.fit} size={16} />
					{t('inspector.fill')}
				</Button>
				<Button variant="secondary" size="sm" disabled={locked} data-testid="inspector-centre" onclick={() => commit(boxOps(clip, centredBox(box, stage)))}>
					<Icon icon={icons.centre} size={16} />
					{t('inspector.centre')}
				</Button>
			</div>
		{/if}
		{#if media}
			<Field label={t('inspector.fit')}>
				<Select
					options={fits}
					disabled={locked}
					data-testid="inspector-fit"
					bind:value={() => clip.fit ?? 'cover', (value) => commit([{ op: 'update_clip', clip: clip.id, fit: value as Fit }])}
				/>
			</Field>
		{/if}
		<div class="grid grid-cols-2 gap-2">
			{@render number('start', t('inspector.start'), 's')}
			{@render number('duration', t('inspector.duration'), 's')}
			{#if sounds}{@render number('volume', t('inspector.volume'), '%')}{/if}
		</div>
		{@render motion('enter')}
		{@render motion('exit')}
	{/if}
</section>
