<script lang="ts">
	import type { ChallengeGame } from '$api/Api';
	import { Button } from '$lib/components/ui/button';

	let {
		game,
		rowIndex,
		onAssign
	}: {
		game?: ChallengeGame;
		rowIndex: number;
		onAssign: () => void;
	} = $props();
</script>

<div class="bg-card border-border flex h-12 items-center rounded-md border px-3">
	{#if game}
		<div class="min-w-0 flex-1">
			<p
				class="text-foreground truncate text-sm font-medium"
				title={game.displayName || `Game ${game.gameId}`}
			>
				{game.displayName || `Game ${game.gameId}`}
			</p>
			<p class="text-muted-foreground text-xs capitalize">
				{game.gameType}
			</p>
		</div>
		<button
			type="button"
			class="text-muted-foreground hover:text-foreground ml-2 text-xs"
			onclick={onAssign}
		>
			Change
		</button>
	{:else}
		<Button variant="ghost" size="sm" class="w-full justify-start" onclick={onAssign}>
			<span class="text-muted-foreground">+ Add game to row {rowIndex + 1}</span>
		</Button>
	{/if}
</div>
