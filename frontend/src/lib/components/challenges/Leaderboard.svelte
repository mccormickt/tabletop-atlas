<script lang="ts">
	import type { LeaderboardEntry } from '$api/Api';

	let { entries }: { entries: LeaderboardEntry[] } = $props();

	function getMedalEmoji(position: number): string {
		switch (position) {
			case 0:
				return '\u{1F947}'; // Gold medal
			case 1:
				return '\u{1F948}'; // Silver medal
			case 2:
				return '\u{1F949}'; // Bronze medal
			default:
				return '';
		}
	}
</script>

{#if entries.length === 0}
	<p class="text-muted-foreground text-sm">No plays recorded yet</p>
{:else}
	<div class="space-y-3">
		{#each entries as entry, index (entry.userId)}
			<div class="flex items-center gap-3">
				<div class="w-6 text-center">
					{#if index < 3}
						<span class="text-lg">{getMedalEmoji(index)}</span>
					{:else}
						<span class="text-muted-foreground text-sm">{index + 1}</span>
					{/if}
				</div>

				{#if entry.pictureUrl}
					<img src={entry.pictureUrl} alt="" class="h-8 w-8 rounded-full" />
				{:else}
					<div
						class="bg-muted text-muted-foreground flex h-8 w-8 items-center justify-center rounded-full text-sm"
					>
						{(entry.displayName || 'U')[0].toUpperCase()}
					</div>
				{/if}

				<div class="min-w-0 flex-1">
					<p class="text-foreground truncate text-sm font-medium">
						{entry.displayName || 'Unknown'}
					</p>
					<p class="text-muted-foreground text-xs">
						{entry.wins} win{entry.wins === 1 ? '' : 's'} • {(entry.winPercentage ?? 0).toFixed(0)}%
					</p>
				</div>

				<div class="text-right">
					<p class="text-foreground text-sm font-semibold">{entry.wins}</p>
					<p class="text-muted-foreground text-xs">{entry.totalPlays} games</p>
				</div>
			</div>
		{/each}
	</div>
{/if}
