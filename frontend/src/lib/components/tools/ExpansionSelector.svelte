<script lang="ts">
	import type { Expansion } from '$lib';

	let {
		expansions,
		enabledExpansions = $bindable([])
	}: {
		expansions: Expansion[];
		enabledExpansions: string[];
	} = $props();

	function toggleExpansion(id: string) {
		if (enabledExpansions.includes(id)) {
			enabledExpansions = enabledExpansions.filter((e) => e !== id);
		} else {
			enabledExpansions = [...enabledExpansions, id];
		}
	}
</script>

<div>
	<label class="text-foreground mb-2 block text-sm font-medium">Expansions</label>
	<div class="flex flex-wrap gap-2">
		{#each expansions as expansion (expansion.id)}
			<button
				type="button"
				class="rounded-full border px-3 py-1 text-sm font-medium transition-colors
					{enabledExpansions.includes(expansion.id)
					? 'border-primary bg-primary text-primary-foreground'
					: 'border-border bg-muted text-muted-foreground hover:border-primary'}"
				onclick={() => toggleExpansion(expansion.id)}
			>
				{expansion.displayName}
			</button>
		{/each}
	</div>
</div>
