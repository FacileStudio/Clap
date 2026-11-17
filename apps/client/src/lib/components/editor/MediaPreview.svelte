<script lang="ts">
	import { Modal } from '@facile/muse';
	import { kindOfAsset } from '$lib/timeline/adds';
	import type { Asset } from '$lib/types';

	let { asset = $bindable(null) }: { asset: Asset | null } = $props();

	const kind = $derived(asset ? kindOfAsset(asset) : null);
	const name = $derived(asset?.name.split('/').pop() ?? '');
</script>

<Modal open={asset !== null} title={name} size="lg" showClose onClose={() => (asset = null)}>
	{#if asset}
		<div class="flex max-h-[70dvh] items-center justify-center overflow-hidden rounded-fc-md bg-fc-bg" data-testid="media-preview">
			{#if kind === 'video'}
				<video src={asset.url} poster={asset.poster_url ?? undefined} controls autoplay class="max-h-[70dvh] max-w-full">
					<track kind="captions" />
				</video>
			{:else if kind === 'audio'}
				<audio src={asset.url} controls autoplay class="w-full p-6"></audio>
			{:else}
				<img src={asset.url} alt={name} class="max-h-[70dvh] max-w-full object-contain" />
			{/if}
		</div>
	{/if}
</Modal>
