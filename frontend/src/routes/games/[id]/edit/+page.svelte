<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { api, type Game, type UpdateGameRequest } from '$lib';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui';
	import { Input, Label, Textarea } from '$lib/components/ui';
	import { useHeader } from '$lib/stores/header';

	// Configure header for this page
	const header = useHeader();
	header.configure({
		showSearch: true,
		currentGame: null
	});

	// Get game ID from URL parameters
	let gameId = $derived(parseInt(page.params.id));

	// State management
	let game = $state<Game | null>(null);
	let loading = $state(true);
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);
	let success = $state(false);

	// Form data
	let formData = $state({
		name: '',
		description: '',
		publisher: '',
		yearPublished: '',
		minPlayers: '',
		maxPlayers: '',
		playTimeMinutes: '',
		complexityRating: '',
		bggId: ''
	});

	// Form validation
	let errors = $state({
		name: '',
		yearPublished: '',
		minPlayers: '',
		maxPlayers: '',
		playTimeMinutes: '',
		complexityRating: '',
		bggId: ''
	});

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
				// Populate form data with existing game data
				formData = {
					name: game.name || '',
					description: game.description || '',
					publisher: game.publisher || '',
					yearPublished: game.yearPublished?.toString() || '',
					minPlayers: game.minPlayers?.toString() || '',
					maxPlayers: game.maxPlayers?.toString() || '',
					playTimeMinutes: game.playTimeMinutes?.toString() || '',
					complexityRating: game.complexityRating?.toString() || '',
					bggId: game.bggId?.toString() || ''
				};
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

	function validateForm(): boolean {
		// Reset errors
		errors = {
			name: '',
			yearPublished: '',
			minPlayers: '',
			maxPlayers: '',
			playTimeMinutes: '',
			complexityRating: '',
			bggId: ''
		};

		let isValid = true;

		// Required field validation
		if (!formData.name.trim()) {
			errors.name = 'Game name is required';
			isValid = false;
		}

		// Numeric field validation
		if (
			formData.yearPublished &&
			(isNaN(+formData.yearPublished) ||
				+formData.yearPublished < 1800 ||
				+formData.yearPublished > new Date().getFullYear() + 5)
		) {
			errors.yearPublished =
				'Please enter a valid year between 1800 and ' + (new Date().getFullYear() + 5);
			isValid = false;
		}

		if (formData.minPlayers && (isNaN(+formData.minPlayers) || +formData.minPlayers < 1)) {
			errors.minPlayers = 'Minimum players must be at least 1';
			isValid = false;
		}

		if (formData.maxPlayers && (isNaN(+formData.maxPlayers) || +formData.maxPlayers < 1)) {
			errors.maxPlayers = 'Maximum players must be at least 1';
			isValid = false;
		}

		if (formData.minPlayers && formData.maxPlayers && +formData.minPlayers > +formData.maxPlayers) {
			errors.maxPlayers = 'Maximum players must be greater than or equal to minimum players';
			isValid = false;
		}

		if (
			formData.playTimeMinutes &&
			(isNaN(+formData.playTimeMinutes) || +formData.playTimeMinutes < 1)
		) {
			errors.playTimeMinutes = 'Play time must be at least 1 minute';
			isValid = false;
		}

		if (
			formData.complexityRating &&
			(isNaN(+formData.complexityRating) ||
				+formData.complexityRating < 1 ||
				+formData.complexityRating > 5)
		) {
			errors.complexityRating = 'Complexity rating must be between 1.0 and 5.0';
			isValid = false;
		}

		if (formData.bggId && (isNaN(+formData.bggId) || +formData.bggId < 1)) {
			errors.bggId = 'BoardGameGeek ID must be a positive number';
			isValid = false;
		}

		return isValid;
	}

	function buildRequestData(): UpdateGameRequest {
		const data: UpdateGameRequest = {};

		// Only include fields that have changed or are not empty
		if (formData.name.trim()) data.name = formData.name.trim();
		if (formData.description.trim()) data.description = formData.description.trim();
		if (formData.publisher.trim()) data.publisher = formData.publisher.trim();
		if (formData.yearPublished) data.yearPublished = +formData.yearPublished;
		if (formData.minPlayers) data.minPlayers = +formData.minPlayers;
		if (formData.maxPlayers) data.maxPlayers = +formData.maxPlayers;
		if (formData.playTimeMinutes) data.playTimeMinutes = +formData.playTimeMinutes;
		if (formData.complexityRating) data.complexityRating = +formData.complexityRating;
		if (formData.bggId) data.bggId = +formData.bggId;

		return data;
	}

	async function handleSubmit(event: Event) {
		event.preventDefault();

		if (!validateForm()) return;

		isSubmitting = true;
		error = null;
		success = false;

		try {
			const requestData = buildRequestData();
			const result = await api.methods.updateGame({
				path: { id: gameId },
				body: requestData
			});

			if (result.type === 'success') {
				success = true;
				// Navigate to the game detail page after a short delay
				setTimeout(() => {
					goto(`/games/${gameId}`);
				}, 1500);
			} else if (result.type === 'error') {
				error = result.data.message || 'An error occurred while updating the game';
			} else if (result.type === 'client_error') {
				error = result.error.message || 'An error occurred while updating the game';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			isSubmitting = false;
		}
	}

	function handleCancel() {
		goto(`/games/${gameId}`);
	}
</script>

<svelte:head>
	<title>{game ? `Edit ${game.name} - Tabletop Atlas` : 'Edit Game - Tabletop Atlas'}</title>
	<meta name="description" content="Edit board game details" />
</svelte:head>

<!-- Main Content -->
<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Back Navigation -->
	<div class="mb-6">
		<Button
			variant="ghost"
			onclick={handleCancel}
			class="text-muted-foreground hover:text-foreground"
		>
			← Back to Game Details
		</Button>
	</div>

	<!-- Loading State -->
	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="h-8 w-8 animate-spin rounded-full border-b-2 border-blue-600"></div>
			<span class="ml-2 text-gray-600">Loading game details...</span>
		</div>
	{/if}

	<!-- Error State (Loading Error) -->
	{#if error && loading}
		<Card class="mx-auto w-full max-w-2xl text-center">
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
					<Button variant="outline" onclick={() => goto('/games')}>Go Back</Button>
				</div>
			</CardContent>
		</Card>
	{/if}

	<!-- Form Card -->
	{#if !loading && !error}
		<Card class="mx-auto w-full max-w-2xl">
			<CardHeader>
				<CardTitle class="text-2xl">Edit Game</CardTitle>
				<CardDescription>Update the game information below.</CardDescription>
			</CardHeader>
			<CardContent>
				{#if error}
					<div class="mb-4 rounded-md border border-red-200 bg-red-50 p-3">
						<p class="text-sm text-red-700">{error}</p>
					</div>
				{/if}

				{#if success}
					<div class="mb-4 rounded-md border border-green-200 bg-green-50 p-3">
						<p class="text-sm text-green-700">
							Game updated successfully! Redirecting to game details...
						</p>
					</div>
				{/if}

				<form onsubmit={handleSubmit} class="space-y-4">
					<!-- Game Name -->
					<div>
						<Label for="name">Game Name *</Label>
						<Input
							id="name"
							bind:value={formData.name}
							class={errors.name ? 'border-red-500' : ''}
							placeholder="Enter the game name"
							disabled={isSubmitting}
						/>
						{#if errors.name}
							<p class="mt-1 text-sm text-red-600">{errors.name}</p>
						{/if}
					</div>

					<!-- Description -->
					<div>
						<Label for="description">Description</Label>
						<Textarea
							id="description"
							bind:value={formData.description}
							placeholder="Brief description of the game"
							disabled={isSubmitting}
							rows={3}
						/>
					</div>

					<!-- Publisher -->
					<div>
						<Label for="publisher">Publisher</Label>
						<Input
							id="publisher"
							bind:value={formData.publisher}
							placeholder="Game publisher"
							disabled={isSubmitting}
						/>
					</div>

					<!-- Year Published -->
					<div>
						<Label for="yearPublished">Year Published</Label>
						<Input
							id="yearPublished"
							type="number"
							bind:value={formData.yearPublished}
							class={errors.yearPublished ? 'border-red-500' : ''}
							placeholder="e.g. 2023"
							disabled={isSubmitting}
						/>
						{#if errors.yearPublished}
							<p class="mt-1 text-sm text-red-600">{errors.yearPublished}</p>
						{/if}
					</div>

					<!-- Player Count -->
					<div class="grid grid-cols-2 gap-4">
						<div>
							<Label for="minPlayers">Min Players</Label>
							<Input
								id="minPlayers"
								type="number"
								bind:value={formData.minPlayers}
								class={errors.minPlayers ? 'border-red-500' : ''}
								placeholder="1"
								disabled={isSubmitting}
							/>
							{#if errors.minPlayers}
								<p class="mt-1 text-sm text-red-600">{errors.minPlayers}</p>
							{/if}
						</div>
						<div>
							<Label for="maxPlayers">Max Players</Label>
							<Input
								id="maxPlayers"
								type="number"
								bind:value={formData.maxPlayers}
								class={errors.maxPlayers ? 'border-red-500' : ''}
								placeholder="4"
								disabled={isSubmitting}
							/>
							{#if errors.maxPlayers}
								<p class="mt-1 text-sm text-red-600">{errors.maxPlayers}</p>
							{/if}
						</div>
					</div>

					<!-- Play Time -->
					<div>
						<Label for="playTimeMinutes">Play Time (minutes)</Label>
						<Input
							id="playTimeMinutes"
							type="number"
							bind:value={formData.playTimeMinutes}
							class={errors.playTimeMinutes ? 'border-red-500' : ''}
							placeholder="60"
							disabled={isSubmitting}
						/>
						{#if errors.playTimeMinutes}
							<p class="mt-1 text-sm text-red-600">{errors.playTimeMinutes}</p>
						{/if}
					</div>

					<!-- Complexity Rating -->
					<div>
						<Label for="complexityRating">Complexity Rating (1.0 - 5.0)</Label>
						<Input
							id="complexityRating"
							type="number"
							step="0.1"
							min="1"
							max="5"
							bind:value={formData.complexityRating}
							class={errors.complexityRating ? 'border-red-500' : ''}
							placeholder="2.5"
							disabled={isSubmitting}
						/>
						{#if errors.complexityRating}
							<p class="mt-1 text-sm text-red-600">{errors.complexityRating}</p>
						{/if}
					</div>

					<!-- BoardGameGeek ID -->
					<div>
						<Label for="bggId">BoardGameGeek ID</Label>
						<Input
							id="bggId"
							type="number"
							bind:value={formData.bggId}
							class={errors.bggId ? 'border-red-500' : ''}
							placeholder="Optional BGG ID"
							disabled={isSubmitting}
						/>
						{#if errors.bggId}
							<p class="mt-1 text-sm text-red-600">{errors.bggId}</p>
						{/if}
					</div>

					<!-- Form Actions -->
					<div class="flex justify-end space-x-3 pt-4">
						<Button type="button" variant="outline" onclick={handleCancel} disabled={isSubmitting}>
							Cancel
						</Button>
						<Button type="submit" disabled={isSubmitting}>
							{isSubmitting ? 'Updating...' : 'Update Game'}
						</Button>
					</div>
				</form>
			</CardContent>
		</Card>
	{/if}
</main>
