<script lang="ts">
	import type { DropTarget } from '$lib/timeline/ghost';

	let { onpick }: { onpick: (files: File[], target: DropTarget) => void } = $props();

	let input = $state<HTMLInputElement>();
	let aimed: DropTarget | null = null;

	/** Asks for media files, to place at `target` once chosen. */
	export function pick(target: DropTarget): void {
		aimed = target;
		input?.click();
	}
</script>

<input
	bind:this={input}
	type="file"
	multiple
	accept="video/*,audio/*,image/*"
	class="hidden"
	data-testid="media-picker"
	onchange={(event) => {
		const files = [...(event.currentTarget.files ?? [])];
		if (files.length > 0 && aimed) onpick(files, aimed);
		event.currentTarget.value = '';
	}}
/>
