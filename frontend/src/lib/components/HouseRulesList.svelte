<script lang="ts">
	import { formatDate } from '$lib';
	import { Button, Badge, Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui';
	import { EmptyState, LoadingSpinner } from '$lib/components/ui';
	import HouseRuleForm from './HouseRuleForm.svelte';
	import type { HouseRule } from '$lib';

	// Props - data passed from parent
	let {
		gameId,
		houseRules = [],
		isLoading = false,
		error = null,
		currentPage = 1,
		totalPages = 1,
		total = 0,
		onPageChange,
		onDelete,
		onSaved
	}: {
		gameId: number;
		houseRules: HouseRule[];
		isLoading?: boolean;
		error?: string | null;
		currentPage?: number;
		totalPages?: number;
		total?: number;
		onPageChange?: (page: number) => void;
		onDelete?: (rule: HouseRule) => Promise<void>;
		onSaved?: (rule: HouseRule) => void;
	} = $props();

	// Local UI state only
	let showForm = $state(false);
	let editingRule = $state<HouseRule | null>(null);
	let deleteError = $state<string | null>(null);

	// Handle delete with confirmation
	async function handleDelete(rule: HouseRule) {
		if (!confirm(`Are you sure you want to delete "${rule.title}"?`)) {
			return;
		}

		deleteError = null;
		try {
			await onDelete?.(rule);
		} catch (err) {
			deleteError = err instanceof Error ? err.message : 'Failed to delete house rule';
		}
	}

	// Handle form saved
	function handleSaved(rule: HouseRule) {
		showForm = false;
		editingRule = null;
		onSaved?.(rule);
	}

	// Handle form cancel
	function handleCancel() {
		showForm = false;
		editingRule = null;
	}

	// Start editing a rule
	function startEdit(rule: HouseRule) {
		editingRule = rule;
		showForm = true;
	}

	// Start creating a new rule
	function startCreate() {
		editingRule = null;
		showForm = true;
	}

	// Handle page change
	function goToPage(page: number) {
		onPageChange?.(page);
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-xl font-semibold">House Rules</h2>
			<p class="text-muted-foreground text-sm">
				{total} house rule{total === 1 ? '' : 's'}
			</p>
		</div>
		{#if !showForm}
			<Button onclick={startCreate}>Add House Rule</Button>
		{/if}
	</div>

	<!-- Error from parent -->
	{#if error}
		<div class="rounded-md border border-red-200 bg-red-50 p-4">
			<p class="text-sm text-red-700">{error}</p>
		</div>
	{/if}

	<!-- Delete error (local) -->
	{#if deleteError}
		<div class="rounded-md border border-red-200 bg-red-50 p-4">
			<p class="text-sm text-red-700">{deleteError}</p>
		</div>
	{/if}

	<!-- Form -->
	{#if showForm}
		<Card>
			<CardHeader>
				<CardTitle>{editingRule ? 'Edit House Rule' : 'Add House Rule'}</CardTitle>
			</CardHeader>
			<CardContent>
				<HouseRuleForm
					{gameId}
					existingRule={editingRule}
					onSaved={handleSaved}
					onCancel={handleCancel}
				/>
			</CardContent>
		</Card>
	{/if}

	<!-- Loading -->
	{#if isLoading}
		<div class="flex justify-center py-8">
			<LoadingSpinner />
		</div>
	{:else if houseRules.length === 0}
		<!-- Empty state -->
		<EmptyState
			title="No House Rules"
			description="Add custom rules and variants for this game. House rules can be included in chat context for personalized rule lookups."
		>
			{#if !showForm}
				<Button onclick={startCreate} class="mt-4">Add Your First House Rule</Button>
			{/if}
		</EmptyState>
	{:else}
		<!-- House rules list -->
		<div class="space-y-4">
			{#each houseRules as rule (rule.id)}
				<Card class={rule.isActive ? '' : 'opacity-60'}>
					<CardContent class="pt-6">
						<div class="flex items-start justify-between">
							<div class="flex-1">
								<div class="flex items-center gap-2">
									<h3 class="font-semibold">{rule.title}</h3>
									<Badge variant={rule.isActive ? 'default' : 'secondary'}>
										{rule.isActive ? 'Active' : 'Inactive'}
									</Badge>
									{#if rule.category}
										<Badge variant="outline">{rule.category}</Badge>
									{/if}
								</div>
								<p class="text-muted-foreground mt-2 text-sm">{rule.description}</p>
								<p class="text-muted-foreground mt-2 text-xs">
									Created {formatDate(new Date(rule.createdAt))}
								</p>
							</div>
							<div class="flex space-x-2">
								<Button variant="outline" size="sm" onclick={() => startEdit(rule)}>Edit</Button>
								<Button variant="destructive" size="sm" onclick={() => handleDelete(rule)}
									>Delete</Button
								>
							</div>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>

		<!-- Pagination -->
		{#if totalPages > 1}
			<div class="flex items-center justify-center gap-2 pt-4">
				<Button
					variant="outline"
					size="sm"
					disabled={currentPage <= 1}
					onclick={() => goToPage(currentPage - 1)}
				>
					Previous
				</Button>
				<span class="text-muted-foreground text-sm">
					Page {currentPage} of {totalPages}
				</span>
				<Button
					variant="outline"
					size="sm"
					disabled={currentPage >= totalPages}
					onclick={() => goToPage(currentPage + 1)}
				>
					Next
				</Button>
			</div>
		{/if}
	{/if}
</div>
