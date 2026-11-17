<script lang="ts">
	import { t } from '$lib/i18n';
	import type { Markers } from '$lib/project/markers.svelte';
	import TitleDialog from '../TitleDialog.svelte';

	let {
		markers,
		naming = $bindable(null)
	}: {
		markers: Markers;
		/** The id of the marker being named; null keeps the dialog closed. */
		naming?: string | null;
	} = $props();
</script>

<TitleDialog
	open={naming !== null}
	heading={t('menu.marker.rename')}
	label={t('marker.name')}
	value={markers.list.find((marker) => marker.id === naming)?.label ?? ''}
	confirm={t('action.save')}
	onsubmit={async (label) => {
		if (naming) await markers.rename(naming, label);
		naming = null;
	}}
	onclose={() => (naming = null)}
/>
