<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { createAuthState } from '$lib/stores/auth.svelte';

	let { children } = $props();

	const auth = createAuthState();

	$effect(() => {
		if (!auth.isLoading && (!auth.user || auth.user.role !== 'admin')) {
			goto(resolve('/'));
		}
	});
</script>

{#if auth.isLoading}
	<div class="flex min-h-[50vh] items-center justify-center">
		<div class="text-center">
			<div
				class="border-game-blue mx-auto h-12 w-12 animate-spin rounded-full border-4 border-t-transparent"
			></div>
			<p class="text-muted-foreground mt-4">Loading...</p>
		</div>
	</div>
{:else if auth.isAdmin}
	{@render children()}
{:else}
	<div class="flex min-h-[50vh] items-center justify-center">
		<div class="text-center">
			<h2 class="text-foreground text-2xl font-bold">Access Denied</h2>
			<p class="text-muted-foreground mt-2">You need admin privileges to access this page.</p>
		</div>
	</div>
{/if}
