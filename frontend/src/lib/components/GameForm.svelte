<script lang="ts">
	import { api, type Game, type CreateGameRequest, type UpdateGameRequest } from '$lib';
	import { Button, Card, Badge, Label, Input, Textarea } from '$lib/components/ui';
	import { createEventDispatcher } from 'svelte';

	interface Props {
		game?: Game;
		onSubmit?: (game: Game) => void;
		onCancel?: () => void;
	}

	let { game, onSubmit, onCancel }: Props = $props();

	const dispatch = createEventDispatcher<{
		submit: Game;
		cancel: void;
	}>();

	// Form state
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);
	let success = $state(false);

	// Form data
	let formData = $state({
		name: game?.name || '',
		description: game?.description || '',
		publisher: game?.publisher || '',
		yearPublished: game?.yearPublished?.toString() || '',
		minPlayers: game?.minPlayers?.toString() || '',
		maxPlayers: game?.maxPlayers?.toString() || '',
		playTimeMinutes: game?.playTimeMinutes?.toString() || '',
		complexityRating: game?.complexityRating?.toString() || '',
		bggId: game?.bggId?.toString() || ''
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

	function buildRequestData(): CreateGameRequest | UpdateGameRequest {
		const data: any = {
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

	async function handleSubmit() {
		if (!validateForm()) return;

		isSubmitting = true;
		error = null;
		success = false;

		try {
			const requestData = buildRequestData();
			let result;

			if (game) {
				// Update existing game
				result = await api.methods.updateGame({
					path: { id: game.id },
					body: requestData as UpdateGameRequest
				});
			} else {
				// Create new game
				result = await api.methods.createGame({
					body: requestData as CreateGameRequest
				});
			}

			if (result.success) {
				success = true;
				onSubmit?.(result.data);
				dispatch('submit', result.data);
			} else {
				error = result.error?.message || 'An error occurred while saving the game';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			isSubmitting = false;
		}
	}

	function handleCancel() {
		onCancel?.();
		dispatch('cancel');
	}
</script>

<Card class="mx-auto w-full max-w-2xl p-6">
	<div class="mb-6">
		<h2 class="text-2xl font-bold text-gray-900">
			{game ? 'Edit Game' : 'Add New Game'}
		</h2>
		<p class="mt-1 text-sm text-gray-600">
			{game ? 'Update the game information below.' : 'Fill in the details for your new board game.'}
		</p>
	</div>

	{#if error}
		<div class="mb-4 rounded-md border border-red-200 bg-red-50 p-3">
			<p class="text-sm text-red-700">{error}</p>
		</div>
	{/if}

	{#if success}
		<div class="mb-4 rounded-md border border-green-200 bg-green-50 p-3">
			<p class="text-sm text-green-700">
				Game {game ? 'updated' : 'created'} successfully!
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
				error={!!errors.name}
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
				rows="3"
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
				error={!!errors.yearPublished}
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
					error={!!errors.minPlayers}
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
					error={!!errors.maxPlayers}
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
				error={!!errors.playTimeMinutes}
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
				error={!!errors.complexityRating}
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
				error={!!errors.bggId}
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
				{isSubmitting ? 'Saving...' : game ? 'Update Game' : 'Create Game'}
			</Button>
		</div>
	</form>
</Card>
