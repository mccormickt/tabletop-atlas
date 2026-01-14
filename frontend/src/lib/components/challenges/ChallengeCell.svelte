<script lang="ts">
	import type { ChallengePlayWithParticipants, ChallengeParticipant } from '$api/Api';

	let {
		play,
		participants,
		disabled = false,
		onClick
	}: {
		play?: ChallengePlayWithParticipants;
		participants: ChallengeParticipant[];
		disabled?: boolean;
		onClick: () => void;
	} = $props();

	// Get winner info
	const winners = $derived(play?.participants.filter((p) => p.isWinner) || []);
	const hasWinner = $derived(winners.length > 0);
	const participantCount = $derived(play?.participants.length || 0);

	// Compute cell styling based on state
	const cellClass = $derived(() => {
		if (disabled) {
			return 'bg-muted/30 border-muted cursor-not-allowed';
		}
		if (!play) {
			return 'bg-card border-border hover:border-primary cursor-pointer';
		}
		if (hasWinner) {
			return 'cursor-pointer border-green-200 bg-green-50 hover:border-green-400 dark:border-green-800 dark:bg-green-900/20 dark:hover:border-green-600';
		}
		return 'cursor-pointer border-blue-200 bg-blue-50 hover:border-blue-400 dark:border-blue-800 dark:bg-blue-900/20 dark:hover:border-blue-600';
	});

	function getParticipantInfo(userId: number): ChallengeParticipant | undefined {
		return participants.find((p) => p.userId === userId);
	}
</script>

<button
	type="button"
	class="flex h-12 flex-1 items-center justify-center rounded-md border transition-colors {cellClass()}"
	onclick={onClick}
	{disabled}
>
	{#if play}
		{#if hasWinner}
			<!-- Show winner avatar(s) -->
			<div class="flex -space-x-2">
				{#each winners.slice(0, 3) as winner (winner.userId)}
					{@const info = getParticipantInfo(winner.userId)}
					{#if info?.pictureUrl}
						<img
							src={info.pictureUrl}
							alt={info.displayName || ''}
							class="h-6 w-6 rounded-full border-2 border-white dark:border-gray-800"
							title={info.displayName || 'Winner'}
						/>
					{:else}
						<div
							class="flex h-6 w-6 items-center justify-center rounded-full border-2 border-white bg-green-500 text-xs text-white dark:border-gray-800"
							title={info?.displayName || 'Winner'}
						>
							{(info?.displayName || 'W')[0].toUpperCase()}
						</div>
					{/if}
				{/each}
				{#if winners.length > 3}
					<div
						class="flex h-6 w-6 items-center justify-center rounded-full border-2 border-white bg-gray-200 text-xs dark:border-gray-800 dark:bg-gray-600"
					>
						+{winners.length - 3}
					</div>
				{/if}
			</div>
		{:else}
			<!-- Played but no winner marked -->
			<div class="text-muted-foreground text-xs">
				{participantCount}
				{participantCount === 1 ? 'player' : 'players'}
			</div>
		{/if}
	{:else if !disabled}
		<span class="text-muted-foreground text-lg">+</span>
	{/if}
</button>
