<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { useAuth, type AuthState } from '$lib/stores/auth';
	import { getStatusColor } from '$lib';
	import { Button } from '$lib/components/ui/button';
	import ChallengeGrid from '$lib/components/challenges/ChallengeGrid.svelte';
	import Leaderboard from '$lib/components/challenges/Leaderboard.svelte';
	import CellEditModal from '$lib/components/challenges/CellEditModal.svelte';
	import GamePicker from '$lib/components/challenges/GamePicker.svelte';
	import type { ChallengeGridView, ChallengeGame, ChallengePlayWithParticipants } from '$api/Api';

	const auth = useAuth();

	let authState = $state<AuthState>({ user: null, isLoading: true, error: null });
	let gridData = $state<ChallengeGridView | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	// Modal states
	let showCellEditModal = $state(false);
	let editingCell = $state<{
		game: ChallengeGame;
		colIndex: number;
		play?: ChallengePlayWithParticipants;
	} | null>(null);
	let showGamePicker = $state(false);
	let selectedRowIndex = $state<number | null>(null);

	$effect(() => {
		const unsubscribe = auth.subscribe((state) => {
			authState = state;
			if (!state.isLoading && !state.user) {
				goto('/auth/login');
			} else if (state.user && $page.params.id) {
				loadGrid();
			}
		});
		return unsubscribe;
	});

	async function loadGrid() {
		isLoading = true;
		error = null;
		try {
			const response = await fetch(`/api/challenges/${$page.params.id}/grid`, {
				credentials: 'include'
			});
			if (response.ok) {
				gridData = await response.json();
			} else if (response.status === 401) {
				goto('/auth/login');
			} else if (response.status === 403) {
				error = 'You are not a participant in this challenge';
			} else if (response.status === 404) {
				error = 'Challenge not found';
			} else {
				error = 'Failed to load challenge';
			}
		} catch {
			error = 'Failed to load challenge';
		} finally {
			isLoading = false;
		}
	}

	function handleCellClick(game: ChallengeGame, colIndex: number) {
		const existingPlay = gridData?.plays.find(
			(p) => p.challengeGameId === game.id && p.colIndex === colIndex
		);
		editingCell = { game, colIndex, play: existingPlay };
		showCellEditModal = true;
	}

	function handleAssignGame(rowIndex: number) {
		selectedRowIndex = rowIndex;
		showGamePicker = true;
	}

	async function handleGameSelected(gameType: string, gameId: number, displayName: string) {
		if (selectedRowIndex === null || !gridData) return;

		try {
			const response = await fetch(`/api/challenges/${gridData.challenge.id}/games`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({
					rowIndex: selectedRowIndex,
					gameType,
					gameId,
					displayName
				})
			});

			if (response.ok) {
				await loadGrid();
			}
		} catch (e) {
			console.error('Failed to assign game:', e);
		}

		showGamePicker = false;
		selectedRowIndex = null;
	}

	async function closeModalAndRefresh() {
		showCellEditModal = false;
		editingCell = null;
		await loadGrid();
	}
</script>

<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	{#if authState.isLoading || isLoading}
		<div class="flex justify-center py-12">
			<div
				class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
			></div>
		</div>
	{:else if error}
		<div class="bg-destructive/10 border-destructive rounded-lg border p-4 text-center">
			<p class="text-destructive">{error}</p>
			<div class="mt-4 flex justify-center gap-4">
				<Button variant="outline" onclick={loadGrid}>Retry</Button>
				<Button href="/challenges">Back to Challenges</Button>
			</div>
		</div>
	{:else if gridData}
		<div class="mb-8">
			<a href="/challenges" class="text-muted-foreground hover:text-foreground text-sm">
				&larr; Back to Challenges
			</a>
			<div class="mt-4 flex items-start justify-between">
				<div>
					<div class="flex items-center gap-3">
						<h1 class="text-foreground text-3xl font-bold">{gridData.challenge.name}</h1>
						<span
							class="rounded-full px-2 py-1 text-xs font-medium {getStatusColor(
								gridData.challenge.status
							)}"
						>
							{gridData.challenge.status}
						</span>
					</div>
					{#if gridData.challenge.description}
						<p class="text-muted-foreground mt-2">{gridData.challenge.description}</p>
					{/if}
				</div>
				<div class="flex gap-2">
					<Button variant="outline" href="/challenges/{gridData.challenge.id}/stats">
						View Stats
					</Button>
				</div>
			</div>
		</div>

		<!-- Progress -->
		<div class="bg-card border-border mb-8 rounded-lg border p-4">
			<div class="mb-2 flex items-center justify-between">
				<span class="text-muted-foreground text-sm">Progress</span>
				<span class="text-foreground font-medium">
					{gridData.stats.completedCells ?? 0} / {gridData.stats.totalCells ?? 0} sessions
				</span>
			</div>
			<div class="bg-muted h-3 overflow-hidden rounded-full">
				<div
					class="bg-primary h-full transition-all"
					style="width: {Math.min(gridData.stats.completionPercentage ?? 0, 100)}%"
				></div>
			</div>
			<p class="text-muted-foreground mt-1 text-xs">
				{(gridData.stats.completionPercentage ?? 0).toFixed(1)}% complete
			</p>
		</div>

		<div class="grid gap-8 lg:grid-cols-4">
			<!-- Grid -->
			<div class="lg:col-span-3">
				<ChallengeGrid
					challenge={gridData.challenge}
					games={gridData.games}
					plays={gridData.plays}
					participants={gridData.participants}
					onCellClick={handleCellClick}
					onAssignGame={handleAssignGame}
				/>
			</div>

			<!-- Sidebar -->
			<div class="space-y-6">
				<!-- Leaderboard -->
				<div class="bg-card border-border rounded-lg border p-4">
					<h3 class="text-foreground mb-4 font-semibold">Leaderboard</h3>
					<Leaderboard entries={gridData.stats.leaderboard} />
				</div>

				<!-- Participants -->
				<div class="bg-card border-border rounded-lg border p-4">
					<h3 class="text-foreground mb-4 font-semibold">
						Participants ({gridData.participants.length})
					</h3>
					<div class="space-y-2">
						{#each gridData.participants as participant (participant.id)}
							<div class="flex items-center gap-2">
								{#if participant.pictureUrl}
									<img src={participant.pictureUrl} alt="" class="h-8 w-8 rounded-full" />
								{:else}
									<div
										class="bg-muted text-muted-foreground flex h-8 w-8 items-center justify-center rounded-full text-sm"
									>
										{(participant.displayName || 'U')[0].toUpperCase()}
									</div>
								{/if}
								<span class="text-foreground text-sm">
									{participant.displayName || 'Unknown'}
								</span>
								{#if participant.role === 'owner'}
									<span class="text-muted-foreground text-xs">(Owner)</span>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			</div>
		</div>
	{/if}
</div>

<!-- Cell Edit Modal -->
{#if showCellEditModal && editingCell && gridData}
	<CellEditModal
		challengeId={gridData.challenge.id}
		game={editingCell.game}
		colIndex={editingCell.colIndex}
		existingPlay={editingCell.play}
		participants={gridData.participants}
		onSave={closeModalAndRefresh}
		onDelete={closeModalAndRefresh}
		onClose={() => {
			showCellEditModal = false;
			editingCell = null;
		}}
	/>
{/if}

<!-- Game Picker Modal -->
{#if showGamePicker && selectedRowIndex !== null}
	<GamePicker
		onSelect={handleGameSelected}
		onClose={() => {
			showGamePicker = false;
			selectedRowIndex = null;
		}}
	/>
{/if}
