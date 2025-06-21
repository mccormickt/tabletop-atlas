<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { api, type Game, type RulesInfoResponse, type UploadResponse, formatDate } from '$lib';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui';
	import { Badge } from '$lib/components/ui';
	import PDFUpload from '$lib/components/PDFUpload.svelte';

	// Get game ID from URL parameters
	let gameId = $derived(parseInt($page.params.id));

	// State management
	let game = $state<Game | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let deleting = $state(false);
	let rulesInfo = $state<RulesInfoResponse | null>(null);
	let showUpload = $state(false);

	onMount(() => {
		if (gameId && !isNaN(gameId)) {
			loadGame();
		} else {
			error = 'Invalid game ID';
			loading = false;
		}
	});

	async function loadGame() {
		loading = true;
		error = null;

		try {
			const result = await api.methods.getGame({
				path: { id: gameId }
			});

			if (result.type === 'success') {
				game = result.data;
				// Also load rules info
				await loadRulesInfo();
			} else if (result.type === 'error') {
				error = result.data.message || 'Failed to load game details';
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Failed to load game details';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			loading = false;
		}
	}

	async function loadRulesInfo() {
		if (!gameId) return;

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

	async function handleDelete() {
		if (!game) return;

		if (!confirm(`Are you sure you want to delete "${game.name}"? This action cannot be undone.`)) {
			return;
		}

		deleting = true;

		try {
			const result = await api.methods.deleteGame({
				path: { id: game.id }
			});

			if (result.type === 'success') {
				// Navigate back to games list
				goto('/games');
			} else if (result.type === 'error') {
				alert(result.data.message || 'Failed to delete game');
			} else if (result.type === 'client_error') {
				alert(result.error.message || 'Failed to delete game');
			}
		} catch (err) {
			alert(err instanceof Error ? err.message : 'An unexpected error occurred');
		} finally {
			deleting = false;
		}
	}

	function handleEdit() {
		if (game) {
			goto(`/games/${game.id}/edit`);
		}
	}

	function handleBack() {
		goto('/games');
	}

	function handleUploadSuccess(event: CustomEvent<UploadResponse>) {
		// Refresh game and rules info
		loadGame();
		showUpload = false;
	}

	function handleUploadDeleted() {
		// Refresh game and rules info
		loadGame();
	}

	function toggleUpload() {
		showUpload = !showUpload;
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

	function formatPlayTime(minutes?: number | null): string {
		if (!minutes) return 'Not specified';
		if (minutes < 60) return `${minutes} minutes`;
		const hours = Math.floor(minutes / 60);
		const remainingMinutes = minutes % 60;
		if (remainingMinutes === 0) return `${hours} hour${hours === 1 ? '' : 's'}`;
		return `${hours}h ${remainingMinutes}m`;
	}

	function formatComplexity(rating?: number | null): string {
		if (!rating) return 'Not rated';
		const stars = '★'.repeat(Math.round(rating)) + '☆'.repeat(5 - Math.round(rating));
		return `${rating.toFixed(1)}/5.0 ${stars}`;
	}
</script>

<svelte:head>
	<title>{game ? `${game.name} - Tabletop Atlas` : 'Game Details - Tabletop Atlas'}</title>
	<meta name="description" content={game ? `Details for ${game.name}` : 'Board game details'} />
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
		<div class="mx-auto w-full max-w-4xl">
			<!-- Loading State -->
			{#if loading}
				<div class="flex items-center justify-center py-12">
					<div class="h-8 w-8 animate-spin rounded-full border-b-2 border-blue-600"></div>
					<span class="ml-2 text-gray-600">Loading game details...</span>
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
						<h3 class="mb-2 text-lg font-semibold text-gray-900">Unable to Load Game</h3>
						<p class="mb-4 text-gray-600">{error}</p>
						<div class="flex justify-center space-x-3">
							<Button onclick={loadGame}>Try Again</Button>
							<Button variant="outline" onclick={handleBack}>Go Back</Button>
						</div>
					</CardContent>
				</Card>
			{/if}

			<!-- Game Details -->
			{#if game && !loading && !error}
				<!-- Header -->
				<div class="mb-6">
					<div class="mb-4 flex items-center justify-between">
						<Button variant="ghost" onclick={handleBack} class="text-gray-600 hover:text-gray-900">
							← Back to Games
						</Button>
						<div class="flex space-x-3">
							<Button variant="outline" onclick={handleEdit}>Edit Game</Button>
							<Button variant="destructive" onclick={handleDelete} disabled={deleting}>
								{deleting ? 'Deleting...' : 'Delete Game'}
							</Button>
						</div>
					</div>

					<h1 class="mb-2 text-3xl font-bold text-gray-900">{game.name}</h1>
					{#if game.publisher}
						<p class="text-lg text-gray-600">Published by {game.publisher}</p>
					{/if}
				</div>

				<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
					<!-- Main Information -->
					<div class="space-y-6 lg:col-span-2">
						<!-- Description -->
						{#if game.description}
							<Card>
								<CardHeader>
									<CardTitle>Description</CardTitle>
								</CardHeader>
								<CardContent>
									<p class="whitespace-pre-wrap text-gray-700">{game.description}</p>
								</CardContent>
							</Card>
						{/if}

						<!-- Game Statistics -->
						<Card>
							<CardHeader>
								<CardTitle>Game Information</CardTitle>
							</CardHeader>
							<CardContent>
								<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
									<div class="space-y-3">
										<div>
											<dt class="text-sm font-medium text-gray-500">Year Published</dt>
											<dd class="text-sm text-gray-900">{game.yearPublished || 'Not specified'}</dd>
										</div>
										<div>
											<dt class="text-sm font-medium text-gray-500">Player Count</dt>
											<dd class="text-sm text-gray-900">
												{formatPlayerCount(game.minPlayers, game.maxPlayers)}
											</dd>
										</div>
										<div>
											<dt class="text-sm font-medium text-gray-500">Play Time</dt>
											<dd class="text-sm text-gray-900">{formatPlayTime(game.playTimeMinutes)}</dd>
										</div>
									</div>
									<div class="space-y-3">
										<div>
											<dt class="text-sm font-medium text-gray-500">Complexity Rating</dt>
											<dd class="text-sm text-gray-900">
												{formatComplexity(game.complexityRating)}
											</dd>
										</div>
										{#if game.bggId}
											<div>
												<dt class="text-sm font-medium text-gray-500">BoardGameGeek</dt>
												<dd class="text-sm text-gray-900">
													<a
														href="https://boardgamegeek.com/boardgame/{game.bggId}"
														target="_blank"
														rel="noopener noreferrer"
														class="text-blue-600 underline hover:text-blue-800"
													>
														View on BGG #{game.bggId}
													</a>
												</dd>
											</div>
										{/if}
										<div>
											<dt class="text-sm font-medium text-gray-500">Added to Library</dt>
											<dd class="text-sm text-gray-900">{formatDate(new Date(game.createdAt))}</dd>
										</div>
									</div>
								</div>
							</CardContent>
						</Card>

						<!-- Rules and Files -->
						<Card>
							<CardHeader>
								<CardTitle>Rules & Documentation</CardTitle>
							</CardHeader>
							<CardContent class="space-y-4">
								{#if rulesInfo?.hasRulesPdf}
									<div
										class="flex items-center justify-between rounded-md border border-green-200 bg-green-50 p-3"
									>
										<div class="flex items-center">
											<svg
												class="mr-2 h-5 w-5 text-green-600"
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
											<div>
												<span class="text-sm font-medium text-green-800">PDF Rules Available</span>
												{#if rulesInfo}
													<p class="text-xs text-green-600">
														{rulesInfo.chunkCount} chunks processed
														{#if rulesInfo.textLength}
															({rulesInfo.textLength.toLocaleString()} characters)
														{/if}
													</p>
												{/if}
											</div>
										</div>
										<div class="flex space-x-2">
											<Button size="sm" variant="outline" onclick={toggleUpload}>
												{showUpload ? 'Cancel' : 'Replace'}
											</Button>
											<Button size="sm" variant="outline">Search Rules</Button>
										</div>
									</div>
								{:else}
									<div
										class="flex items-center justify-between rounded-md border border-gray-200 bg-gray-50 p-3"
									>
										<div class="flex items-center">
											<svg
												class="mr-2 h-5 w-5 text-gray-400"
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
											<span class="text-sm text-gray-600">No PDF rules uploaded</span>
										</div>
										<Button size="sm" onclick={toggleUpload}>Upload Rules</Button>
									</div>
								{/if}

								{#if showUpload}
									<div class="mt-4">
										<PDFUpload
											gameId={game.id}
											gameName={game.name}
											existingRulesInfo={rulesInfo}
											on:uploaded={handleUploadSuccess}
											on:deleted={handleUploadDeleted}
											on:error={(e) => (error = e.detail)}
										/>
									</div>
								{/if}

								{#if game.rulesText && !showUpload}
									<div>
										<h3 class="mb-2 text-sm font-medium text-gray-500">Extracted Rules Text</h3>
										<div
											class="max-h-40 overflow-y-auto rounded-md border border-gray-200 bg-gray-50 p-3"
										>
											<p class="text-sm whitespace-pre-wrap text-gray-700">
												{game.rulesText.substring(0, 500)}{game.rulesText.length > 500 ? '...' : ''}
											</p>
										</div>
									</div>
								{/if}
							</CardContent>
						</Card>
					</div>

					<!-- Sidebar -->
					<div class="space-y-6">
						<!-- Quick Stats -->
						<Card>
							<CardHeader>
								<CardTitle>Quick Stats</CardTitle>
							</CardHeader>
							<CardContent class="space-y-3">
								<div class="flex items-center justify-between">
									<span class="text-sm text-gray-600">Last Updated</span>
									<span class="text-sm font-medium text-gray-900"
										>{formatDate(new Date(game.updatedAt))}</span
									>
								</div>
								<div class="flex items-center justify-between">
									<span class="text-sm text-gray-600">Has PDF Rules</span>
									<Badge variant={rulesInfo?.hasRulesPdf ? 'default' : 'secondary'}>
										{rulesInfo?.hasRulesPdf ? 'Yes' : 'No'}
									</Badge>
								</div>
								{#if rulesInfo?.hasRulesPdf}
									<div class="flex items-center justify-between">
										<span class="text-sm text-gray-600">Text Chunks</span>
										<span class="text-sm font-medium text-gray-900">{rulesInfo.chunkCount}</span>
									</div>
								{/if}
							</CardContent>
						</Card>

						<!-- Actions -->
						<Card>
							<CardHeader>
								<CardTitle>Actions</CardTitle>
							</CardHeader>
							<CardContent class="space-y-3">
								<Button class="w-full" variant="outline">View House Rules</Button>
								<Button class="w-full" variant="outline">Start Chat Session</Button>
								{#if !rulesInfo?.hasRulesPdf}
									<Button class="w-full" onclick={toggleUpload}>Upload Rules PDF</Button>
								{:else}
									<Button class="w-full" variant="outline">Search Rules</Button>
								{/if}
							</CardContent>
						</Card>

						<!-- Game ID -->
						<Card>
							<CardHeader>
								<CardTitle>Technical Info</CardTitle>
							</CardHeader>
							<CardContent class="space-y-2">
								<div>
									<dt class="text-xs font-medium tracking-wide text-gray-500 uppercase">Game ID</dt>
									<dd class="font-mono text-sm text-gray-900">{game.id}</dd>
								</div>
								{#if game.bggId}
									<div>
										<dt class="text-xs font-medium tracking-wide text-gray-500 uppercase">
											BGG ID
										</dt>
										<dd class="font-mono text-sm text-gray-900">{game.bggId}</dd>
									</div>
								{/if}
							</CardContent>
						</Card>
					</div>
				</div>
			{/if}
		</div>
	</main>
</div>
