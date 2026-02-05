<script lang="ts">
	import CategoryInput from './CategoryInput.svelte';
	import type { ScoringCategory, PlayerScoreInput } from '$lib';

	let {
		categories,
		players,
		onScoreChange
	}: {
		categories: ScoringCategory[];
		players: PlayerScoreInput[];
		onScoreChange: (playerIndex: number, categoryId: string, value: number) => void;
	} = $props();

	function handleScienceChange(
		playerIndex: number,
		tablets: number,
		compasses: number,
		gears: number
	) {
		onScoreChange(playerIndex, 'scienceTablets', tablets);
		onScoreChange(playerIndex, 'scienceCompasses', compasses);
		onScoreChange(playerIndex, 'scienceGears', gears);
	}
</script>

<div class="overflow-x-auto">
	<table class="w-full">
		<thead>
			<tr class="border-border border-b">
				<th class="text-muted-foreground px-2 py-3 text-left text-sm font-medium">Category</th>
				{#each players as player, i (i)}
					<th class="text-foreground min-w-[120px] px-2 py-3 text-center text-sm font-medium">
						{player.name}
					</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each categories as category (category.id)}
				<tr class="border-border border-b">
					<td class="text-foreground px-2 py-3 text-sm font-medium">
						{category.displayName}
					</td>
					{#each players as player, playerIndex (playerIndex)}
						<td class="px-2 py-3">
							<CategoryInput
								{category}
								value={player.scores[category.id] ?? 0}
								onChange={(value) => onScoreChange(playerIndex, category.id, value)}
								onScienceChange={(t, c, g) => handleScienceChange(playerIndex, t, c, g)}
							/>
						</td>
					{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>
