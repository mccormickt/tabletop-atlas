<script lang="ts">
	import type { ChallengeParticipant } from '$api/Api';
	import { Input } from '$lib/components/ui/input';

	let {
		participants,
		selectedParticipants,
		onToggleParticipant,
		onToggleWinner,
		onUpdateScore
	}: {
		participants: ChallengeParticipant[];
		selectedParticipants: Map<number, { isWinner: boolean; score: number | null }>;
		onToggleParticipant: (userId: number) => void;
		onToggleWinner: (userId: number) => void;
		onUpdateScore: (userId: number, score: number | null) => void;
	} = $props();
</script>

<div class="space-y-2">
	{#each participants as participant (participant.id)}
		{@const isSelected = selectedParticipants.has(participant.userId)}
		{@const data = selectedParticipants.get(participant.userId)}

		<div
			class="border-border rounded-lg border p-3 transition-colors {isSelected
				? 'bg-primary/5 border-primary'
				: 'bg-card'}"
		>
			<div class="flex items-center gap-3">
				<!-- Checkbox -->
				<label class="flex cursor-pointer items-center">
					<input
						type="checkbox"
						checked={isSelected}
						onchange={() => onToggleParticipant(participant.userId)}
						class="h-4 w-4 rounded border-gray-300"
					/>
				</label>

				<!-- Avatar -->
				{#if participant.pictureUrl}
					<img src={participant.pictureUrl} alt="" class="h-8 w-8 rounded-full" />
				{:else}
					<div
						class="bg-muted text-muted-foreground flex h-8 w-8 items-center justify-center rounded-full text-sm"
					>
						{(participant.displayName || 'U')[0].toUpperCase()}
					</div>
				{/if}

				<!-- Name -->
				<span class="text-foreground flex-1 text-sm font-medium">
					{participant.displayName || 'Unknown'}
				</span>

				<!-- Winner toggle (only if selected) -->
				{#if isSelected}
					<button
						type="button"
						class="rounded-full px-3 py-1 text-xs font-medium transition-colors {data?.isWinner
							? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
							: 'bg-muted text-muted-foreground hover:bg-muted/80'}"
						onclick={() => onToggleWinner(participant.userId)}
					>
						{data?.isWinner ? '\u{1F3C6} Winner' : 'Mark Winner'}
					</button>
				{/if}
			</div>

			<!-- Score input (only if selected) -->
			{#if isSelected}
				<div class="mt-2 flex items-center gap-2 pl-7">
					<label class="text-muted-foreground text-xs">Score:</label>
					<Input
						type="number"
						placeholder="Optional"
						value={data?.score ?? ''}
						onchange={(e) => {
							const val = (e.target as HTMLInputElement).value;
							onUpdateScore(participant.userId, val ? parseInt(val) : null);
						}}
						class="h-7 w-24 text-sm"
					/>
				</div>
			{/if}
		</div>
	{/each}
</div>
