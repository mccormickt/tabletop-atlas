<script lang="ts">
	import { page } from '$app/state';
	import { Button } from '$lib/components/ui/button';
	import ScoreCalculator from '$lib/components/tools/ScoreCalculator.svelte';
	import type { ToolDetails, ScoreInput, ScoreOutput } from '$lib';

	let toolId = $derived(page.params.toolId);
	let tool = $state<ToolDetails | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let result = $state<ScoreOutput | null>(null);
	let isCalculating = $state(false);

	$effect(() => {
		if (toolId) {
			loadTool();
		}
	});

	async function loadTool() {
		isLoading = true;
		error = null;
		result = null;
		try {
			const response = await fetch(`/api/tools/${toolId}`);
			if (response.ok) {
				tool = await response.json();
			} else if (response.status === 404) {
				error = 'Tool not found';
			} else {
				error = 'Failed to load tool';
			}
		} catch (e) {
			console.error('Failed to load tool:', e);
			error = 'Failed to load tool';
		} finally {
			isLoading = false;
		}
	}

	async function handleCalculate(input: ScoreInput) {
		isCalculating = true;
		error = null;
		try {
			const response = await fetch(`/api/tools/${toolId}/calculate`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify(input)
			});
			if (response.ok) {
				result = await response.json();
			} else {
				const data = await response.json();
				error = data.message || 'Calculation failed';
			}
		} catch (e) {
			console.error('Score calculation failed:', e);
			error = 'Calculation failed';
		} finally {
			isCalculating = false;
		}
	}

	function handleReset() {
		result = null;
	}
</script>

<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	{#if isLoading}
		<div class="flex justify-center py-12">
			<div
				class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
			></div>
		</div>
	{:else if error && !tool}
		<div class="bg-destructive/10 border-destructive rounded-lg border p-4 text-center">
			<p class="text-destructive">{error}</p>
			<div class="mt-4 flex justify-center gap-2">
				<Button variant="outline" onclick={loadTool}>Retry</Button>
				<Button href="/tools">Back to Tools</Button>
			</div>
		</div>
	{:else if tool}
		<div class="mb-6">
			<div class="flex items-center gap-2">
				<Button variant="ghost" href="/tools" class="text-muted-foreground hover:text-foreground">
					← Back
				</Button>
			</div>
			<h1 class="text-foreground mt-2 text-3xl font-bold">{tool.displayName}</h1>
			<p class="text-muted-foreground mt-1">
				{tool.playerRange.min}-{tool.playerRange.max} players
			</p>
		</div>

		{#if error}
			<div class="bg-destructive/10 border-destructive mb-4 rounded-lg border p-4">
				<p class="text-destructive">{error}</p>
			</div>
		{/if}

		<ScoreCalculator
			{tool}
			{result}
			{isCalculating}
			onCalculate={handleCalculate}
			onReset={handleReset}
		/>
	{/if}
</div>
