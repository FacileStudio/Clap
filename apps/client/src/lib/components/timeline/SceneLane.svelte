<script lang="ts">
	import { t } from '$lib/i18n';
	import { FPS, shortTime } from '$lib/project/timecode';
	import { clipGesture, type Grip } from '$lib/timeline/clip-gesture';
	import { drag } from '$lib/timeline/drag';
	import type { Engine } from '$lib/timeline/engine.svelte';
	import type { GestureHost } from '$lib/timeline/host';
	import { HEAD, ROW } from '$lib/timeline/layout';
	import { spanOf } from '$lib/timeline/link';
	import { round } from '$lib/timeline/model';
	import { nudgeOps } from '$lib/timeline/nudge';
	import { reorderGesture, resizeGesture } from '$lib/timeline/scene-gesture';
	import type { Clip, Scene } from '$lib/types';
	import ClipBlock from './ClipBlock.svelte';

	let {
		scene,
		start,
		zoom,
		rows,
		engine,
		host,
		posters,
		readonly,
		nodes
	}: {
		scene: Scene;
		/** The project time the scene starts at. */
		start: number;
		zoom: number;
		/** The track of each row, top first. */
		rows: number[];
		engine: Engine;
		host: GestureHost;
		posters: ReadonlyMap<string, string>;
		readonly: boolean;
		/** Where the drawn clips and scenes are kept for the gestures. */
		nodes: { clips: Map<string, HTMLElement>; lanes: Map<number, HTMLElement> };
	} = $props();

	const count = $derived(engine.timeline.scenes.length);
	const locked = $derived(readonly || engine.busy);

	function registerClip(node: HTMLElement, id: string) {
		nodes.clips.set(id, node);
		return { destroy: () => void (nodes.clips.get(id) === node && nodes.clips.delete(id)) };
	}

	function registerLane(node: HTMLElement, index: number) {
		nodes.lanes.set(index, node);
		return { destroy: () => void (nodes.lanes.get(index) === node && nodes.lanes.delete(index)) };
	}

	function label(clip: Clip): string {
		const found = spanOf(engine.timeline, engine.spans, clip.id)?.label;
		return found || clip.src?.split('/').pop() || t(`kind.${clip.kind}`);
	}

	function grab(event: PointerEvent, clip: Clip, grip: Grip) {
		if (event.shiftKey) engine.select(clip.id, true);
		else if (!engine.selection.has(clip.id)) engine.select(clip.id);
		const gesture = locked ? null : clipGesture(host, clip.id, grip);
		if (gesture) drag(event, gesture);
	}

	function clipKey(event: KeyboardEvent, clip: Clip) {
		if (event.key === 'Enter') return engine.select(clip.id, event.shiftKey);
		const ops = locked ? null : nudgeOps(clip, scene, event);
		if (!ops) return;
		event.preventDefault();
		event.stopPropagation();
		host.commit(ops);
	}

	function sceneKey(event: KeyboardEvent) {
		const side = { ArrowLeft: -1, ArrowRight: 1 }[event.key];
		const to = scene.index + (side ?? 0);
		if (locked || !side || !event.altKey || to < 0 || to >= count) return;
		event.preventDefault();
		event.stopPropagation();
		host.commit([{ op: 'reorder_scene', scene: scene.index, to }]);
	}

	function edgeKey(event: KeyboardEvent) {
		const side = { ArrowLeft: -1, ArrowRight: 1 }[event.key];
		if (locked || !side) return;
		event.preventDefault();
		event.stopPropagation();
		const duration = round(Math.max(0.1, scene.duration + (event.shiftKey ? side : side / FPS)));
		host.commit([{ op: 'set_scene_duration', scene: scene.index, duration }]);
	}
</script>

<div
	use:registerLane={scene.index}
	class="lane absolute"
	style="--x: {start * zoom}px; --w: {scene.duration * zoom}px; height: {HEAD + rows.length * ROW}px"
	data-testid="scene"
	data-scene={scene.index}
	data-duration={scene.duration}
>
	<div class="absolute inset-y-0 right-px left-px overflow-hidden rounded-fc-md bg-fc-component">
		<div
			class="flex touch-none items-center gap-2 bg-fc-surface px-2.5 text-fc-xs whitespace-nowrap text-fc-fg-muted select-none
				focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-fc-ring
				{locked ? '' : 'cursor-grab active:cursor-grabbing'}"
			style="height: {HEAD}px"
			role="button"
			tabindex="0"
			aria-label={t('timeline.scene.move', { scene: scene.index + 1, duration: shortTime(scene.duration) })}
			data-testid="scene-head"
			onpointerdown={(event) => !locked && event.button === 0 && drag(event, reorderGesture(host, scene.index))}
			onkeydown={sceneKey}
		>
			<span class="font-semibold text-fc-fg">{t('timeline.scene', { scene: scene.index + 1 })}</span>
			<span class="tabular-nums">{shortTime(scene.duration)}</span>
		</div>
		<div class="relative" style="height: {rows.length * ROW}px">
			{#each rows as track, row (track)}
				<div
					class="absolute inset-x-0 {row % 2 === 0 ? 'bg-fc-surface/30' : ''}"
					style="top: {row * ROW}px; height: {ROW}px"
				></div>
			{/each}
			{#each scene.clips as clip (clip.id)}
				<ClipBlock
					{clip}
					row={Math.max(0, rows.indexOf(clip.track))}
					{zoom}
					label={label(clip)}
					poster={posters.get(clip.src ?? '') ?? null}
					selected={engine.selection.has(clip.id)}
					{readonly}
					register={registerClip}
					ongrab={(event, grip) => grab(event, clip, grip)}
					onkeydown={(event) => clipKey(event, clip)}
				/>
			{/each}
		</div>
	</div>
	{#if !readonly}
		<div
			class="absolute inset-y-0 right-0 z-10 w-2 cursor-ew-resize touch-none rounded-fc-sm hover:bg-fc-fg/40
				focus-visible:bg-fc-fg/40 focus-visible:outline-2 focus-visible:outline-fc-ring"
			role="slider"
			tabindex="0"
			aria-label={t('timeline.scene.resize', { scene: scene.index + 1 })}
			aria-valuenow={scene.duration}
			aria-valuetext={shortTime(scene.duration)}
			data-testid="scene-edge"
			onpointerdown={(event) => {
				event.stopPropagation();
				if (!locked && event.button === 0) drag(event, resizeGesture(host, scene.index));
			}}
			onkeydown={edgeKey}
		></div>
	{/if}
</div>

<style>
	.lane {
		top: 0;
		left: calc(var(--x) + var(--dx, 0px));
		width: calc(var(--w) + var(--dw, 0px));
	}
</style>
