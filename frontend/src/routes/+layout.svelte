<script lang="ts">
	import '../app.css';
	import Header from '$lib/components/Header.svelte';
	import MobileNav from '$lib/components/MobileNav.svelte';
	import { createHeaderStore, setHeaderContext } from '$lib/stores/header';
	import type { Game } from '$lib';

	let { children } = $props();

	// Create and provide header context
	const headerStore = createHeaderStore();
	setHeaderContext(headerStore);

	// Subscribe to header config with proper cleanup
	let headerConfig = $state({
		currentGame: null as Game | null,
		showSearch: true,
		title: undefined as string | undefined
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
</script>

<div class="bg-background flex min-h-screen flex-col">
	<!-- Global Header -->
	<Header currentGame={headerConfig.currentGame} showSearch={headerConfig.showSearch} />

	<!-- Page Content with bottom padding for mobile nav -->
	<main class="flex-1 pb-20 md:pb-0">
		{@render children()}
	</main>

	<!-- Mobile Bottom Navigation -->
	<MobileNav />
</div>
