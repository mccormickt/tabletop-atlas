<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';

	interface Game {
		id: number;
		name: string;
		type: 'master' | 'custom' | 'collection';
	}

	let {
		onSelect,
		onClose
	}: {
		onSelect: (gameType: string, gameId: number, displayName: string) => void;
		onClose: () => void;
	} = $props();

	let searchQuery = $state('');
	let games = $state<Game[]>([]);
	let isLoading = $state(true);
	let error = $state<string | null>(null);
	let activeTab = $state<'master' | 'collection' | 'custom'>('master');

	async function loadGames() {
		isLoading = true;
		error = null;

		try {
			// Load games based on active tab
			let response;
			if (activeTab === 'master') {
				response = await fetch('/api/games?limit=100', { credentials: 'include' });
			} else if (activeTab === 'collection') {
				response = await fetch('/api/collection?limit=100', { credentials: 'include' });
			} else {
				response = await fetch('/api/custom-games?limit=100', { credentials: 'include' });
			}

			if (response.ok) {
				const data = await response.json();
				const items = data.items || [];
				games = items.map((item: Record<string, unknown>) => ({
					id: item.id || item.masterGameId || item.gameId,
					name: item.name || item.gameName || item.game_name || `Game ${item.id}`,
					type: activeTab
				}));
			} else {
				error = 'Failed to load games';
			}
		} catch {
			error = 'Failed to load games';
		} finally {
			isLoading = false;
		}
	}

	// Load games on mount and when tab changes
	$effect(() => {
		// Access activeTab to create reactive dependency
		const currentTab = activeTab;
		void currentTab;
		loadGames();
	});

	const filteredGames = $derived(
		games.filter((g) => g.name.toLowerCase().includes(searchQuery.toLowerCase()))
	);

	function handleSelect(game: Game) {
		onSelect(game.type, game.id, game.name);
	}
</script>

<!-- Modal backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
	onclick={(e) => e.target === e.currentTarget && onClose()}
>
	<div
		class="bg-background flex max-h-[80vh] w-full max-w-lg flex-col rounded-lg shadow-lg"
		onclick={(e) => e.stopPropagation()}
	>
		<div class="border-border border-b p-4">
			<div class="flex items-start justify-between">
				<h2 class="text-foreground text-lg font-semibold">Select a Game</h2>
				<button type="button" class="text-muted-foreground hover:text-foreground" onclick={onClose}>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						viewBox="0 0 20 20"
						fill="currentColor"
					>
						<path
							fill-rule="evenodd"
							d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
							clip-rule="evenodd"
						/>
					</svg>
				</button>
			</div>

			<!-- Tabs -->
			<div class="mt-4 flex gap-2">
				<button
					type="button"
					class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors {activeTab ===
					'master'
						? 'bg-primary text-primary-foreground'
						: 'bg-muted text-muted-foreground hover:text-foreground'}"
					onclick={() => (activeTab = 'master')}
				>
					All Games
				</button>
				<button
					type="button"
					class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors {activeTab ===
					'collection'
						? 'bg-primary text-primary-foreground'
						: 'bg-muted text-muted-foreground hover:text-foreground'}"
					onclick={() => (activeTab = 'collection')}
				>
					My Collection
				</button>
				<button
					type="button"
					class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors {activeTab ===
					'custom'
						? 'bg-primary text-primary-foreground'
						: 'bg-muted text-muted-foreground hover:text-foreground'}"
					onclick={() => (activeTab = 'custom')}
				>
					Custom Games
				</button>
			</div>

			<!-- Search -->
			<div class="mt-4">
				<Input type="search" placeholder="Search games..." bind:value={searchQuery} />
			</div>
		</div>

		<div class="flex-1 overflow-y-auto p-4">
			{#if isLoading}
				<div class="flex justify-center py-8">
					<div
						class="border-game-blue h-6 w-6 animate-spin rounded-full border-4 border-t-transparent"
					></div>
				</div>
			{:else if error}
				<div class="py-8 text-center">
					<p class="text-destructive text-sm">{error}</p>
					<Button variant="outline" size="sm" class="mt-2" onclick={loadGames}>Retry</Button>
				</div>
			{:else if filteredGames.length === 0}
				<p class="text-muted-foreground py-8 text-center text-sm">
					{searchQuery ? 'No games match your search' : 'No games available'}
				</p>
			{:else}
				<div class="space-y-1">
					{#each filteredGames as game (game.id)}
						<button
							type="button"
							class="bg-card border-border hover:border-primary hover:bg-muted/50 w-full rounded-lg border p-3 text-left transition-colors"
							onclick={() => handleSelect(game)}
						>
							<p class="text-foreground font-medium">{game.name}</p>
						</button>
					{/each}
				</div>
			{/if}
		</div>

		<div class="border-border border-t p-4">
			<Button variant="outline" class="w-full" onclick={onClose}>Cancel</Button>
		</div>
	</div>
</div>
