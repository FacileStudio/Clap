<script lang="ts">
	import { Alert, Button, Field, Input, Modal, Progress, RadioGroup, Select, icons, toast } from '@facile/muse';
	import { backend } from '$lib/backend';
	import { t } from '$lib/i18n';
	import { RenderJob, ended, renderSize } from '$lib/project/render.svelte';
	import type { Render } from '$lib/types';
	import RenderHistory from './RenderHistory.svelte';

	let {
		open = $bindable(),
		id,
		ratio,
		duration,
		playhead
	}: {
		open: boolean;
		id: string;
		/** The stage's width over its height. */
		ratio: number;
		/** The project's length in seconds. */
		duration: number;
		playhead: number;
	} = $props();

	const HEIGHTS = ['1080', '720', '480'];
	const RATES = ['30', '25', '24'];
	const LIMIT = 600;
	const job = new RenderJob();

	let height = $state('1080');
	let fps = $state('30');
	let range = $state('all');
	let from = $state('0');
	let to = $state('0');
	let renders = $state<Render[]>([]);

	const status = $derived(job.status);
	const span = $derived(range === 'all' ? duration : Number(to) - Number(from));
	const valid = $derived(span > 0 && span <= LIMIT && (range === 'all' || (Number(from) >= 0 && Number(to) <= duration)));
	const sizes = $derived(
		HEIGHTS.map((value) => {
			const size = renderSize(Number(value), ratio);
			return { value, label: `${size.width} × ${size.height}` };
		})
	);

	const refresh = () =>
		backend
			.renders(id)
			.then((found) => (renders = found))
			.catch(() => undefined);

	$effect(() => {
		if (!open) return;
		from = playhead.toFixed(2);
		to = duration.toFixed(2);
		void refresh();
	});

	$effect(() => {
		if (status && ended(status.state)) void refresh();
	});

	async function start() {
		const size = renderSize(Number(height), ratio);
		const part = range === 'all' ? {} : { range: [Number(from), Number(to)] as [number, number] };
		try {
			await job.start(id, { ...size, fps: Number(fps), ...part });
		} catch (error) {
			toast.danger(error instanceof Error && error.message ? error.message : t('render.failed'));
		}
	}

	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));
</script>

<Modal bind:open title={t('render.title')} size="md" showClose>
	<div class="flex flex-col gap-5" data-testid="render-dialog" data-state={status?.state ?? 'idle'} data-job={status?.id ?? ''}>
		{#if !status}
			<div class="grid gap-4 sm:grid-cols-2">
				<Field label={t('render.resolution')}>
					{#snippet children({ id: field })}<Select id={field} bind:value={height} options={sizes} />{/snippet}
				</Field>
				<Field label={t('render.fps')}>
					{#snippet children({ id: field })}
						<Select id={field} bind:value={fps} options={RATES.map((value) => ({ value, label: value }))} />
					{/snippet}
				</Field>
			</div>
			<RadioGroup
				aria-label={t('render.range')}
				orientation="horizontal"
				bind:value={range}
				options={[
					{ value: 'all', label: t('render.range.all') },
					{ value: 'part', label: t('render.range.part') }
				]}
			/>
			{#if range === 'part'}
				<div class="grid gap-4 sm:grid-cols-2">
					<Field label={t('render.in')}>
						{#snippet children({ id: field })}
							<Input id={field} type="number" min="0" step="0.01" bind:value={from} />
						{/snippet}
					</Field>
					<Field label={t('render.out')}>
						{#snippet children({ id: field })}
							<Input id={field} type="number" min="0" step="0.01" bind:value={to} />
						{/snippet}
					</Field>
				</div>
			{/if}
			<p class="text-fc-xs text-fc-fg-muted">{t('preview.note')} {t('render.limits')}</p>
		{:else}
			<div class="flex flex-col gap-3" role="status">
				<div class="flex items-center justify-between gap-3 text-fc-sm">
					<span class="font-semibold whitespace-nowrap text-fc-fg">{t(`render.state.${status.state}`)}</span>
					{#if status.frames > 0}
						<span class="whitespace-nowrap text-fc-fg-muted tabular-nums" data-testid="render-progress">
							{t('render.progress', { frame: status.frame, frames: status.frames })}
						</span>
					{/if}
				</div>
				{#if !ended(status.state)}
					<Progress value={status.frames > 0 ? status.frame : null} max={Math.max(status.frames, 1)} />
				{/if}
				{#if status.state === 'failed'}
					<Alert tone="danger">{status.error || t('render.failed')}</Alert>
				{/if}
				{#if status.state === 'done' && status.download_url}
					<video class="w-full rounded-fc-md bg-fc-surface" src={status.download_url} controls preload="metadata">
						<track kind="captions" />
					</video>
				{/if}
			</div>
		{/if}
		<RenderHistory {renders} onfollow={(render) => job.follow(render.id, render)} />
	</div>
	{#snippet footer()}
		{#if !status}
			<Button variant="secondary" onclick={() => (open = false)}>{t('render.close')}</Button>
			<Button disabled={!valid || job.starting} onclick={start} data-testid="render-start">{t('render.start')}</Button>
		{:else if !ended(status.state)}
			<Button variant="secondary" onclick={() => void job.cancel().catch(failed)} data-testid="render-cancel">
				{t('render.cancel')}
			</Button>
		{:else}
			<Button variant="secondary" onclick={() => job.reset()}>{t('render.again')}</Button>
			{#if status.state === 'done' && status.download_url}
				<Button icon={icons.download} href={status.download_url} download data-testid="render-download">
					{t('render.download')}
				</Button>
			{/if}
		{/if}
	{/snippet}
</Modal>
