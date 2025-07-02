<script lang="ts">
	import { api, type SearchResult } from '$lib';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui';
	import { Input, Label } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';

	// Props
	let {
		gameId,
		gameName = '',
		placeholder = 'Search game rules...',
		maxResults = 5,
		showResultsInline = true,
		compact = false,
		onSearch = () => {},
		onError = () => {},
		onResultClick = () => {}
	}: {
		gameId: number;
		gameName?: string;
		placeholder?: string;
		maxResults?: number;
		showResultsInline?: boolean;
		compact?: boolean;
		onSearch?: (data: { query: string; results: SearchResult[]; totalResults: number }) => void;
		onError?: (error: string) => void;
		onResultClick?: (result: SearchResult) => void;
	} = $props();

	// State
	let searching = $state(false);
	let searchQuery = $state('');
	let searchResults = $state<SearchResult[]>([]);
	let totalResults = $state(0);
	let error = $state<string | null>(null);

	async function performSearch() {
		if (!gameId || !searchQuery.trim()) {
			return;
		}

		searching = true;
		error = null;

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
				totalResults = result.data.totalResults;
				onSearch({
					query: searchQuery.trim(),
					results: searchResults,
					totalResults
				});
			} else if (result.type === 'error') {
				error = result.data.message || 'Search failed';
				searchResults = [];
				totalResults = 0;
				onError(error);
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Search failed';
				searchResults = [];
				totalResults = 0;
				onError(error);
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			searchResults = [];
			totalResults = 0;
			onError(error);
		} finally {
			searching = false;
		}
	}

	function handleSearchSubmit(event: Event) {
		event.preventDefault();
		performSearch();
	}

	function handleResultClick(result: SearchResult) {
		onResultClick(result);
	}

	function formatSimilarityScore(score: number): string {
		return (score * 100).toFixed(1) + '%';
	}

	function getSimilarityColor(score: number): string {
		if (score >= 0.8) return 'text-green-600';
		if (score >= 0.6) return 'text-yellow-600';
		return 'text-gray-600';
	}

	function truncateText(text: string, maxLength: number = 150): string {
		if (text.length <= maxLength) return text;
		return text.substring(0, maxLength) + '...';
	}

	// Clear results when gameId changes
	$effect(() => {
		searchResults = [];
		totalResults = 0;
		error = null;
		searchQuery = '';
	});
</script>

<div class="rules-search {compact ? 'compact' : ''}">
	<!-- Search Form -->
	<Card class={compact ? 'shadow-sm' : ''}>
		<CardHeader class={compact ? 'pb-3' : ''}>
			<CardTitle class={compact ? 'text-base' : ''}>
				Search Rules
				{#if gameName}
					<span class="text-sm font-normal text-gray-600">- {gameName}</span>
				{/if}
			</CardTitle>
			{#if !compact}
				<CardDescription>
					Ask natural language questions or search for specific game concepts
				</CardDescription>
			{/if}
		</CardHeader>
		<CardContent class={compact ? 'pt-0' : ''}>
			<form onsubmit={handleSearchSubmit} class="space-y-4">
				<div>
					{#if !compact}
						<Label for="searchQuery">Search Query</Label>
					{/if}
					<Input
						id="searchQuery"
						bind:value={searchQuery}
						{placeholder}
						disabled={searching}
						class={compact ? 'text-sm' : 'mt-1'}
					/>
					{#if !compact}
						<p class="mt-1 text-xs text-gray-500">
							Ask questions like "How do I win?" or "What happens during combat?"
						</p>
					{/if}
				</div>

				<div class="flex items-center justify-between">
					<Button
						type="submit"
						disabled={!searchQuery.trim() || searching}
						size={compact ? 'sm' : 'default'}
						class="flex items-center"
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
							Search
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
		</CardContent>
	</Card>

	<!-- Search Results -->
	{#if showResultsInline && searchResults.length > 0}
		<Card class="mt-6">
			<CardHeader>
				<CardTitle class={compact ? 'text-base' : ''}>Search Results</CardTitle>
				<CardDescription>
					Found {totalResults} relevant passage{totalResults === 1 ? '' : 's'}
					{#if searchQuery}for "{searchQuery}"{/if}
				</CardDescription>
			</CardHeader>
			<CardContent class="space-y-4">
				{#each searchResults as result, index (result.chunkId)}
					<button
						onclick={() => handleResultClick(result)}
						class="w-full space-y-3 rounded-lg border border-gray-200 p-4 text-left transition-colors hover:bg-gray-50"
					>
						<div class="flex items-start justify-between">
							<div class="flex items-center space-x-2">
								<Badge variant="outline" class="text-xs">
									#{index + 1}
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
							<p class="text-sm leading-relaxed text-gray-900">
								{compact ? truncateText(result.chunkText, 100) : result.chunkText}
							</p>
						</div>

						{#if !compact}
							<div class="flex items-center justify-between text-xs text-gray-500">
								<span>Chunk ID: {result.chunkId}</span>
								<span>Similarity: {result.similarityScore.toFixed(3)}</span>
							</div>
						{/if}
					</button>
				{/each}
			</CardContent>
		</Card>
	{:else if showResultsInline && searchQuery && !searching && searchResults.length === 0}
		<!-- No Results -->
		<Card class="mt-6">
			<CardContent class="p-6 text-center">
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
					No relevant passages found for "{searchQuery}"{#if gameName}
						in {gameName}{/if}.
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
	{/if}
</div>

<style>
	.rules-search.compact .prose p {
		font-size: 0.875rem;
		line-height: 1.5;
	}
</style>
