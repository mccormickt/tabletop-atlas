<script lang="ts">
	import type { ChallengeGame, ChallengePlayWithParticipants, ChallengeParticipant } from '$lib';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import { SvelteMap } from 'svelte/reactivity';
	import WinnerSelector from './WinnerSelector.svelte';

	let {
		challengeId,
		game,
		colIndex,
		existingPlay,
		participants,
		onSave,
		onDelete,
		onClose
	}: {
		challengeId: number;
		game: ChallengeGame;
		colIndex: number;
		existingPlay?: ChallengePlayWithParticipants;
		participants: ChallengeParticipant[];
		onSave: () => void;
		onDelete: () => void;
		onClose: () => void;
	} = $props();

	let isSubmitting = $state(false);
	let isDeleting = $state(false);
	let error = $state<string | null>(null);

	// Form state
	let playedAt = $state(existingPlay?.playedAt || new Date().toISOString().split('T')[0]);
	let notes = $state(existingPlay?.notes || '');
	let selectedParticipants = new SvelteMap<number, { isWinner: boolean; score: number | null }>(
		existingPlay?.participants.map((p) => [
			p.userId,
			{ isWinner: p.isWinner, score: p.score ?? null }
		]) || []
	);

	const isEditing = $derived(!!existingPlay);

	function toggleParticipant(userId: number) {
		const newMap = new SvelteMap(selectedParticipants);
		if (newMap.has(userId)) {
			newMap.delete(userId);
		} else {
			newMap.set(userId, { isWinner: false, score: null });
		}
		selectedParticipants = newMap;
	}

	function toggleWinner(userId: number) {
		const current = selectedParticipants.get(userId);
		if (current) {
			const newMap = new SvelteMap(selectedParticipants);
			newMap.set(userId, { ...current, isWinner: !current.isWinner });
			selectedParticipants = newMap;
		}
	}

	function updateScore(userId: number, score: number | null) {
		const current = selectedParticipants.get(userId);
		if (current) {
			const newMap = new SvelteMap(selectedParticipants);
			newMap.set(userId, { ...current, score });
			selectedParticipants = newMap;
		}
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();

		if (selectedParticipants.size === 0) {
			error = 'Please select at least one participant';
			return;
		}

		isSubmitting = true;
		error = null;

		const participantsData = Array.from(selectedParticipants.entries()).map(([userId, data]) => ({
			userId,
			isWinner: data.isWinner,
			score: data.score
		}));

		try {
			let response;
			if (isEditing && existingPlay) {
				response = await fetch(`/api/challenges/${challengeId}/plays/${existingPlay.id}`, {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					credentials: 'include',
					body: JSON.stringify({
						playedAt,
						notes: notes.trim() || null,
						participants: participantsData
					})
				});
			} else {
				response = await fetch(`/api/challenges/${challengeId}/plays`, {
					method: 'POST',
					headers: { 'Content-Type': 'application/json' },
					credentials: 'include',
					body: JSON.stringify({
						challengeGameId: game.id,
						colIndex,
						playedAt,
						notes: notes.trim() || null,
						participants: participantsData
					})
				});
			}

			if (response.ok) {
				onSave();
			} else {
				const data = await response.json();
				error = data.message || 'Failed to save play';
			}
		} catch {
			error = 'Failed to save play';
		} finally {
			isSubmitting = false;
		}
	}

	async function handleDelete() {
		if (!existingPlay || !confirm('Are you sure you want to delete this play session?')) {
			return;
		}

		isDeleting = true;
		error = null;

		try {
			const response = await fetch(`/api/challenges/${challengeId}/plays/${existingPlay.id}`, {
				method: 'DELETE',
				credentials: 'include'
			});

			if (response.ok) {
				onDelete();
			} else {
				error = 'Failed to delete play';
			}
		} catch {
			error = 'Failed to delete play';
		} finally {
			isDeleting = false;
		}
	}
</script>

<!-- Modal backdrop -->
<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
	onclick={(e) => e.target === e.currentTarget && onClose()}
>
	<div
		class="bg-background max-h-[90vh] w-full max-w-lg overflow-y-auto rounded-lg shadow-lg"
		onclick={(e) => e.stopPropagation()}
	>
		<div class="border-border border-b p-4">
			<div class="flex items-start justify-between">
				<div>
					<h2 class="text-foreground text-lg font-semibold">
						{isEditing ? 'Edit Play Session' : 'Record Play Session'}
					</h2>
					<p class="text-muted-foreground text-sm">
						{game.displayName || `Game ${game.gameId}`} - Play #{colIndex + 1}
					</p>
				</div>
				<button type="button" class="text-muted-foreground hover:text-foreground" onclick={onClose}>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						viewBox="0 0 20 20"
						fill="currentColor"
					>
						<path
							fill-rule="evenodd"
							d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
							clip-rule="evenodd"
						/>
					</svg>
				</button>
			</div>
		</div>

		<form onsubmit={handleSubmit} class="space-y-4 p-4">
			{#if error}
				<div class="bg-destructive/10 border-destructive rounded-lg border p-3">
					<p class="text-destructive text-sm">{error}</p>
				</div>
			{/if}

			<div class="space-y-2">
				<Label for="playedAt">Date Played</Label>
				<Input id="playedAt" type="date" bind:value={playedAt} required />
			</div>

			<div class="space-y-2">
				<Label>Participants & Winners</Label>
				<WinnerSelector
					{participants}
					{selectedParticipants}
					onToggleParticipant={toggleParticipant}
					onToggleWinner={toggleWinner}
					onUpdateScore={updateScore}
				/>
			</div>

			<div class="space-y-2">
				<Label for="notes">Notes (optional)</Label>
				<Textarea
					id="notes"
					bind:value={notes}
					placeholder="Any notes about this play session..."
					rows={2}
				/>
			</div>

			<div class="border-border flex gap-3 border-t pt-4">
				{#if isEditing}
					<Button
						type="button"
						variant="destructive"
						onclick={handleDelete}
						disabled={isDeleting || isSubmitting}
						class="mr-auto"
					>
						{isDeleting ? 'Deleting...' : 'Delete'}
					</Button>
				{/if}
				<Button type="button" variant="outline" onclick={onClose} class={isEditing ? '' : 'flex-1'}>
					Cancel
				</Button>
				<Button
					type="submit"
					disabled={isSubmitting || isDeleting}
					class={isEditing ? '' : 'flex-1'}
				>
					{isSubmitting ? 'Saving...' : isEditing ? 'Save Changes' : 'Record Play'}
				</Button>
			</div>
		</form>
	</div>
</div>
