<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import {
		api,
		type Game,
		type GameSummary,
		type SearchResult,
		type RulesSearchResponse
	} from '$lib';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui';
	import { Input, Label } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';
	import { useHeader } from '$lib/stores/header';

	// State management
	let games = $state<GameSummary[]>([]);
	let loading = $state(true);
	let searching = $state(false);
	let error = $state<string | null>(null);
	let searchResults = $state<SearchResult[]>([]);
	let selectedGameId = $state<number | null>(null);
	let selectedGame = $state<GameSummary | null>(null);
	let searchQuery = $state('');
	let searchLimit = $state(5);
	let totalResults = $state(0);

	// URL search params (read once on mount to avoid reactive loops)
	let initialUrlParams: { gameId: string | null; query: string | null } = {
		gameId: null,
		query: null
	};
	let hasInitialized = $state(false);

	// Configure header for this page
	const header = useHeader();
	header.configure({
		showSearch: false,
		currentGame: null
	});

	onMount(() => {
		// Read URL params once on mount
		const params = $page.url.searchParams;
		initialUrlParams.gameId = params.get('gameId');
		initialUrlParams.query = params.get('q');

		loadGames();

		// Set initial values from URL params
		if (initialUrlParams.gameId) {
			selectedGameId = parseInt(initialUrlParams.gameId);
		}
		if (initialUrlParams.query) {
			searchQuery = initialUrlParams.query;
		}

		hasInitialized = true;
	});

	// Watch for selectedGameId changes to update selectedGame
	$effect(() => {
		if (selectedGameId && games.length > 0) {
			selectedGame = games.find((g) => g.id === selectedGameId) || null;

			// Only perform initial search if we have URL params and haven't searched yet
			if (
				hasInitialized &&
				selectedGame &&
				initialUrlParams.query &&
				searchQuery &&
				!searching &&
				searchResults.length === 0
			) {
				performSearch();
			}
		}
	});

	async function loadGames() {
		loading = true;
		error = null;

		try {
			const result = await api.methods.listGames({
				query: { limit: 100 } // Get all games for selection
			});

			if (result.type === 'success') {
				games = result.data.items;
			} else if (result.type === 'error') {
				error = result.data.message || 'Failed to load games';
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Failed to load games';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			loading = false;
		}
	}

	function selectGame(gameId: number) {
		const game = games.find((g) => g.id === gameId);
		if (!game) return;

		selectedGameId = gameId;
		selectedGame = game;
		// Clear previous results when switching games
		searchResults = [];
		totalResults = 0;
		error = null;

		// Show warning if game has no rules
		if (!game.hasRulesPdf) {
			error = `${game.name} doesn't have uploaded rules yet. Please upload rules first to enable search.`;
		}

		// Update URL when user manually selects a game (not during initialization)
		if (hasInitialized) {
			updateUrlParams();
		}
	}

	function updateUrlParams() {
		const params = new URLSearchParams();
		if (selectedGameId) {
			params.set('gameId', selectedGameId.toString());
		}
		if (searchQuery.trim()) {
			params.set('q', searchQuery.trim());
		}
		// Use replaceState to avoid creating browser history entries
		const newUrl = `/search?${params.toString()}`;
		if (newUrl !== window.location.pathname + window.location.search) {
			goto(newUrl, { replaceState: true });
		}
	}

	async function performSearch() {
		if (!selectedGameId || !selectedGame?.hasRulesPdf || !searchQuery.trim()) {
			if (selectedGameId && !selectedGame?.hasRulesPdf) {
				error = 'This game does not have uploaded rules. Please upload rules first.';
			}
			return;
		}

		searching = true;
		error = null;

		try {
			const result = await api.methods.searchRules({
				query: {
					gameId: selectedGameId,
					query: searchQuery.trim(),
					limit: searchLimit
				}
			});

			if (result.type === 'success') {
				searchResults = result.data.results;
				totalResults = result.data.totalResults;
			} else if (result.type === 'error') {
				error = result.data.message || 'Search failed';
				searchResults = [];
				totalResults = 0;
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Search failed';
				searchResults = [];
				totalResults = 0;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			searchResults = [];
			totalResults = 0;
		} finally {
			searching = false;
		}
	}

	function handleSearchSubmit(event: Event) {
		event.preventDefault();
		if (hasInitialized) {
			updateUrlParams();
		}
		performSearch();
	}

	function goToGame(gameId: number) {
		goto(`/games/${gameId}`);
	}

	function formatSimilarityScore(score: number): string {
		return (score * 100).toFixed(1) + '%';
	}

	function getSimilarityColor(score: number): string {
		if (score >= 0.8) return 'text-green-600';
		if (score >= 0.6) return 'text-yellow-600';
		return 'text-gray-600';
	}

	function truncateText(text: string, maxLength: number = 200): string {
		if (text.length <= maxLength) return text;
		return text.substring(0, maxLength) + '...';
	}
</script>

<svelte:head>
	<title>Search Rules - Tabletop Atlas</title>
	<meta
		name="description"
		content="Search through your board game rules using AI-powered semantic search"
	/>
</svelte:head>

<!-- Main Content -->
<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Header -->
	<div class="mb-8">
		<h1 class="text-3xl font-bold text-gray-900">Search Game Rules</h1>
		<p class="mt-2 text-gray-600">
			Use AI-powered semantic search to find specific information in your uploaded game rules
		</p>
	</div>

	<div class="grid grid-cols-1 gap-8 lg:grid-cols-4">
		<!-- Game Selection Sidebar -->
		<div class="lg:col-span-1">
			<Card>
				<CardHeader>
					<CardTitle>Select Game</CardTitle>
					<CardDescription>Choose which game's rules to search</CardDescription>
				</CardHeader>
				<CardContent>
					{#if loading}
						<div class="flex items-center justify-center py-8">
							<div class="h-6 w-6 animate-spin rounded-full border-b-2 border-blue-600"></div>
							<span class="ml-2 text-sm text-gray-600">Loading games...</span>
						</div>
					{:else if error && games.length === 0}
						<div class="text-center">
							<p class="text-sm text-red-600">{error}</p>
							<Button onclick={loadGames} class="mt-4" size="sm">Try Again</Button>
						</div>
					{:else if games.length === 0}
						<div class="text-center">
							<p class="text-sm text-gray-600">No games found. Add a game first.</p>
							<Button href="/games/add" class="mt-4" size="sm">Add Game</Button>
						</div>
					{:else}
						<div class="max-h-96 space-y-2 overflow-y-auto">
							{#each games as game (game.id)}
								<button
									onclick={() => selectGame(game.id)}
									class="w-full rounded-lg border p-3 text-left transition-colors hover:bg-gray-50
										{selectedGameId === game.id ? 'border-blue-500 bg-blue-50' : 'border-gray-200'}
										{!game.hasRulesPdf ? 'opacity-75' : ''}"
								>
									<div class="flex items-start justify-between">
										<div class="min-w-0 flex-1">
											<h3 class="truncate text-sm font-medium text-gray-900">{game.name}</h3>
											{#if game.publisher}
												<p class="truncate text-xs text-gray-500">{game.publisher}</p>
											{/if}
											<div class="mt-1 flex items-center space-x-1">
												{#if game.yearPublished}
													<Badge variant="secondary" class="text-xs">{game.yearPublished}</Badge>
												{/if}
												{#if game.hasRulesPdf}
													<Badge variant="default" class="text-xs">Has PDF</Badge>
												{:else}
													<Badge variant="outline" class="border-red-200 text-xs text-red-600"
														>No Rules</Badge
													>
												{/if}
											</div>
										</div>
										{#if selectedGameId === game.id}
											<svg
												class="h-4 w-4 flex-shrink-0 text-blue-600"
												fill="currentColor"
												viewBox="0 0 20 20"
											>
												<path
													fill-rule="evenodd"
													d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
													clip-rule="evenodd"
												></path>
											</svg>
										{/if}
									</div>
								</button>
							{/each}

							{#if games.length === 0}
								<div class="py-8 text-center">
									<div class="mb-2 text-gray-400">
										<svg
											class="mx-auto h-12 w-12"
											fill="none"
											stroke="currentColor"
											viewBox="0 0 24 24"
										>
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
											></path>
										</svg>
									</div>
									<p class="mb-2 text-sm text-gray-600">No games found</p>
									<Button href="/games/add" size="sm" variant="outline">Add Games</Button>
								</div>
							{:else if games.filter((g) => g.hasRulesPdf).length === 0}
								<div class="mt-4 rounded-md border border-yellow-200 bg-yellow-50 p-3">
									<div class="flex">
										<div class="ml-3">
											<h3 class="text-sm font-medium text-yellow-800">No Rules Uploaded</h3>
											<div class="mt-2 text-sm text-yellow-700">
												<p>
													You have games but no uploaded rules. Upload PDF rules to enable search.
												</p>
												<Button href="/upload" size="sm" variant="outline" class="mt-2"
													>Upload Rules</Button
												>
											</div>
										</div>
									</div>
								</div>
							{/if}
						</div>
					{/if}
				</CardContent>
			</Card>

			{#if selectedGame}
				<!-- Selected Game Info -->
				<Card class="mt-6">
					<CardHeader>
						<CardTitle class="text-base">{selectedGame.name}</CardTitle>
						{#if selectedGame.publisher}
							<CardDescription class="text-sm">{selectedGame.publisher}</CardDescription>
						{/if}
					</CardHeader>
					<CardContent class="space-y-2">
						{#if selectedGame.yearPublished}
							<div class="text-sm">
								<span class="font-medium text-gray-500">Year:</span>
								<span class="text-gray-900">{selectedGame.yearPublished}</span>
							</div>
						{/if}
						<div class="pt-2">
							<Button
								variant="outline"
								onclick={() => selectedGame && goToGame(selectedGame.id)}
								class="w-full"
								size="sm"
							>
								View Game Details
							</Button>
						</div>
					</CardContent>
				</Card>
			{/if}
		</div>

		<!-- Search Interface and Results -->
		<div class="space-y-6 lg:col-span-3">
			<!-- Search Form -->
			<Card>
				<CardHeader>
					<CardTitle>Search Rules</CardTitle>
					<CardDescription>
						{#if selectedGame}
							Search through the rules for {selectedGame.name}
						{:else}
							Select a game to start searching its rules
						{/if}
					</CardDescription>
				</CardHeader>
				<CardContent>
					<form onsubmit={handleSearchSubmit} class="space-y-4">
						<div>
							<Label for="searchQuery">Search Query</Label>
							<Input
								id="searchQuery"
								bind:value={searchQuery}
								placeholder="e.g. How do I win the game? What happens during combat?"
								disabled={!selectedGameId || !selectedGame?.hasRulesPdf || searching}
								class="mt-1"
							/>
							<p class="mt-1 text-xs text-gray-500">
								Ask natural language questions or search for specific game concepts
							</p>
						</div>

						<div class="flex items-center space-x-4">
							<div class="flex-1">
								<Label for="searchLimit" class="text-sm">Results limit</Label>
								<Input
									id="searchLimit"
									type="number"
									bind:value={searchLimit}
									min="1"
									max="20"
									disabled={searching}
									class="mt-1"
								/>
							</div>
							<div class="flex-shrink-0 pt-6">
								<Button
									type="submit"
									disabled={!selectedGameId ||
										!selectedGame?.hasRulesPdf ||
										!searchQuery.trim() ||
										searching}
								>
									{#if searching}
										<svg class="mr-2 -ml-1 h-4 w-4 animate-spin" fill="none" viewBox="0 0 24 24">
											<circle
												class="opacity-25"
												cx="12"
												cy="12"
												r="10"
												stroke="currentColor"
												stroke-width="4"
											></circle>
											<path
												class="opacity-75"
												fill="currentColor"
												d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
											></path>
										</svg>
										Searching...
									{:else}
										Search Rules
									{/if}
								</Button>
							</div>
						</div>
					</form>

					{#if error}
						<div class="mt-4 rounded-md border border-red-200 bg-red-50 p-3">
							<p class="text-sm text-red-700">{error}</p>
						</div>
					{/if}
				</CardContent>
			</Card>

			<!-- Search Results -->
			{#if searchResults.length > 0}
				<Card>
					<CardHeader>
						<CardTitle>Search Results</CardTitle>
						<CardDescription>
							Found {totalResults} relevant passage{totalResults === 1 ? '' : 's'}
							{#if searchQuery}for "{searchQuery}"{/if}
						</CardDescription>
					</CardHeader>
					<CardContent class="space-y-4">
						{#each searchResults as result, index (result.chunkId)}
							<div class="space-y-3 rounded-lg border border-gray-200 p-4">
								<div class="flex items-start justify-between">
									<div class="flex items-center space-x-2">
										<Badge variant="outline" class="text-xs">
											Result #{index + 1}
										</Badge>
										<Badge
											variant="secondary"
											class="text-xs {getSimilarityColor(result.similarityScore)}"
										>
											{formatSimilarityScore(result.similarityScore)} match
										</Badge>
									</div>
									{#if result.metadata}
										<div class="text-xs text-gray-500">
											{result.metadata}
										</div>
									{/if}
								</div>

								<div class="prose prose-sm max-w-none">
									<p class="leading-relaxed text-gray-900">
										{result.chunkText}
									</p>
								</div>

								<div class="flex items-center justify-between text-xs text-gray-500">
									<span>Chunk ID: {result.chunkId}</span>
									<span>Similarity: {result.similarityScore.toFixed(3)}</span>
								</div>
							</div>
						{/each}
					</CardContent>
				</Card>
			{:else if searchQuery && selectedGameId && !searching && searchResults.length === 0}
				<!-- No Results -->
				<Card>
					<CardContent class="p-8 text-center">
						<div class="mb-4 text-gray-400">
							<svg class="mx-auto h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
								></path>
							</svg>
						</div>
						<h3 class="mb-2 text-lg font-medium text-gray-900">No results found</h3>
						<p class="mb-4 text-gray-600">
							No relevant passages found for "{searchQuery}"{#if selectedGame}
								in {selectedGame.name}{/if}.
						</p>
						<div class="text-sm text-gray-500">
							<p>Try:</p>
							<ul class="mt-1 list-inside list-disc space-y-1">
								<li>Using different keywords or phrases</li>
								<li>Asking more general questions</li>
								<li>Checking if the rules PDF was properly processed</li>
							</ul>
						</div>
					</CardContent>
				</Card>
			{:else if !selectedGameId}
				<!-- No Game Selected -->
				<Card>
					<CardContent class="p-8 text-center">
						<div class="mb-4 text-gray-400">
							<svg class="mx-auto h-16 w-16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
								></path>
							</svg>
						</div>
						<h3 class="mb-2 text-lg font-medium text-gray-900">Ready to Search</h3>
						<p class="text-gray-600">
							Select a game from the sidebar to start searching through its rules.
						</p>
					</CardContent>
				</Card>
			{/if}
		</div>
	</div>

	<!-- Tips Section -->
	<div class="mt-12">
		<h2 class="mb-4 text-xl font-semibold text-gray-900">Search Tips</h2>
		<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
			<Card>
				<CardContent class="p-6">
					<div class="mb-3 flex items-center">
						<div class="flex-shrink-0">
							<svg
								class="h-6 w-6 text-blue-600"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
								></path>
							</svg>
						</div>
						<h3 class="ml-3 text-lg font-medium text-gray-900">Ask Natural Questions</h3>
					</div>
					<p class="text-sm text-gray-600">
						Ask questions like "How do I win?" or "What happens during combat?" instead of just
						searching for keywords.
					</p>
				</CardContent>
			</Card>

			<Card>
				<CardContent class="p-6">
					<div class="mb-3 flex items-center">
						<div class="flex-shrink-0">
							<svg
								class="h-6 w-6 text-green-600"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M13 10V3L4 14h7v7l9-11h-7z"
								></path>
							</svg>
						</div>
						<h3 class="ml-3 text-lg font-medium text-gray-900">Semantic Search</h3>
					</div>
					<p class="text-sm text-gray-600">
						Our AI understands context and meaning, so you can find relevant information even if
						your exact words don't appear in the rules.
					</p>
				</CardContent>
			</Card>

			<Card>
				<CardContent class="p-6">
					<div class="mb-3 flex items-center">
						<div class="flex-shrink-0">
							<svg
								class="h-6 w-6 text-purple-600"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
								></path>
							</svg>
						</div>
						<h3 class="ml-3 text-lg font-medium text-gray-900">Similarity Scores</h3>
					</div>
					<p class="text-sm text-gray-600">
						Results are ranked by relevance. Higher similarity scores mean the passage is more
						likely to answer your question.
					</p>
				</CardContent>
			</Card>
		</div>
	</div>
</main>
