<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { EmptyState } from '$lib/components/ui/empty-state';
	import ToolCard from '$lib/components/tools/ToolCard.svelte';
	import { api } from '$lib';
	import type { ToolSummary } from '$api/Api';

	let tools = $state<ToolSummary[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			loadTools();
		}
	});

	async function loadTools() {
		isLoading = true;
		error = null;
		try {
			const result = await api.methods.listTools({});
			if (result.type === 'success') {
				tools = result.data;
			} else {
				error = 'Failed to load tools';
			}
		} catch (e) {
			console.error('Failed to load tools:', e);
			error = 'Failed to load tools';
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<div class="mb-8">
		<h1 class="text-foreground text-3xl font-bold">Game Tools</h1>
		<p class="text-muted-foreground mt-2">Score calculators, timers, and more</p>
	</div>

	{#if isLoading}
		<div class="flex justify-center py-12">
			<div
				class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
			></div>
		</div>
	{:else if error}
		<div class="bg-destructive/10 border-destructive rounded-lg border p-4 text-center">
			<p class="text-destructive">{error}</p>
			<Button variant="outline" class="mt-4" onclick={loadTools}>Retry</Button>
		</div>
	{:else if tools.length === 0}
		<EmptyState
			title="No tools available"
			description="Score calculators and other game tools will appear here."
		>
			<svg
				slot="icon"
				xmlns="http://www.w3.org/2000/svg"
				class="text-muted-foreground h-12 w-12"
				fill="none"
				viewBox="0 0 24 24"
				stroke="currentColor"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M9 7h6m0 10v-3m-3 3h.01M9 17h.01M9 14h.01M12 14h.01M15 11h.01M12 11h.01M9 11h.01M7 21h10a2 2 0 002-2V5a2 2 0 00-2-2H7a2 2 0 00-2 2v14a2 2 0 002 2z"
				/>
			</svg>
		</EmptyState>
	{:else}
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each tools as tool (tool.id)}
				<ToolCard {tool} />
			{/each}
		</div>
	{/if}
</div>
