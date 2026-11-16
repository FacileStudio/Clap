<script lang="ts">
	let { class: className = '' }: { class?: string } = $props();
</script>

<!--
@component
The stand-in for a holiday clip: a flat beach in muse's chart tints, so a frame of footage is drawn without shipping a video.
-->

<span class="footage {className}" aria-hidden="true">
	<span class="sun"></span>
	<span class="sea"></span>
	<span class="sand"></span>
</span>

<style>
	.footage {
		position: absolute;
		inset: 0;
		display: block;
		overflow: hidden;
		background: color-mix(in oklab, var(--color-fc-chart-3) 55%, var(--color-fc-on-inverse));
	}
	.sun {
		position: absolute;
		top: 16%;
		left: 16%;
		width: 16%;
		aspect-ratio: 1;
		border-radius: 999px;
		background: color-mix(in oklab, var(--color-fc-chart-2) 70%, var(--color-fc-on-inverse));
	}
	.sea {
		position: absolute;
		inset: 52% 0 0;
		background: color-mix(in oklab, var(--color-fc-info) 70%, var(--color-fc-chart-3));
	}
	.sand {
		position: absolute;
		inset: 80% -20% -40% -20%;
		border-radius: 50% 50% 0 0;
		background: color-mix(in oklab, var(--color-fc-chart-2) 55%, var(--color-fc-on-inverse));
	}
</style>
