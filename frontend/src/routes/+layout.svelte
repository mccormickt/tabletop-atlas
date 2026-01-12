<script lang="ts">
	import '../app.css';
	import Header from '$lib/components/Header.svelte';
	import MobileNav from '$lib/components/MobileNav.svelte';
	import { createHeaderStore, setHeaderContext } from '$lib/stores/header';
	import { createAuthStore, setAuthContext, type UserInfo } from '$lib/stores/auth';
	import type { Game } from '$lib';

	let { children } = $props();

	// Create and provide header context
	const headerStore = createHeaderStore();
	setHeaderContext(headerStore);

	// Create and provide auth context
	const authStore = createAuthStore();
	setAuthContext(authStore);

	// Subscribe to header config with proper cleanup
	let headerConfig = $state({
		currentGame: null as Game | null,
		showSearch: true,
		title: undefined as string | undefined
	});

	// Subscribe to auth state
	let authState = $state({
		user: null as UserInfo | null,
		isLoading: true
	});

	// Check auth on mount
	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			authStore.checkAuth();
		}
	});

	$effect(() => {
		const unsubscribe = headerStore.subscribe((config) => {
			headerConfig = {
				currentGame: config.currentGame ?? null,
				showSearch: config.showSearch ?? true,
				title: config.title
			};
		});
		return unsubscribe;
	});

	$effect(() => {
		const unsubscribe = authStore.subscribe((state) => {
			authState = {
				user: state.user,
				isLoading: state.isLoading
			};
		});
		return unsubscribe;
	});
</script>

<div class="bg-background flex min-h-screen flex-col">
	<!-- Global Header -->
	<Header
		currentGame={headerConfig.currentGame}
		showSearch={headerConfig.showSearch}
		user={authState.user}
		isAuthLoading={authState.isLoading}
	/>

	<!-- Page Content with bottom padding for mobile nav -->
	<main class="flex-1 pb-20 md:pb-0">
		{@render children()}
	</main>

	<!-- Mobile Bottom Navigation -->
	<MobileNav />
</div>
