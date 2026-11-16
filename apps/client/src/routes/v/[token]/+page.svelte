<script lang="ts">
	import { page } from '$app/state';
	import PublicPlayer from '$lib/components/PublicPlayer.svelte';
</script>

{#key page.params.token}
	<PublicPlayer token={page.params.token ?? ''} />
{/key}
