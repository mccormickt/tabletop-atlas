<script lang="ts">
	import { goto } from '$app/navigation';
	import { useAuth, type AuthState } from '$lib/stores/auth';
	import { Button } from '$lib/components/ui/button';
	import { EmptyState } from '$lib/components/ui/empty-state';
	import { Dice } from '$lib/components/icons';

	interface CustomGame {
		id: number;
		name: string;
		description: string | null;
		publisher: string | null;
		year_published: number | null;
		min_players: number | null;
		max_players: number | null;
		is_public: boolean;
	}

	const auth = useAuth();

	let authState = $state<AuthState>({ user: null, isLoading: true, error: null });
	let customGames = $state<CustomGame[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	$effect(() => {
		const unsubscribe = auth.subscribe((state) => {
			authState = state;
			if (!state.isLoading && !state.user) {
				goto('/auth/login');
			} else if (state.user) {
				loadCustomGames();
			}
		});
		return unsubscribe;
	});

	async function loadCustomGames() {
		isLoading = true;
		error = null;
		try {
			const response = await fetch('/api/custom-games', {
				credentials: 'include'
			});
			if (response.ok) {
				const data = await response.json();
				customGames = data.items || [];
			} else if (response.status === 401) {
				goto('/auth/login');
			} else {
				error = 'Failed to load custom games';
			}
		} catch {
			error = 'Failed to load custom games';
		} finally {
			isLoading = false;
		}
	}

	async function deleteGame(gameId: number) {
		try {
			const response = await fetch(`/api/custom-games/${gameId}`, {
				method: 'DELETE',
				credentials: 'include'
			});
			if (response.ok) {
				customGames = customGames.filter((g) => g.id !== gameId);
			}
		} catch (e) {
			console.error('Failed to delete custom game:', e);
		}
	}

	async function toggleVisibility(game: CustomGame) {
		try {
			const response = await fetch(`/api/custom-games/${game.id}`, {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({ is_public: !game.is_public })
			});
			if (response.ok) {
				const updated = await response.json();
				customGames = customGames.map((g) => (g.id === game.id ? updated : g));
			}
		} catch (e) {
			console.error('Failed to update custom game:', e);
		}
	}
</script>

<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<div class="mb-8 flex items-center justify-between">
		<div>
			<h1 class="text-foreground text-3xl font-bold">My Custom Games</h1>
			<p class="text-muted-foreground mt-2">Games you've created that aren't in the main library</p>
		</div>
		<Button href="/games/custom/add">Create Custom Game</Button>
	</div>

	{#if authState.isLoading || isLoading}
		<div class="flex justify-center py-12">
			<div
				class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
			></div>
		</div>
	{:else if error}
		<div class="bg-destructive/10 border-destructive rounded-lg border p-4 text-center">
			<p class="text-destructive">{error}</p>
			<Button variant="outline" class="mt-4" onclick={loadCustomGames}>Retry</Button>
		</div>
	{:else if customGames.length === 0}
		<EmptyState
			title="No custom games yet"
			description="Create your own games that aren't in the main library."
		>
			<Dice slot="icon" size={48} class="text-muted-foreground" />
			<Button href="/games/custom/add">Create Your First Custom Game</Button>
		</EmptyState>
	{:else}
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each customGames as game (game.id)}
				<div class="bg-card border-border rounded-lg border p-4">
					<div class="mb-2 flex items-start justify-between">
						<div>
							<h3 class="text-foreground font-semibold">{game.name}</h3>
							{#if game.publisher}
								<p class="text-muted-foreground text-sm">{game.publisher}</p>
							{/if}
						</div>
						<button
							onclick={() => toggleVisibility(game)}
							class="text-muted-foreground hover:text-foreground text-xs"
							title={game.is_public ? 'Make private' : 'Make public'}
						>
							{game.is_public ? '🌐 Public' : '🔒 Private'}
						</button>
					</div>
					{#if game.description}
						<p class="text-muted-foreground mb-2 line-clamp-2 text-sm">{game.description}</p>
					{/if}
					<div class="text-muted-foreground mb-3 flex gap-4 text-xs">
						{#if game.year_published}
							<span>{game.year_published}</span>
						{/if}
						{#if game.min_players && game.max_players}
							<span>{game.min_players}-{game.max_players} players</span>
						{/if}
					</div>
					<div class="flex gap-2">
						<Button variant="destructive" size="sm" onclick={() => deleteGame(game.id)}
							>Delete</Button
						>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
