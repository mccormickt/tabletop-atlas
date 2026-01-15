<script lang="ts">
	import { goto } from '$app/navigation';
	import { api, type GameSummary } from '$lib';
	import { Button, GameBox, LoadingSpinner } from '$lib/components/ui';
	import CollectionDashboard from '$lib/components/CollectionDashboard.svelte';
	import FilterPanel from '$lib/components/FilterPanel.svelte';
	import { GameBoxIcon, Rulebook } from '$lib/components/icons';
	import { useHeader } from '$lib/stores/header';

	const header = useHeader();
	header.configure({
		showSearch: true,
		currentGame: null
	});

	let games = $state<GameSummary[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let currentPage = $state(1);
	let totalPages = $state(1);
	let total = $state(0);
	let limit = $state(20);

	let showFilters = $state(false);
	let filters = $state({});

	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			loadGames(1);
		}
	});

	async function loadGames(page: number = 1) {
		loading = true;
		error = null;

		try {
			const result = await api.methods.listGames({
				query: { page, limit }
			});

			if (result.type === 'success') {
				games = result.data.items;
				currentPage = result.data.page;
				totalPages = result.data.totalPages;
				total = result.data.total;
			} else if (result.type === 'error') {
				error = result.data.message || 'Failed to load games';
				games = [];
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Failed to load games';
				games = [];
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			games = [];
		} finally {
			loading = false;
		}
	}

	async function handleDelete(game: GameSummary) {
		if (!confirm(`Are you sure you want to delete "${game.name}"? This action cannot be undone.`)) {
			return;
		}

		try {
			const result = await api.methods.deleteGame({
				path: { id: game.id }
			});

			if (result.type === 'success') {
				await loadGames(currentPage);
			} else if (result.type === 'error') {
				alert(result.data.message || 'Failed to delete game');
			} else if (result.type === 'client_error') {
				alert(result.error.message || 'Failed to delete game');
			}
		} catch (err) {
			alert(err instanceof Error ? err.message : 'An unexpected error occurred');
		}
	}

	function navigateToAddGame() {
		goto('/games/add');
	}

	function handlePageChange(page: number) {
		loadGames(page);
	}

	function toggleFilters() {
		showFilters = !showFilters;
	}
</script>

<svelte:head>
	<title>Games - Tabletop Atlas</title>
	<meta name="description" content="Browse and manage your board game collection" />
</svelte:head>

<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Page Header with Rulebook Style -->
	<div class="mb-8">
		<div class="rulebook-header">
			<h1 class="text-3xl md:text-4xl">Game Library</h1>
		</div>
		<p class="text-muted-foreground font-body text-center">
			{#if total > 0}
				{total} game{total === 1 ? '' : 's'} in your collection
			{:else}
				Your collection awaits
			{/if}
		</p>
	</div>

	<!-- Action Bar -->
	<div class="mb-6 flex flex-wrap items-center justify-between gap-4">
		<div class="flex items-center gap-2">
			<Button variant="game-secondary" size="sm" onclick={toggleFilters} class="md:hidden">
				{showFilters ? 'Hide Filters' : 'Filters'}
			</Button>
		</div>

		<!-- Desktop: Regular button -->
		<Button variant="game-primary" onclick={navigateToAddGame} class="hidden gap-2 md:flex">
			<GameBoxIcon size={18} />
			Add New Game
		</Button>
	</div>

	<!-- Mobile: Floating Action Button -->
	<button
		onclick={navigateToAddGame}
		class="bg-game-blue hover:bg-game-blue/90 fixed right-4 bottom-24 z-40 flex h-14 w-14 items-center justify-center rounded-full text-white shadow-lg transition-all active:scale-95 md:hidden"
		aria-label="Add new game"
	>
		<GameBoxIcon size={24} />
	</button>

	<div class="flex gap-6">
		<!-- Filter Sidebar (Desktop) -->
		<aside class="hidden w-64 flex-shrink-0 md:block">
			<FilterPanel bind:filters onApply={() => loadGames(1)} onClear={() => loadGames(1)} />
		</aside>

		<!-- Mobile Filter Drawer -->
		{#if showFilters}
			<div class="fixed inset-0 z-40 md:hidden">
				<div class="absolute inset-0 bg-black/50" onclick={toggleFilters}></div>
				<div
					class="bg-background absolute top-0 bottom-0 left-0 w-72 overflow-y-auto p-4 shadow-lg"
				>
					<div class="mb-4 flex items-center justify-between">
						<h2 class="font-display text-lg font-semibold">Filters</h2>
						<button onclick={toggleFilters} class="text-muted-foreground hover:text-foreground">
							<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M6 18L18 6M6 6l12 12"
								/>
							</svg>
						</button>
					</div>
					<FilterPanel
						bind:filters
						onApply={() => {
							loadGames(1);
							showFilters = false;
						}}
						onClear={() => {
							loadGames(1);
							showFilters = false;
						}}
					/>
				</div>
			</div>
		{/if}

		<!-- Main Content -->
		<div class="min-w-0 flex-1">
			<!-- Loading State -->
			{#if loading}
				<div class="game-box-lid p-12 text-center">
					<LoadingSpinner class="mx-auto mb-4" />
					<p class="text-muted-foreground font-body">Loading your collection...</p>
				</div>
			{/if}

			<!-- Error State -->
			{#if error && !loading}
				<GameBox variant="default" class="text-center">
					<div class="py-8">
						<div class="text-destructive mb-4">
							<svg class="mx-auto h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
								></path>
							</svg>
						</div>
						<h3 class="font-display mb-2 text-lg font-semibold">Unable to Load Games</h3>
						<p class="text-muted-foreground font-body mb-4">{error}</p>
						<Button variant="game-primary" onclick={() => loadGames(currentPage)}>Try Again</Button>
					</div>
				</GameBox>
			{/if}

			<!-- Empty State -->
			{#if !loading && !error && games.length === 0}
				<GameBox variant="featured" showCorners={true} class="text-center">
					<div class="py-12">
						<div class="mb-6">
							<div
								class="bg-parchment-dark mx-auto flex h-24 w-24 items-center justify-center rounded-full"
							>
								<Rulebook size={48} class="text-game-blue" />
							</div>
						</div>
						<h3 class="font-display mb-3 text-xl font-semibold">No Games Yet</h3>
						<p class="text-muted-foreground font-body mx-auto mb-6 max-w-md">
							Your game library is empty. Start building your collection by adding your first board
							game.
						</p>
						<Button variant="game-accent" onclick={navigateToAddGame} class="gap-2">
							<GameBoxIcon size={18} />
							Add Your First Game
						</Button>
					</div>
				</GameBox>
			{/if}

			<!-- Games Dashboard -->
			{#if !loading && !error && games.length > 0}
				<CollectionDashboard
					{games}
					{currentPage}
					{totalPages}
					{total}
					onPageChange={handlePageChange}
					onDelete={handleDelete}
				/>
			{/if}
		</div>
	</div>
</main>
