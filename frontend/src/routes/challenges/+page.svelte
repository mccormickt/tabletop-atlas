<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { useAuth, type AuthState } from '$lib/stores/auth';
	import { Button } from '$lib/components/ui/button';
	import { EmptyState } from '$lib/components/ui/empty-state';
	import ChallengeCard from '$lib/components/challenges/ChallengeCard.svelte';
	import type { ChallengeSummary } from '$lib';

	const auth = useAuth();

	let authState = $state<AuthState>({ user: null, isLoading: true, error: null });
	let challenges = $state<ChallengeSummary[]>([]);
	let isLoadingChallenges = $state(true);
	let error = $state<string | null>(null);

	$effect(() => {
		const unsubscribe = auth.subscribe((state) => {
			authState = state;
			if (!state.isLoading && !state.user) {
				goto(resolve('/auth/login'));
			} else if (state.user) {
				loadChallenges();
			}
		});
		return unsubscribe;
	});

	async function loadChallenges() {
		isLoadingChallenges = true;
		error = null;
		try {
			const response = await fetch('/api/challenges', {
				credentials: 'include'
			});
			if (response.ok) {
				const data = await response.json();
				challenges = data.items || [];
			} else if (response.status === 401) {
				goto(resolve('/auth/login'));
			} else {
				error = 'Failed to load challenges';
			}
		} catch {
			error = 'Failed to load challenges';
		} finally {
			isLoadingChallenges = false;
		}
	}
</script>

<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-foreground text-3xl font-bold">Challenges</h1>
			<p class="text-muted-foreground mt-2">Track your gaming sessions and compete with friends</p>
		</div>
		<Button href="/challenges/new">Create Challenge</Button>
	</div>

	{#if authState.isLoading || isLoadingChallenges}
		<div class="flex justify-center py-12">
			<div
				class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
			></div>
		</div>
	{:else if error}
		<div class="bg-destructive/10 border-destructive rounded-lg border p-4 text-center">
			<p class="text-destructive">{error}</p>
			<Button variant="outline" class="mt-4" onclick={loadChallenges}>Retry</Button>
		</div>
	{:else if challenges.length === 0}
		<EmptyState
			icon="document"
			title="No challenges yet"
			description="Create your first challenge to start tracking game sessions with friends."
			actionText="Create Challenge"
			onAction={() => goto(resolve('/challenges/new'))}
		/>
	{:else}
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each challenges as challenge (challenge.id)}
				<ChallengeCard {challenge} />
			{/each}
		</div>
	{/if}
</div>
