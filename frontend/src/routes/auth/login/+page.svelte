<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { useAuth } from '$lib/stores/auth';
	import { createAuthState } from '$lib/stores/auth.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Meeple } from '$lib/components/icons';

	const authStore = useAuth();
	const auth = createAuthState();

	let isRedirecting = $state(false);

	$effect(() => {
		if (!auth.isLoading && auth.user) {
			goto(resolve('/'));
		}
	});

	function handleLogin() {
		isRedirecting = true;
		authStore.login();
	}
</script>

<div class="flex min-h-[80vh] items-center justify-center px-4">
	<div class="bg-card border-border w-full max-w-md rounded-lg border p-8 shadow-lg">
		<div class="mb-8 text-center">
			<div
				class="bg-game-blue mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full"
			>
				<Meeple size={32} color="current" class="text-white" />
			</div>
			<h1 class="text-foreground mb-2 text-2xl font-bold">Welcome to Tabletop Atlas</h1>
			<p class="text-muted-foreground">Sign in to access your game collection and more</p>
		</div>

		{#if auth.isLoading || isRedirecting}
			<div class="flex justify-center">
				<div
					class="border-game-blue h-8 w-8 animate-spin rounded-full border-4 border-t-transparent"
				></div>
			</div>
		{:else}
			<Button class="w-full" size="lg" onclick={handleLogin}>
				<svg class="mr-2 h-5 w-5" viewBox="0 0 24 24">
					<path
						fill="currentColor"
						d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
					/>
					<path
						fill="currentColor"
						d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
					/>
					<path
						fill="currentColor"
						d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
					/>
					<path
						fill="currentColor"
						d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
					/>
				</svg>
				Sign in with Google
			</Button>
		{/if}

		<p class="text-muted-foreground mt-6 text-center text-sm">
			By signing in, you agree to our Terms of Service and Privacy Policy.
		</p>
	</div>
</div>
