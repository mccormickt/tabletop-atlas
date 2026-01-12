<script lang="ts">
	import { goto } from '$app/navigation';
	import { useAuth } from '$lib/stores/auth';
	import { Meeple } from '$lib/components/icons';

	const auth = useAuth();

	let error = $state<string | null>(null);

	$effect(() => {
		// Check authentication status after the callback
		auth.checkAuth().then((user) => {
			if (user) {
				goto('/');
			} else {
				error = 'Authentication failed. Please try again.';
			}
		});
	});
</script>

<div class="flex min-h-[80vh] items-center justify-center px-4">
	<div class="text-center">
		{#if error}
			<div class="bg-destructive/10 border-destructive rounded-lg border p-6">
				<p class="text-destructive mb-4">{error}</p>
				<a href="/auth/login" class="text-primary hover:underline">Try again</a>
			</div>
		{:else}
			<div class="mb-4 flex justify-center">
				<div
					class="bg-game-blue flex h-16 w-16 animate-pulse items-center justify-center rounded-full"
				>
					<Meeple size={32} color="current" class="text-white" />
				</div>
			</div>
			<h2 class="text-foreground mb-2 text-xl font-semibold">Completing sign in...</h2>
			<p class="text-muted-foreground">Please wait while we set up your account.</p>
		{/if}
	</div>
</div>
