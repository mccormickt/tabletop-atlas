<script lang="ts">
	import { goto } from '$app/navigation';
	import { api, type Game, type CreateGameRequest } from '$lib';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui';
	import { Input, Label, Textarea } from '$lib/components/ui';

	// Form state
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

	function buildRequestData(): CreateGameRequest {
		const data: CreateGameRequest = {
			name: formData.name.trim()
		};

		// Only include non-empty optional fields
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
			const result = await api.methods.createGame({
				body: requestData
			});

			if (result.type === 'success') {
				success = true;
				// Navigate to the game detail page after a short delay
				setTimeout(() => {
					goto(`/games/${result.data.id}`);
				}, 1500);
			} else if (result.type === 'error') {
				error = result.data.message || 'An error occurred while saving the game';
			} else if (result.type === 'client_error') {
				error = result.error.message || 'An error occurred while saving the game';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			isSubmitting = false;
		}
	}

	function handleCancel() {
		goto('/games');
	}
</script>

<svelte:head>
	<title>Add New Game - Tabletop Atlas</title>
	<meta name="description" content="Add a new board game to your collection" />
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
		<!-- Back Navigation -->
		<div class="mb-6">
			<Button
				variant="ghost"
				onclick={handleCancel}
				class="text-muted-foreground hover:text-foreground"
			>
				← Back to Games
			</Button>
		</div>

		<!-- Form Card -->
		<Card class="mx-auto w-full max-w-2xl">
			<CardHeader>
				<CardTitle class="text-2xl">Add New Game</CardTitle>
				<CardDescription>Fill in the details for your new board game.</CardDescription>
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
							Game created successfully! Redirecting to game details...
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
							placeholder="e.g., 2023"
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
							{isSubmitting ? 'Creating...' : 'Create Game'}
						</Button>
					</div>
				</form>
			</CardContent>
		</Card>
	</main>
</div>
