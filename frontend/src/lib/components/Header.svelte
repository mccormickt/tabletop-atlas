<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import type { Game } from '$lib';
	import HeaderSearch from './HeaderSearch.svelte';
	import { Meeple, Dice } from './icons';

	let {
		currentGame = null,
		showSearch = true
	}: {
		currentGame?: Game | null;
		showSearch?: boolean;
	} = $props();

	let currentPath = $derived(page.url.pathname);

	function isActivePath(path: string): boolean {
		return currentPath === path || currentPath.startsWith(path + '/');
	}

	function navigateHome() {
		goto('/');
	}

	const navItems = [
		{ path: '/games', label: 'Games' },
		{ path: '/upload', label: 'Upload' },
		{ path: '/search', label: 'Search' },
		{ path: '/chat', label: 'Chat' }
	];
</script>

<header class="header-board-edge relative bg-card border-b-4 border-wood-dark shadow-lg">
	<!-- Decorative corner pieces -->
	<div class="absolute top-2 left-2 w-4 h-4 border-l-3 border-t-3 border-gold-foil opacity-70 hidden lg:block"></div>
	<div class="absolute top-2 right-2 w-4 h-4 border-r-3 border-t-3 border-gold-foil opacity-70 hidden lg:block"></div>

	<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
		<div class="flex items-center justify-between py-4">
			<!-- Logo with game-piece styling -->
			<div class="flex items-center gap-3">
				<button
					onclick={navigateHome}
					class="group flex items-center gap-3 text-foreground hover:text-primary transition-all duration-200"
				>
					<!-- Compass/Map logo with game pieces -->
					<div class="relative">
						<div class="w-10 h-10 rounded-lg bg-game-blue flex items-center justify-center shadow-md group-hover:scale-105 transition-transform">
							<Meeple size={24} color="current" class="text-white" />
						</div>
						<div class="absolute -top-1 -right-1 w-4 h-4 rounded-full bg-game-orange flex items-center justify-center shadow-sm">
							<Dice size={12} value={6} class="text-foreground" />
						</div>
					</div>
					<div class="flex flex-col">
						<span class="font-display text-xl font-bold tracking-wide leading-none">
							Tabletop Atlas
						</span>
						<span class="text-xs text-muted-foreground font-ui hidden sm:block">
							Your Game Library
						</span>
					</div>
				</button>
			</div>

			<!-- Center - Search (when available and appropriate) -->
			{#if showSearch && currentGame}
				<div class="mx-8 hidden max-w-lg flex-1 lg:flex">
					<HeaderSearch {currentGame} showSearchButton={false} />
				</div>
			{/if}

			<!-- Navigation -->
			<nav class="flex items-center gap-2">
				<!-- Main Navigation Links (desktop) -->
				<div class="hidden items-center md:flex">
					{#each navItems as item, i}
						{#if i > 0}
							<!-- Dice dot divider -->
							<span class="mx-2 text-gold-foil opacity-60">◆</span>
						{/if}
						<a
							href={item.path}
							class="nav-link relative px-3 py-2 font-display text-sm font-medium tracking-wide transition-all duration-200
								{isActivePath(item.path)
									? 'text-game-blue'
									: 'text-muted-foreground hover:text-foreground'}"
						>
							{item.label}
							{#if isActivePath(item.path)}
								<span class="absolute bottom-0 left-1/2 -translate-x-1/2 w-6 h-0.5 bg-game-blue rounded-full"></span>
							{/if}
						</a>
					{/each}
				</div>

				<!-- Search Button/Controls -->
				{#if showSearch}
					<div class="flex items-center ml-4">
						<HeaderSearch {currentGame} showQuickSearch={false} showSearchButton={true} />
					</div>
				{/if}
			</nav>
		</div>
	</div>

	<!-- Bottom decorative border pattern -->
	<div class="absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r from-transparent via-gold-foil/30 to-transparent"></div>
</header>

<style>
	.border-l-3 {
		border-left-width: 3px;
	}
	.border-t-3 {
		border-top-width: 3px;
	}
	.border-r-3 {
		border-right-width: 3px;
	}

	.nav-link::before {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: 0.375rem;
		opacity: 0;
		background: var(--parchment-dark);
		transition: opacity 0.2s;
		z-index: -1;
	}

	.nav-link:hover::before {
		opacity: 1;
	}
</style>
