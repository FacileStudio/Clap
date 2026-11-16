<script lang="ts">
	import { untrack } from 'svelte';
	import { Alert, Button, ConfirmModal, CopyButton, Field, Input, Modal, Spinner, toast } from '@facile/muse';
	import { formatMoment, t } from '$lib/i18n';
	import { Sharing, embedSnippet } from '$lib/project/sharing.svelte';

	let {
		open = $bindable(),
		id,
		title,
		width,
		height,
		onrender
	}: {
		open: boolean;
		id: string;
		title: string;
		/** The stage size, for the embed snippet's ratio. */
		width: number;
		height: number;
		/** Opens the render dialog: a link shows only a finished render. */
		onrender: () => void;
	} = $props();

	const sharing = new Sharing(untrack(() => id));

	let confirming = $state<'rotate' | 'revoke' | null>(null);

	const embed = $derived(sharing.url ? embedSnippet(sharing.url, title, width, height) : '');
	const failed = (error: unknown) => toast.danger(error instanceof Error ? error.message : t('error.generic'));

	$effect(() => {
		if (open) void sharing.load();
		else sharing.forget();
	});

	async function confirm() {
		const action = confirming;
		confirming = null;
		if (action === 'rotate') await sharing.create(true).catch(failed);
		if (action === 'revoke') await sharing.revoke().catch(failed);
	}

	function render() {
		open = false;
		onrender();
	}
</script>

{#snippet copyable(label: string, value: string, copy: string, testid: string, helper?: string)}
	<Field {label} {helper}>
		{#snippet children({ id: field })}
			<span class="flex items-center gap-2">
				<Input id={field} {value} readonly class="font-fc-mono text-fc-sm" data-testid={testid} />
				<CopyButton text={value} label={copy} variant="secondary" />
			</span>
		{/snippet}
	</Field>
{/snippet}

<Modal {open} title={t('share.title')} size="md" onClose={() => (open = false)}>
	<div
		class="flex flex-col gap-4"
		data-testid="share-dialog"
		data-status={sharing.status}
		data-rendered={sharing.rendered ?? 'unknown'}
	>
		{#if sharing.status === 'loading'}
			<div class="flex justify-center py-6"><Spinner /></div>
		{:else if sharing.status === 'failed'}
			<Alert tone="danger">{t('share.failed')}</Alert>
		{:else}
			{#if sharing.rendered === false}
				<Alert tone="info" data-testid="share-norender">
					{t('share.norender')}
					{#snippet action()}
						<Button size="sm" variant="secondary" onclick={render}>{t('share.render')}</Button>
					{/snippet}
				</Alert>
			{/if}
			{#if sharing.status === 'off'}
				<p class="text-fc-sm text-fc-fg-muted">{t('share.off')}</p>
				<Button disabled={sharing.busy} onclick={() => sharing.create().catch(failed)} data-testid="share-create">
					{t('share.create')}
				</Button>
			{:else}
				{#if sharing.url}
					<Alert tone="warning" title={t('share.once')}>{t('share.once.hint')}</Alert>
					{@render copyable(t('share.link'), sharing.url, t('share.copy'), 'share-url')}
					{@render copyable(t('share.embed'), embed, t('share.embed.copy'), 'share-embed', t('share.embed.hint'))}
				{:else}
					<p class="text-fc-sm text-fc-fg" data-testid="share-live">
						{t('share.on', { moment: formatMoment(sharing.createdAt ?? '') })}
					</p>
					<p class="text-fc-sm text-fc-fg-muted">{t('share.hidden')}</p>
				{/if}
				<div class="flex flex-wrap gap-2">
					<Button variant="secondary" disabled={sharing.busy} onclick={() => (confirming = 'rotate')} data-testid="share-rotate">
						{t('share.rotate')}
					</Button>
					<Button
						variant="ghost-danger"
						disabled={sharing.busy}
						onclick={() => (confirming = 'revoke')}
						data-testid="share-revoke"
					>
						{t('share.revoke')}
					</Button>
				</div>
			{/if}
		{/if}
	</div>
</Modal>

<ConfirmModal
	open={confirming !== null}
	tone="danger"
	title={t(confirming === 'revoke' ? 'share.revoke.title' : 'share.rotate.title')}
	description={t(confirming === 'revoke' ? 'share.revoke.hint' : 'share.rotate.hint')}
	confirmLabel={t(confirming === 'revoke' ? 'share.revoke' : 'share.rotate')}
	cancelLabel={t('action.cancel')}
	onConfirm={confirm}
	onCancel={() => (confirming = null)}
/>
