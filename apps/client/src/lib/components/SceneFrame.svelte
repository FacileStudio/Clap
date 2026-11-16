<script lang="ts">
	import { untrack } from 'svelte';
	import { stageBoxes, stageMessage, type StageBox, type StageBoxes, type StageInfo, type StageState } from '$lib/project/stage';

	type Layer = { key: number; src: string; phase: 'loading' | 'seeking' | 'shown'; painted: boolean; drawn: StageState | null };

	const PATIENCE_MS = 2500;

	let {
		src,
		title,
		time = 0,
		playing = false,
		lazy = false,
		ghost = null,
		onready,
		onstate,
		onboxes
	}: {
		/** The URL of a compiled document. A new URL loads behind the shown one and replaces it once drawn. */
		src: string;
		title: string;
		/** The project time to draw, in seconds. */
		time?: number;
		/** Whether the clock runs: the document then lets its videos and sounds play between two seeks. */
		playing?: boolean;
		lazy?: boolean;
		/** A clip drawn at a temporary box while it is dragged; null draws every clip where its source puts it. */
		ghost?: Omit<StageBox, 'kind'> | null;
		onready?: (info: StageInfo) => void;
		onstate?: (state: StageState) => void;
		/** The boxes of the clips shown, in stage pixels, after every seek of the shown document, and its URL. */
		onboxes?: (boxes: StageBoxes, src: string) => void;
	} = $props();

	let layers = $state<Layer[]>([]);
	let serial = 0;
	const frames = new Map<number, HTMLIFrameElement>();

	$effect(() => {
		const next = src;
		untrack(() => {
			const shown = layers.filter((layer) => layer.phase === 'shown').slice(-1);
			layers = [...shown, { key: (serial += 1), src: next, phase: 'loading', painted: shown.length === 0, drawn: null }];
		});
	});

	$effect(() => {
		const t = time;
		for (const layer of layers) {
			if (layer.phase !== 'loading') frames.get(layer.key)?.contentWindow?.postMessage({ type: 'scene:seek', t }, '*');
		}
	});

	$effect(() => {
		const message = { type: 'clap:playing', playing };
		for (const layer of layers) {
			if (layer.phase !== 'loading') frames.get(layer.key)?.contentWindow?.postMessage(message, '*');
		}
	});

	$effect(() => {
		const message = ghost ? { type: 'clap:ghost', ...ghost } : { type: 'clap:ghost' };
		for (const layer of layers) {
			if (layer.phase !== 'loading') frames.get(layer.key)?.contentWindow?.postMessage(message, '*');
		}
	});

	function register(node: HTMLIFrameElement, key: number) {
		frames.set(key, node);
		return { destroy: () => frames.delete(key) };
	}

	function promote(layer: Layer) {
		if (!layers.includes(layer) || layer.phase === 'loading' || !layer.painted || !layer.drawn) return;
		layer.phase = 'shown';
		if (layers.length > 1) layers = layers.filter((known) => known.key >= layer.key);
		onstate?.(layer.drawn);
	}

	function painted(layer: Layer) {
		layer.painted = true;
		promote(layer);
	}

	function onmessage(event: MessageEvent) {
		const layer = layers.find((known) => frames.get(known.key)?.contentWindow === event.source);
		const boxes = layer?.phase === 'shown' ? stageBoxes(event.data) : null;
		if (boxes && layer) onboxes?.(boxes, layer.src);
		if (layer && (event.data as { type?: unknown } | null)?.type === 'clap:painted') return painted(layer);
		const message = layer ? stageMessage(event.data) : null;
		if (!layer || !message) return;
		if (message.type === 'scene:ready') {
			layer.phase = 'seeking';
			setTimeout(() => painted(layer), PATIENCE_MS);
			onready?.(message);
			return;
		}
		if (layer.phase === 'loading') return;
		layer.drawn = message;
		promote(layer);
	}
</script>

<svelte:window {onmessage} />

<div class="relative size-full overflow-hidden bg-fc-surface">
	{#each layers as layer (layer.key)}
		<iframe
			use:register={layer.key}
			src={layer.src}
			{title}
			sandbox="allow-scripts"
			referrerpolicy="no-referrer"
			loading={lazy ? 'lazy' : 'eager'}
			tabindex="-1"
			class="pointer-events-none absolute inset-0 size-full {layer.phase === 'shown' ? '' : 'opacity-0'}"
		></iframe>
	{/each}
</div>
