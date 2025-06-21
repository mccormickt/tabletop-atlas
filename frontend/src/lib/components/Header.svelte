<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import type { Game } from '$lib';
	import HeaderSearch from './HeaderSearch.svelte';

	// Props
	let {
		currentGame = null,
		showSearch = true
	}: {
		currentGame?: Game | null;
		showSearch?: boolean;
	} = $props();

	// Derive current path for active navigation
	let currentPath = $derived($page.url.pathname);

	function isActivePath(path: string): boolean {
		return currentPath === path || currentPath.startsWith(path + '/');
	}

	function navigateHome() {
		goto('/');
	}

	function navigateToGames() {
		goto('/games');
	}

	function navigateToUpload() {
		goto('/upload');
	}

	function navigateToSearch() {
		goto('/search');
	}

	function navigateToChat() {
		// TODO: Implement chat route
		console.log('Navigate to chat');
	}
</script>

<header class="bg-card border-b shadow-sm">
	<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
		<div class="flex items-center justify-between py-6">
			<!-- Logo -->
			<div class="flex items-center">
				<button
					onclick={navigateHome}
					class="text-foreground hover:text-primary flex items-center text-2xl font-bold transition-colors"
				>
					🎲 Tabletop Atlas
				</button>
			</div>

			<!-- Center - Search (when available and appropriate) -->
			{#if showSearch && currentGame}
				<div class="mx-8 hidden max-w-lg flex-1 lg:flex">
					<HeaderSearch {currentGame} showSearchButton={false} />
				</div>
			{/if}

			<!-- Navigation -->
			<nav class="flex items-center space-x-6">
				<!-- Main Navigation Links -->
				<div class="hidden items-center space-x-6 md:flex">
					<a
						href="/games"
						class="transition-colors {isActivePath('/games')
							? 'text-foreground font-medium'
							: 'text-muted-foreground hover:text-foreground'}"
					>
						Games
					</a>
					<a
						href="/upload"
						class="transition-colors {isActivePath('/upload')
							? 'text-foreground font-medium'
							: 'text-muted-foreground hover:text-foreground'}"
					>
						Upload
					</a>
					<a
						href="/search"
						class="transition-colors {isActivePath('/search')
							? 'text-foreground font-medium'
							: 'text-muted-foreground hover:text-foreground'}"
					>
						Search
					</a>
					<button
						onclick={navigateToChat}
						class="text-muted-foreground hover:text-foreground transition-colors"
					>
						Chat
					</button>
				</div>

				<!-- Search Button/Controls -->
				{#if showSearch}
					<div class="flex items-center">
						<HeaderSearch {currentGame} showQuickSearch={false} showSearchButton={true} />
					</div>
				{/if}

				<!-- Mobile Menu Button -->
				<div class="md:hidden">
					<button
						onclick={() => {
							// TODO: Implement mobile menu
							console.log('Toggle mobile menu');
						}}
						class="text-muted-foreground hover:text-foreground p-2"
						aria-label="Toggle mobile menu"
					>
						<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M4 6h16M4 12h16M4 18h16"
							></path>
						</svg>
					</button>
				</div>
			</nav>
		</div>

		<!-- Mobile Search (when on game page) -->
		{#if showSearch && currentGame}
			<div class="pb-4 lg:hidden">
				<HeaderSearch {currentGame} showSearchButton={false} />
			</div>
		{/if}
	</div>
</header>
