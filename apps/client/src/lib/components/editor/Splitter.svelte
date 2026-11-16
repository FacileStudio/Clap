<script lang="ts">
	import { t } from '$lib/i18n';
	import { drag } from '$lib/timeline/drag';
	import { clamp } from '$lib/timeline/model';

	let {
		value = $bindable(),
		axis,
		min = 0.2,
		max = 0.8
	}: {
		/** The share of the parent the first panel takes, from 0 to 1. */
		value: number;
		/** `x` splits left and right, `y` splits top and bottom. */
		axis: 'x' | 'y';
		min?: number;
		max?: number;
	} = $props();

	function grab(event: PointerEvent) {
		const parent = (event.currentTarget as HTMLElement).parentElement;
		if (!parent || event.button !== 0) return;
		const rect = parent.getBoundingClientRect();
		const share = (next: PointerEvent) =>
			axis === 'x' ? (next.clientX - rect.left) / rect.width : (next.clientY - rect.top) / rect.height;
		drag(event, { move: (_, __, next) => (value = clamp(share(next), min, max)), end: () => undefined });
	}

	function onkeydown(event: KeyboardEvent) {
		const steps: Record<string, number> = { ArrowLeft: -0.02, ArrowUp: -0.02, ArrowRight: 0.02, ArrowDown: 0.02 };
		const step = steps[event.key];
		if (!step) return;
		event.preventDefault();
		event.stopPropagation();
		value = clamp(value + step, min, max);
	}
</script>

<div
	class="group flex shrink-0 touch-none items-center justify-center focus-visible:outline-2 focus-visible:outline-fc-ring
		{axis === 'x' ? 'w-3 cursor-col-resize' : 'h-3 cursor-row-resize'}"
	role="slider"
	tabindex="0"
	aria-orientation={axis === 'x' ? 'vertical' : 'horizontal'}
	aria-valuenow={Math.round(value * 100)}
	aria-valuemin={Math.round(min * 100)}
	aria-valuemax={Math.round(max * 100)}
	aria-label={t('editor.resize')}
	onpointerdown={grab}
	{onkeydown}
>
	<span
		class="rounded-fc-full bg-fc-field group-hover:bg-fc-fg-muted group-focus-visible:bg-fc-fg-muted
			{axis === 'x' ? 'h-10 w-1' : 'h-1 w-10'}"
	></span>
</div>
