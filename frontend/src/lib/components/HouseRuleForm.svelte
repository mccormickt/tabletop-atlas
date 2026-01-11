<script lang="ts">
	import { api } from '$lib';
	import { Button, Input, Textarea, Label } from '$lib/components/ui';
	import type { HouseRule, CreateHouseRuleRequest, UpdateHouseRuleRequest } from '$lib';

	// Props
	let {
		gameId,
		existingRule = null,
		onSaved,
		onCancel
	}: {
		gameId: number;
		existingRule?: HouseRule | null;
		onSaved?: (rule: HouseRule) => void;
		onCancel?: () => void;
	} = $props();

	// Form state
	let title = $state(existingRule?.title ?? '');
	let description = $state(existingRule?.description ?? '');
	let category = $state(existingRule?.category ?? '');
	let isActive = $state(existingRule?.isActive ?? true);
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);

	// Category options
	const categories = ['Setup', 'Gameplay', 'Scoring', 'Variants', 'Other'];

	const isEditMode = $derived(!!existingRule);

	async function handleSubmit(event: Event) {
		event.preventDefault();
		if (!title.trim() || !description.trim()) {
			error = 'Title and description are required';
			return;
		}

		isSubmitting = true;
		error = null;

		try {
			if (isEditMode && existingRule) {
				// Update existing rule
				const updateRequest: UpdateHouseRuleRequest = {
					title: title.trim(),
					description: description.trim(),
					category: category.trim() || null,
					isActive
				};

				const result = await api.methods.updateHouseRule({
					path: { id: existingRule.id },
					body: updateRequest
				});

				if (result.type === 'success') {
					onSaved?.(result.data);
				} else if (result.type === 'error') {
					error = result.data.message || 'Failed to update house rule';
				} else if (result.type === 'client_error') {
					error = result.error.message || 'Failed to update house rule';
				}
			} else {
				// Create new rule
				const createRequest: CreateHouseRuleRequest = {
					gameId,
					title: title.trim(),
					description: description.trim(),
					category: category.trim() || undefined,
					isActive
				};

				const result = await api.methods.createHouseRule({
					body: createRequest
				});

				if (result.type === 'success') {
					onSaved?.(result.data);
					// Reset form for next entry
					title = '';
					description = '';
					category = '';
					isActive = true;
				} else if (result.type === 'error') {
					error = result.data.message || 'Failed to create house rule';
				} else if (result.type === 'client_error') {
					error = result.error.message || 'Failed to create house rule';
				}
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			isSubmitting = false;
		}
	}

	function handleCancel() {
		onCancel?.();
	}
</script>

<form onsubmit={handleSubmit} class="space-y-4">
	{#if error}
		<div class="rounded-md border border-red-200 bg-red-50 p-3">
			<p class="text-sm text-red-700">{error}</p>
		</div>
	{/if}

	<div class="space-y-2">
		<Label for="title">Title *</Label>
		<Input
			id="title"
			bind:value={title}
			placeholder="e.g., Quick Start Variant"
			required
			disabled={isSubmitting}
		/>
	</div>

	<div class="space-y-2">
		<Label for="description">Description *</Label>
		<Textarea
			id="description"
			bind:value={description}
			placeholder="Describe the house rule in detail..."
			rows={4}
			required
			disabled={isSubmitting}
		/>
	</div>

	<div class="space-y-2">
		<Label for="category">Category</Label>
		<select
			id="category"
			bind:value={category}
			class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
			disabled={isSubmitting}
		>
			<option value="">Select a category (optional)</option>
			{#each categories as cat}
				<option value={cat}>{cat}</option>
			{/each}
		</select>
	</div>

	<div class="flex items-center space-x-2">
		<input
			type="checkbox"
			id="isActive"
			bind:checked={isActive}
			disabled={isSubmitting}
			class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
		/>
		<Label for="isActive" class="text-sm font-medium leading-none">
			Active (include in chat context)
		</Label>
	</div>

	<div class="flex justify-end space-x-3 pt-4">
		<Button type="button" variant="outline" onclick={handleCancel} disabled={isSubmitting}>
			Cancel
		</Button>
		<Button type="submit" disabled={isSubmitting}>
			{#if isSubmitting}
				{isEditMode ? 'Updating...' : 'Creating...'}
			{:else}
				{isEditMode ? 'Update Rule' : 'Create Rule'}
			{/if}
		</Button>
	</div>
</form>
