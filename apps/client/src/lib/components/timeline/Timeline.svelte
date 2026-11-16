<script lang="ts">
	import { tick } from 'svelte';
	import { t } from '$lib/i18n';
	import type { Transport } from '$lib/project/transport.svelte';
	import { runCommand } from '$lib/timeline/commands';
	import { drag } from '$lib/timeline/drag';
	import { textOps } from '$lib/timeline/adds';
	import { bringsFiles, broughtKind, carriedAsset, placeableFiles, trackAt } from '$lib/timeline/drop';
	import type { Engine } from '$lib/timeline/engine.svelte';
	import { aimedGhost, type DropTarget, type Ghost } from '$lib/timeline/ghost';
	import type { GestureHost } from '$lib/timeline/host';
	import { ASSET_MIME, GUTTER, HEAD, ROW, RULER, TAIL } from '$lib/timeline/layout';
	import { clamp, sceneStarts, total, trackRows } from '$lib/timeline/model';
	import { fitZoom, scrollFor, zoomBy } from '$lib/timeline/scale';
	import { SNAP_PX, snap, snapTargets } from '$lib/timeline/snap';
	import { watchViewport } from '$lib/timeline/viewport';
	import type { LocalMedia } from '$lib/project/local-media';
	import type { Asset } from '$lib/types';
	import DropGhost from './DropGhost.svelte';
	import Ruler from './Ruler.svelte';
	import SceneLane from './SceneLane.svelte';
	import TimelineToolbar from './TimelineToolbar.svelte';

	let {
		engine,
		transport,
		assets,
		readonly = false,
		ghosts = [],
		measured = {},
		ondropasset,
		ondropfiles
	}: {
		engine: Engine;
		transport: Transport;
		assets: Asset[];
		readonly?: boolean;
		/** Dropped files still uploading or waiting for their probe, drawn where they will land. */
		ghosts?: Ghost[];
		/** What the browser read of each uploaded file, by asset name, for lengths the server has not probed yet. */
		measured?: Record<string, LocalMedia>;
		ondropasset?: (asset: Asset, target: DropTarget) => void;
		ondropfiles?: (files: File[], target: DropTarget) => void;
	} = $props();

	let zoom = $state(60);
	let snapping = $state(true);
	let guide = $state<number | null>(null);
	let aimed = $state<Ghost | null>(null);
	let view = $state({ left: 0, width: 1200 });
	let viewport = $state<HTMLDivElement>();
	let track = $state<HTMLDivElement>();
	let fitted = false;
	const nodes = { clips: new Map<string, HTMLElement>(), lanes: new Map<number, HTMLElement>() };

	const rows = $derived(trackRows(engine.timeline));
	const starts = $derived(sceneStarts(engine.timeline));
	const length = $derived(total(engine.timeline));
	const clipCount = $derived(engine.timeline.scenes.reduce((sum, scene) => sum + scene.clips.length, 0));
	const media = $derived(new Map(assets.map((asset) => [asset.name, asset])));

	const host: GestureHost = {
		timeline: () => engine.timeline,
		selection: () => engine.selection,
		zoom: () => zoom,
		playhead: () => transport.time,
		snaps: (event) => snapping && !event.altKey,
		clip: (id) => nodes.clips.get(id),
		lane: (index) => nodes.lanes.get(index),
		guide: (time) => (guide = time),
		commit: (ops) => void (ops.length > 0 && engine.commit(ops))
	};

	const fit = () => fitZoom(length, (viewport?.clientWidth ?? 1200) - GUTTER - 64);

	$effect(() => {
		if (fitted || length <= 0 || !viewport) return;
		fitted = true;
		zoom = fit();
	});

	$effect(() => {
		const x = transport.time * zoom;
		if (!transport.playing || !viewport) return;
		const seen = viewport.clientWidth - GUTTER;
		if (x < viewport.scrollLeft || x > viewport.scrollLeft + seen - 32) viewport.scrollLeft = Math.max(0, x - 32);
	});

	function timeAt(event: MouseEvent): number {
		const x = event.clientX - (track?.getBoundingClientRect().left ?? 0);
		const targets = snapTargets(engine.timeline, x / zoom, new Set());
		const free = !snapping || event.altKey;
		return clamp(snap(x / zoom, targets, free ? 0 : SNAP_PX / zoom).value, 0, length);
	}

	function scrub(event: PointerEvent) {
		if (event.button !== 0) return;
		transport.pause();
		transport.seek(timeAt(event));
		drag(event, { move: (_, __, next) => transport.seek(timeAt(next)), end: () => undefined });
	}

	async function rezoom(next: number, anchor: number) {
		const left = scrollFor(anchor, viewport?.scrollLeft ?? 0, zoom, next);
		zoom = next;
		await tick();
		if (viewport) viewport.scrollLeft = left;
	}

	const watch = (node: HTMLElement) =>
		watchViewport(node, {
			onzoom: (factor, x) => void rezoom(zoomBy(zoom, factor), x - GUTTER),
			onview: (left, width) => (view = { left, width })
		});

	function ondragover(event: DragEvent) {
		const brought = bringsFiles(event.dataTransfer) ? broughtKind(event.dataTransfer) : null;
		const target = { time: timeAt(event), track: trackAt(event.clientY, track, rows) };
		const ghost = readonly ? null : aimedGhost(carriedAsset(), brought, measured, target);
		aimed = ghost;
		if (!ghost || !event.dataTransfer) return;
		event.preventDefault();
		event.dataTransfer.dropEffect = 'copy';
	}

	function ondrop(event: DragEvent) {
		aimed = null;
		const name = event.dataTransfer?.getData(ASSET_MIME);
		const asset = name ? assets.find((known) => known.name === name) : undefined;
		const files = asset ? [] : placeableFiles(event.dataTransfer);
		if (readonly || (!asset && files.length === 0) || !track) return;
		event.preventDefault();
		const target = { time: timeAt(event), track: trackAt(event.clientY, track, rows) };
		if (asset) ondropasset?.(asset, target);
		else ondropfiles?.(files, target);
	}
</script>

<section
	class="flex size-full min-h-0 flex-col"
	aria-label={t('timeline.title')}
	data-testid="timeline"
	data-busy={engine.busy}
	data-clips={clipCount}
>
	{#if !readonly}
		<TimelineToolbar
			{engine}
			bind:snapping
			oncommand={(command) => runCommand(command, engine, transport)}
			ontext={() => host.commit(textOps(engine.timeline, transport.time, t('timeline.text.default')))}
			onzoom={(factor) => void rezoom(factor ? zoomBy(zoom, factor) : fit(), factor ? (view.width - GUTTER) / 2 : 0)}
		/>
	{/if}
	<div
		bind:this={viewport}
		use:watch
		class="relative min-h-0 flex-1 overflow-auto overscroll-contain"
		role="application"
		aria-label={t('timeline.title')}
		{ondragover}
		ondragleave={(event) => void (!viewport?.contains(event.relatedTarget as Node | null) && (aimed = null))}
		{ondrop}
	>
		<div class="flex min-h-full" style="width: {GUTTER + length * zoom + TAIL}px">
			<div class="sticky left-0 z-20 shrink-0 bg-fc-component" style="width: {GUTTER}px">
				<div style="height: {RULER + HEAD}px"></div>
				{#each rows as row (row)}
					<div
						class="flex items-center justify-center text-[11px] text-fc-fg-muted tabular-nums"
						style="height: {ROW}px"
						title={t('timeline.track', { track: row })}
					>
						{row}
					</div>
				{/each}
			</div>
			<div bind:this={track} class="relative min-w-0 flex-1" style="min-height: {RULER + HEAD + rows.length * ROW}px">
				<Ruler {zoom} from={(view.left - GUTTER) / zoom} to={(view.left + view.width) / zoom} onpointerdown={scrub} />
				<div class="relative" role="presentation" onpointerdown={() => engine.select(null)}>
					{#each engine.timeline.scenes as scene (scene.index)}
						<SceneLane {scene} start={starts[scene.index] ?? 0} {zoom} {rows} {engine} {host} {media} {readonly} {nodes} />
					{/each}
				</div>
				{#if clipCount === 0 && !readonly}
					<p class="pointer-events-none absolute top-24 left-4 text-fc-sm text-fc-fg-muted">{t('timeline.empty')}</p>
				{/if}
				{#each aimed ? [...ghosts, aimed] : ghosts as ghost (ghost.id)}
					<DropGhost {ghost} timeline={engine.timeline} {rows} {zoom} />
				{/each}
				{#if guide !== null}
					<div class="pointer-events-none absolute inset-y-0 z-10 w-px bg-fc-warning" style="left: {guide * zoom}px"></div>
				{/if}
				<div
					class="pointer-events-none absolute inset-y-0 left-0 z-10 w-0.5 bg-fc-danger will-change-transform"
					style="transform: translateX({transport.time * zoom - 1}px)"
					data-testid="playhead"
					data-time={transport.time.toFixed(3)}
				>
					<span class="absolute -top-0 -left-[5px] size-3 rounded-fc-sm bg-fc-danger"></span>
				</div>
			</div>
		</div>
	</div>
</section>
