<script lang="ts">
	import { Button, EmptyState, Icon, Spinner, icons } from '@facile/muse';
	import { logoIcon } from '$lib/brand';
	import { t } from '$lib/i18n';
	import { shortTime } from '$lib/project/timecode';
	import { Watching } from '$lib/project/watching.svelte';

	let {
		token,
		embedded = false
	}: {
		token: string;
		/** Inside another site's frame: the player alone, and the only way out is the watch page in a new tab. */
		embedded?: boolean;
	} = $props();

	const watching = new Watching();

	let player = $state<HTMLVideoElement>();
	let playing = $state(false);

	$effect(() => void watching.load(token));

	const video = $derived(watching.video);

	function onkeydown(event: KeyboardEvent) {
		if (event.key !== ' ' || event.metaKey || event.ctrlKey || event.altKey || !player) return;
		const target = event.target as HTMLElement | null;
		if (target === player || target?.closest('button, a, input, textarea')) return;
		event.preventDefault();
		if (player.paused) void player.play().catch(() => undefined);
		else player.pause();
	}
</script>

<svelte:head>
	<title>{video?.title ?? 'Clap'} · Clap</title>
	<meta name="robots" content="noindex" />
</svelte:head>

<svelte:window {onkeydown} />

{#if watching.gone}
	<div class="flex h-dvh w-full items-center justify-center bg-fc-page p-4" data-testid="link-gone">
		<EmptyState bare icon={icons.eyeClosed} title={t('public.gone')} description={t('public.gone.hint')} />
	</div>
{:else if watching.failed}
	<div class="flex h-dvh w-full items-center justify-center bg-fc-page p-4" data-testid="link-failed">
		<EmptyState bare icon={icons.warning} title={t('error.generic')} description={t('error.hint')}>
			<Button variant="secondary" onclick={() => watching.load(token)}>{t('action.retry')}</Button>
		</EmptyState>
	</div>
{:else if !video}
	<div class="flex h-dvh w-full items-center justify-center bg-fc-inverse"><Spinner /></div>
{:else}
	<div
		class="relative flex h-dvh w-full flex-col bg-fc-inverse text-fc-on-inverse {embedded ? '' : 'gap-4 p-4 sm:p-6'}"
		data-testid="player"
		data-playing={playing}
	>
		{#if !embedded}
			<header class="flex shrink-0 flex-wrap items-center gap-x-4 gap-y-2">
				<div class="min-w-0 flex-1">
					<h1 class="truncate text-fc-xl font-semibold" data-testid="player-title">{video.title}</h1>
					<p class="text-fc-sm text-fc-on-inverse-muted tabular-nums" data-testid="player-duration">
						{shortTime(video.duration_ms / 1000)}
					</p>
				</div>
				<a
					class="flex min-h-11 items-center gap-2 rounded-fc-pill bg-fc-bg px-4 text-fc-sm text-fc-fg shadow-lg
						focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
					href="/"
					data-sveltekit-reload
					data-testid="made-with"
				>
					<Icon icon={logoIcon} size={18} />{t('public.made')}
				</a>
			</header>
		{/if}
		<div class="flex min-h-0 flex-1 items-center justify-center [container-type:size]">
			<video
				bind:this={player}
				class="max-h-full w-full bg-fc-inverse {embedded ? 'h-full' : 'rounded-fc-lg shadow-lg'}"
				style={embedded ? '' : `aspect-ratio: ${video.width / video.height}; max-width: calc(100cqh * ${video.width / video.height})`}
				src={video.video}
				poster={video.poster ?? undefined}
				controls
				playsinline
				preload="metadata"
				aria-label={video.title}
				data-testid="player-video"
				onplay={() => (playing = true)}
				onpause={() => (playing = false)}
			>
				<track kind="captions" />
			</video>
		</div>
		{#if embedded}
			<a
				class="absolute top-2 right-2 flex size-11 items-center justify-center rounded-fc-pill bg-fc-bg text-fc-fg shadow-lg
					focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-fc-ring"
				href="/v/{token}"
				target="_blank"
				rel="noopener"
				aria-label={t('public.open')}
				title={t('public.open')}
				data-testid="player-open"
			>
				<Icon icon={icons.external} size={18} />
			</a>
		{/if}
	</div>
{/if}
