<script lang="ts">
	import { Badge, Button, icons } from '@facile/muse';
	import { formatMoment, t } from '$lib/i18n';
	import type { Render } from '$lib/types';

	let { renders, onfollow }: { renders: Render[]; onfollow: (render: Render) => void } = $props();

	const TONE = { queued: 'neutral', running: 'info', done: 'success', failed: 'danger', cancelled: 'neutral' } as const;
</script>

<section class="flex flex-col gap-2" aria-label={t('render.history')} data-testid="render-history">
	<h3 class="text-fc-sm font-semibold text-fc-fg">{t('render.history')}</h3>
	{#if renders.length === 0}
		<p class="text-fc-sm text-fc-fg-muted">{t('render.history.empty')}</p>
	{/if}
	<ul class="flex max-h-48 flex-col gap-1 overflow-auto">
		{#each renders as render (render.id)}
			<li class="flex min-h-11 items-center gap-2 rounded-fc-md bg-fc-surface py-1 pr-1 pl-3" data-state={render.state}>
				<Badge tone={TONE[render.state]} class="whitespace-nowrap">{t(`render.state.${render.state}`)}</Badge>
				<span class="min-w-0 flex-1 truncate text-fc-xs text-fc-fg-muted">
					{render.created_at ? formatMoment(render.created_at) : render.id}
				</span>
				{#if render.state === 'done' && render.download_url}
					<Button size="sm" variant="ghost" icon={icons.download} href={render.download_url} download>MP4</Button>
				{:else if render.state === 'queued' || render.state === 'running'}
					<Button size="sm" variant="ghost" icon={icons.eye} onclick={() => onfollow(render)}>
						{t('render.state.running')}
					</Button>
				{/if}
			</li>
		{/each}
	</ul>
</section>
