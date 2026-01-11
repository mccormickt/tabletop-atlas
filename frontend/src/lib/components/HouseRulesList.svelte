<script lang="ts">
	import { api } from '$lib';
	import { Button, Badge, Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui';
	import { EmptyState, LoadingSpinner } from '$lib/components/ui';
	import HouseRuleForm from './HouseRuleForm.svelte';
	import type { HouseRule, PaginatedResponse_for_HouseRule } from '$lib';

	// Props
	let { gameId }: { gameId: number } = $props();

	// State
	let houseRules = $state<HouseRule[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let showForm = $state(false);
	let editingRule = $state<HouseRule | null>(null);
	let currentPage = $state(1);
	let totalPages = $state(1);
	let total = $state(0);
	const limit = 10;

	// Load house rules
	async function loadHouseRules() {
		isLoading = true;
		error = null;

		try {
			const result = await api.methods.listHouseRules({
				query: { gameId, page: currentPage, limit }
			});

			if (result.type === 'success') {
				houseRules = result.data.items;
				totalPages = result.data.totalPages;
				total = result.data.total;
			} else if (result.type === 'error') {
				error = result.data.message || 'Failed to load house rules';
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Failed to load house rules';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			isLoading = false;
		}
	}

	// Delete a house rule
	async function deleteRule(rule: HouseRule) {
		if (!confirm(`Are you sure you want to delete "${rule.title}"?`)) {
			return;
		}

		try {
			const result = await api.methods.deleteHouseRule({
				path: { id: rule.id }
			});

			if (result.type === 'success') {
				await loadHouseRules();
			} else if (result.type === 'error') {
				error = result.data.message || 'Failed to delete house rule';
			} else if (result.type === 'client_error') {
				error = result.error.message || 'Failed to delete house rule';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		}
	}

	// Handle form saved
	function handleSaved(rule: HouseRule) {
		showForm = false;
		editingRule = null;
		loadHouseRules();
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

	// Format date
	function formatDate(date: Date | string): string {
		const d = typeof date === 'string' ? new Date(date) : date;
		return d.toLocaleDateString();
	}

	// Load on mount
	$effect(() => {
		loadHouseRules();
	});

	// Reload when page changes
	$effect(() => {
		if (currentPage > 0) {
			loadHouseRules();
		}
	});
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h2 class="text-xl font-semibold">House Rules</h2>
			<p class="text-sm text-muted-foreground">
				{total} house rule{total === 1 ? '' : 's'}
			</p>
		</div>
		{#if !showForm}
			<Button onclick={startCreate}>Add House Rule</Button>
		{/if}
	</div>

	<!-- Error -->
	{#if error}
		<div class="rounded-md border border-red-200 bg-red-50 p-4">
			<p class="text-sm text-red-700">{error}</p>
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
								<p class="mt-2 text-sm text-muted-foreground">{rule.description}</p>
								<p class="mt-2 text-xs text-muted-foreground">
									Created {formatDate(rule.createdAt)}
								</p>
							</div>
							<div class="flex space-x-2">
								<Button variant="outline" size="sm" onclick={() => startEdit(rule)}>Edit</Button>
								<Button variant="destructive" size="sm" onclick={() => deleteRule(rule)}
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
					onclick={() => (currentPage = currentPage - 1)}
				>
					Previous
				</Button>
				<span class="text-sm text-muted-foreground">
					Page {currentPage} of {totalPages}
				</span>
				<Button
					variant="outline"
					size="sm"
					disabled={currentPage >= totalPages}
					onclick={() => (currentPage = currentPage + 1)}
				>
					Next
				</Button>
			</div>
		{/if}
	{/if}
</div>
