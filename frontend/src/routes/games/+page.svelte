<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { api, type GameSummary } from '$lib';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';

	// State management
	let games = $state<GameSummary[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let currentPage = $state(1);
	let totalPages = $state(1);
	let total = $state(0);
	let limit = $state(20);

	onMount(() => {
		loadGames(1);
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
				// Reload the current page
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

	function handleView(game: GameSummary) {
		goto(`/games/${game.id}`);
	}

	function handleEdit(game: GameSummary) {
		goto(`/games/${game.id}/edit`);
	}

	function navigateToAddGame() {
		goto('/games/add');
	}

	function nextPage() {
		if (currentPage < totalPages) {
			loadGames(currentPage + 1);
		}
	}

	function prevPage() {
		if (currentPage > 1) {
			loadGames(currentPage - 1);
		}
	}

	function goToPage(page: number) {
		if (page >= 1 && page <= totalPages) {
			loadGames(page);
		}
	}
</script>

<svelte:head>
	<title>Games - Tabletop Atlas</title>
	<meta name="description" content="Browse and manage your board game collection" />
</svelte:head>

<div class="bg-background min-h-screen">
	<!-- Header -->
	<header class="bg-card border-b shadow-sm">
		<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
			<div class="flex items-center justify-between py-6">
				<div class="flex items-center">
					<a
						href="/"
						class="text-foreground hover:text-primary flex items-center text-2xl font-bold transition-colors"
					>
						🎲 Tabletop Atlas
					</a>
				</div>
				<nav class="flex space-x-8">
					<a href="/games" class="text-foreground font-medium transition-colors"> Games </a>
					<button class="text-muted-foreground hover:text-foreground transition-colors">
						Upload Rules
					</button>
					<button class="text-muted-foreground hover:text-foreground transition-colors">
						Chat
					</button>
				</nav>
			</div>
		</div>
	</header>

	<!-- Main Content -->
	<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
		<!-- Header -->
		<div class="mb-6 flex items-center justify-between">
			<div>
				<h1 class="text-3xl font-bold text-gray-900">Game Library</h1>
				<p class="text-sm text-gray-600">
					{total > 0
						? `${total} game${total === 1 ? '' : 's'} in your collection`
						: 'No games in your collection yet'}
				</p>
			</div>
			<Button onclick={navigateToAddGame}>Add New Game</Button>
		</div>

		<!-- Loading State -->
		{#if loading}
			<div class="flex items-center justify-center py-12">
				<div class="h-8 w-8 animate-spin rounded-full border-b-2 border-blue-600"></div>
				<span class="ml-2 text-gray-600">Loading games...</span>
			</div>
		{/if}

		<!-- Error State -->
		{#if error && !loading}
			<Card class="text-center">
				<CardContent class="p-6">
					<div class="mb-4 text-red-600">
						<svg class="mx-auto h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
							></path>
						</svg>
					</div>
					<h3 class="mb-2 text-lg font-semibold text-gray-900">Unable to Load Games</h3>
					<p class="mb-4 text-gray-600">{error}</p>
					<Button onclick={() => loadGames(currentPage)}>Try Again</Button>
				</CardContent>
			</Card>
		{/if}

		<!-- Empty State -->
		{#if !loading && !error && games.length === 0}
			<Card class="text-center">
				<CardContent class="p-8">
					<div class="mb-4 text-gray-400">
						<svg class="mx-auto h-16 w-16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
							></path>
						</svg>
					</div>
					<h3 class="mb-2 text-lg font-semibold text-gray-900">No Games Yet</h3>
					<p class="mb-4 text-gray-600">
						Start building your board game library by adding your first game.
					</p>
					<Button onclick={navigateToAddGame}>Add Your First Game</Button>
				</CardContent>
			</Card>
		{/if}

		<!-- Games Grid -->
		{#if !loading && !error && games.length > 0}
			<div class="mb-8 grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
				{#each games as game (game.id)}
					<Card class="transition-shadow hover:shadow-lg">
						<CardHeader>
							<CardTitle class="text-lg">{game.name}</CardTitle>
							{#if game.publisher}
								<p class="text-sm text-gray-600">{game.publisher}</p>
							{/if}
						</CardHeader>
						<CardContent>
							<!-- Game Details -->
							<div class="mb-4 space-y-2 text-sm text-gray-600">
								{#if game.yearPublished}
									<div class="flex items-center">
										<span class="w-20 font-medium">Year:</span>
										<span>{game.yearPublished}</span>
									</div>
								{/if}

								{#if game.minPlayers && game.maxPlayers}
									<div class="flex items-center">
										<span class="w-20 font-medium">Players:</span>
										<span
											>{game.minPlayers === game.maxPlayers
												? game.minPlayers
												: `${game.minPlayers}-${game.maxPlayers}`}</span
										>
									</div>
								{/if}

								{#if game.complexityRating}
									<div class="flex items-center">
										<span class="w-20 font-medium">Complexity:</span>
										<span>{game.complexityRating.toFixed(1)}/5.0</span>
									</div>
								{/if}
							</div>

							<!-- Badges -->
							<div class="mb-4 flex flex-wrap gap-2">
								{#if game.hasRulesPdf}
									<Badge variant="secondary" class="text-xs">PDF Rules</Badge>
								{/if}

								{#if game.houseRulesCount > 0}
									<Badge variant="outline" class="text-xs">
										{game.houseRulesCount} House Rule{game.houseRulesCount === 1 ? '' : 's'}
									</Badge>
								{/if}
							</div>

							<!-- Actions -->
							<div class="flex items-center justify-between border-t border-gray-200 pt-4">
								<Button
									variant="ghost"
									size="sm"
									onclick={() => handleView(game)}
									class="text-blue-600 hover:text-blue-800"
								>
									View Details
								</Button>

								<div class="flex space-x-2">
									<Button variant="outline" size="sm" onclick={() => handleEdit(game)}>Edit</Button>
									<Button variant="destructive" size="sm" onclick={() => handleDelete(game)}>
										Delete
									</Button>
								</div>
							</div>
						</CardContent>
					</Card>
				{/each}
			</div>

			<!-- Pagination -->
			{#if totalPages > 1}
				<div class="flex items-center justify-between">
					<div class="text-sm text-gray-700">
						Showing page {currentPage} of {totalPages} ({total} total games)
					</div>

					<div class="flex items-center space-x-2">
						<Button variant="outline" size="sm" onclick={prevPage} disabled={currentPage <= 1}>
							Previous
						</Button>

						<!-- Page Numbers -->
						<div class="flex items-center space-x-1">
							{#each Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
								const startPage = Math.max(1, currentPage - 2);
								return startPage + i;
							}).filter((page) => page <= totalPages) as page (page)}
								<Button
									variant={page === currentPage ? 'default' : 'outline'}
									size="sm"
									onclick={() => goToPage(page)}
									class="h-8 w-8 p-0"
								>
									{page}
								</Button>
							{/each}
						</div>

						<Button
							variant="outline"
							size="sm"
							onclick={nextPage}
							disabled={currentPage >= totalPages}
						>
							Next
						</Button>
					</div>
				</div>
			{/if}
		{/if}
	</main>
</div>
