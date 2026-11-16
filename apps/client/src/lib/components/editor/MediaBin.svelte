<script lang="ts">
	import { Badge, Dropzone, Icon, IconButton, UploadProgress } from '@facile/muse';
	import { KIND_ICON, icons } from '$lib/icons';
	import { t } from '$lib/i18n';
	import { shortTime } from '$lib/project/timecode';
	import type { Uploads } from '$lib/project/uploads.svelte';
	import { kindOfAsset } from '$lib/timeline/adds';
	import { carry } from '$lib/timeline/drop';
	import { ASSET_MIME } from '$lib/timeline/layout';
	import type { Asset } from '$lib/types';

	let {
		assets,
		uploads,
		readonly = false,
		onadd,
		onremove
	}: {
		assets: Asset[];
		uploads: Uploads;
		readonly?: boolean;
		/** Places the file on the timeline at the playhead. */
		onadd: (asset: Asset) => void;
		onremove: (asset: Asset) => void;
	} = $props();

	function ondragstart(event: DragEvent, asset: Asset) {
		event.dataTransfer?.setData(ASSET_MIME, asset.name);
		event.dataTransfer?.setData('text/plain', asset.name);
		if (event.dataTransfer) event.dataTransfer.effectAllowed = 'copy';
		carry(asset);
	}

	const short = (asset: Asset) => asset.name.split('/').pop() ?? asset.name;
	const known = (asset: Asset) =>
		asset.state === 'ready' ||
		(asset.state === 'probing' && (kindOfAsset(asset) === 'image' || (uploads.measured[asset.name]?.duration ?? null) !== null));
	const placeable = (asset: Asset) => !readonly && known(asset) && kindOfAsset(asset) !== null;
</script>

<div class="flex size-full min-h-0 flex-col gap-3 overflow-auto p-3" data-testid="media-bin">
	<Dropzone
		multiple
		accept="video/*,audio/*,image/*"
		label={t('media.drop')}
		browseLabel={t('media.browse')}
		onFiles={(files) => uploads.add(files)}
	/>
	{#if uploads.items.length > 0}
		<UploadProgress items={uploads.items} onCancel={(id) => uploads.cancel(id)} aria-label={t('media.uploads')} />
	{/if}
	{#if assets.length === 0 && uploads.items.length === 0}
		<p class="px-1 text-fc-sm text-fc-fg-muted">{t('media.empty')}</p>
	{/if}
	<ul class="grid grid-cols-[repeat(auto-fill,minmax(132px,1fr))] gap-2">
		{#each assets as asset (asset.name)}
			{@const kind = kindOfAsset(asset) ?? 'html'}
			<li
				class="group relative flex flex-col gap-1 rounded-fc-md bg-fc-surface p-1.5
					{placeable(asset) ? 'cursor-grab active:cursor-grabbing' : ''}"
				draggable={placeable(asset)}
				ondragstart={(event) => ondragstart(event, asset)}
				ondragend={() => carry(null)}
				data-testid="media-item"
				data-name={asset.name}
				data-state={asset.state}
			>
				<div class="relative flex aspect-video items-center justify-center overflow-hidden rounded-fc-sm bg-fc-field">
					{#if asset.poster_url}
						<img src={asset.poster_url} alt="" class="size-full object-cover" draggable="false" loading="lazy" />
					{:else}
						<Icon icon={KIND_ICON[kind]} size={24} class="text-fc-fg-muted" />
					{/if}
					{#if asset.state !== 'ready'}
						<Badge tone={asset.state === 'failed' ? 'danger' : 'neutral'} class="absolute top-1 left-1 whitespace-nowrap">
							{asset.state === 'failed' ? t('media.unreadable') : t('media.probing')}
						</Badge>
					{:else if asset.duration_ms}
						<span class="absolute right-1 bottom-1 rounded-fc-xs bg-fc-bg/80 px-1 text-[11px] text-fc-fg tabular-nums">
							{shortTime(asset.duration_ms / 1000)}
						</span>
					{/if}
				</div>
				<div class="flex items-center gap-0.5">
					<span class="min-w-0 flex-1 truncate px-1 text-fc-xs text-fc-fg" title={asset.name}>{short(asset)}</span>
					{#if placeable(asset)}
						<IconButton
							variant="ghost"
							class="size-8"
							aria-label={t('media.add', { name: short(asset) })}
							data-testid="media-add"
							onclick={() => onadd(asset)}
						>
							<Icon icon={icons.plus} size={16} />
						</IconButton>
					{/if}
					{#if !readonly}
						<IconButton
							variant="ghost-danger"
							class="size-8"
							aria-label={t('media.delete', { name: short(asset) })}
							onclick={() => onremove(asset)}
						>
							<Icon icon={icons.remove} size={16} />
						</IconButton>
					{/if}
				</div>
			</li>
		{/each}
	</ul>
</div>
