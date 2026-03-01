<script lang="ts">
	import { api, type EnrichmentStats } from '$lib';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';

	interface AdminStats {
		master_games_count: number;
	}

	let stats = $state<AdminStats | null>(null);
	let enrichmentStats = $state<EnrichmentStats | null>(null);
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
			// Fetch both stats in parallel
			const [statsResponse, enrichmentResult] = await Promise.all([
				fetch('/api/admin/stats', { credentials: 'include' }),
				api.methods.getEnrichmentStats({})
			]);

			if (statsResponse.ok) {
				stats = await statsResponse.json();
			} else if (statsResponse.status === 403) {
				error = 'Access denied. Admin privileges required.';
				return;
			} else {
				error = 'Failed to load admin stats';
				return;
			}

			if (enrichmentResult.type === 'success') {
				enrichmentStats = enrichmentResult.data;
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

			<!-- Upload Rules Card -->
			<Card.Root>
				<Card.Header>
					<Card.Title>Upload Rules</Card.Title>
					<Card.Description>Upload PDF rule books for games</Card.Description>
				</Card.Header>
				<Card.Content>
					<Button href="/admin/upload">Upload Rules PDF</Button>
				</Card.Content>
			</Card.Root>

			<!-- BGG Enrichment Card -->
			{#if enrichmentStats}
				<Card.Root>
					<Card.Header>
						<Card.Title>BGG Enrichment</Card.Title>
						<Card.Description>Games with missing data from BoardGameGeek</Card.Description>
					</Card.Header>
					<Card.Content>
						<div class="space-y-3">
							<p class="text-foreground text-2xl font-bold">
								{enrichmentStats.missingAny}
								<span class="text-muted-foreground text-base font-normal">need enrichment</span>
							</p>
							<ul class="text-muted-foreground space-y-1 text-sm">
								<li>{enrichmentStats.missingYear} missing year</li>
								<li>{enrichmentStats.missingPlayers} missing player count</li>
								<li>{enrichmentStats.missingPlayTime} missing play time</li>
								<li>{enrichmentStats.missingComplexity} missing complexity</li>
							</ul>
							<Button href="/admin/games/enrich" class="mt-2 w-full">Enrich from BGG</Button>
						</div>
					</Card.Content>
				</Card.Root>
			{/if}
		</div>
	{/if}
</div>
