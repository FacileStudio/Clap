<script lang="ts">
	import { Alert } from '@facile/muse';
	import { t } from '$lib/i18n';

	let { message }: { message: string } = $props();
</script>

<Alert tone="warning" title={t('problem.title')} data-testid="project-problem">
	{t('problem.body')}
	<details class="mt-2 text-fc-sm">
		<summary class="cursor-pointer rounded-fc-sm focus-visible:outline-2 focus-visible:outline-fc-ring">{t('problem.detail')}</summary>
		<p class="mt-1 whitespace-pre-line">{message}</p>
	</details>
</Alert>
