<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';

	interface BggGamePreview {
		row: number;
		name: string;
		bgg_id: number;
		year_published: number | null;
		min_players: number | null;
		max_players: number | null;
		play_time_minutes: number | null;
		complexity_rating: number | null;
	}

	interface FieldChange {
		field: string;
		old_value: string | null;
		new_value: string | null;
	}

	interface BggGameUpdatePreview {
		row: number;
		existing_id: number;
		bgg_id: number;
		name: string;
		changes: FieldChange[];
	}

	interface BggParseError {
		row: number;
		message: string;
	}

	interface PreviewResponse {
		games_to_insert: BggGamePreview[];
		games_to_update: BggGameUpdatePreview[];
		errors: BggParseError[];
		total_rows: number;
	}

	interface ImportResponse {
		inserted_count: number;
		updated_count: number;
		errors: BggParseError[];
	}

	const PAGE_SIZE = 50;

	let selectedFile = $state<File | null>(null);
	let preview = $state<PreviewResponse | null>(null);
	let importResult = $state<ImportResponse | null>(null);
	let isLoading = $state(false);
	let error = $state<string | null>(null);

	// Pagination state for each table
	let insertPage = $state(1);
	let updatePage = $state(1);
	let errorPage = $state(1);

	let fileInput: HTMLInputElement;

	// Computed paginated data
	let paginatedInserts = $derived.by(() => {
		if (!preview) return [];
		const start = (insertPage - 1) * PAGE_SIZE;
		return preview.games_to_insert.slice(start, start + PAGE_SIZE);
	});

	let paginatedUpdates = $derived.by(() => {
		if (!preview) return [];
		const start = (updatePage - 1) * PAGE_SIZE;
		return preview.games_to_update.slice(start, start + PAGE_SIZE);
	});

	let paginatedErrors = $derived.by(() => {
		if (!preview) return [];
		const start = (errorPage - 1) * PAGE_SIZE;
		return preview.errors.slice(start, start + PAGE_SIZE);
	});

	// Total pages for each table
	let insertTotalPages = $derived(
		preview ? Math.ceil(preview.games_to_insert.length / PAGE_SIZE) : 0
	);
	let updateTotalPages = $derived(
		preview ? Math.ceil(preview.games_to_update.length / PAGE_SIZE) : 0
	);
	let errorTotalPages = $derived(preview ? Math.ceil(preview.errors.length / PAGE_SIZE) : 0);

	function handleFileSelect(event: Event) {
		const input = event.target as HTMLInputElement;
		if (input.files && input.files.length > 0) {
			const file = input.files[0];
			if (!file.name.endsWith('.csv')) {
				error = 'Please select a CSV file';
				selectedFile = null;
				return;
			}
			if (file.size > 15 * 1024 * 1024) {
				error = 'File too large. Maximum size is 15MB';
				selectedFile = null;
				return;
			}
			selectedFile = file;
			error = null;
			preview = null;
			importResult = null;
		}
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		if (event.dataTransfer?.files && event.dataTransfer.files.length > 0) {
			const file = event.dataTransfer.files[0];
			if (!file.name.endsWith('.csv')) {
				error = 'Please select a CSV file';
				return;
			}
			if (file.size > 15 * 1024 * 1024) {
				error = 'File too large. Maximum size is 15MB';
				return;
			}
			selectedFile = file;
			error = null;
			preview = null;
			importResult = null;
		}
	}

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
	}

	async function loadPreview() {
		if (!selectedFile) return;

		isLoading = true;
		error = null;

		try {
			const response = await fetch('/api/admin/games/import/preview', {
				method: 'POST',
				body: selectedFile,
				credentials: 'include'
			});

			if (response.ok) {
				preview = await response.json();
				// Reset pagination
				insertPage = 1;
				updatePage = 1;
				errorPage = 1;
			} else {
				const data = await response.json().catch(() => ({}));
				error = data.message || 'Failed to preview CSV';
			}
		} catch {
			error = 'Failed to preview CSV';
		} finally {
			isLoading = false;
		}
	}

	async function executeImport() {
		if (!selectedFile) return;

		isLoading = true;
		error = null;

		try {
			const response = await fetch('/api/admin/games/import', {
				method: 'POST',
				body: selectedFile,
				credentials: 'include'
			});

			if (response.ok) {
				importResult = await response.json();
				preview = null;
				selectedFile = null;
			} else {
				const data = await response.json().catch(() => ({}));
				error = data.message || 'Failed to import CSV';
			}
		} catch {
			error = 'Failed to import CSV';
		} finally {
			isLoading = false;
		}
	}

	function resetForm() {
		selectedFile = null;
		preview = null;
		importResult = null;
		error = null;
		insertPage = 1;
		updatePage = 1;
		errorPage = 1;
		if (fileInput) {
			fileInput.value = '';
		}
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
				<h1 class="text-foreground text-3xl font-bold">Import Games from BGG</h1>
				<p class="text-muted-foreground mt-2">
					Upload a BoardGameGeek CSV export to add or update master games
				</p>
			</div>
		</div>
	</div>

	{#if importResult}
		<!-- Import Result -->
		<Card.Root class="border-green-500">
			<Card.Header>
				<Card.Title class="text-green-600">Import Complete</Card.Title>
			</Card.Header>
			<Card.Content>
				<div class="space-y-2">
					<p class="text-foreground">
						<span class="font-semibold">{importResult.inserted_count.toLocaleString()}</span> games inserted
					</p>
					<p class="text-foreground">
						<span class="font-semibold">{importResult.updated_count.toLocaleString()}</span> games updated
					</p>
					{#if importResult.errors.length > 0}
						<p class="text-destructive">
							<span class="font-semibold">{importResult.errors.length.toLocaleString()}</span> rows had
							errors
						</p>
					{/if}
				</div>
				<Button class="mt-4" onclick={resetForm}>Import Another File</Button>
			</Card.Content>
		</Card.Root>
	{:else if preview}
		<!-- Preview Results -->
		<div class="space-y-6">
			<!-- Summary -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Import Preview</Card.Title>
					<Card.Description>Review changes before importing</Card.Description>
				</Card.Header>
				<Card.Content>
					<div class="grid grid-cols-3 gap-4 text-center">
						<div class="rounded-lg bg-green-50 p-4 dark:bg-green-950">
							<p class="text-2xl font-bold text-green-600">
								{preview.games_to_insert.length.toLocaleString()}
							</p>
							<p class="text-muted-foreground text-sm">New Games</p>
						</div>
						<div class="rounded-lg bg-yellow-50 p-4 dark:bg-yellow-950">
							<p class="text-2xl font-bold text-yellow-600">
								{preview.games_to_update.length.toLocaleString()}
							</p>
							<p class="text-muted-foreground text-sm">Updates</p>
						</div>
						<div class="rounded-lg bg-red-50 p-4 dark:bg-red-950">
							<p class="text-2xl font-bold text-red-600">
								{preview.errors.length.toLocaleString()}
							</p>
							<p class="text-muted-foreground text-sm">Errors</p>
						</div>
					</div>
					<div class="mt-6 flex gap-4">
						<Button onclick={executeImport} disabled={isLoading}>
							{isLoading ? 'Importing...' : 'Confirm Import'}
						</Button>
						<Button variant="outline" onclick={resetForm}>Cancel</Button>
					</div>
				</Card.Content>
			</Card.Root>

			<!-- Games to Insert -->
			{#if preview.games_to_insert.length > 0}
				<Card.Root>
					<Card.Header>
						<Card.Title class="text-green-600">
							New Games to Add ({preview.games_to_insert.length.toLocaleString()})
						</Card.Title>
						<Card.Description>
							Showing {((insertPage - 1) * PAGE_SIZE + 1).toLocaleString()} - {Math.min(
								insertPage * PAGE_SIZE,
								preview.games_to_insert.length
							).toLocaleString()}
							of {preview.games_to_insert.length.toLocaleString()}
						</Card.Description>
					</Card.Header>
					<Card.Content>
						<div class="max-h-96 overflow-auto">
							<table class="w-full text-sm">
								<thead class="bg-muted sticky top-0">
									<tr>
										<th class="px-2 py-1 text-left">Row</th>
										<th class="px-2 py-1 text-left">Name</th>
										<th class="px-2 py-1 text-left">BGG ID</th>
										<th class="px-2 py-1 text-left">Year</th>
									</tr>
								</thead>
								<tbody>
									{#each paginatedInserts as game (game.row)}
										<tr class="border-border border-b">
											<td class="px-2 py-1">{game.row}</td>
											<td class="px-2 py-1">{game.name}</td>
											<td class="px-2 py-1">{game.bgg_id}</td>
											<td class="px-2 py-1">{game.year_published ?? '-'}</td>
										</tr>
									{/each}
								</tbody>
							</table>
						</div>
						<!-- Pagination -->
						{#if insertTotalPages > 1}
							<div class="mt-4 flex items-center justify-between">
								<Button
									variant="outline"
									size="sm"
									disabled={insertPage <= 1}
									onclick={() => (insertPage = Math.max(1, insertPage - 1))}
								>
									Previous
								</Button>
								<span class="text-muted-foreground text-sm">
									Page {insertPage} of {insertTotalPages}
								</span>
								<Button
									variant="outline"
									size="sm"
									disabled={insertPage >= insertTotalPages}
									onclick={() => (insertPage = Math.min(insertTotalPages, insertPage + 1))}
								>
									Next
								</Button>
							</div>
						{/if}
					</Card.Content>
				</Card.Root>
			{/if}

			<!-- Games to Update -->
			{#if preview.games_to_update.length > 0}
				<Card.Root>
					<Card.Header>
						<Card.Title class="text-yellow-600">
							Games to Update ({preview.games_to_update.length.toLocaleString()})
						</Card.Title>
						<Card.Description>
							Showing {((updatePage - 1) * PAGE_SIZE + 1).toLocaleString()} - {Math.min(
								updatePage * PAGE_SIZE,
								preview.games_to_update.length
							).toLocaleString()}
							of {preview.games_to_update.length.toLocaleString()}
						</Card.Description>
					</Card.Header>
					<Card.Content>
						<div class="max-h-96 overflow-auto">
							<table class="w-full text-sm">
								<thead class="bg-muted sticky top-0">
									<tr>
										<th class="px-2 py-1 text-left">Row</th>
										<th class="px-2 py-1 text-left">Name</th>
										<th class="px-2 py-1 text-left">Changes</th>
									</tr>
								</thead>
								<tbody>
									{#each paginatedUpdates as game (game.row)}
										<tr class="border-border border-b">
											<td class="px-2 py-1">{game.row}</td>
											<td class="px-2 py-1">{game.name}</td>
											<td class="px-2 py-1">
												{#each game.changes as change (change.field)}
													<div class="text-xs">
														<span class="font-medium">{formatFieldName(change.field)}:</span>
														<span class="text-red-500 line-through"
															>{change.old_value ?? 'null'}</span
														>
														&rarr;
														<span class="text-green-500">{change.new_value ?? 'null'}</span>
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
							Parsing Errors ({preview.errors.length.toLocaleString()})
						</Card.Title>
						<Card.Description>
							These rows will be skipped. Showing {(
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
										<th class="px-2 py-1 text-left">Row</th>
										<th class="px-2 py-1 text-left">Error</th>
									</tr>
								</thead>
								<tbody>
									{#each paginatedErrors as err (err.row)}
										<tr class="border-border border-b">
											<td class="px-2 py-1">{err.row}</td>
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
		<!-- File Upload -->
		<Card.Root>
			<Card.Header>
				<Card.Title>Upload CSV File</Card.Title>
				<Card.Description>
					Upload a BGG collection export or the official BGG ranks data dump. The CSV must contain
					at minimum the columns: name/objectname and id/objectid
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<!-- Drop Zone -->
				<div
					class="border-border hover:border-game-blue cursor-pointer rounded-lg border-2 border-dashed p-8 text-center transition-colors"
					ondrop={handleDrop}
					ondragover={handleDragOver}
					role="button"
					tabindex="0"
					onkeydown={(e) => e.key === 'Enter' && fileInput?.click()}
					onclick={() => fileInput?.click()}
				>
					<input
						bind:this={fileInput}
						type="file"
						accept=".csv"
						class="hidden"
						onchange={handleFileSelect}
					/>
					{#if selectedFile}
						<div class="text-foreground">
							<p class="text-lg font-semibold">{selectedFile.name}</p>
							<p class="text-muted-foreground text-sm">
								{(selectedFile.size / 1024).toFixed(1)} KB
							</p>
						</div>
					{:else}
						<div class="text-muted-foreground">
							<p class="text-lg">Drop your CSV file here</p>
							<p class="text-sm">or click to browse</p>
						</div>
					{/if}
				</div>

				{#if error}
					<div class="text-destructive mt-4 text-sm">{error}</div>
				{/if}

				<div class="mt-6 flex gap-4">
					<Button onclick={loadPreview} disabled={!selectedFile || isLoading}>
						{isLoading ? 'Loading Preview...' : 'Preview Import'}
					</Button>
					{#if selectedFile}
						<Button variant="outline" onclick={resetForm}>Clear</Button>
					{/if}
				</div>
			</Card.Content>
		</Card.Root>

		<!-- CSV Format Info -->
		<Card.Root class="mt-6">
			<Card.Header>
				<Card.Title>BGG CSV Column Mapping</Card.Title>
				<Card.Description>
					Supports both BGG collection exports and BGG ranks data dumps
				</Card.Description>
			</Card.Header>
			<Card.Content>
				<table class="text-muted-foreground w-full text-sm">
					<thead>
						<tr>
							<th class="px-2 py-1 text-left">BGG Column</th>
							<th class="px-2 py-1 text-left">Database Field</th>
							<th class="px-2 py-1 text-left">Required</th>
						</tr>
					</thead>
					<tbody>
						<tr>
							<td class="px-2 py-1">objectname / name</td>
							<td class="px-2 py-1">name</td>
							<td class="px-2 py-1">Yes</td>
						</tr>
						<tr>
							<td class="px-2 py-1">objectid / id</td>
							<td class="px-2 py-1">bgg_id</td>
							<td class="px-2 py-1">Yes</td>
						</tr>
						<tr>
							<td class="px-2 py-1">yearpublished</td>
							<td class="px-2 py-1">year_published</td>
							<td class="px-2 py-1">No</td>
						</tr>
						<tr>
							<td class="px-2 py-1">minplayers</td>
							<td class="px-2 py-1">min_players</td>
							<td class="px-2 py-1">No</td>
						</tr>
						<tr>
							<td class="px-2 py-1">maxplayers</td>
							<td class="px-2 py-1">max_players</td>
							<td class="px-2 py-1">No</td>
						</tr>
						<tr>
							<td class="px-2 py-1">playingtime</td>
							<td class="px-2 py-1">play_time_minutes</td>
							<td class="px-2 py-1">No</td>
						</tr>
						<tr>
							<td class="px-2 py-1">avgweight</td>
							<td class="px-2 py-1">complexity_rating</td>
							<td class="px-2 py-1">No</td>
						</tr>
					</tbody>
				</table>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
