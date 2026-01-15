<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import PlayerSetup from './PlayerSetup.svelte';
	import ExpansionSelector from './ExpansionSelector.svelte';
	import ScoringGrid from './ScoringGrid.svelte';
	import ScoreSummary from './ScoreSummary.svelte';
	import type { ToolDetails, ScoreInput, ScoreOutput, PlayerScoreInput } from '$api/Api';

	let {
		tool,
		result,
		isCalculating,
		onCalculate,
		onReset
	}: {
		tool: ToolDetails;
		result: ScoreOutput | null;
		isCalculating: boolean;
		onCalculate: (input: ScoreInput) => Promise<void>;
		onReset: () => void;
	} = $props();

	// Phase: setup, scoring, or results
	let phase = $state<'setup' | 'scoring' | 'results'>('setup');
	let players = $state<PlayerScoreInput[]>([]);
	let enabledExpansions = $state<string[]>([]);

	// When result changes, switch to results phase
	$effect(() => {
		if (result) {
			phase = 'results';
		}
	});

	function handlePlayersSetup(playerNames: string[]) {
		players = playerNames.map((name) => ({
			name,
			scores: {}
		}));
		phase = 'scoring';
	}

	function handleScoreChange(playerIndex: number, categoryId: string, value: number) {
		players[playerIndex].scores[categoryId] = value;
	}

	async function handleCalculate() {
		await onCalculate({
			players,
			enabledExpansions
		});
	}

	function handleNewGame() {
		players = [];
		enabledExpansions = [];
		phase = 'setup';
		onReset();
	}

	function handleBackToScoring() {
		phase = 'scoring';
		onReset();
	}

	// Filter categories based on enabled expansions
	let activeCategories = $derived(
		tool.schema.categories.filter(
			(cat) => !cat.requiresExpansion || enabledExpansions.includes(cat.requiresExpansion)
		)
	);
</script>

<div class="bg-card border-border rounded-lg border p-6">
	{#if phase === 'setup'}
		<div class="space-y-6">
			<h2 class="text-foreground text-xl font-semibold">Game Setup</h2>

			{#if tool.schema.expansions.length > 0}
				<ExpansionSelector expansions={tool.schema.expansions} bind:enabledExpansions />
			{/if}

			<PlayerSetup
				minPlayers={tool.playerRange.min}
				maxPlayers={tool.playerRange.max}
				onSetup={handlePlayersSetup}
			/>
		</div>
	{:else if phase === 'scoring'}
		<div class="space-y-6">
			<div class="flex items-center justify-between">
				<h2 class="text-foreground text-xl font-semibold">Enter Scores</h2>
				<Button variant="outline" onclick={() => (phase = 'setup')}>← Back to Setup</Button>
			</div>

			<ScoringGrid categories={activeCategories} {players} onScoreChange={handleScoreChange} />

			<div class="flex justify-end">
				<Button onclick={handleCalculate} disabled={isCalculating}>
					{isCalculating ? 'Calculating...' : 'Calculate Scores'}
				</Button>
			</div>
		</div>
	{:else if phase === 'results' && result}
		<div class="space-y-6">
			<div class="flex items-center justify-between">
				<h2 class="text-foreground text-xl font-semibold">Results</h2>
			</div>

			<ScoreSummary {result} categories={activeCategories} />

			<div class="flex justify-end gap-2">
				<Button variant="outline" onclick={handleBackToScoring}>Edit Scores</Button>
				<Button onclick={handleNewGame}>New Game</Button>
			</div>
		</div>
	{/if}
</div>
