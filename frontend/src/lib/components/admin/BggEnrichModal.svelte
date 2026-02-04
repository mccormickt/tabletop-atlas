<script lang="ts">
	import { api, type BggEnrichPreviewResponse, type FieldChange } from '$lib';
	import { SvelteSet } from 'svelte/reactivity';
	import { Button } from '$lib/components/ui/button';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Label } from '$lib/components/ui/label';

	let {
		gameId,
		bggId,
		onClose,
		onSuccess
	}: {
		gameId: number;
		bggId: number;
		onClose: () => void;
		onSuccess: () => void;
	} = $props();

	let isLoading = $state(true);
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);
	let preview = $state<BggEnrichPreviewResponse | null>(null);
	let selectedFields = $state<Set<string>>(new Set());

	// Load preview on mount
	$effect(() => {
		loadPreview();
	});

	async function loadPreview() {
		isLoading = true;
		error = null;

		const result = await api.methods.previewBggEnrich({ path: { id: gameId } });

		if (result.type === 'success') {
			preview = result.data;
			// Pre-select all fields with changes
			selectedFields = new Set(preview.changes.map((c) => c.field));
		} else if (result.type === 'error') {
			error = result.data.message || 'Failed to fetch BGG data';
		} else if (result.type === 'client_error') {
			error = result.error.message || 'Failed to fetch BGG data';
		}

		isLoading = false;
	}

	function toggleField(field: string) {
		const newSet = new SvelteSet(selectedFields);
		if (newSet.has(field)) {
			newSet.delete(field);
		} else {
			newSet.add(field);
		}
		selectedFields = newSet;
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();

		if (selectedFields.size === 0) {
			error = 'Please select at least one field to update';
			return;
		}

		isSubmitting = true;
		error = null;

		const result = await api.methods.executeBggEnrich({
			path: { id: gameId },
			body: { fieldsToUpdate: Array.from(selectedFields) }
		});

		if (result.type === 'success') {
			onSuccess();
		} else if (result.type === 'error') {
			error = result.data.message || 'Failed to update game';
		} else if (result.type === 'client_error') {
			error = result.error.message || 'Failed to update game';
		}

		isSubmitting = false;
	}

	function formatFieldName(field: string): string {
		return field.replace(/_/g, ' ').replace(/\b\w/g, (l) => l.toUpperCase());
	}

	function formatValue(value: string | null | undefined): string {
		if (value === null || value === undefined) return '-';
		if (value.length > 60) return value.slice(0, 60) + '...';
		return value;
	}

	function getChangeForField(field: string): FieldChange | undefined {
		return preview?.changes.find((c) => c.field === field);
	}

	const allFields = [
		'description',
		'year_published',
		'min_players',
		'max_players',
		'play_time_minutes',
		'complexity_rating'
	];
</script>

<!-- Modal backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
	onclick={(e) => e.target === e.currentTarget && onClose()}
>
	<div
		class="bg-background max-h-[90vh] w-full max-w-2xl overflow-y-auto rounded-lg shadow-lg"
		onclick={(e) => e.stopPropagation()}
	>
		<div class="border-border border-b p-4">
			<div class="flex items-start justify-between">
				<div>
					<h2 class="text-foreground text-lg font-semibold">Update from BoardGameGeek</h2>
					<p class="text-muted-foreground text-sm">
						BGG ID: {bggId}
					</p>
				</div>
				<button type="button" class="text-muted-foreground hover:text-foreground" onclick={onClose}>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						viewBox="0 0 20 20"
						fill="currentColor"
					>
						<path
							fill-rule="evenodd"
							d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
							clip-rule="evenodd"
						/>
					</svg>
				</button>
			</div>
		</div>

		<div class="p-4">
			{#if isLoading}
				<div class="flex items-center justify-center py-8">
					<div class="text-muted-foreground">Fetching data from BGG...</div>
				</div>
			{:else if error && !preview}
				<div class="bg-destructive/10 border-destructive rounded-lg border p-3">
					<p class="text-destructive text-sm">{error}</p>
				</div>
			{:else if preview}
				{#if preview.changes.length === 0}
					<div class="bg-muted/50 rounded-lg p-4 text-center">
						<p class="text-muted-foreground">Game data is already up to date with BGG.</p>
					</div>
				{:else}
					<form onsubmit={handleSubmit} class="space-y-4">
						{#if error}
							<div class="bg-destructive/10 border-destructive rounded-lg border p-3">
								<p class="text-destructive text-sm">{error}</p>
							</div>
						{/if}

						<p class="text-muted-foreground text-sm">Select which fields to update from BGG:</p>

						<div class="space-y-3">
							{#each allFields as field (field)}
								{@const change = getChangeForField(field)}
								{@const hasChange = !!change}
								{@const currentValue = hasChange
									? change.oldValue
									: ((preview.currentValues as Record<string, unknown>)[
											field === 'complexity_rating'
												? 'complexityRating'
												: field.replace(/_([a-z])/g, (_, l) => l.toUpperCase())
										] as string | null)}
								{@const bggValue = hasChange
									? change.newValue
									: ((preview.bggValues as Record<string, unknown>)[
											field === 'complexity_rating'
												? 'complexityRating'
												: field.replace(/_([a-z])/g, (_, l) => l.toUpperCase())
										] as string | null)}

								<div
									class="border-border rounded-lg border p-3 {hasChange
										? 'bg-muted/30'
										: 'opacity-50'}"
								>
									<div class="flex items-start gap-3">
										<Checkbox
											id={field}
											checked={selectedFields.has(field)}
											disabled={!hasChange}
											onCheckedChange={() => toggleField(field)}
										/>
										<div class="flex-1">
											<Label
												for={field}
												class="font-medium {!hasChange ? 'text-muted-foreground' : ''}"
											>
												{formatFieldName(field)}
											</Label>
											<div class="mt-1 grid grid-cols-2 gap-2 text-sm">
												<div>
													<span class="text-muted-foreground">Current:</span>
													<span class={hasChange ? 'text-muted-foreground line-through' : ''}>
														{formatValue(currentValue?.toString())}
													</span>
												</div>
												<div>
													<span class="text-muted-foreground">BGG:</span>
													<span class={hasChange ? 'font-medium text-green-600' : ''}>
														{formatValue(bggValue?.toString())}
													</span>
												</div>
											</div>
										</div>
									</div>
								</div>
							{/each}
						</div>

						<div class="border-border flex gap-3 border-t pt-4">
							<Button type="button" variant="outline" onclick={onClose} class="flex-1">
								Cancel
							</Button>
							<Button
								type="submit"
								disabled={isSubmitting || selectedFields.size === 0}
								class="flex-1"
							>
								{isSubmitting
									? 'Updating...'
									: `Update ${selectedFields.size} field${selectedFields.size === 1 ? '' : 's'}`}
							</Button>
						</div>
					</form>
				{/if}
			{/if}
		</div>
	</div>
</div>
