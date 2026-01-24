<script lang="ts">
	import { goto } from '$app/navigation';
	import { browser } from '$app/environment';
	import {
		api,
		type GameSummary,
		type CollectionEntryWithGame,
		type CustomGameSummary
	} from '$lib';
	import { Button, GameBox, LoadingSpinner } from '$lib/components/ui';
	import CollectionDashboard from '$lib/components/CollectionDashboard.svelte';
	import FilterPanel from '$lib/components/FilterPanel.svelte';
	import BulkActionsBar from '$lib/components/BulkActionsBar.svelte';
	import { GameBoxIcon, Rulebook } from '$lib/components/icons';
	import { useHeader } from '$lib/stores/header';
	import { useAuth, type AuthState } from '$lib/stores/auth';

	type TabType = 'library' | 'collection' | 'custom';

	const header = useHeader();
	header.configure({
		showSearch: true,
		currentGame: null
	});

	const auth = useAuth();
	let authState = $state<AuthState>({ user: null, isLoading: true, error: null });
	let isAdmin = $state(false);

	$effect(() => {
		const unsubscribe = auth.subscribe((state) => {
			authState = state;
			isAdmin = state.user?.role === 'admin';
		});
		return unsubscribe;
	});

	// Tab state - read from URL on init
	function getInitialTab(): TabType {
		if (browser) {
			const urlTab = new URLSearchParams(window.location.search).get('tab');
			if (urlTab === 'collection' || urlTab === 'custom') {
				return urlTab;
			}
		}
		return 'library';
	}

	let activeTab = $state<TabType>(getInitialTab());

	// Library state
	let libraryGames = $state<GameSummary[]>([]);
	let libraryLoading = $state(true);
	let libraryError = $state<string | null>(null);
	let libraryPage = $state(1);
	let libraryTotalPages = $state(1);
	let libraryTotal = $state(0);

	// Collection state
	let collectionItems = $state<CollectionEntryWithGame[]>([]);
	let collectionLoading = $state(false);
	let collectionError = $state<string | null>(null);
	let collectionPage = $state(1);
	let collectionTotalPages = $state(1);
	let collectionTotal = $state(0);

	// Custom games state
	let customGames = $state<CustomGameSummary[]>([]);
	let customLoading = $state(false);
	let customError = $state<string | null>(null);
	let customPage = $state(1);
	let customTotalPages = $state(1);
	let customTotal = $state(0);

	// Shared state
	let limit = $state(24);
	let searchQuery = $state('');
	let showFilters = $state(false);
	let filters = $state<{
		search?: string;
		minPlayers?: number;
		maxPlayers?: number;
		minComplexity?: number;
		maxComplexity?: number;
		hasRules?: boolean;
		hasHouseRules?: boolean;
	}>({});

	// Selection state
	let selectedGameIds = $state<Set<string>>(new Set());

	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			loadLibrary(1);
		}
	});

	// Load data when tab changes
	$effect(() => {
		if (
			initialized &&
			activeTab === 'collection' &&
			collectionItems.length === 0 &&
			authState.user
		) {
			loadCollection(1);
		}
		if (initialized && activeTab === 'custom' && customGames.length === 0 && authState.user) {
			loadCustomGames(1);
		}
		// Clear selection when tab changes
		selectedGameIds = new Set();
	});

	// Update URL when tab changes
	$effect(() => {
		if (browser && initialized) {
			const url = new URL(window.location.href);
			if (activeTab === 'library') {
				url.searchParams.delete('tab');
			} else {
				url.searchParams.set('tab', activeTab);
			}
			window.history.replaceState({}, '', url.toString());
		}
	});

	async function loadLibrary(pageNum: number = 1) {
		libraryLoading = true;
		libraryError = null;

		try {
			const result = await api.methods.listGames({
				query: {
					page: pageNum,
					limit,
					search: searchQuery || undefined
				}
			});

			if (result.type === 'success') {
				libraryGames = result.data.items;
				libraryPage = result.data.page;
				libraryTotalPages = result.data.totalPages;
				libraryTotal = result.data.total;
			} else if (result.type === 'error') {
				libraryError = result.data.message || 'Failed to load games';
				libraryGames = [];
			} else if (result.type === 'client_error') {
				libraryError = result.error.message || 'Failed to load games';
				libraryGames = [];
			}
		} catch (err) {
			libraryError = err instanceof Error ? err.message : 'An unexpected error occurred';
			libraryGames = [];
		} finally {
			libraryLoading = false;
		}
	}

	async function loadCollection(pageNum: number = 1) {
		collectionLoading = true;
		collectionError = null;

		try {
			const result = await api.methods.listCollection({
				query: {
					page: pageNum,
					limit
				}
			});

			if (result.type === 'success') {
				collectionItems = result.data.items;
				collectionPage = result.data.page;
				collectionTotalPages = result.data.totalPages;
				collectionTotal = result.data.total;
			} else if (result.type === 'error') {
				collectionError = result.data.message || 'Failed to load collection';
				collectionItems = [];
			} else if (result.type === 'client_error') {
				collectionError = result.error.message || 'Failed to load collection';
				collectionItems = [];
			}
		} catch (err) {
			collectionError = err instanceof Error ? err.message : 'An unexpected error occurred';
			collectionItems = [];
		} finally {
			collectionLoading = false;
		}
	}

	async function loadCustomGames(pageNum: number = 1) {
		customLoading = true;
		customError = null;

		try {
			const result = await api.methods.listCustomGames({
				query: {
					page: pageNum,
					limit
				}
			});

			if (result.type === 'success') {
				customGames = result.data.items;
				customPage = result.data.page;
				customTotalPages = result.data.totalPages;
				customTotal = result.data.total;
			} else if (result.type === 'error') {
				customError = result.data.message || 'Failed to load custom games';
				customGames = [];
			} else if (result.type === 'client_error') {
				customError = result.error.message || 'Failed to load custom games';
				customGames = [];
			}
		} catch (err) {
			customError = err instanceof Error ? err.message : 'An unexpected error occurred';
			customGames = [];
		} finally {
			customLoading = false;
		}
	}

	function handleSearchChange(search: string) {
		searchQuery = search;
		if (activeTab === 'library') {
			loadLibrary(1);
		}
	}

	async function handleAddToCollection() {
		const gameIds = Array.from(selectedGameIds).map((id) => parseInt(id, 10));
		let successCount = 0;

		for (const gameId of gameIds) {
			try {
				const result = await api.methods.addToCollection({
					body: { masterGameId: gameId }
				});
				if (result.type === 'success') {
					successCount++;
				}
			} catch {
				// Continue with other games
			}
		}

		selectedGameIds = new Set();
		if (successCount > 0) {
			// Reload collection if on that tab
			if (activeTab === 'collection') {
				await loadCollection(collectionPage);
			}
		}
	}

	async function handleRemoveFromCollection() {
		const entryIds = Array.from(selectedGameIds).map((id) => parseInt(id, 10));
		let successCount = 0;

		for (const entryId of entryIds) {
			try {
				const result = await api.methods.removeFromCollection({
					path: { id: entryId }
				});
				if (result.type === 'success') {
					successCount++;
				}
			} catch {
				// Continue with other items
			}
		}

		selectedGameIds = new Set();
		if (successCount > 0) {
			await loadCollection(collectionPage);
		}
	}

	async function handleDeleteCustomGames() {
		if (
			!confirm(
				`Are you sure you want to delete ${selectedGameIds.size} custom game${selectedGameIds.size === 1 ? '' : 's'}? This action cannot be undone.`
			)
		) {
			return;
		}

		const gameIds = Array.from(selectedGameIds).map((id) => parseInt(id, 10));
		let successCount = 0;

		for (const gameId of gameIds) {
			try {
				const result = await api.methods.deleteCustomGame({
					path: { id: gameId }
				});
				if (result.type === 'success') {
					successCount++;
				}
			} catch {
				// Continue with other games
			}
		}

		selectedGameIds = new Set();
		if (successCount > 0) {
			await loadCustomGames(customPage);
		}
	}

	async function handleDeleteGame(game: GameSummary) {
		if (!confirm(`Are you sure you want to delete "${game.name}"? This action cannot be undone.`)) {
			return;
		}

		try {
			const result = await api.methods.deleteGame({
				path: { id: game.id }
			});

			if (result.type === 'success') {
				await loadLibrary(libraryPage);
			} else if (result.type === 'error') {
				alert(result.data.message || 'Failed to delete game');
			} else if (result.type === 'client_error') {
				alert(result.error.message || 'Failed to delete game');
			}
		} catch (err) {
			alert(err instanceof Error ? err.message : 'An unexpected error occurred');
		}
	}

	function navigateToAddGame() {
		if (activeTab === 'custom' || !isAdmin) {
			goto('/games/custom/add');
		} else {
			goto('/games/add');
		}
	}

	function handleLibraryPageChange(pageNum: number) {
		loadLibrary(pageNum);
	}

	function handleCollectionPageChange(pageNum: number) {
		loadCollection(pageNum);
	}

	function handleCustomPageChange(pageNum: number) {
		loadCustomGames(pageNum);
	}

	function handleSelectionChange(newSelection: Set<string>) {
		selectedGameIds = newSelection;
	}

	function toggleFilters() {
		showFilters = !showFilters;
	}

	function setActiveTab(tab: TabType) {
		activeTab = tab;
	}

	// Get current tab's data
	let currentLoading = $derived(
		activeTab === 'library'
			? libraryLoading
			: activeTab === 'collection'
				? collectionLoading
				: customLoading
	);
	let currentError = $derived(
		activeTab === 'library'
			? libraryError
			: activeTab === 'collection'
				? collectionError
				: customError
	);
</script>

<svelte:head>
	<title>Games - Tabletop Atlas</title>
	<meta name="description" content="Browse and manage your board game collection" />
</svelte:head>

<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Page Header with Rulebook Style -->
	<div class="mb-8">
		<div class="rulebook-header">
			<h1 class="text-3xl md:text-4xl">Game Library</h1>
		</div>
		<p class="text-muted-foreground font-body text-center">
			{#if activeTab === 'library'}
				{#if libraryTotal > 0}
					{libraryTotal.toLocaleString()} games in the library
				{:else}
					Browse the game library
				{/if}
			{:else if activeTab === 'collection'}
				{#if collectionTotal > 0}
					{collectionTotal} game{collectionTotal === 1 ? '' : 's'} in your collection
				{:else}
					Your collection awaits
				{/if}
			{:else if customTotal > 0}
				{customTotal} custom game{customTotal === 1 ? '' : 's'}
			{:else}
				Create your own games
			{/if}
		</p>
	</div>

	<!-- Tab Navigation -->
	<div class="mb-6 border-b border-gray-200">
		<nav class="-mb-px flex space-x-8" aria-label="Tabs">
			<button
				onclick={() => setActiveTab('library')}
				class="border-b-2 px-1 py-4 text-sm font-medium whitespace-nowrap {activeTab === 'library'
					? 'border-game-blue text-game-blue'
					: 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'}"
			>
				Browse Library
			</button>
			<button
				onclick={() => setActiveTab('collection')}
				class="border-b-2 px-1 py-4 text-sm font-medium whitespace-nowrap {activeTab ===
				'collection'
					? 'border-game-blue text-game-blue'
					: 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'}"
			>
				My Collection
				{#if collectionTotal > 0}
					<span
						class="bg-game-blue/10 text-game-blue ml-2 rounded-full px-2 py-0.5 text-xs font-medium"
					>
						{collectionTotal}
					</span>
				{/if}
			</button>
			<button
				onclick={() => setActiveTab('custom')}
				class="border-b-2 px-1 py-4 text-sm font-medium whitespace-nowrap {activeTab === 'custom'
					? 'border-game-blue text-game-blue'
					: 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'}"
			>
				Custom Games
				{#if customTotal > 0}
					<span
						class="bg-game-blue/10 text-game-blue ml-2 rounded-full px-2 py-0.5 text-xs font-medium"
					>
						{customTotal}
					</span>
				{/if}
			</button>
		</nav>
	</div>

	<!-- Action Bar -->
	<div class="mb-6 flex flex-wrap items-center justify-between gap-4">
		<div class="flex items-center gap-2">
			{#if activeTab === 'library'}
				<Button variant="game-secondary" size="sm" onclick={toggleFilters} class="md:hidden">
					{showFilters ? 'Hide Filters' : 'Filters'}
				</Button>
			{/if}
		</div>

		<!-- Desktop: Regular button -->
		{#if activeTab === 'library' && isAdmin}
			<Button variant="game-primary" onclick={navigateToAddGame} class="hidden gap-2 md:flex">
				<GameBoxIcon size={18} />
				Add to Library
			</Button>
		{:else if activeTab === 'custom'}
			<Button variant="game-primary" onclick={navigateToAddGame} class="hidden gap-2 md:flex">
				<GameBoxIcon size={18} />
				Create Custom Game
			</Button>
		{/if}
	</div>

	<!-- Mobile: Floating Action Button -->
	{#if (activeTab === 'library' && isAdmin) || activeTab === 'custom'}
		<button
			onclick={navigateToAddGame}
			class="bg-game-blue hover:bg-game-blue/90 fixed right-4 bottom-24 z-40 flex h-14 w-14 items-center justify-center rounded-full text-white shadow-lg transition-all active:scale-95 md:hidden"
			aria-label={activeTab === 'custom' ? 'Create custom game' : 'Add new game'}
		>
			<GameBoxIcon size={24} />
		</button>
	{/if}

	<div class="flex gap-6">
		<!-- Filter Sidebar (Desktop) - Only for Library tab -->
		{#if activeTab === 'library'}
			<aside class="hidden w-64 flex-shrink-0 md:block">
				<FilterPanel
					bind:filters
					onApply={() => loadLibrary(1)}
					onClear={() => {
						searchQuery = '';
						loadLibrary(1);
					}}
					onSearchChange={handleSearchChange}
				/>
			</aside>
		{/if}

		<!-- Mobile Filter Drawer -->
		{#if showFilters && activeTab === 'library'}
			<div class="fixed inset-0 z-40 md:hidden">
				<div class="absolute inset-0 bg-black/50" onclick={toggleFilters}></div>
				<div
					class="bg-background absolute top-0 bottom-0 left-0 w-72 overflow-y-auto p-4 shadow-lg"
				>
					<div class="mb-4 flex items-center justify-between">
						<h2 class="font-display text-lg font-semibold">Filters</h2>
						<button onclick={toggleFilters} class="text-muted-foreground hover:text-foreground">
							<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M6 18L18 6M6 6l12 12"
								/>
							</svg>
						</button>
					</div>
					<FilterPanel
						bind:filters
						onApply={() => {
							loadLibrary(1);
							showFilters = false;
						}}
						onClear={() => {
							searchQuery = '';
							loadLibrary(1);
							showFilters = false;
						}}
						onSearchChange={handleSearchChange}
					/>
				</div>
			</div>
		{/if}

		<!-- Main Content -->
		<div class="min-w-0 flex-1">
			<!-- Auth Required Message for Collection/Custom -->
			{#if (activeTab === 'collection' || activeTab === 'custom') && !authState.user && !authState.isLoading}
				<GameBox variant="featured" showCorners={true} class="text-center">
					<div class="py-12">
						<div class="mb-6">
							<div
								class="bg-parchment-dark mx-auto flex h-24 w-24 items-center justify-center rounded-full"
							>
								<Rulebook size={48} class="text-game-blue" />
							</div>
						</div>
						<h3 class="font-display mb-3 text-xl font-semibold">Sign In Required</h3>
						<p class="text-muted-foreground font-body mx-auto mb-6 max-w-md">
							{#if activeTab === 'collection'}
								Sign in to view and manage your personal game collection.
							{:else}
								Sign in to create and manage your custom games.
							{/if}
						</p>
						<Button variant="game-accent" onclick={() => auth.login()} class="gap-2">
							Sign In
						</Button>
					</div>
				</GameBox>
			{/if}

			<!-- Loading State -->
			{#if currentLoading}
				<div class="game-box-lid p-12 text-center">
					<LoadingSpinner class="mx-auto mb-4" />
					<p class="text-muted-foreground font-body">
						{#if activeTab === 'library'}
							Loading game library...
						{:else if activeTab === 'collection'}
							Loading your collection...
						{:else}
							Loading custom games...
						{/if}
					</p>
				</div>
			{/if}

			<!-- Error State -->
			{#if currentError && !currentLoading}
				<GameBox variant="default" class="text-center">
					<div class="py-8">
						<div class="text-destructive mb-4">
							<svg class="mx-auto h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path
									stroke-linecap="round"
									stroke-linejoin="round"
									stroke-width="2"
									d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
								></path>
							</svg>
						</div>
						<h3 class="font-display mb-2 text-lg font-semibold">Unable to Load Games</h3>
						<p class="text-muted-foreground font-body mb-4">{currentError}</p>
						<Button
							variant="game-primary"
							onclick={() => {
								if (activeTab === 'library') loadLibrary(libraryPage);
								else if (activeTab === 'collection') loadCollection(collectionPage);
								else loadCustomGames(customPage);
							}}>Try Again</Button
						>
					</div>
				</GameBox>
			{/if}

			<!-- Library Tab Content -->
			{#if activeTab === 'library' && !libraryLoading && !libraryError}
				{#if libraryGames.length === 0}
					<GameBox variant="featured" showCorners={true} class="text-center">
						<div class="py-12">
							<div class="mb-6">
								<div
									class="bg-parchment-dark mx-auto flex h-24 w-24 items-center justify-center rounded-full"
								>
									<Rulebook size={48} class="text-game-blue" />
								</div>
							</div>
							<h3 class="font-display mb-3 text-xl font-semibold">No Games Found</h3>
							<p class="text-muted-foreground font-body mx-auto mb-6 max-w-md">
								{#if searchQuery}
									No games match your search. Try a different search term.
								{:else}
									The game library is empty.
								{/if}
							</p>
						</div>
					</GameBox>
				{:else}
					<CollectionDashboard
						mode="library"
						games={libraryGames}
						currentPage={libraryPage}
						totalPages={libraryTotalPages}
						total={libraryTotal}
						{isAdmin}
						selectedIds={selectedGameIds}
						onPageChange={handleLibraryPageChange}
						onDelete={isAdmin ? handleDeleteGame : undefined}
						onSelectionChange={handleSelectionChange}
					/>
				{/if}
			{/if}

			<!-- Collection Tab Content -->
			{#if activeTab === 'collection' && authState.user && !collectionLoading && !collectionError}
				{#if collectionItems.length === 0}
					<GameBox variant="featured" showCorners={true} class="text-center">
						<div class="py-12">
							<div class="mb-6">
								<div
									class="bg-parchment-dark mx-auto flex h-24 w-24 items-center justify-center rounded-full"
								>
									<Rulebook size={48} class="text-game-blue" />
								</div>
							</div>
							<h3 class="font-display mb-3 text-xl font-semibold">Your Collection is Empty</h3>
							<p class="text-muted-foreground font-body mx-auto mb-6 max-w-md">
								Start building your collection by adding games from the library.
							</p>
							<Button variant="game-accent" onclick={() => setActiveTab('library')} class="gap-2">
								Browse Library
							</Button>
						</div>
					</GameBox>
				{:else}
					<CollectionDashboard
						mode="collection"
						{collectionItems}
						currentPage={collectionPage}
						totalPages={collectionTotalPages}
						total={collectionTotal}
						selectedIds={selectedGameIds}
						onPageChange={handleCollectionPageChange}
						onSelectionChange={handleSelectionChange}
					/>
				{/if}
			{/if}

			<!-- Custom Games Tab Content -->
			{#if activeTab === 'custom' && authState.user && !customLoading && !customError}
				{#if customGames.length === 0}
					<GameBox variant="featured" showCorners={true} class="text-center">
						<div class="py-12">
							<div class="mb-6">
								<div
									class="bg-parchment-dark mx-auto flex h-24 w-24 items-center justify-center rounded-full"
								>
									<GameBoxIcon size={48} class="text-game-blue" />
								</div>
							</div>
							<h3 class="font-display mb-3 text-xl font-semibold">No Custom Games Yet</h3>
							<p class="text-muted-foreground font-body mx-auto mb-6 max-w-md">
								Create your own games that aren't in the main library.
							</p>
							<Button variant="game-accent" onclick={navigateToAddGame} class="gap-2">
								<GameBoxIcon size={18} />
								Create Your First Custom Game
							</Button>
						</div>
					</GameBox>
				{:else}
					<CollectionDashboard
						mode="custom"
						{customGames}
						currentPage={customPage}
						totalPages={customTotalPages}
						total={customTotal}
						selectedIds={selectedGameIds}
						onPageChange={handleCustomPageChange}
						onSelectionChange={handleSelectionChange}
					/>
				{/if}
			{/if}
		</div>
	</div>
</main>

<!-- Bulk Actions Bar -->
<BulkActionsBar
	selectedCount={selectedGameIds.size}
	mode={activeTab}
	onAddToCollection={activeTab === 'library' && authState.user ? handleAddToCollection : undefined}
	onRemoveFromCollection={activeTab === 'collection' ? handleRemoveFromCollection : undefined}
	onDelete={activeTab === 'custom' ? handleDeleteCustomGames : undefined}
	onClearSelection={() => (selectedGameIds = new Set())}
/>
