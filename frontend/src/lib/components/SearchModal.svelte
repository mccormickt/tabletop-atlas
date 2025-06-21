<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Game, type GameSummary, type SearchResult } from '$lib';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui';
	import { Label } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';
	import {
		SearchInput,
		SearchResult as SearchResultComponent,
		EmptyState,
		LoadingSpinner
	} from '$lib/components/ui';
	import { createEventDispatcher } from 'svelte';

	// Props
	let {
		isOpen = $bindable(false),
		initialGameId = null,
		initialQuery = ''
	}: {
		isOpen?: boolean;
		initialGameId?: number | null;
		initialQuery?: string;
	} = $props();

	// State
	let games = $state<GameSummary[]>([]);
	let loading = $state(true);
	let searching = $state(false);
	let searchResults = $state<SearchResult[]>([]);
	let selectedGameId = $state<number | null>(initialGameId);
	let selectedGame = $state<GameSummary | null>(null);
	let searchQuery = $state(initialQuery);
	let totalResults = $state(0);
	let error = $state<string | null>(null);
	let modalRef: HTMLDivElement | null = $state(null);
	let searchInputRef: HTMLElement | null = $state(null);

	// Event dispatcher
	const dispatch = createEventDispatcher<{
		close: void;
		resultSelect: { result: SearchResult; game: GameSummary };
		gameSelect: GameSummary;
	}>();

	// Load games when modal opens
	$effect(() => {
		if (isOpen && games.length === 0) {
			loadGames();
		}
	});

	// Focus search input when modal opens
	$effect(() => {
		if (isOpen && searchInputRef) {
			setTimeout(() => searchInputRef?.focus(), 100);
		}
	});

	// Update selected game when selectedGameId changes
	$effect(() => {
		if (selectedGameId && games.length > 0) {
			selectedGame = games.find((g) => g.id === selectedGameId) || null;
		}
	});

	// Handle escape key and backdrop clicks
	$effect(() => {
		if (!isOpen) return;

		function handleKeydown(e: KeyboardEvent) {
			if (e.key === 'Escape') {
				closeModal();
			}
		}

		function handleBackdropClick(e: MouseEvent) {
			if (modalRef && !modalRef.contains(e.target as Node)) {
				closeModal();
			}
		}

		document.addEventListener('keydown', handleKeydown);
		document.addEventListener('mousedown', handleBackdropClick);

		return () => {
			document.removeEventListener('keydown', handleKeydown);
			document.removeEventListener('mousedown', handleBackdropClick);
		};
	});

	async function loadGames() {
		loading = true;
		error = null;

		try {
			const result = await api.methods.listGames({
				query: { limit: 100 }
			});

			if (result.type === 'success') {
				games = result.data.items.filter((game) => game.hasRulesPdf);
				if (selectedGameId && !games.find((g) => g.id === selectedGameId)) {
					selectedGameId = null;
					selectedGame = null;
				}
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

	function selectGame(game: GameSummary) {
		selectedGameId = game.id;
		selectedGame = game;
		searchResults = [];
		totalResults = 0;
		dispatch('gameSelect', game);
	}

	async function performSearch() {
		if (!selectedGameId || !searchQuery.trim()) {
			return;
		}

		searching = true;
		error = null;

		try {
			const result = await api.methods.searchRules({
				query: {
					gameId: selectedGameId,
					query: searchQuery.trim(),
					limit: 8
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
		performSearch();
	}

	function handleResultClick(result: SearchResult) {
		if (selectedGame) {
			dispatch('resultSelect', { result, game: selectedGame });
		}
		closeModal();
	}

	function closeModal() {
		isOpen = false;
		dispatch('close');
	}

	function formatSimilarityScore(score: number): string {
		return (score * 100).toFixed(1) + '%';
	}

	function getSimilarityColor(score: number): string {
		if (score >= 0.8) return 'text-green-600';
		if (score >= 0.6) return 'text-yellow-600';
		return 'text-gray-600';
	}

	function truncateText(text: string, maxLength: number = 180): string {
		if (text.length <= maxLength) return text;
		return text.substring(0, maxLength) + '...';
	}
</script>

{#if isOpen}
	<!-- Modal Backdrop -->
	<div class="fixed inset-0 z-50 flex items-start justify-center bg-black/50 px-4 pt-16">
		<!-- Modal Content -->
		<div
			bind:this={modalRef}
			class="max-h-[80vh] w-full max-w-4xl overflow-hidden rounded-lg bg-white shadow-xl"
		>
			<!-- Modal Header -->
			<div class="border-b border-gray-200 px-6 py-4">
				<div class="flex items-center justify-between">
					<div>
						<h2 class="text-xl font-semibold text-gray-900">Search Game Rules</h2>
						<p class="mt-1 text-sm text-gray-600">
							Find specific information in your uploaded game rules
						</p>
					</div>
					<button
						onclick={closeModal}
						class="text-gray-400 transition-colors hover:text-gray-600"
						aria-label="Close search modal"
					>
						<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M6 18L18 6M6 6l12 12"
							></path>
						</svg>
					</button>
				</div>
			</div>

			<!-- Modal Body -->
			<div class="flex max-h-[calc(80vh-120px)] flex-col lg:flex-row">
				<!-- Game Selection Sidebar -->
				<div class="overflow-y-auto border-r border-gray-200 p-4 lg:w-1/3">
					<div class="mb-4">
						<Label class="text-sm font-medium text-gray-700">Select Game</Label>
						<p class="mt-1 text-xs text-gray-500">Choose which game's rules to search</p>
					</div>

					{#if loading}
						<div class="py-8">
							<LoadingSpinner text="Loading games..." class="justify-center" />
						</div>
					{:else if error && games.length === 0}
						<div class="py-4 text-center">
							<p class="text-sm text-red-600">{error}</p>
							<Button onclick={loadGames} class="mt-2" size="sm">Try Again</Button>
						</div>
					{:else if games.length === 0}
						<EmptyState
							icon="document"
							title="No games found"
							description="No games with uploaded rules found"
							actionText="Upload Rules First"
							size="sm"
							onAction={closeModal}
						/>
					{:else}
						<div class="space-y-2">
							{#each games as game (game.id)}
								<button
									onclick={() => selectGame(game)}
									class="w-full rounded-lg border p-3 text-left transition-colors hover:bg-gray-50
									{selectedGameId === game.id ? 'border-blue-500 bg-blue-50' : 'border-gray-200'}"
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
												<Badge variant="default" class="text-xs">Has PDF</Badge>
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
						</div>
					{/if}
				</div>

				<!-- Search Interface and Results -->
				<div class="overflow-y-auto p-4 lg:w-2/3">
					<!-- Search Form -->
					<div class="mb-6">
						<form onsubmit={handleSearchSubmit} class="space-y-4">
							<div>
								<Label for="searchQuery" class="text-sm font-medium text-gray-700">
									Search Query
									{#if selectedGame}
										<span class="font-normal text-gray-500">- {selectedGame.name}</span>
									{/if}
								</Label>
								<SearchInput
									bind:ref={searchInputRef}
									bind:value={searchQuery}
									placeholder="e.g. How do I win the game? What happens during combat?"
									disabled={!selectedGameId || searching}
									loading={searching}
									class="mt-1"
								/>
								<p class="mt-1 text-xs text-gray-500">
									Ask natural language questions or search for specific game concepts
								</p>
							</div>

							<div class="flex items-center justify-between">
								<Button
									type="submit"
									disabled={!selectedGameId || !searchQuery.trim() || searching}
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
										<svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
											></path>
										</svg>
										Search Rules
									{/if}
								</Button>

								{#if searchResults.length > 0}
									<span class="text-sm text-gray-600">
										{totalResults} result{totalResults === 1 ? '' : 's'} found
									</span>
								{/if}
							</div>
						</form>

						{#if error}
							<div class="mt-4 rounded-md border border-red-200 bg-red-50 p-3">
								<p class="text-sm text-red-700">{error}</p>
							</div>
						{/if}
					</div>

					<!-- Search Results -->
					{#if searchResults.length > 0}
						<div class="space-y-3">
							<h3 class="text-sm font-medium text-gray-700">Search Results</h3>
							{#each searchResults as result, index (result.chunkId)}
								<SearchResultComponent
									chunkId={result.chunkId}
									chunkText={result.chunkText}
									similarityScore={result.similarityScore}
									metadata={result.metadata}
									{index}
									on:click={() => handleResultClick(result)}
								/>
							{/each}
						</div>
					{:else if searchQuery && selectedGameId && !searching && searchResults.length === 0}
						<!-- No Results -->
						<EmptyState
							icon="search"
							title="No results found"
							description="No relevant passages found for '{searchQuery}' in {selectedGame?.name}."
							size="default"
						/>
					{:else if !selectedGameId && games.length > 0}
						<!-- No Game Selected -->
						<div class="py-8 text-center">
							<div class="mb-4 text-gray-400">
								<svg
									class="mx-auto h-16 w-16"
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
									></path>
								</svg>
							</div>
							<h3 class="mb-2 text-lg font-medium text-gray-900">Ready to Search</h3>
							<p class="text-gray-600">Select a game to start searching through its rules.</p>
						</div>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}
