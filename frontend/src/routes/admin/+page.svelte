<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';

	interface AdminStats {
		master_games_count: number;
	}

	let stats = $state<AdminStats | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			loadStats();
		}
	});

	async function loadStats() {
		isLoading = true;
		error = null;
		try {
			const response = await fetch('/api/admin/stats', {
				credentials: 'include'
			});
			if (response.ok) {
				stats = await response.json();
			} else if (response.status === 403) {
				error = 'Access denied. Admin privileges required.';
			} else {
				error = 'Failed to load admin stats';
			}
		} catch {
			error = 'Failed to load admin stats';
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<div class="mb-8">
		<h1 class="text-foreground text-3xl font-bold">Admin Dashboard</h1>
		<p class="text-muted-foreground mt-2">Manage system data and settings</p>
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
			<Button variant="outline" class="mt-4" onclick={loadStats}>Retry</Button>
		</div>
	{:else if stats}
		<div class="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
			<!-- Stats Card -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Master Games</Card.Title>
					<Card.Description>Total games in the system database</Card.Description>
				</Card.Header>
				<Card.Content>
					<p class="text-foreground text-4xl font-bold">{stats.master_games_count}</p>
				</Card.Content>
			</Card.Root>

			<!-- Import Games Card -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Import Games</Card.Title>
					<Card.Description>Import games from BoardGameGeek CSV export</Card.Description>
				</Card.Header>
				<Card.Content>
					<Button href="/admin/games/import">Import BGG CSV</Button>
				</Card.Content>
			</Card.Root>
		</div>
	{/if}
</div>
