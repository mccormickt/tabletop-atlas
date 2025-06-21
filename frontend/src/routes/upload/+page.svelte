<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		api,
		type Game,
		type GameSummary,
		type RulesInfoResponse,
		type UploadResponse
	} from '$lib';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';
	import PDFUpload from '$lib/components/PDFUpload.svelte';
	import { useHeader } from '$lib/stores/header';

	// Configure header for this page
	const header = useHeader();
	header.configure({
		showSearch: true,
		currentGame: null
	});

	// State management
	let games = $state<GameSummary[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let selectedGameId = $state<number | null>(null);
	let selectedGame = $state<GameSummary | null>(null);
	let rulesInfo = $state<RulesInfoResponse | null>(null);
	let uploadSuccess = $state(false);

	onMount(() => {
		loadGames();
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

	async function selectGame(gameId: number) {
		selectedGameId = gameId;
		selectedGame = games.find((g) => g.id === gameId) || null;
		uploadSuccess = false;

		// Load existing rules info for this game
		try {
			const result = await api.methods.getRulesInfo({
				path: { id: gameId }
			});

			if (result.type === 'success') {
				rulesInfo = result.data;
			} else {
				rulesInfo = null;
			}
		} catch {
			rulesInfo = null;
		}
	}

	function handleUploadSuccess(event: CustomEvent<UploadResponse>) {
		uploadSuccess = true;
		// Refresh rules info
		if (selectedGameId) {
			selectGame(selectedGameId);
		}
	}

	function handleUploadDeleted() {
		// Refresh rules info
		if (selectedGameId) {
			selectGame(selectedGameId);
		}
	}

	function handleUploadError(event: CustomEvent<string>) {
		error = event.detail;
	}

	function goToGame(gameId: number) {
		goto(`/games/${gameId}`);
	}

	function formatPlayerCount(min?: number | null, max?: number | null): string {
		if (!min && !max) return 'Not specified';
		if (min && max) {
			return min === max ? `${min} player${min === 1 ? '' : 's'}` : `${min}-${max} players`;
		}
		if (min) return `${min}+ players`;
		if (max) return `Up to ${max} players`;
		return 'Not specified';
	}
</script>

<svelte:head>
	<title>Upload Rules - Tabletop Atlas</title>
	<meta name="description" content="Upload PDF rule books for your board games" />
</svelte:head>

<!-- Main Content -->
<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Header -->
	<div class="mb-8">
		<h1 class="text-3xl font-bold text-gray-900">Upload Game Rules</h1>
		<p class="mt-2 text-gray-600">
			Upload PDF rule books to enable AI-powered search and question answering
		</p>
	</div>

	<div class="grid grid-cols-1 gap-8 lg:grid-cols-3">
		<!-- Game Selection -->
		<div class="lg:col-span-1">
			<Card>
				<CardHeader>
					<CardTitle>Select a Game</CardTitle>
					<CardDescription>Choose which game you want to upload rules for</CardDescription>
				</CardHeader>
				<CardContent>
					{#if loading}
						<div class="flex items-center justify-center py-8">
							<div class="h-6 w-6 animate-spin rounded-full border-b-2 border-blue-600"></div>
							<span class="ml-2 text-sm text-gray-600">Loading games...</span>
						</div>
					{:else if error}
						<div class="text-center">
							<p class="text-red-600">{error}</p>
							<Button onclick={loadGames} class="mt-4">Try Again</Button>
						</div>
					{:else if games.length === 0}
						<div class="text-center">
							<p class="text-gray-600">No games found. Add a game first.</p>
							<Button href="/games/add" class="mt-4">Add Game</Button>
						</div>
					{:else}
						<div class="max-h-96 space-y-2 overflow-y-auto">
							{#each games as game (game.id)}
								<button
									onclick={() => selectGame(game.id)}
									class="w-full rounded-lg border p-3 text-left transition-colors hover:bg-gray-50
										{selectedGameId === game.id ? 'border-blue-500 bg-blue-50' : 'border-gray-200'}"
								>
									<div class="flex items-start justify-between">
										<div class="min-w-0 flex-1">
											<h3 class="truncate font-medium text-gray-900">{game.name}</h3>
											{#if game.publisher}
												<p class="truncate text-sm text-gray-500">{game.publisher}</p>
											{/if}
											<div class="mt-1 flex items-center space-x-2">
												{#if game.yearPublished}
													<Badge variant="secondary" class="text-xs">{game.yearPublished}</Badge>
												{/if}
												{#if game.hasRulesPdf}
													<Badge variant="default" class="text-xs">Has PDF</Badge>
												{/if}
											</div>
										</div>
										{#if selectedGameId === game.id}
											<svg class="h-5 w-5 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
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
				</CardContent>
			</Card>

			{#if selectedGame}
				<!-- Selected Game Info -->
				<Card class="mt-6">
					<CardHeader>
						<CardTitle class="text-lg">{selectedGame.name}</CardTitle>
						{#if selectedGame.publisher}
							<CardDescription>{selectedGame.publisher}</CardDescription>
						{/if}
					</CardHeader>
					<CardContent class="space-y-3">
						{#if selectedGame.yearPublished}
							<div>
								<span class="text-sm font-medium text-gray-500">Year:</span>
								<span class="text-sm text-gray-900">{selectedGame.yearPublished}</span>
							</div>
						{/if}
						<div>
							<span class="text-sm font-medium text-gray-500">Players:</span>
							<span class="text-sm text-gray-900">
								{formatPlayerCount(selectedGame.minPlayers, selectedGame.maxPlayers)}
							</span>
						</div>
						{#if selectedGame.complexityRating}
							<div>
								<span class="text-sm font-medium text-gray-500">Complexity:</span>
								<span class="text-sm text-gray-900">{selectedGame.complexityRating}/5.0</span>
							</div>
						{/if}
						<div class="pt-3">
							<Button
								variant="outline"
								onclick={() => selectedGame && goToGame(selectedGame.id)}
								class="w-full"
							>
								View Game Details
							</Button>
						</div>
					</CardContent>
				</Card>
			{/if}
		</div>

		<!-- Upload Section -->
		<div class="lg:col-span-2">
			{#if selectedGame}
				<PDFUpload
					gameId={selectedGame.id}
					gameName={selectedGame.name}
					existingRulesInfo={rulesInfo}
					on:uploaded={handleUploadSuccess}
					on:deleted={handleUploadDeleted}
					on:error={handleUploadError}
				/>

				{#if uploadSuccess}
					<Card class="mt-6">
						<CardHeader>
							<CardTitle class="text-green-800">Upload Complete!</CardTitle>
							<CardDescription>Your PDF has been successfully processed and indexed</CardDescription
							>
						</CardHeader>
						<CardContent class="space-y-4">
							<div class="flex items-center space-x-4">
								<svg
									class="h-8 w-8 text-green-600"
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
									></path>
								</svg>
								<div>
									<h3 class="font-medium text-gray-900">Ready for AI Chat</h3>
									<p class="text-sm text-gray-600">
										You can now ask questions about the rules using our AI chat interface
									</p>
								</div>
							</div>
							<div class="flex space-x-3">
								<Button onclick={() => selectedGame && goToGame(selectedGame.id)}
									>View Game Details</Button
								>
								<Button variant="outline">Start Chat Session</Button>
							</div>
						</CardContent>
					</Card>
				{/if}
			{:else}
				<!-- No Game Selected -->
				<Card class="h-96">
					<CardContent class="flex h-full items-center justify-center">
						<div class="text-center">
							<svg
								class="mx-auto h-16 w-16 text-gray-400"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
								></path>
							</svg>
							<h3 class="mt-4 text-lg font-medium text-gray-900">Select a Game</h3>
							<p class="mt-2 text-gray-600">Choose a game from the list to upload its rules</p>
						</div>
					</CardContent>
				</Card>
			{/if}
		</div>
	</div>

	<!-- Additional Features -->
	<div class="mt-12">
		<h2 class="mb-4 text-xl font-semibold text-gray-900">What happens after upload?</h2>
		<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
			<Card>
				<CardContent class="p-6">
					<div class="flex items-center">
						<div class="flex-shrink-0">
							<svg
								class="h-8 w-8 text-blue-600"
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
						<div class="ml-4">
							<h3 class="text-lg font-medium text-gray-900">Text Extraction</h3>
							<p class="text-sm text-gray-600">
								We extract and index all text content from your PDF for searching
							</p>
						</div>
					</div>
				</CardContent>
			</Card>

			<Card>
				<CardContent class="p-6">
					<div class="flex items-center">
						<div class="flex-shrink-0">
							<svg
								class="h-8 w-8 text-green-600"
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
						<div class="ml-4">
							<h3 class="text-lg font-medium text-gray-900">AI Processing</h3>
							<p class="text-sm text-gray-600">
								Content is processed for semantic search and question answering
							</p>
						</div>
					</div>
				</CardContent>
			</Card>

			<Card>
				<CardContent class="p-6">
					<div class="flex items-center">
						<div class="flex-shrink-0">
							<svg
								class="h-8 w-8 text-purple-600"
								fill="none"
								stroke="currentColor"
								viewBox="0 0 24 24"
							>
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
								></path>
							</svg>
						</div>
						<div class="ml-4">
							<h3 class="text-lg font-medium text-gray-900">Chat Ready</h3>
							<p class="text-sm text-gray-600">
								Ask questions about rules and get instant, accurate answers
							</p>
						</div>
					</div>
				</CardContent>
			</Card>
		</div>
	</div>
</main>
