<script lang="ts">
	import { Kbd, Modal } from '@facile/muse';
	import { t } from '$lib/i18n';
	import type { AppKey } from '$lib/i18n/messages';
	import { SHORTCUTS, type Command } from '$lib/timeline/keys';

	let { open = $bindable(false) }: { open?: boolean } = $props();

	const GROUPS: { title: AppKey; commands: Command[] }[] = [
		{
			title: 'help.playback',
			commands: ['toggle', 'back', 'pause', 'forward', 'prevFrame', 'nextFrame', 'prevSecond', 'nextSecond', 'home', 'end']
		},
		{ title: 'help.editing', commands: ['split', 'copy', 'paste', 'duplicate', 'delete', 'ripple', 'undo', 'redo', 'help'] }
	];
</script>

<Modal bind:open title={t('help.open')} size="md" showClose>
	<div class="flex flex-col gap-5" data-testid="shortcuts-dialog">
		{#each GROUPS as group (group.title)}
			<section class="flex flex-col gap-1">
				<h3 class="text-fc-sm font-semibold text-fc-fg">{t(group.title)}</h3>
				<dl class="flex flex-col">
					{#each group.commands as command (command)}
						<div class="flex min-h-9 items-center justify-between gap-4 rounded-fc-sm px-2 even:bg-fc-surface/50">
							<dt class="text-fc-sm text-fc-fg">{t(`command.${command}`)}</dt>
							<dd><Kbd keys={SHORTCUTS[command]} /></dd>
						</div>
					{/each}
				</dl>
			</section>
		{/each}
		<p class="text-fc-sm text-fc-fg-muted">{t('help.menus')}</p>
	</div>
</Modal>
