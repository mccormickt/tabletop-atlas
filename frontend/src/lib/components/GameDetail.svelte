<script lang="ts">
	import { onMount } from 'svelte';
	import { api, type Game, formatDate, formatDateTime } from '$lib';
	import Button from './ui/Button.svelte';
	import Card from './ui/Card.svelte';
	import Badge from './ui/Badge.svelte';
	import { createEventDispatcher } from 'svelte';

	interface Props {
		gameId: number;
		onEdit?: (game: Game) => void;
		onDelete?: (game: Game) => void;
		onBack?: () => void;
	}

	let { gameId, onEdit, onDelete, onBack }: Props = $props();

	const dispatch = createEventDispatcher<{
		edit: Game;
		delete: Game;
		back: void;
	}>();

	// State management
	let game = $state<Game | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let deleting = $state(false);

	onMount(() => {
		loadGame();
	});

	async function loadGame() {
		loading = true;
		error = null;

		try {
			const result = await api.methods.getGame({
				path: { id: gameId }
			});

			if (result.success) {
				game = result.data;
			} else {
				error = result.error?.message || 'Failed to load game details';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			loading = false;
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

			if (result.success) {
				onDelete?.(game);
				dispatch('delete', game);
			} else {
				alert(result.error?.message || 'Failed to delete game');
			}
		} catch (err) {
			alert(err instanceof Error ? err.message : 'An unexpected error occurred');
		} finally {
			deleting = false;
		}
	}

	function handleEdit() {
		if (game) {
			onEdit?.(game);
			dispatch('edit', game);
		}
	}

	function handleBack() {
		onBack?.();
		dispatch('back');
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

<div class="w-full max-w-4xl mx-auto">
	<!-- Loading State -->
	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
			<span class="ml-2 text-gray-600">Loading game details...</span>
		</div>
	{/if}

	<!-- Error State -->
	{#if error && !loading}
		<Card class="p-6 text-center">
			<div class="text-red-600 mb-4">
				<svg class="mx-auto h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
				</svg>
			</div>
			<h3 class="text-lg font-semibold text-gray-900 mb-2">Unable to Load Game</h3>
			<p class="text-gray-600 mb-4">{error}</p>
			<div class="flex justify-center space-x-3">
				<Button onclick={loadGame}>
					Try Again
				</Button>
				<Button variant="outline" onclick={handleBack}>
					Go Back
				</Button>
			</div>
		</Card>
	{/if}

	<!-- Game Details -->
	{#if game && !loading && !error}
		<!-- Header -->
		<div class="mb-6">
			<div class="flex items-center justify-between mb-4">
				<Button variant="ghost" onclick={handleBack} class="text-gray-600 hover:text-gray-900">
					← Back to Games
				</Button>
				<div class="flex space-x-3">
					<Button variant="outline" onclick={handleEdit}>
						Edit Game
					</Button>
					<Button
						variant="destructive"
						onclick={handleDelete}
						disabled={deleting}
					>
						{deleting ? 'Deleting...' : 'Delete Game'}
					</Button>
				</div>
			</div>

			<h1 class="text-3xl font-bold text-gray-900 mb-2">{game.name}</h1>
			{#if game.publisher}
				<p class="text-lg text-gray-600">Published by {game.publisher}</p>
			{/if}
		</div>

		<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
			<!-- Main Information -->
			<div class="lg:col-span-2 space-y-6">
				<!-- Description -->
				{#if game.description}
					<Card class="p-6">
						<h2 class="text-xl font-semibold text-gray-900 mb-3">Description</h2>
						<p class="text-gray-700 whitespace-pre-wrap">{game.description}</p>
					</Card>
				{/if}

				<!-- Game Statistics -->
				<Card class="p-6">
					<h2 class="text-xl font-semibold text-gray-900 mb-4">Game Information</h2>
					<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
						<div class="space-y-3">
							<div>
								<dt class="text-sm font-medium text-gray-500">Year Published</dt>
								<dd class="text-sm text-gray-900">{game.yearPublished || 'Not specified'}</dd>
							</div>
							<div>
								<dt class="text-sm font-medium text-gray-500">Player Count</dt>
								<dd class="text-sm text-gray-900">{formatPlayerCount(game.minPlayers, game.maxPlayers)}</dd>
							</div>
							<div>
								<dt class="text-sm font-medium text-gray-500">Play Time</dt>
								<dd class="text-sm text-gray-900">{formatPlayTime(game.playTimeMinutes)}</dd>
							</div>
						</div>
						<div class="space-y-3">
							<div>
								<dt class="text-sm font-medium text-gray-500">Complexity Rating</dt>
								<dd class="text-sm text-gray-900">{formatComplexity(game.complexityRating)}</dd>
							</div>
							{#if game.bggId}
								<div>
									<dt class="text-sm font-medium text-gray-500">BoardGameGeek</dt>
									<dd class="text-sm text-gray-900">
										<a
											href="https://boardgamegeek.com/boardgame/{game.bggId}"
											target="_blank"
											rel="noopener noreferrer"
											class="text-blue-600 hover:text-blue-800 underline"
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
				</Card>

				<!-- Rules and Files -->
				<Card class="p-6">
					<h2 class="text-xl font-semibold text-gray-900 mb-4">Rules & Documentation</h2>
					<div class="space-y-4">
						{#if game.rulesPdfPath}
							<div class="flex items-center justify-between p-3 bg-green-50 border border-green-200 rounded-md">
								<div class="flex items-center">
									<svg class="h-5 w-5 text-green-600 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
									</svg>
									<span class="text-sm font-medium text-green-800">PDF Rules Available</span>
								</div>
								<Button size="sm" variant="outline">
									Download Rules
								</Button>
							</div>
						{:else}
							<div class="flex items-center justify-between p-3 bg-gray-50 border border-gray-200 rounded-md">
								<div class="flex items-center">
									<svg class="h-5 w-5 text-gray-400 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
									</svg>
									<span class="text-sm text-gray-600">No PDF rules uploaded</span>
								</div>
								<Button size="sm">
									Upload Rules
								</Button>
							</div>
						{/if}

						{#if game.rulesText}
							<div>
								<h3 class="text-sm font-medium text-gray-500 mb-2">Extracted Rules Text</h3>
								<div class="p-3 bg-gray-50 border border-gray-200 rounded-md max-h-40 overflow-y-auto">
									<p class="text-sm text-gray-700 whitespace-pre-wrap">{game.rulesText.substring(0, 500)}{game.rulesText.length > 500 ? '...' : ''}</p>
								</div>
							</div>
						{/if}
					</div>
				</Card>
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Quick Stats -->
				<Card class="p-6">
					<h2 class="text-xl font-semibold text-gray-900 mb-4">Quick Stats</h2>
					<div class="space-y-3">
						<div class="flex items-center justify-between">
							<span class="text-sm text-gray-600">Last Updated</span>
							<span class="text-sm font-medium text-gray-900">{formatDate(new Date(game.updatedAt))}</span>
						</div>
						<div class="flex items-center justify-between">
							<span class="text-sm text-gray-600">Has PDF Rules</span>
							<Badge variant={game.rulesPdfPath ? "default" : "secondary"}>
								{game.rulesPdfPath ? "Yes" : "No"}
							</Badge>
						</div>
					</div>
				</Card>

				<!-- Actions -->
				<Card class="p-6">
					<h2 class="text-xl font-semibold text-gray-900 mb-4">Actions</h2>
					<div class="space-y-3">
						<Button class="w-full" variant="outline">
							View House Rules
						</Button>
						<Button class="w-full" variant="outline">
							Start Chat Session
						</Button>
						{#if !game.rulesPdfPath}
							<Button class="w-full">
								Upload Rules PDF
							</Button>
						{/if}
					</div>
				</Card>

				<!-- Game ID -->
				<Card class="p-6">
					<h2 class="text-xl font-semibold text-gray-900 mb-4">Technical Info</h2>
					<div class="space-y-2">
						<div>
							<dt class="text-xs font-medium text-gray-500 uppercase tracking-wide">Game ID</dt>
							<dd class="text-sm font-mono text-gray-900">{game.id}</dd>
						</div>
						{#if game.bggId}
							<div>
								<dt class="text-xs font-medium text-gray-500 uppercase tracking-wide">BGG ID</dt>
								<dd class="text-sm font-mono text-gray-900">{game.bggId}</dd>
							</div>
						{/if}
					</div>
				</Card>
			</div>
		</div>
	{/if}
</div>
