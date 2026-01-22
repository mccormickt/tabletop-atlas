<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { useAuth, type AuthState } from '$lib/stores/auth';
	import { api } from '$lib';
	import { Button } from '$lib/components/ui/button';
	import type { ChallengeStats, Challenge } from '$api/Api';

	const auth = useAuth();

	let authState = $state<AuthState>({ user: null, isLoading: true, error: null });
	let stats = $state<ChallengeStats | null>(null);
	let challenge = $state<Challenge | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	$effect(() => {
		const unsubscribe = auth.subscribe((state) => {
			authState = state;
			if (!state.isLoading && !state.user) {
				goto('/auth/login');
			} else if (state.user && $page.params.id) {
				loadData();
			}
		});
		return unsubscribe;
	});

	async function loadData() {
		isLoading = true;
		error = null;
		try {
			const challengeId = Number($page.params.id);

			// Fetch stats and challenge info in parallel
			const [statsResult, challengeResult] = await Promise.all([
				api.methods.getChallengeStats({ path: { id: challengeId } }),
				api.methods.getChallenge({ path: { id: challengeId } })
			]);

			if (statsResult.type === 'success') {
				stats = statsResult.data;
			} else if (statsResult.statusCode === 401) {
				goto('/auth/login');
				return;
			} else if (statsResult.statusCode === 403) {
				error = 'You are not a participant in this challenge';
				return;
			} else if (statsResult.statusCode === 404) {
				error = 'Challenge not found';
				return;
			} else {
				error = 'Failed to load stats';
				return;
			}

			if (challengeResult.type === 'success') {
				challenge = challengeResult.data;
			}
		} catch {
			error = 'Failed to load challenge stats';
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>{challenge ? `${challenge.name} Stats` : 'Challenge Stats'} - Tabletop Atlas</title>
</svelte:head>

<div class="mx-auto max-w-4xl px-4 py-8 sm:px-6 lg:px-8">
	{#if authState.isLoading || isLoading}
		<div class="flex justify-center py-12">
			<div
				class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
			></div>
		</div>
	{:else if error}
		<div class="bg-destructive/10 border-destructive rounded-lg border p-4 text-center">
			<p class="text-destructive">{error}</p>
			<div class="mt-4 flex justify-center gap-4">
				<Button variant="outline" onclick={loadData}>Retry</Button>
				<Button href="/challenges">Back to Challenges</Button>
			</div>
		</div>
	{:else if stats}
		<!-- Header -->
		<div class="mb-8">
			<a
				href="/challenges/{$page.params.id}"
				class="text-muted-foreground hover:text-foreground text-sm"
			>
				&larr; Back to Challenge
			</a>
			<h1 class="text-foreground mt-4 text-3xl font-bold">
				{challenge?.name ?? 'Challenge'} Stats
			</h1>
			{#if challenge?.description}
				<p class="text-muted-foreground mt-2">{challenge.description}</p>
			{/if}
		</div>

		<!-- Progress Overview -->
		<div class="bg-card border-border mb-8 rounded-lg border p-6">
			<h2 class="text-foreground mb-4 text-xl font-semibold">Progress Overview</h2>
			<div class="grid gap-6 sm:grid-cols-3">
				<div class="text-center">
					<p class="text-foreground text-4xl font-bold">{stats.completedCells}</p>
					<p class="text-muted-foreground text-sm">Sessions Completed</p>
				</div>
				<div class="text-center">
					<p class="text-foreground text-4xl font-bold">{stats.totalCells}</p>
					<p class="text-muted-foreground text-sm">Total Sessions</p>
				</div>
				<div class="text-center">
					<p class="text-foreground text-4xl font-bold">
						{stats.completionPercentage.toFixed(1)}%
					</p>
					<p class="text-muted-foreground text-sm">Complete</p>
				</div>
			</div>

			<!-- Progress Bar -->
			<div class="mt-6">
				<div class="bg-muted h-4 overflow-hidden rounded-full">
					<div
						class="bg-primary h-full transition-all"
						style="width: {Math.min(stats.completionPercentage, 100)}%"
					></div>
				</div>
			</div>
		</div>

		<!-- Leaderboard -->
		<div class="bg-card border-border rounded-lg border p-6">
			<h2 class="text-foreground mb-6 text-xl font-semibold">Leaderboard</h2>
			{#if stats.leaderboard.length === 0}
				<p class="text-muted-foreground text-center">No plays recorded yet</p>
			{:else}
				<div class="space-y-4">
					{#each stats.leaderboard as entry, index (entry.userId)}
						<div
							class="bg-muted/50 flex items-center gap-4 rounded-lg p-4 {index === 0
								? 'ring-2 ring-yellow-400'
								: ''}"
						>
							<!-- Rank -->
							<div class="flex h-10 w-10 items-center justify-center">
								{#if index === 0}
									<span class="text-3xl">🥇</span>
								{:else if index === 1}
									<span class="text-3xl">🥈</span>
								{:else if index === 2}
									<span class="text-3xl">🥉</span>
								{:else}
									<span class="text-muted-foreground text-xl font-bold">{index + 1}</span>
								{/if}
							</div>

							<!-- Avatar -->
							{#if entry.pictureUrl}
								<img src={entry.pictureUrl} alt="" class="h-12 w-12 rounded-full" />
							{:else}
								<div
									class="bg-muted text-muted-foreground flex h-12 w-12 items-center justify-center rounded-full text-lg font-medium"
								>
									{(entry.displayName || 'U')[0].toUpperCase()}
								</div>
							{/if}

							<!-- Name and Stats -->
							<div class="min-w-0 flex-1">
								<p class="text-foreground truncate text-lg font-semibold">
									{entry.displayName || 'Unknown'}
								</p>
								<p class="text-muted-foreground text-sm">
									{entry.totalPlays} game{entry.totalPlays === 1 ? '' : 's'} played
								</p>
							</div>

							<!-- Wins -->
							<div class="text-right">
								<p class="text-foreground text-2xl font-bold">{entry.wins}</p>
								<p class="text-muted-foreground text-sm">
									win{entry.wins === 1 ? '' : 's'} ({entry.winPercentage.toFixed(0)}%)
								</p>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>

		<!-- Back Button -->
		<div class="mt-8 text-center">
			<Button href="/challenges/{$page.params.id}">Back to Challenge Grid</Button>
		</div>
	{/if}
</div>
