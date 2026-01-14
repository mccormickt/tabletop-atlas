<script lang="ts">
	import { goto } from '$app/navigation';
	import { useAuth, type AuthState } from '$lib/stores/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';

	const auth = useAuth();

	let authState = $state<AuthState>({ user: null, isLoading: true, error: null });
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);

	let name = $state('');
	let description = $state('');
	let gridRows = $state(8);
	let gridCols = $state(8);
	let startDate = $state('');
	let endDate = $state('');

	$effect(() => {
		const unsubscribe = auth.subscribe((state) => {
			authState = state;
			if (!state.isLoading && !state.user) {
				goto('/auth/login');
			}
		});
		return unsubscribe;
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!name.trim()) {
			error = 'Name is required';
			return;
		}

		isSubmitting = true;
		error = null;

		try {
			const response = await fetch('/api/challenges', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({
					name: name.trim(),
					description: description.trim() || null,
					gridRows,
					gridCols,
					startDate: startDate || null,
					endDate: endDate || null
				})
			});

			if (response.ok) {
				const challenge = await response.json();
				goto(`/challenges/${challenge.id}`);
			} else if (response.status === 401) {
				goto('/auth/login');
			} else {
				const data = await response.json();
				error = data.message || 'Failed to create challenge';
			}
		} catch {
			error = 'Failed to create challenge';
		} finally {
			isSubmitting = false;
		}
	}

	function handleGridSizeChange(type: 'rows' | 'cols', value: number) {
		const clamped = Math.max(1, Math.min(10, value));
		if (type === 'rows') {
			gridRows = clamped;
		} else {
			gridCols = clamped;
		}
	}
</script>

<div class="mx-auto max-w-2xl px-4 py-8 sm:px-6 lg:px-8">
	<div class="mb-8">
		<a href="/challenges" class="text-muted-foreground hover:text-foreground text-sm">
			&larr; Back to Challenges
		</a>
		<h1 class="text-foreground mt-4 text-3xl font-bold">Create Challenge</h1>
		<p class="text-muted-foreground mt-2">Set up a new gaming challenge to track with friends</p>
	</div>

	{#if authState.isLoading}
		<div class="flex justify-center py-12">
			<div
				class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
			></div>
		</div>
	{:else}
		<form onsubmit={handleSubmit} class="space-y-6">
			{#if error}
				<div class="bg-destructive/10 border-destructive rounded-lg border p-4">
					<p class="text-destructive text-sm">{error}</p>
				</div>
			{/if}

			<div class="space-y-2">
				<Label for="name">Challenge Name *</Label>
				<Input id="name" bind:value={name} placeholder="Summer Gaming Challenge 2024" required />
			</div>

			<div class="space-y-2">
				<Label for="description">Description</Label>
				<Textarea
					id="description"
					bind:value={description}
					placeholder="Describe your challenge..."
					rows={3}
				/>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="gridRows">Number of Games (Rows)</Label>
					<Input
						id="gridRows"
						type="number"
						min={1}
						max={10}
						value={gridRows}
						onchange={(e) =>
							handleGridSizeChange('rows', parseInt((e.target as HTMLInputElement).value) || 8)}
					/>
					<p class="text-muted-foreground text-xs">1-10 games</p>
				</div>

				<div class="space-y-2">
					<Label for="gridCols">Plays per Game (Columns)</Label>
					<Input
						id="gridCols"
						type="number"
						min={1}
						max={10}
						value={gridCols}
						onchange={(e) =>
							handleGridSizeChange('cols', parseInt((e.target as HTMLInputElement).value) || 8)}
					/>
					<p class="text-muted-foreground text-xs">1-10 plays each</p>
				</div>
			</div>

			<div class="bg-muted/50 rounded-lg p-4">
				<p class="text-muted-foreground text-sm">
					Your challenge will have <strong class="text-foreground">{gridRows}</strong> games, each
					played
					<strong class="text-foreground">{gridCols}</strong>
					times, for a total of
					<strong class="text-foreground">{gridRows * gridCols}</strong> sessions to complete.
				</p>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="startDate">Start Date (optional)</Label>
					<Input id="startDate" type="date" bind:value={startDate} />
				</div>

				<div class="space-y-2">
					<Label for="endDate">End Date (optional)</Label>
					<Input id="endDate" type="date" bind:value={endDate} />
				</div>
			</div>

			<div class="flex gap-4 pt-4">
				<Button type="button" variant="outline" href="/challenges" class="flex-1">Cancel</Button>
				<Button type="submit" class="flex-1" disabled={isSubmitting}>
					{#if isSubmitting}
						Creating...
					{:else}
						Create Challenge
					{/if}
				</Button>
			</div>
		</form>
	{/if}
</div>
