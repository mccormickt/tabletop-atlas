<script lang="ts">
	import type { ChallengeSummary } from '$api/Api';
	import { formatDate, getStatusColor } from '$lib';

	let { challenge }: { challenge: ChallengeSummary } = $props();
</script>

<a
	href="/challenges/{challenge.id}"
	class="bg-card border-border hover:border-primary block rounded-lg border p-4 transition-colors"
>
	<div class="mb-2 flex items-start justify-between">
		<h3 class="text-foreground font-semibold">{challenge.name}</h3>
		<span class="rounded-full px-2 py-1 text-xs font-medium {getStatusColor(challenge.status)}">
			{challenge.status}
		</span>
	</div>

	{#if challenge.description}
		<p class="text-muted-foreground mb-3 line-clamp-2 text-sm">{challenge.description}</p>
	{/if}

	<div class="mb-3">
		<div class="bg-muted h-2 overflow-hidden rounded-full">
			<div
				class="bg-primary h-full transition-all"
				style="width: {Math.min(challenge.completionPercentage ?? 0, 100)}%"
			></div>
		</div>
		<p class="text-muted-foreground mt-1 text-xs">
			{(challenge.completionPercentage ?? 0).toFixed(0)}% complete
		</p>
	</div>

	<div class="text-muted-foreground flex items-center justify-between text-sm">
		<span>
			{challenge.gridRows ?? 0}x{challenge.gridCols ?? 0} grid
		</span>
		<span>
			{challenge.participantCount ?? 0}
			{(challenge.participantCount ?? 0) === 1 ? 'player' : 'players'}
		</span>
	</div>

	{#if challenge.startDate || challenge.endDate}
		<div class="text-muted-foreground mt-2 text-xs">
			{#if challenge.startDate}
				<span>Started: {formatDate(challenge.startDate)}</span>
			{/if}
			{#if challenge.startDate && challenge.endDate}
				<span class="mx-1">•</span>
			{/if}
			{#if challenge.endDate}
				<span>Ends: {formatDate(challenge.endDate)}</span>
			{/if}
		</div>
	{/if}
</a>
