<script lang="ts">
	import { Field, Icon, Input, Popover } from '@facile/muse';
	import { icons } from '$lib/icons';
	import { t } from '$lib/i18n';
	import { SCENE_TRANSITIONS, SCENE_TRANSITION_SECONDS } from '$lib/stage/inspect';
	import { round } from '$lib/timeline/model';
	import type { Op, Scene, SceneTransition } from '$lib/types';

	let {
		scene,
		locked,
		oncommit
	}: {
		/** The scene the transition leads into. */
		scene: Scene;
		locked: boolean;
		oncommit: (ops: Op[]) => void;
	} = $props();

	let open = $state(false);
	let chip = $state<HTMLButtonElement | null>(null);

	const current = $derived(scene.transition ?? null);
	const name = $derived(current ? t(`transition.${current}`) : t('inspector.none'));
	const choices = $derived([null, ...SCENE_TRANSITIONS]);

	const pick = (transition: SceneTransition | null) => {
		oncommit([{ op: 'set_scene_transition', scene: scene.index, transition }]);
		if (transition === null) open = false;
	};
	const length = (event: Event & { currentTarget: HTMLInputElement }) => {
		const value = Number(event.currentTarget.value.replace(',', '.'));
		const typed = event.currentTarget.value.trim();
		if (!current || (typed !== '' && !(value > 0))) return;
		oncommit([{ op: 'set_scene_transition', scene: scene.index, transition: current, duration: typed ? round(value) : null }]);
	};
</script>

<button
	bind:this={chip}
	type="button"
	class="relative flex h-5 items-center gap-1 rounded-full px-1.5 text-[10px] font-medium shadow-sm
		after:absolute after:-inset-3 after:content-[''] focus-visible:outline-2 focus-visible:outline-fc-ring
		{current ? 'bg-fc-accent text-fc-accent-fg' : 'bg-fc-field text-fc-fg-muted hover:text-fc-fg'}"
	aria-label={t('transition.scene', { scene: scene.index + 1, name })}
	aria-haspopup="dialog"
	aria-expanded={open}
	disabled={locked}
	data-testid="transition-chip"
	data-transition={current ?? ''}
	onpointerdown={(event) => event.stopPropagation()}
	onclick={() => (open = !open)}
>
	<Icon icon={icons.transition} size={12} />
	{#if current}<span>{name}</span>{/if}
</button>
<Popover label={t('transition.scene', { scene: scene.index + 1, name })} anchor={chip} bind:open>
	<div class="flex flex-col gap-3" data-testid="transition-picker">
		<div class="grid grid-cols-2 gap-1.5">
			{#each choices as choice (choice ?? 'none')}
				<button
					type="button"
					class="min-h-11 rounded-fc-md px-3 text-left text-fc-sm focus-visible:outline-2 focus-visible:outline-fc-ring
						{choice === current ? 'bg-fc-accent text-fc-accent-fg' : 'bg-fc-field text-fc-fg hover:bg-fc-surface'}"
					aria-pressed={choice === current}
					data-testid="transition-choice"
					data-choice={choice ?? ''}
					onclick={() => pick(choice)}
				>
					{choice ? t(`transition.${choice}`) : t('inspector.none')}
				</button>
			{/each}
		</div>
		{#if current}
			<Field label={t('transition.length')}>
				<Input
					type="text"
					inputmode="decimal"
					value={scene.transition_duration ?? ''}
					placeholder={String(SCENE_TRANSITION_SECONDS)}
					data-testid="transition-length"
					onchange={length}
				/>
			</Field>
		{/if}
	</div>
</Popover>
