<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { api, type GameSummary, type SearchResult } from '$lib';
	import {
		Button,
		Input,
		Label,
		Badge,
		GameBox,
		CardSleeve,
		LoadingSpinner
	} from '$lib/components/ui';
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
	let gameFilterQuery = $state('');

	let filteredGames = $derived(
		gameFilterQuery.trim()
			? games.filter((g) => g.name.toLowerCase().includes(gameFilterQuery.toLowerCase()))
			: games
	);

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
		// Track searchQuery changes to reset hasSearched
		void searchQuery;
		hasSearched = false;
	});

	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
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
		}
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
	<meta
		name="description"
		content="Search for keywords and concepts in your uploaded game rules."
	/>
</svelte:head>

<main class="mx-auto max-w-6xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Page Header -->
	<div class="mb-10">
		<div class="rulebook-header">
			<h1 class="text-2xl md:text-3xl">Rule Search</h1>
		</div>
		<p class="text-muted-foreground font-body text-center">
			Search for keywords in your uploaded game rules
		</p>
	</div>

	<div class="flex flex-col gap-8 lg:flex-row">
		<!-- Game Selection Sidebar -->
		<div class="flex-shrink-0 lg:w-80">
			<ComponentTray title="Select Game">
				{#if loading}
					<ComponentTraySection>
						<LoadingSpinner text="Loading games..." />
					</ComponentTraySection>
				{:else if games.length === 0}
					<ComponentTraySection>
						<p class="text-parchment/70 mb-2 text-center text-sm">No games found</p>
						<Button variant="game-primary" href="/games/add" size="sm" class="w-full"
							>Add Game</Button
						>
					</ComponentTraySection>
				{:else}
					<div class="mb-2">
						<Input
							bind:value={gameFilterQuery}
							placeholder="Filter games..."
							class="bg-parchment text-foreground placeholder:text-foreground/50 h-8 text-sm"
						/>
					</div>
					{#if filteredGames.length === 0}
						<p class="text-parchment/70 py-2 text-center text-sm">No games match filter</p>
					{:else}
						<div class="max-h-80 space-y-2 overflow-y-auto">
							{#each filteredGames as game (game.id)}
								<button
									onclick={() => selectGame(game.id)}
									class="w-full rounded-lg p-3 text-left transition-all
										{selectedGameId === game.id
										? 'bg-game-blue text-white'
										: 'bg-parchment hover:bg-parchment-dark text-foreground'}
										{!game.hasRulesPdf ? 'opacity-60' : ''}"
								>
									<div class="font-display text-sm font-medium">{game.name}</div>
									<div class="mt-1 flex items-center gap-1">
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
				{/if}
			</ComponentTray>

			{#if selectedGame}
				<ComponentTray title="Selected Game" class="mt-4">
					<ComponentTraySection>
						<div class="flex items-center gap-3">
							<div class="bg-game-blue flex h-10 w-10 items-center justify-center rounded-lg">
								<GameBoxIcon size={20} class="text-white" />
							</div>
							<div class="min-w-0 flex-1">
								<p class="font-display truncate text-sm font-medium">{selectedGame.name}</p>
								{#if selectedGame.publisher}
									<p class="text-xs opacity-70">{selectedGame.publisher}</p>
								{/if}
							</div>
						</div>
						<Button
							variant="ghost"
							size="sm"
							onclick={() => selectedGame && goToGame(selectedGame.id)}
							class="mt-3 w-full"
						>
							View Details
						</Button>
					</ComponentTraySection>
				</ComponentTray>
			{/if}
		</div>

		<!-- Search Interface and Results -->
		<div class="min-w-0 flex-1 space-y-6">
			<!-- Search Form -->
			<GameBox variant="default" class="p-6">
				<form onsubmit={handleSearchSubmit} class="space-y-4">
					<div class="flex gap-2">
						<div class="relative flex-1">
							<SearchGlass
								size={18}
								class="text-muted-foreground absolute top-1/2 left-3 -translate-y-1/2"
							/>
							<Input
								bind:value={searchQuery}
								placeholder="e.g. win conditions, combat, movement..."
								disabled={!selectedGameId || !selectedGame?.hasRulesPdf || searching}
								class="bg-card pl-10"
							/>
						</div>
						<Button
							type="submit"
							variant="game-primary"
							disabled={!selectedGameId ||
								!selectedGame?.hasRulesPdf ||
								!searchQuery.trim() ||
								searching}
						>
							{#if searching}
								<LoadingSpinner size="sm" />
							{:else}
								Search
							{/if}
						</Button>
					</div>

					<div class="flex items-center justify-between">
						<p class="text-muted-foreground font-ui text-xs">
							For Q&A, try our <a
								href="/chat"
								class="text-game-blue inline-flex items-center gap-1 hover:underline"
							>
								<ChatBubble size={12} /> Chat
							</a>
						</p>
						<div class="flex items-center gap-2">
							<Label for="searchLimit" class="text-muted-foreground text-xs">Results:</Label>
							<Input
								id="searchLimit"
								type="number"
								bind:value={searchLimit}
								min="1"
								max="20"
								disabled={searching}
								class="h-8 w-16 text-center text-sm"
							/>
						</div>
					</div>
				</form>

				{#if error}
					<div class="border-game-red bg-game-red/10 mt-4 rounded-lg border-2 p-3">
						<p class="text-game-red font-ui text-sm">{error}</p>
					</div>
				{/if}
			</GameBox>

			<!-- Search Results - Card Sleeves -->
			{#if searchResults.length > 0}
				<div>
					<div class="mb-4 flex items-center justify-between">
						<h2 class="font-display text-lg font-semibold">Results</h2>
						<Badge variant="secondary" class="font-ui">
							{totalResults} match{totalResults === 1 ? '' : 'es'}
						</Badge>
					</div>

					<div class="space-y-3">
						{#each searchResults as result, index (result.chunkId)}
							<CardSleeve variant="default" class="p-4">
								<div class="flex items-start gap-4">
									<!-- Dice Score -->
									<div class="flex flex-shrink-0 flex-col items-center">
										<Dice
											size={32}
											value={getDiceValue(result.similarityScore)}
											class="text-foreground"
										/>
										<span class="font-ui text-muted-foreground mt-1 text-xs">
											{formatSimilarityScore(result.similarityScore)}
										</span>
									</div>

									<!-- Result Content -->
									<div class="min-w-0 flex-1">
										<div class="mb-2 flex items-center gap-2">
											<Badge variant="outline" class="font-ui text-xs">#{index + 1}</Badge>
											{#if result.metadata}
												<span class="text-muted-foreground text-xs">{result.metadata}</span>
											{/if}
										</div>
										<p class="font-body text-sm leading-relaxed">{result.chunkText}</p>
									</div>
								</div>
							</CardSleeve>
						{/each}
					</div>
				</div>
			{:else if hasSearched && searchQuery && selectedGameId && !searching}
				<GameBox variant="default" class="py-12 text-center">
					<div class="mb-4">
						<Dice size={48} value={1} class="text-muted-foreground mx-auto" />
					</div>
					<h3 class="font-display mb-2 text-lg font-semibold">No Matches Found</h3>
					<p class="text-muted-foreground font-body mb-4 text-sm">
						Try different keywords or check our <a
							href="/chat"
							class="text-game-blue hover:underline">Chat</a
						> for Q&A
					</p>
					<div class="flex flex-wrap justify-center gap-2">
						{#each ['victory', 'combat', 'movement', 'setup', 'turn', 'scoring'] as term (term)}
							<button
								onclick={() => {
									searchQuery = term;
								}}
								class="bg-parchment-dark font-ui hover:bg-primary hover:text-primary-foreground rounded-full px-3 py-1 text-sm transition-colors"
							>
								{term}
							</button>
						{/each}
					</div>
				</GameBox>
			{:else if !selectedGameId}
				<GameBox variant="default" showCorners={true} class="py-16 text-center lg:py-24">
					<div class="mb-6">
						<SearchGlass size={56} class="text-muted-foreground mx-auto" />
					</div>
					<h3 class="font-display mb-3 text-xl font-semibold">Select a Game</h3>
					<p class="text-muted-foreground font-body">
						Choose a game from the sidebar to search its rules
					</p>
				</GameBox>
			{/if}
		</div>
	</div>
</main>
