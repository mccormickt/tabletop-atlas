<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';

	let {
		minPlayers,
		maxPlayers,
		onSetup
	}: {
		minPlayers: number;
		maxPlayers: number;
		onSetup: (players: string[]) => void;
	} = $props();

	let playerCount = $state(Math.max(minPlayers, 2));
	let playerNames = $state<string[]>(Array(playerCount).fill(''));

	$effect(() => {
		// Resize array when count changes
		if (playerNames.length !== playerCount) {
			const newNames = [...playerNames];
			if (playerCount > playerNames.length) {
				for (let i = playerNames.length; i < playerCount; i++) {
					newNames.push(`Player ${i + 1}`);
				}
			} else {
				newNames.length = playerCount;
			}
			playerNames = newNames;
		}
	});

	function handleSubmit() {
		// Use default names for empty inputs
		const names = playerNames.map((name, i) => name.trim() || `Player ${i + 1}`);
		onSetup(names);
	}

	let canSubmit = $derived(playerCount >= minPlayers && playerCount <= maxPlayers);
</script>

<div class="space-y-4">
	<div>
		<label class="text-foreground mb-2 block text-sm font-medium">Number of Players</label>
		<div class="flex items-center gap-2">
			<Button
				variant="outline"
				size="sm"
				disabled={playerCount <= minPlayers}
				onclick={() => (playerCount = Math.max(minPlayers, playerCount - 1))}
			>
				-
			</Button>
			<span class="text-foreground w-8 text-center font-semibold">{playerCount}</span>
			<Button
				variant="outline"
				size="sm"
				disabled={playerCount >= maxPlayers}
				onclick={() => (playerCount = Math.min(maxPlayers, playerCount + 1))}
			>
				+
			</Button>
			<span class="text-muted-foreground ml-2 text-sm">
				({minPlayers}-{maxPlayers} players)
			</span>
		</div>
	</div>

	<div class="space-y-2">
		<label class="text-foreground mb-2 block text-sm font-medium">Player Names</label>
		{#each Array.from({ length: playerCount }, (_, i) => i) as i (i)}
			<Input type="text" placeholder={`Player ${i + 1}`} bind:value={playerNames[i]} />
		{/each}
	</div>

	<Button onclick={handleSubmit} disabled={!canSubmit} class="w-full">Start Scoring</Button>
</div>
