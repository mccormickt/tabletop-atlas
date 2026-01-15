<script lang="ts">
	import type { ScoreOutput, ScoringCategory } from '$api/Api';

	let {
		result,
		categories
	}: {
		result: ScoreOutput;
		categories: ScoringCategory[];
	} = $props();

	// Sort players by total score (highest first)
	let sortedPlayers = $derived(
		[...result.players]
			.map((player, originalIndex) => ({ ...player, originalIndex }))
			.sort((a, b) => b.total - a.total)
	);
</script>

<div class="space-y-4">
	<!-- Winner announcement -->
	{#if result.winnerIndex !== null && result.winnerIndex !== undefined}
		<div class="bg-game-blue/10 border-game-blue rounded-lg border p-4 text-center">
			<span class="text-2xl">🏆</span>
			<h3 class="text-game-blue mt-2 text-xl font-bold">
				{result.players[result.winnerIndex].name} Wins!
			</h3>
			<p class="text-muted-foreground">
				with {result.players[result.winnerIndex].total} points
			</p>
		</div>
	{/if}

	<!-- Score breakdown table -->
	<div class="overflow-x-auto">
		<table class="w-full">
			<thead>
				<tr class="border-border border-b">
					<th class="text-muted-foreground px-2 py-3 text-left text-sm font-medium">Category</th>
					{#each sortedPlayers as player, i (player.originalIndex)}
						<th
							class="min-w-[100px] px-2 py-3 text-center text-sm font-medium
							{i === 0 ? 'text-game-blue' : 'text-foreground'}"
						>
							{#if i === 0}🏆{/if}
							{player.name}
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each categories as category (category.id)}
					<tr class="border-border border-b">
						<td class="text-foreground px-2 py-3 text-sm">{category.displayName}</td>
						{#each sortedPlayers as player (player.originalIndex)}
							<td class="text-foreground px-2 py-3 text-center text-sm">
								{player.categoryScores[category.id] ?? 0}
							</td>
						{/each}
					</tr>
				{/each}
				<!-- Total row -->
				<tr class="bg-muted/50 font-bold">
					<td class="text-foreground px-2 py-3 text-sm">Total</td>
					{#each sortedPlayers as player, i (player.originalIndex)}
						<td
							class="px-2 py-3 text-center text-sm
							{i === 0 ? 'text-game-blue' : 'text-foreground'}"
						>
							{player.total}
						</td>
					{/each}
				</tr>
			</tbody>
		</table>
	</div>
</div>
