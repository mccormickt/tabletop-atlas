<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import HeaderSearch from './HeaderSearch.svelte';
	import UserMenu from './UserMenu.svelte';
	import { Meeple, Dice } from './icons';
	import type { UserInfo } from '$lib/stores/auth';

	let {
		user = null,
		isAuthLoading = false
	}: {
		user?: UserInfo | null;
		isAuthLoading?: boolean;
	} = $props();

	let currentPath = $derived(page.url.pathname);

	function isActivePath(path: string): boolean {
		return currentPath === path || currentPath.startsWith(path + '/');
	}

	function navigateHome() {
		goto(resolve('/'));
	}

	const navItems = [
		{ path: '/games', label: 'Games' },
		{ path: '/challenges', label: 'Challenges' },
		{ path: '/tools', label: 'Tools' },
		{ path: '/search', label: 'Search' },
		{ path: '/chat', label: 'Chat' }
	];
</script>

<header class="header-board-edge bg-card border-wood-dark relative border-b-4 shadow-lg">
	<!-- Decorative corner pieces -->
	<div
		class="border-gold-foil absolute top-2 left-2 hidden h-4 w-4 border-t-3 border-l-3 opacity-70 lg:block"
	></div>
	<div
		class="border-gold-foil absolute top-2 right-2 hidden h-4 w-4 border-t-3 border-r-3 opacity-70 lg:block"
	></div>

	<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
		<div class="flex items-center justify-between py-4">
			<!-- Logo with game-piece styling -->
			<div class="flex items-center gap-3">
				<button
					onclick={navigateHome}
					class="group text-foreground hover:text-primary flex items-center gap-3 transition-all duration-200"
				>
					<!-- Compass/Map logo with game pieces -->
					<div class="relative">
						<div
							class="bg-game-blue flex h-10 w-10 items-center justify-center rounded-lg shadow-md transition-transform group-hover:scale-105"
						>
							<Meeple size={24} color="current" class="text-white" />
						</div>
						<div
							class="bg-game-orange absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full shadow-sm"
						>
							<Dice size={12} value={6} class="text-foreground" />
						</div>
					</div>
					<div class="flex flex-col">
						<span class="font-display text-xl leading-none font-bold tracking-wide">
							Tabletop Atlas
						</span>
						<span class="text-muted-foreground font-ui hidden text-xs sm:block">
							Your Game Library
						</span>
					</div>
				</button>
			</div>

			<!-- Navigation -->
			<nav class="flex items-center gap-2">
				<!-- Main Navigation Links (desktop) -->
				<div class="hidden items-center md:flex">
					{#each navItems as item, i (item.path)}
						{#if i > 0}
							<!-- Dice dot divider -->
							<span class="text-gold-foil mx-2 opacity-60">◆</span>
						{/if}
						<a
							href={resolve(item.path as '/')}
							class="nav-link font-display relative px-3 py-2 text-sm font-medium tracking-wide transition-all duration-200
								{isActivePath(item.path) ? 'text-game-blue' : 'text-muted-foreground hover:text-foreground'}"
						>
							{item.label}
							{#if isActivePath(item.path)}
								<span
									class="bg-game-blue absolute bottom-0 left-1/2 h-0.5 w-6 -translate-x-1/2 rounded-full"
								></span>
							{/if}
						</a>
					{/each}
				</div>

				<!-- Search Rules -->
				<div class="ml-4 hidden lg:block">
					<HeaderSearch />
				</div>

				<!-- User Menu -->
				<div class="ml-4">
					<UserMenu {user} isLoading={isAuthLoading} />
				</div>
			</nav>
		</div>
	</div>

	<!-- Bottom decorative border pattern -->
	<div
		class="via-gold-foil/30 absolute right-0 bottom-0 left-0 h-1 bg-gradient-to-r from-transparent to-transparent"
	></div>
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
