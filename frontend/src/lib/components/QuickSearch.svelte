<script lang="ts">
	import { api, type SearchResult } from '$lib';
	import { SearchInput, SearchResult as SearchResultComponent } from '$lib/components/ui';
	import { createEventDispatcher } from 'svelte';

	// Props
	let {
		gameId,
		gameName = '',
		placeholder = 'Quick search...',
		maxResults = 3,
		autoFocus = false
	}: {
		gameId: number;
		gameName?: string;
		placeholder?: string;
		maxResults?: number;
		autoFocus?: boolean;
	} = $props();

	// State
	let searching = $state(false);
	let searchQuery = $state('');
	let searchResults = $state<SearchResult[]>([]);
	let showResults = $state(false);
	let inputRef: HTMLInputElement | null = $state(null);

	// Event dispatcher
	const dispatch = createEventDispatcher<{
		search: { query: string; results: SearchResult[] };
		resultSelect: SearchResult;
		clear: void;
	}>();

	// Auto-focus input if requested
	$effect(() => {
		if (autoFocus && inputRef) {
			inputRef.focus();
		}
	});

	async function performSearch() {
		if (!gameId || !searchQuery.trim()) {
			searchResults = [];
			showResults = false;
			return;
		}

		searching = true;

		try {
			const result = await api.methods.searchRules({
				query: {
					gameId,
					query: searchQuery.trim(),
					limit: maxResults
				}
			});

			if (result.type === 'success') {
				searchResults = result.data.results;
				showResults = searchResults.length > 0;
				dispatch('search', {
					query: searchQuery.trim(),
					results: searchResults
				});
			} else {
				searchResults = [];
				showResults = false;
			}
		} catch (err) {
			searchResults = [];
			showResults = false;
		} finally {
			searching = false;
		}
	}

	function handleInput() {
		if (searchQuery.trim()) {
			performSearch();
		} else {
			searchResults = [];
			showResults = false;
		}
	}

	function handleResultClick(result: SearchResult) {
		dispatch('resultSelect', result);
		clearSearch();
	}

	function clearSearch() {
		searchQuery = '';
		searchResults = [];
		showResults = false;
		dispatch('clear');
	}

	// Close results when clicking outside
	function handleClickOutside(event: MouseEvent) {
		const target = event.target as Element;
		if (!target.closest('.quick-search-container')) {
			showResults = false;
		}
	}

	$effect(() => {
		document.addEventListener('click', handleClickOutside);
		return () => {
			document.removeEventListener('click', handleClickOutside);
		};
	});
</script>

<div class="quick-search-container relative">
	<div class="flex items-center space-x-2">
		<div class="relative flex-1">
			<SearchInput
				bind:ref={inputRef}
				bind:value={searchQuery}
				{placeholder}
				loading={searching}
				disabled={searching}
				onClear={clearSearch}
				on:input={handleInput}
				autocomplete="off"
			/>
		</div>

		{#if gameName}
			<div class="text-xs whitespace-nowrap text-gray-500">
				{gameName}
			</div>
		{/if}
	</div>

	<!-- Search Results Dropdown -->
	{#if showResults}
		<div
			class="absolute top-full right-0 left-0 z-50 mt-1 max-h-96 overflow-auto rounded-md border border-gray-200 bg-white shadow-lg"
		>
			{#if searchResults.length > 0}
				<div class="p-2">
					<div class="mb-2 text-xs font-medium text-gray-500">
						{searchResults.length} result{searchResults.length === 1 ? '' : 's'} found
					</div>

					{#each searchResults as result, index (result.chunkId)}
						<div class="border-b border-gray-100 last:border-b-0">
							<SearchResultComponent
								chunkId={String(result.chunkId)}
								chunkText={result.chunkText}
								similarityScore={result.similarityScore}
								metadata={result.metadata}
								{index}
								variant="compact"
								maxTextLength={120}
								showIndex={false}
								class="rounded-none border-0 hover:bg-gray-50"
								on:click={() => handleResultClick(result)}
							/>
						</div>
					{/each}
				</div>
			{:else}
				<div class="p-4 text-center text-sm text-gray-500">
					No results found for "{searchQuery}"
				</div>
			{/if}
		</div>
	{/if}
</div>

<style>
	.quick-search-container {
		min-width: 280px;
	}
</style>
