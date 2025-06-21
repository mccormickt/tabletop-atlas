<script lang="ts">
	import '../app.css';
	import Header from '$lib/components/Header.svelte';
	import { createHeaderStore, setHeaderContext } from '$lib/stores/header';
	import { onMount } from 'svelte';
	import type { Game } from '$lib';

	let { children } = $props();

	// Create and provide header context
	const headerStore = createHeaderStore();
	setHeaderContext(headerStore);

	// Subscribe to header config
	let headerConfig = $state({
		currentGame: null as Game | null,
		showSearch: true,
		title: undefined as string | undefined
	});

	headerStore.subscribe((config) => {
		headerConfig = {
			currentGame: config.currentGame ?? null,
			showSearch: config.showSearch ?? true,
			title: config.title
		};
	});
</script>

<div class="bg-background min-h-screen">
	<!-- Global Header -->
	<Header currentGame={headerConfig.currentGame} showSearch={headerConfig.showSearch} />

	<!-- Page Content -->
	{@render children()}
</div>
