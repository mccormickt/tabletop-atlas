<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { createAuthState } from '$lib/stores/auth.svelte';
	import { Button } from '$lib/components/ui/button';
	import { EmptyState } from '$lib/components/ui/empty-state';

	interface CollectionEntry {
		id: number;
		game_name: string;
		rating: number | null;
		play_count: number;
		notes: string | null;
	}

	const auth = createAuthState();

	let collection = $state<CollectionEntry[]>([]);
	let isLoadingCollection = $state(true);
	let error = $state<string | null>(null);

	$effect(() => {
		if (!auth.isLoading && !auth.user) {
			goto(resolve('/auth/login'));
		} else if (auth.user) {
			loadCollection();
		}
	});

	async function loadCollection() {
		isLoadingCollection = true;
		error = null;
		try {
			const response = await fetch('/api/collection', {
				credentials: 'include'
			});
			if (response.ok) {
				const data = await response.json();
				collection = data.items || [];
			} else if (response.status === 401) {
				goto(resolve('/auth/login'));
			} else {
				error = 'Failed to load collection';
			}
		} catch {
			error = 'Failed to load collection';
		} finally {
			isLoadingCollection = false;
		}
	}

	async function removeFromCollection(entryId: number) {
		try {
			const response = await fetch(`/api/collection/${entryId}`, {
				method: 'DELETE',
				credentials: 'include'
			});
			if (response.ok) {
				collection = collection.filter((e) => e.id !== entryId);
			}
		} catch (e) {
			console.error('Failed to remove from collection:', e);
		}
	}
</script>

<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<div class="mb-8">
		<h1 class="text-foreground text-3xl font-bold">My Collection</h1>
		<p class="text-muted-foreground mt-2">Games you own or want to track</p>
	</div>

	{#if auth.isLoading || isLoadingCollection}
		<div class="flex justify-center py-12">
			<div
				class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
			></div>
		</div>
	{:else if error}
		<div class="bg-destructive/10 border-destructive rounded-lg border p-4 text-center">
			<p class="text-destructive">{error}</p>
			<Button variant="outline" class="mt-4" onclick={loadCollection}>Retry</Button>
		</div>
	{:else if collection.length === 0}
		<EmptyState
			title="Your collection is empty"
			description="Start adding games to your collection from the Games page."
			icon="game"
			actionText="Browse Games"
			onAction={() => goto(resolve('/games'))}
		/>
	{:else}
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{#each collection as entry (entry.id)}
				<div class="bg-card border-border rounded-lg border p-4">
					<div class="mb-2 flex items-start justify-between">
						<h3 class="text-foreground font-semibold">{entry.game_name}</h3>
						<button
							onclick={() => removeFromCollection(entry.id)}
							class="text-muted-foreground hover:text-destructive text-sm"
						>
							Remove
						</button>
					</div>
					{#if entry.rating}
						<p class="text-muted-foreground text-sm">Rating: {entry.rating}/10</p>
					{/if}
					{#if entry.play_count > 0}
						<p class="text-muted-foreground text-sm">Plays: {entry.play_count}</p>
					{/if}
					{#if entry.notes}
						<p class="text-muted-foreground mt-2 text-sm">{entry.notes}</p>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</div>
