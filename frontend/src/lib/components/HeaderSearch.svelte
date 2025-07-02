<script lang="ts">
	import { onMount } from 'svelte';
	import { searchStore, searchUtils, type SearchState } from '$lib/stores/search';
	import { Button } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';
	import QuickSearch from './QuickSearch.svelte';
	import SearchModal from './SearchModal.svelte';
	import type { Game, GameSummary, SearchResult } from '$lib';

	// Props
	let {
		currentGame = null,
		showQuickSearch = true,
		showSearchButton = true
	}: {
		currentGame?: Game | null;
		showQuickSearch?: boolean;
		showSearchButton?: boolean;
	} = $props();

	// State
	let showQuickSearchDropdown = $state(false);
	let isModalOpen = $state(false);
	let searchState = $state<SearchState>({
		isModalOpen: false,
		recentSearches: [],
		favoriteResults: [],
		currentGame: null
	});

	// Subscribe to search store
	searchStore.subscribe((state) => {
		searchState = state;
		isModalOpen = state.isModalOpen;
	});

	onMount(() => {
		// Load persisted search data
		searchUtils.loadPersistedData();

		// Set current game context if provided
		if (currentGame) {
			searchUtils.setCurrentGame(currentGame);
		}

		// Initialize keyboard shortcuts
		const cleanup = initKeyboardShortcuts();
		return cleanup;
	});

	function initKeyboardShortcuts() {
		function handleKeydown(event: KeyboardEvent) {
			// Cmd/Ctrl + K to open search modal
			if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
				event.preventDefault();
				openSearchModal();
			}
		}

		document.addEventListener('keydown', handleKeydown);
		return () => document.removeEventListener('keydown', handleKeydown);
	}

	function openSearchModal() {
		searchUtils.openModal();
	}

	function closeSearchModal() {
		searchUtils.closeModal();
	}

	function handleQuickSearchResult(event: CustomEvent<SearchResult>) {
		const result = event.detail;
		console.log('Quick search result selected:', result);
		// You can dispatch this up to parent or handle navigation here
	}

	function handleModalResultSelect(event: { result: SearchResult; game: GameSummary }) {
		console.log('Modal search result selected:', event.result, 'from game:', event.game);
		// Handle navigation to result or display in context
	}

	function handleQuickSearch(event: CustomEvent<{ query: string; results: SearchResult[] }>) {
		const { query, results } = event.detail;
		if (currentGame) {
			searchUtils.addToHistory(query, currentGame, results.length);
		}
	}

	function toggleQuickSearch() {
		showQuickSearchDropdown = !showQuickSearchDropdown;
	}

	// Format keyboard shortcut for display
	function getShortcutText(): string {
		if (typeof navigator !== 'undefined') {
			const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
			return isMac ? '⌘K' : 'Ctrl+K';
		}
		return 'Ctrl+K';
	}
</script>

<div class="header-search flex items-center space-x-3">
	<!-- Quick Search (for when on a specific game page) -->
	{#if showQuickSearch && currentGame}
		<div class="relative">
			<QuickSearch
				gameId={currentGame.id}
				gameName={currentGame.name}
				placeholder="Search {currentGame.name} rules..."
				maxResults={5}
				autoFocus={false}
				on:search={handleQuickSearch}
				on:resultSelect={handleQuickSearchResult}
			/>
		</div>
	{/if}

	<!-- Global Search Button -->
	{#if showSearchButton}
		<Button
			variant="outline"
			size="sm"
			onclick={openSearchModal}
			class="flex items-center space-x-2 text-sm"
		>
			<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
				></path>
			</svg>
			<span class="hidden sm:inline">Search Rules</span>
			<Badge variant="secondary" class="hidden px-1.5 py-0.5 text-xs lg:inline">
				{getShortcutText()}
			</Badge>
		</Button>
	{/if}

	<!-- Recent Searches Indicator -->
	{#if searchState.recentSearches.length > 0}
		<div class="relative">
			<button
				onclick={toggleQuickSearch}
				class="text-gray-400 transition-colors hover:text-gray-600"
				title="Recent searches"
				aria-label="View recent searches"
			>
				<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
					></path>
				</svg>
			</button>

			<!-- Recent Searches Dropdown -->
			{#if showQuickSearchDropdown}
				<div
					class="absolute top-full right-0 z-50 mt-2 w-80 rounded-lg border border-gray-200 bg-white shadow-lg"
				>
					<div class="border-b border-gray-200 p-3">
						<div class="flex items-center justify-between">
							<h3 class="text-sm font-medium text-gray-900">Recent Searches</h3>
							<button
								onclick={() => (showQuickSearchDropdown = false)}
								class="text-gray-400 hover:text-gray-600"
								aria-label="Close recent searches"
							>
								<svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
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

					<div class="max-h-64 overflow-y-auto">
						{#each searchState.recentSearches.slice(0, 8) as search (search.id)}
							<button
								onclick={() => {
									// Navigate to search with this query and game
									showQuickSearchDropdown = false;
									openSearchModal();
								}}
								class="w-full border-b border-gray-100 p-3 text-left transition-colors last:border-b-0 hover:bg-gray-50"
							>
								<div class="flex items-start justify-between">
									<div class="min-w-0 flex-1">
										<p class="truncate text-sm font-medium text-gray-900">
											{search.query}
										</p>
										<p class="mt-1 text-xs text-gray-500">
											{search.gameName} • {search.resultCount} result{search.resultCount === 1
												? ''
												: 's'}
										</p>
									</div>
									<div class="ml-2 text-xs text-gray-400">
										{search.timestamp.toLocaleDateString()}
									</div>
								</div>
							</button>
						{/each}

						{#if searchState.recentSearches.length === 0}
							<div class="p-6 text-center text-sm text-gray-500">No recent searches</div>
						{/if}
					</div>

					{#if searchState.recentSearches.length > 0}
						<div class="border-t border-gray-200 p-3">
							<div class="flex items-center justify-between">
								<Button variant="outline" size="sm" onclick={openSearchModal} class="text-xs">
									Advanced Search
								</Button>
								<button
									onclick={() => {
										searchUtils.clearHistory();
										showQuickSearchDropdown = false;
									}}
									class="text-xs text-gray-500 hover:text-gray-700"
								>
									Clear History
								</button>
							</div>
						</div>
					{/if}
				</div>
			{/if}
		</div>
	{/if}
</div>

<!-- Global Search Modal -->
<SearchModal
	bind:isOpen={isModalOpen}
	onClose={closeSearchModal}
	onResultSelect={handleModalResultSelect}
	onGameSelect={(e) => searchUtils.setCurrentGame(e as unknown as Game)}
/>

<style>
	.header-search {
		position: relative;
	}
</style>
