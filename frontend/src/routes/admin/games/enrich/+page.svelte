<script lang="ts">
	import { api, type BulkEnrichPreviewResponse, type BulkEnrichResponse } from '$lib';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Label } from '$lib/components/ui/label';
	import { Input } from '$lib/components/ui/input';

	const PAGE_SIZE = 50;

	const ENRICHABLE_FIELDS = [
		{ id: 'year_published', label: 'Year Published' },
		{ id: 'min_players', label: 'Min Players' },
		{ id: 'max_players', label: 'Max Players' },
		{ id: 'play_time_minutes', label: 'Play Time' },
		{ id: 'complexity_rating', label: 'Complexity Rating' },
		{ id: 'description', label: 'Description' }
	];

	let selectedFields = $state<Set<string>>(new Set(ENRICHABLE_FIELDS.map((f) => f.id)));
	let batchLimit = $state(50);
	let preview = $state<BulkEnrichPreviewResponse | null>(null);
	let enrichResult = $state<BulkEnrichResponse | null>(null);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// Pagination state
	let updatePage = $state(1);
	let errorPage = $state(1);

	// Computed paginated data
	let paginatedUpdates = $derived.by(() => {
		if (!preview) return [];
		const start = (updatePage - 1) * PAGE_SIZE;
		return preview.gamesToUpdate.slice(start, start + PAGE_SIZE);
	});

	let paginatedErrors = $derived.by(() => {
		if (!preview) return [];
		const start = (errorPage - 1) * PAGE_SIZE;
		return preview.errors.slice(start, start + PAGE_SIZE);
	});

	// Total pages
	let updateTotalPages = $derived(
		preview ? Math.ceil(preview.gamesToUpdate.length / PAGE_SIZE) : 0
	);
	let errorTotalPages = $derived(preview ? Math.ceil(preview.errors.length / PAGE_SIZE) : 0);

	function toggleField(fieldId: string) {
		const newSet = new Set(selectedFields);
		if (newSet.has(fieldId)) {
			newSet.delete(fieldId);
		} else {
			newSet.add(fieldId);
		}
		selectedFields = newSet;
	}

	function toggleAllFields() {
		if (selectedFields.size === ENRICHABLE_FIELDS.length) {
			selectedFields = new Set();
		} else {
			selectedFields = new Set(ENRICHABLE_FIELDS.map((f) => f.id));
		}
	}

	async function loadPreview() {
		if (selectedFields.size === 0) {
			error = 'Please select at least one field to enrich';
			return;
		}

		isLoading = true;
		error = null;

		const result = await api.methods.previewBulkEnrich({
			body: {
				fieldsToEnrich: Array.from(selectedFields),
				limit: batchLimit
			}
		});

		if (result.type === 'success') {
			preview = result.data;
			updatePage = 1;
			errorPage = 1;
		} else if (result.type === 'error') {
			error = result.data.message || 'Failed to preview enrichment';
		} else if (result.type === 'client_error') {
			error = result.error.message || 'Failed to preview enrichment';
		}

		isLoading = false;
	}

	async function executeEnrich() {
		if (selectedFields.size === 0) {
			error = 'Please select at least one field to enrich';
			return;
		}

		isLoading = true;
		error = null;

		const result = await api.methods.executeBulkEnrich({
			body: {
				fieldsToEnrich: Array.from(selectedFields),
				limit: batchLimit
			}
		});

		if (result.type === 'success') {
			enrichResult = result.data;
			preview = null;
		} else if (result.type === 'error') {
			error = result.data.message || 'Failed to enrich games';
		} else if (result.type === 'client_error') {
			error = result.error.message || 'Failed to enrich games';
		}

		isLoading = false;
	}

	function resetForm() {
		preview = null;
		enrichResult = null;
		error = null;
		updatePage = 1;
		errorPage = 1;
	}

	function formatFieldName(field: string): string {
		return field
			.split('_')
			.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
			.join(' ');
	}
</script>

<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<div class="mb-8">
		<div class="flex items-center gap-4">
			<Button variant="outline" href="/admin">&larr; Back</Button>
			<div>
				<h1 class="text-foreground text-3xl font-bold">Enrich Games from BGG</h1>
				<p class="text-muted-foreground mt-2">
					Batch update games with missing data from BoardGameGeek API
				</p>
			</div>
		</div>
	</div>

	{#if enrichResult}
		<!-- Enrichment Result -->
		<Card.Root class="border-green-500">
			<Card.Header>
				<Card.Title class="text-green-600">Enrichment Complete</Card.Title>
			</Card.Header>
			<Card.Content>
				<div class="space-y-2">
					<p class="text-foreground">
						<span class="font-semibold">{enrichResult.updatedCount.toLocaleString()}</span> games updated
					</p>
					{#if enrichResult.errors.length > 0}
						<p class="text-destructive">
							<span class="font-semibold">{enrichResult.errors.length.toLocaleString()}</span> games
							had errors
						</p>
						<div class="bg-destructive/10 mt-2 max-h-48 overflow-auto rounded p-2">
							{#each enrichResult.errors as err (err.gameId)}
								<p class="text-destructive text-sm">Game ID {err.gameId}: {err.message}</p>
							{/each}
						</div>
					{/if}
				</div>
				<Button class="mt-4" onclick={resetForm}>Enrich More Games</Button>
			</Card.Content>
		</Card.Root>
	{:else if preview}
		<!-- Preview Results -->
		<div class="space-y-6">
			<!-- Summary -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Enrichment Preview</Card.Title>
					<Card.Description>Review changes before applying</Card.Description>
				</Card.Header>
				<Card.Content>
					<div class="grid grid-cols-3 gap-4 text-center">
						<div class="rounded-lg bg-blue-50 p-4 dark:bg-blue-950">
							<p class="text-2xl font-bold text-blue-600">
								{preview.totalFetched.toLocaleString()}
							</p>
							<p class="text-muted-foreground text-sm">Games Fetched</p>
						</div>
						<div class="rounded-lg bg-green-50 p-4 dark:bg-green-950">
							<p class="text-2xl font-bold text-green-600">
								{preview.gamesToUpdate.length.toLocaleString()}
							</p>
							<p class="text-muted-foreground text-sm">Games to Update</p>
						</div>
						<div class="rounded-lg bg-red-50 p-4 dark:bg-red-950">
							<p class="text-2xl font-bold text-red-600">
								{preview.errors.length.toLocaleString()}
							</p>
							<p class="text-muted-foreground text-sm">Errors</p>
						</div>
					</div>
					<div class="mt-6 flex gap-4">
						<Button
							onclick={executeEnrich}
							disabled={isLoading || preview.gamesToUpdate.length === 0}
						>
							{isLoading ? 'Enriching...' : 'Confirm Enrichment'}
						</Button>
						<Button variant="outline" onclick={resetForm}>Cancel</Button>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Games to Update -->
			{#if preview.gamesToUpdate.length > 0}
				<Card.Root>
					<Card.Header>
						<Card.Title class="text-green-600">
							Games to Update ({preview.gamesToUpdate.length.toLocaleString()})
						</Card.Title>
						<Card.Description>
							Showing {((updatePage - 1) * PAGE_SIZE + 1).toLocaleString()} - {Math.min(
								updatePage * PAGE_SIZE,
								preview.gamesToUpdate.length
							).toLocaleString()}
							of {preview.gamesToUpdate.length.toLocaleString()}
						</Card.Description>
					</Card.Header>
					<Card.Content>
						<div class="max-h-96 overflow-auto">
							<table class="w-full text-sm">
								<thead class="bg-muted sticky top-0">
									<tr>
										<th class="px-2 py-1 text-left">Game</th>
										<th class="px-2 py-1 text-left">BGG ID</th>
										<th class="px-2 py-1 text-left">Changes</th>
									</tr>
								</thead>
								<tbody>
									{#each paginatedUpdates as game (game.gameId)}
										<tr class="border-border border-b">
											<td class="px-2 py-1">
												<a
													href="/games/{game.gameId}"
													class="text-game-blue hover:underline"
													target="_blank"
												>
													{game.name}
												</a>
											</td>
											<td class="px-2 py-1">{game.bggId}</td>
											<td class="px-2 py-1">
												{#each game.changes as change (change.field)}
													<div class="text-xs">
														<span class="font-medium">{formatFieldName(change.field)}:</span>
														<span class="text-red-500 line-through"
															>{change.oldValue ?? 'null'}</span
														>
														&rarr;
														<span class="text-green-500">{change.newValue ?? 'null'}</span>
													</div>
												{/each}
											</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
						<!-- Pagination -->
						{#if updateTotalPages > 1}
							<div class="mt-4 flex items-center justify-between">
								<Button
									variant="outline"
									size="sm"
									disabled={updatePage <= 1}
									onclick={() => (updatePage = Math.max(1, updatePage - 1))}
								>
									Previous
								</Button>
								<span class="text-muted-foreground text-sm">
									Page {updatePage} of {updateTotalPages}
								</span>
								<Button
									variant="outline"
									size="sm"
									disabled={updatePage >= updateTotalPages}
									onclick={() => (updatePage = Math.min(updateTotalPages, updatePage + 1))}
								>
									Next
								</Button>
							</div>
						{/if}
					</Card.Content>
				</Card.Root>
			{/if}

			<!-- Errors -->
			{#if preview.errors.length > 0}
				<Card.Root class="border-destructive">
					<Card.Header>
						<Card.Title class="text-destructive">
							Fetch Errors ({preview.errors.length.toLocaleString()})
						</Card.Title>
						<Card.Description>
							These games could not be fetched from BGG. Showing {(
								(errorPage - 1) * PAGE_SIZE +
								1
							).toLocaleString()}
							- {Math.min(errorPage * PAGE_SIZE, preview.errors.length).toLocaleString()}
							of {preview.errors.length.toLocaleString()}
						</Card.Description>
					</Card.Header>
					<Card.Content>
						<div class="max-h-64 overflow-auto">
							<table class="w-full text-sm">
								<thead class="bg-muted sticky top-0">
									<tr>
										<th class="px-2 py-1 text-left">Game ID</th>
										<th class="px-2 py-1 text-left">BGG ID</th>
										<th class="px-2 py-1 text-left">Error</th>
									</tr>
								</thead>
								<tbody>
									{#each paginatedErrors as err (err.gameId)}
										<tr class="border-border border-b">
											<td class="px-2 py-1">{err.gameId}</td>
											<td class="px-2 py-1">{err.bggId}</td>
											<td class="text-destructive px-2 py-1">{err.message}</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
						<!-- Pagination -->
						{#if errorTotalPages > 1}
							<div class="mt-4 flex items-center justify-between">
								<Button
									variant="outline"
									size="sm"
									disabled={errorPage <= 1}
									onclick={() => (errorPage = Math.max(1, errorPage - 1))}
								>
									Previous
								</Button>
								<span class="text-muted-foreground text-sm">
									Page {errorPage} of {errorTotalPages}
								</span>
								<Button
									variant="outline"
									size="sm"
									disabled={errorPage >= errorTotalPages}
									onclick={() => (errorPage = Math.min(errorTotalPages, errorPage + 1))}
								>
									Next
								</Button>
							</div>
						{/if}
					</Card.Content>
				</Card.Root>
			{/if}
		</div>
	{:else}
		<!-- Configuration Form -->
		<div class="grid gap-6 lg:grid-cols-2">
			<Card.Root>
				<Card.Header>
					<Card.Title>Fields to Enrich</Card.Title>
					<Card.Description>Select which fields to update for games missing data</Card.Description>
				</Card.Header>
				<Card.Content>
					<div class="space-y-3">
						<div class="border-border flex items-center gap-2 border-b pb-3">
							<Checkbox
								id="select-all"
								checked={selectedFields.size === ENRICHABLE_FIELDS.length}
								onCheckedChange={toggleAllFields}
							/>
							<Label for="select-all" class="font-medium">Select All</Label>
						</div>
						{#each ENRICHABLE_FIELDS as field (field.id)}
							<div class="flex items-center gap-2">
								<Checkbox
									id={field.id}
									checked={selectedFields.has(field.id)}
									onCheckedChange={() => toggleField(field.id)}
								/>
								<Label for={field.id}>{field.label}</Label>
							</div>
						{/each}
					</div>
				</Card.Content>
			</Card.Root>

			<Card.Root>
				<Card.Header>
					<Card.Title>Batch Settings</Card.Title>
					<Card.Description>Configure how many games to process at once</Card.Description>
				</Card.Header>
				<Card.Content>
					<div class="space-y-4">
						<div>
							<Label for="batch-limit">Games per batch</Label>
							<Input
								id="batch-limit"
								type="number"
								bind:value={batchLimit}
								min={1}
								max={200}
								class="mt-1 w-32"
							/>
							<p class="text-muted-foreground mt-1 text-sm">
								Max 200 games per batch. BGG API allows 20 games per request.
							</p>
						</div>

						{#if error}
							<div class="bg-destructive/10 border-destructive rounded-lg border p-3">
								<p class="text-destructive text-sm">{error}</p>
							</div>
						{/if}

						<Button onclick={loadPreview} disabled={isLoading || selectedFields.size === 0}>
							{isLoading ? 'Loading Preview...' : 'Preview Enrichment'}
						</Button>
					</div>
				</Card.Content>
			</Card.Root>
		</div>

		<!-- Info Card -->
		<Card.Root class="mt-6">
			<Card.Header>
				<Card.Title>How Enrichment Works</Card.Title>
			</Card.Header>
			<Card.Content>
				<ul class="text-muted-foreground space-y-2 text-sm">
					<li>
						<strong>1. Find Games:</strong> The system identifies games with a BGG ID but missing the
						selected fields.
					</li>
					<li>
						<strong>2. Fetch from BGG:</strong> Game data is fetched from the BoardGameGeek XML API in
						batches of 20.
					</li>
					<li>
						<strong>3. Preview Changes:</strong> You'll see a comparison of current vs BGG values before
						confirming.
					</li>
					<li>
						<strong>4. Apply Updates:</strong> Only the selected fields with actual changes will be updated.
					</li>
				</ul>
				<p class="text-muted-foreground mt-4 text-sm">
					<strong>Note:</strong> Due to BGG API rate limits, large batches may take some time to process.
					The API enforces a ~500ms delay between requests.
				</p>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
