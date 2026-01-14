<script lang="ts">
	import CounterInput from './CounterInput.svelte';

	let {
		onScienceChange
	}: { onScienceChange?: (tablets: number, compasses: number, gears: number) => void } = $props();

	let tablets = $state(0);
	let compasses = $state(0);
	let gears = $state(0);

	// Emit individual science values whenever they change
	$effect(() => {
		onScienceChange?.(tablets, compasses, gears);
	});

	// Preview score calculation
	let previewScore = $derived.by(() => {
		const sets = Math.min(tablets, compasses, gears);
		return sets * 7 + tablets * tablets + compasses * compasses + gears * gears;
	});
</script>

<div class="space-y-2">
	<div class="flex items-center gap-2">
		<span class="text-foreground w-20 text-sm">📜 Tablets</span>
		<CounterInput value={tablets} onChange={(v) => (tablets = v)} />
	</div>
	<div class="flex items-center gap-2">
		<span class="text-foreground w-20 text-sm">🧭 Compasses</span>
		<CounterInput value={compasses} onChange={(v) => (compasses = v)} />
	</div>
	<div class="flex items-center gap-2">
		<span class="text-foreground w-20 text-sm">⚙️ Gears</span>
		<CounterInput value={gears} onChange={(v) => (gears = v)} />
	</div>
	<div class="text-muted-foreground border-border border-t pt-2 text-xs">
		Preview: {previewScore} pts
	</div>
</div>
