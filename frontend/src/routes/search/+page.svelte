<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, type GameSummary, type SearchResult } from '$lib';
	import { Button, Input, Label, Badge, GameBox, CardSleeve, LoadingSpinner } from '$lib/components/ui';
	import { ComponentTray, ComponentTraySection } from '$lib/components/ui';
	import { Dice, GameBoxIcon, Rulebook, SearchGlass, ChatBubble } from '$lib/components/icons';
	import { useHeader } from '$lib/stores/header';

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
	let hasSearched = $state(false);

	let initialUrlParams: { gameId: string | null; query: string | null } = {
		gameId: null,
		query: null
	};
	let hasInitialized = $state(false);

	const header = useHeader();
	header.configure({
		showSearch: false,
		currentGame: null
	});

	$effect(() => {
		searchQuery;
		hasSearched = false;
	});

	onMount(() => {
		const params = page.url.searchParams;
		initialUrlParams.gameId = params.get('gameId');
		initialUrlParams.query = params.get('q');

		loadGames();

		if (initialUrlParams.gameId) {
			selectedGameId = parseInt(initialUrlParams.gameId);
		}
		if (initialUrlParams.query) {
			searchQuery = initialUrlParams.query;
		}

		hasInitialized = true;
	});

	$effect(() => {
		if (selectedGameId && games.length > 0) {
			selectedGame = games.find((g) => g.id === selectedGameId) || null;

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
				query: { limit: 100 }
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
		searchResults = [];
		totalResults = 0;
		error = null;

		if (!game.hasRulesPdf) {
			error = `${game.name} doesn't have uploaded rules yet. Please upload rules first to enable search.`;
		}

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
		hasSearched = true;

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

	function getDiceValue(score: number): 1 | 2 | 3 | 4 | 5 | 6 {
		if (score >= 0.9) return 6;
		if (score >= 0.75) return 5;
		if (score >= 0.6) return 4;
		if (score >= 0.45) return 3;
		if (score >= 0.3) return 2;
		return 1;
	}

	function formatSimilarityScore(score: number): string {
		return (score * 100).toFixed(0) + '%';
	}
</script>

<svelte:head>
	<title>Search Rules - Tabletop Atlas</title>
	<meta name="description" content="Search for keywords and concepts in your uploaded game rules." />
</svelte:head>

<main class="mx-auto max-w-6xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Page Header -->
	<div class="mb-10">
		<div class="rulebook-header">
			<h1 class="text-2xl md:text-3xl">Rule Search</h1>
		</div>
		<p class="text-center text-muted-foreground font-body">
			Search for keywords in your uploaded game rules
		</p>
	</div>

	<div class="flex flex-col lg:flex-row gap-8">
		<!-- Game Selection Sidebar -->
		<div class="lg:w-80 flex-shrink-0">
			<ComponentTray title="Select Game">
				{#if loading}
					<ComponentTraySection>
						<LoadingSpinner text="Loading games..." />
					</ComponentTraySection>
				{:else if games.length === 0}
					<ComponentTraySection>
						<p class="text-sm text-parchment/70 text-center mb-2">No games found</p>
						<Button variant="game-primary" href="/games/add" size="sm" class="w-full">Add Game</Button>
					</ComponentTraySection>
				{:else}
					<div class="space-y-2 max-h-96 overflow-y-auto">
						{#each games as game (game.id)}
							<button
								onclick={() => selectGame(game.id)}
								class="w-full text-left p-3 rounded-lg transition-all
									{selectedGameId === game.id
										? 'bg-game-blue text-white'
										: 'bg-parchment hover:bg-parchment-dark text-foreground'}
									{!game.hasRulesPdf ? 'opacity-60' : ''}"
							>
								<div class="font-display font-medium text-sm">{game.name}</div>
								<div class="flex items-center gap-1 mt-1">
									{#if game.hasRulesPdf}
										<Rulebook size={12} class="opacity-60" />
									{:else}
										<span class="text-xs opacity-70">No rules</span>
									{/if}
								</div>
							</button>
						{/each}
					</div>
				{/if}
			</ComponentTray>

			{#if selectedGame}
				<ComponentTray title="Selected Game" class="mt-4">
					<ComponentTraySection>
						<div class="flex items-center gap-3">
							<div class="w-10 h-10 rounded-lg bg-game-blue flex items-center justify-center">
								<GameBoxIcon size={20} class="text-white" />
							</div>
							<div class="flex-1 min-w-0">
								<p class="font-display font-medium text-sm truncate">{selectedGame.name}</p>
								{#if selectedGame.publisher}
									<p class="text-xs opacity-70">{selectedGame.publisher}</p>
								{/if}
							</div>
						</div>
						<Button variant="ghost" size="sm" onclick={() => selectedGame && goToGame(selectedGame.id)} class="w-full mt-3">
							View Details
						</Button>
					</ComponentTraySection>
				</ComponentTray>
			{/if}
		</div>

		<!-- Search Interface and Results -->
		<div class="flex-1 min-w-0 space-y-6">
			<!-- Search Form - Index Card Style -->
			<GameBox variant="default" class="relative p-6">
				<div class="absolute -top-3 left-6 bg-game-green text-white px-4 py-1 rounded-t-lg font-display text-sm font-semibold">
					Search
				</div>
				<div class="pt-2">
					<form onsubmit={handleSearchSubmit} class="space-y-4">
						<div class="flex gap-2">
							<div class="relative flex-1">
								<SearchGlass size={18} class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
								<Input
									bind:value={searchQuery}
									placeholder="e.g. win conditions, combat, movement..."
									disabled={!selectedGameId || !selectedGame?.hasRulesPdf || searching}
									class="pl-10 bg-card"
								/>
							</div>
							<Button
								type="submit"
								variant="game-primary"
								disabled={!selectedGameId || !selectedGame?.hasRulesPdf || !searchQuery.trim() || searching}
							>
								{#if searching}
									<LoadingSpinner size="sm" />
								{:else}
									Search
								{/if}
							</Button>
						</div>

						<div class="flex items-center justify-between">
							<p class="text-xs text-muted-foreground font-ui">
								For Q&A, try our <a href="/chat" class="text-game-blue hover:underline inline-flex items-center gap-1">
									<ChatBubble size={12} /> Chat
								</a>
							</p>
							<div class="flex items-center gap-2">
								<Label for="searchLimit" class="text-xs text-muted-foreground">Results:</Label>
								<Input
									id="searchLimit"
									type="number"
									bind:value={searchLimit}
									min="1"
									max="20"
									disabled={searching}
									class="w-16 h-8 text-center text-sm"
								/>
							</div>
						</div>
					</form>

					{#if error}
						<div class="mt-4 rounded-lg border-2 border-game-red bg-game-red/10 p-3">
							<p class="text-sm text-game-red font-ui">{error}</p>
						</div>
					{/if}
				</div>
			</GameBox>

			<!-- Search Results - Card Sleeves -->
			{#if searchResults.length > 0}
				<div>
					<div class="flex items-center justify-between mb-4">
						<h2 class="font-display font-semibold text-lg">Results</h2>
						<Badge variant="secondary" class="font-ui">
							{totalResults} match{totalResults === 1 ? '' : 'es'}
						</Badge>
					</div>

					<div class="space-y-3">
						{#each searchResults as result, index (result.chunkId)}
							<CardSleeve variant="default" class="p-4">
								<div class="flex items-start gap-4">
									<!-- Dice Score -->
									<div class="flex-shrink-0 flex flex-col items-center">
										<Dice size={32} value={getDiceValue(result.similarityScore)} class="text-foreground" />
										<span class="text-xs font-ui text-muted-foreground mt-1">
											{formatSimilarityScore(result.similarityScore)}
										</span>
									</div>

									<!-- Result Content -->
									<div class="flex-1 min-w-0">
										<div class="flex items-center gap-2 mb-2">
											<Badge variant="outline" class="text-xs font-ui">#{index + 1}</Badge>
											{#if result.metadata}
												<span class="text-xs text-muted-foreground">{result.metadata}</span>
											{/if}
										</div>
										<p class="text-sm font-body leading-relaxed">{result.chunkText}</p>
									</div>
								</div>
							</CardSleeve>
						{/each}
					</div>
				</div>
			{:else if hasSearched && searchQuery && selectedGameId && !searching}
				<GameBox variant="default" class="text-center py-12">
					<div class="mb-4">
						<Dice size={48} value={1} class="mx-auto text-muted-foreground" />
					</div>
					<h3 class="font-display font-semibold text-lg mb-2">No Matches Found</h3>
					<p class="text-muted-foreground font-body text-sm mb-4">
						Try different keywords or check our <a href="/chat" class="text-game-blue hover:underline">Chat</a> for Q&A
					</p>
					<div class="flex flex-wrap justify-center gap-2">
						{#each ['victory', 'combat', 'movement', 'setup', 'turn', 'scoring'] as term}
							<button
								onclick={() => { searchQuery = term; }}
								class="px-3 py-1 rounded-full bg-parchment-dark text-sm font-ui hover:bg-primary hover:text-primary-foreground transition-colors"
							>
								{term}
							</button>
						{/each}
					</div>
				</GameBox>
			{:else if !selectedGameId}
				<GameBox variant="default" showCorners={true} class="text-center py-16 lg:py-24">
					<div class="mb-6">
						<SearchGlass size={56} class="mx-auto text-muted-foreground" />
					</div>
					<h3 class="font-display font-semibold text-xl mb-3">Select a Game</h3>
					<p class="text-muted-foreground font-body">
						Choose a game from the sidebar to search its rules
					</p>
				</GameBox>
			{/if}
		</div>
	</div>
</main>
