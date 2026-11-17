<script lang="ts">
	import { untrack } from 'svelte';
	import { ContextMenu } from '@facile/muse';
	import { t } from '$lib/i18n';
	import type { LocalMedia } from '$lib/project/local-media';
	import type { StageBox } from '$lib/project/stage';
	import { StageEditor } from '$lib/stage/editor.svelte';
	import type { Box, Point, Stage } from '$lib/stage/geometry';
	import { CORNERS, SIDES, type Handle } from '$lib/stage/gesture';
	import { stageMenuAt } from '$lib/stage/menu';
	import { dropBox } from '$lib/stage/place';
	import { kindOfAsset } from '$lib/timeline/adds';
	import type { MenuHost } from '$lib/timeline/clip-menu';
	import { bringsFiles, broughtKind, carriedAsset, placeableFiles } from '$lib/timeline/drop';
	import type { Engine } from '$lib/timeline/engine.svelte';
	import { ASSET_MIME } from '$lib/timeline/layout';
	import type { Asset } from '$lib/types';

	let {
		boxes,
		stage,
		engine,
		assets,
		measured,
		readonly,
		actions,
		onghost,
		onlanded,
		ondropasset,
		ondropfiles
	}: {
		/** The clip boxes the preview drew last, in frame pixels. */
		boxes: StageBox[];
		stage: Stage;
		engine: Engine;
		assets: Asset[];
		measured: Record<string, LocalMedia>;
		readonly: boolean;
		/** What the right-click menu acts through. */
		actions: MenuHost;
		onghost: (ghost: (Box & { id: string }) | null) => void;
		onlanded: (sent: boolean) => void;
		ondropasset: (asset: Asset, at: Point) => void;
		ondropfiles: (files: File[], at: Point) => void;
	} = $props();

	let root = $state<HTMLDivElement>();
	let aimed = $state<Box | null>(null);

	const scale = () => (root?.getBoundingClientRect().width ?? stage.width) / stage.width;
	const point = (event: { clientX: number; clientY: number }): Point => {
		const rect = root?.getBoundingClientRect();
		return { x: (event.clientX - (rect?.left ?? 0)) / scale(), y: (event.clientY - (rect?.top ?? 0)) / scale() };
	};
	const editor = new StageEditor({
		engine: untrack(() => engine),
		stage: () => stage,
		boxes: () => boxes,
		point,
		scale,
		locked: () => readonly || engine.busy,
		ghost: (ghost) => onghost(ghost),
		landed: (sent) => onlanded(sent)
	});

	const tracks = $derived(new Map(engine.timeline.scenes.flatMap((scene) => scene.clips.map((clip) => [clip.id, clip.track]))));
	const order = (box: StageBox) => tracks.get(box.id) ?? 0;
	const drawn = $derived([...editor.drawn].sort((a, b) => order(a) - order(b)));
	const chosen = $derived(engine.selection.size === 1 ? drawn.find((box) => engine.selection.has(box.id)) : undefined);
	const pct = (part: number, whole: number) => `${(100 * part) / whole}%`;
	const place = (box: Box) =>
		`left: ${pct(box.x, stage.width)}; top: ${pct(box.y, stage.height)}; width: ${pct(box.w, stage.width)}; ` +
		`height: ${pct(box.h, stage.height)}; transform: rotate(${box.rot}deg)`;
	const SPOT: Record<Handle, string> = {
		nw: 'left: 0; top: 0',
		n: 'left: 12px; right: 12px; top: -6px; height: 12px',
		ne: 'left: 100%; top: 0',
		e: 'top: 12px; bottom: 12px; right: -6px; width: 12px',
		se: 'left: 100%; top: 100%',
		s: 'left: 12px; right: 12px; bottom: -6px; height: 12px',
		sw: 'left: 0; top: 100%',
		w: 'top: 12px; bottom: 12px; left: -6px; width: 12px'
	};
	const CURSOR: Record<Handle, string> = {
		nw: 'nwse-resize',
		se: 'nwse-resize',
		ne: 'nesw-resize',
		sw: 'nesw-resize',
		n: 'ns-resize',
		s: 'ns-resize',
		e: 'ew-resize',
		w: 'ew-resize'
	};

	function onkeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') return engine.select(null);
		if (!editor.key(event)) return;
		event.preventDefault();
		event.stopPropagation();
	}

	function ondragover(event: DragEvent) {
		const asset = carriedAsset();
		const kind = asset ? kindOfAsset(asset) : bringsFiles(event.dataTransfer) ? broughtKind(event.dataTransfer) : null;
		if (readonly || !kind || !event.dataTransfer) return void (aimed = null);
		event.preventDefault();
		event.dataTransfer.dropEffect = 'copy';
		const local = asset ? measured[asset.name] : undefined;
		const size = asset ? { width: asset.width ?? local?.width ?? null, height: asset.height ?? local?.height ?? null } : null;
		aimed = kind === 'audio' ? null : dropBox(stage, point(event), size);
	}

	function ondrop(event: DragEvent) {
		aimed = null;
		const name = event.dataTransfer?.getData(ASSET_MIME);
		const asset = name ? assets.find((known) => known.name === name) : undefined;
		const files = asset ? [] : placeableFiles(event.dataTransfer);
		if (readonly || (!asset && files.length === 0)) return;
		event.preventDefault();
		if (asset) ondropasset(asset, point(event));
		else ondropfiles(files, point(event));
	}
</script>

<ContextMenu
	items={(target) => (readonly ? [] : stageMenuAt(actions, target, drawn, chosen?.id ?? null, stage))}
	label={t('menu.stage')}
	disabled={readonly}
	class="absolute inset-0"
>
	<div
		bind:this={root}
		class="absolute inset-0 touch-none outline-none focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-fc-ring"
		role="listbox"
		tabindex="0"
		aria-label={t('stage.label')}
		aria-multiselectable="true"
		data-testid="stage-overlay"
		data-width={stage.width}
		data-height={stage.height}
		onpointerdown={(event) => {
			root?.focus();
			if (event.button === 0 && !event.shiftKey) engine.select(null);
		}}
		{onkeydown}
		onkeyup={(event) => editor.keyup(event)}
		{ondragover}
		ondragleave={(event) => void (!root?.contains(event.relatedTarget as Node | null) && (aimed = null))}
		{ondrop}
	>
		{#each drawn as box (box.id)}
			<div
				class="absolute cursor-move hover:outline-[1.5px] hover:outline-fc-info {readonly ? 'cursor-default' : ''}
					{engine.selection.has(box.id) ? 'outline-[1.5px] outline-fc-info' : ''}"
				style={place(box)}
				role="option"
				tabindex="-1"
				aria-label={t('stage.clip', { kind: t(`kind.${box.kind}`), id: box.id })}
				aria-selected={engine.selection.has(box.id)}
				data-testid="stage-box"
				data-clip={box.id}
				data-x={Math.round(box.x)}
				data-y={Math.round(box.y)}
				data-w={Math.round(box.w)}
				data-h={Math.round(box.h)}
				data-rot={box.rot}
				onpointerdown={(event) => {
					root?.focus();
					editor.grab(event, box);
				}}
			></div>
		{/each}
		{#each editor.guides.x as x (x)}
			<div class="pointer-events-none absolute inset-y-0 w-px bg-fc-danger" style="left: {pct(x, stage.width)}" data-testid="stage-guide"></div>
		{/each}
		{#each editor.guides.y as y (y)}
			<div class="pointer-events-none absolute inset-x-0 h-px bg-fc-danger" style="top: {pct(y, stage.height)}" data-testid="stage-guide"></div>
		{/each}
		{#if chosen}
			<div class="pointer-events-none absolute outline-[1.5px] outline-fc-info" style={place(chosen)} data-testid="stage-selection">
				{#if !readonly}
					{#each SIDES as handle (handle)}
						<span
							class="pointer-events-auto absolute touch-none"
							style="{SPOT[handle]}; cursor: {CURSOR[handle]}"
							aria-hidden="true"
							data-testid="stage-handle"
							data-handle={handle}
							onpointerdown={(event) => editor.resize(event, chosen, handle)}
						></span>
					{/each}
					{#each CORNERS as handle (handle)}
						<span
							class="knob pointer-events-auto absolute flex -translate-x-1/2 -translate-y-1/2 touch-none items-center justify-center"
							style="{SPOT[handle]}; cursor: {CURSOR[handle]}"
							aria-hidden="true"
							data-testid="stage-handle"
							data-handle={handle}
							onpointerdown={(event) => editor.resize(event, chosen, handle)}
						>
							<span class="size-3 rounded-[3px] bg-fc-info shadow-sm"></span>
						</span>
					{/each}
					<span
						class="knob pointer-events-auto absolute left-1/2 flex -translate-x-1/2 cursor-grab touch-none items-center justify-center active:cursor-grabbing"
						style={chosen.y < stage.height * 0.1 ? 'top: 24px' : 'top: -36px'}
						aria-hidden="true"
						data-testid="stage-rotate"
						onpointerdown={(event) => editor.rotate(event, chosen)}
					>
						<span class="size-3 rounded-fc-pill bg-fc-info shadow-sm"></span>
					</span>
				{/if}
			</div>
		{/if}
		{#if aimed}
			<div class="pointer-events-none absolute rounded-fc-sm bg-fc-info/20 outline-2 outline-fc-info outline-dashed" style={place(aimed)} data-testid="stage-aim"></div>
		{/if}
	</div>
</ContextMenu>

<style>
	.knob {
		width: 24px;
		height: 24px;
	}

	@media (pointer: coarse) {
		.knob {
			width: 44px;
			height: 44px;
		}
	}
</style>
