<script lang="ts">
	import type {
		Challenge,
		ChallengeGame,
		ChallengePlayWithParticipants,
		ChallengeParticipant
	} from '$lib';
	import ChallengeCell from './ChallengeCell.svelte';
	import GameRow from './GameRow.svelte';

	let {
		challenge,
		games,
		plays,
		participants,
		onCellClick,
		onAssignGame
	}: {
		challenge: Challenge;
		games: ChallengeGame[];
		plays: ChallengePlayWithParticipants[];
		participants: ChallengeParticipant[];
		onCellClick: (game: ChallengeGame, colIndex: number) => void;
		onAssignGame: (rowIndex: number) => void;
	} = $props();

	// Build a map for quick lookup
	function getGameForRow(rowIndex: number): ChallengeGame | undefined {
		return games.find((g) => g.rowIndex === rowIndex);
	}

	function getPlayForCell(
		gameId: number,
		colIndex: number
	): ChallengePlayWithParticipants | undefined {
		return plays.find((p) => p.challengeGameId === gameId && p.colIndex === colIndex);
	}

	// Generate row and column indices
	const rowIndices = $derived(
		Array.from({ length: challenge.gridRows }, (_unused, i: number) => i)
	);
	const colIndices = $derived(
		Array.from({ length: challenge.gridCols }, (_unused, i: number) => i)
	);
</script>

<div class="overflow-x-auto">
	<div class="min-w-[600px]">
		<!-- Column headers -->
		<div class="mb-2 flex">
			<div class="w-48 shrink-0"></div>
			<div class="flex flex-1 gap-1">
				{#each colIndices as colNum (colNum)}
					<div
						class="text-muted-foreground flex h-8 flex-1 items-center justify-center text-xs font-medium"
					>
						#{colNum + 1}
					</div>
				{/each}
			</div>
		</div>

		<!-- Grid rows -->
		<div class="space-y-1">
			{#each rowIndices as rowIndex (rowIndex)}
				{@const game = getGameForRow(rowIndex)}
				<div class="flex gap-1">
					<!-- Row header (game info) -->
					<div class="w-48 shrink-0">
						<GameRow {game} {rowIndex} onAssign={() => onAssignGame(rowIndex)} />
					</div>

					<!-- Cells -->
					<div class="flex flex-1 gap-1">
						{#each colIndices as colIndex (colIndex)}
							{@const play = game ? getPlayForCell(game.id, colIndex) : undefined}
							<ChallengeCell
								{play}
								{participants}
								disabled={!game}
								onClick={() => game && onCellClick(game, colIndex)}
							/>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	</div>
</div>
