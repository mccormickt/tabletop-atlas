<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { useAuth, type AuthState } from '$lib/stores/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';

	const auth = useAuth();

	let authState = $state<AuthState>({ user: null, isLoading: true, error: null });
	let isSubmitting = $state(false);
	let error = $state<string | null>(null);

	let formData = $state({
		name: '',
		description: '',
		publisher: '',
		year_published: '',
		min_players: '',
		max_players: '',
		play_time_minutes: '',
		is_public: false
	});

	$effect(() => {
		const unsubscribe = auth.subscribe((state) => {
			authState = state;
			if (!state.isLoading && !state.user) {
				goto(resolve('/auth/login'));
			}
		});
		return unsubscribe;
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		isSubmitting = true;
		error = null;

		try {
			const payload: Record<string, unknown> = {
				name: formData.name,
				is_public: formData.is_public
			};

			if (formData.description) payload.description = formData.description;
			if (formData.publisher) payload.publisher = formData.publisher;
			if (formData.year_published) payload.year_published = parseInt(formData.year_published);
			if (formData.min_players) payload.min_players = parseInt(formData.min_players);
			if (formData.max_players) payload.max_players = parseInt(formData.max_players);
			if (formData.play_time_minutes)
				payload.play_time_minutes = parseInt(formData.play_time_minutes);

			const response = await fetch('/api/custom-games', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify(payload)
			});

			if (response.ok) {
				goto(resolve('/games/custom'));
			} else if (response.status === 401) {
				goto(resolve('/auth/login'));
			} else {
				const data = await response.json();
				error = data.message || 'Failed to create custom game';
			}
		} catch {
			error = 'Failed to create custom game';
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="mx-auto max-w-2xl px-4 py-8 sm:px-6 lg:px-8">
	<div class="mb-8">
		<h1 class="text-foreground text-3xl font-bold">Create Custom Game</h1>
		<p class="text-muted-foreground mt-2">Add a game that isn't in the main library</p>
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
				<Label for="name">Game Name *</Label>
				<Input id="name" bind:value={formData.name} required placeholder="Enter game name" />
			</div>

			<div class="space-y-2">
				<Label for="description">Description</Label>
				<Textarea
					id="description"
					bind:value={formData.description}
					placeholder="Brief description of the game"
					rows={3}
				/>
			</div>

			<div class="space-y-2">
				<Label for="publisher">Publisher</Label>
				<Input id="publisher" bind:value={formData.publisher} placeholder="Publisher name" />
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="year">Year Published</Label>
					<Input
						id="year"
						type="number"
						bind:value={formData.year_published}
						placeholder="2024"
						min="1900"
						max="2100"
					/>
				</div>
				<div class="space-y-2">
					<Label for="playtime">Play Time (minutes)</Label>
					<Input
						id="playtime"
						type="number"
						bind:value={formData.play_time_minutes}
						placeholder="60"
						min="1"
					/>
				</div>
			</div>

			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="min_players">Min Players</Label>
					<Input
						id="min_players"
						type="number"
						bind:value={formData.min_players}
						placeholder="1"
						min="1"
					/>
				</div>
				<div class="space-y-2">
					<Label for="max_players">Max Players</Label>
					<Input
						id="max_players"
						type="number"
						bind:value={formData.max_players}
						placeholder="4"
						min="1"
					/>
				</div>
			</div>

			<div class="flex items-center gap-2">
				<input
					type="checkbox"
					id="is_public"
					bind:checked={formData.is_public}
					class="h-4 w-4 rounded border-gray-300"
				/>
				<Label for="is_public" class="cursor-pointer"
					>Make this game public (others can see it)</Label
				>
			</div>

			<div class="flex gap-4">
				<Button type="submit" disabled={isSubmitting || !formData.name}>
					{isSubmitting ? 'Creating...' : 'Create Game'}
				</Button>
				<Button type="button" variant="outline" onclick={() => goto(resolve('/games/custom'))}
					>Cancel</Button
				>
			</div>
		</form>
	{/if}
</div>
