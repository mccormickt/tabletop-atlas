<script lang="ts">
	import { api, type Game, type GameSummary } from '$lib';
	import GameList from '$lib/components/GameList.svelte';
	import GameForm from '$lib/components/GameForm.svelte';
	import GameDetail from '$lib/components/GameDetail.svelte';
	import { Button, Card } from '$lib/components/ui';

	// State management for different views
	type ViewState = 'list' | 'add' | 'edit' | 'detail';

	let currentView = $state<ViewState>('list');
	let selectedGame = $state<Game | GameSummary | null>(null);
	let refreshTrigger = $state(0);

	// Navigation functions
	function showGameList() {
		currentView = 'list';
		selectedGame = null;
	}

	function showAddGame() {
		currentView = 'add';
		selectedGame = null;
	}

	function showEditGame(game: Game | GameSummary) {
		currentView = 'edit';
		selectedGame = game;
	}

	function showGameDetail(game: GameSummary) {
		currentView = 'detail';
		selectedGame = game;
	}

	// Event handlers
	function handleGameCreated(game: Game) {
		currentView = 'detail';
		selectedGame = game;
		refreshTrigger++;
	}

	function handleGameUpdated(game: Game) {
		currentView = 'detail';
		selectedGame = game;
		refreshTrigger++;
	}

	function handleGameDeleted() {
		currentView = 'list';
		selectedGame = null;
		refreshTrigger++;
	}

	async function handleEditFromDetail(game: Game) {
		// Load fresh game data for editing
		try {
			const result = await api.methods.getGame({
				path: { id: game.id }
			});
			if (result.success) {
				selectedGame = result.data;
				currentView = 'edit';
			}
		} catch (err) {
			console.error('Failed to load game for editing:', err);
		}
	}
</script>

<svelte:head>
	<title>Tabletop Atlas - Board Game Management</title>
	<meta name="description" content="Comprehensive board game rules management system" />
</svelte:head>

<div class="bg-background min-h-screen">
	<!-- Header -->
	<header class="bg-card border-b shadow-sm">
		<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
			<div class="flex items-center justify-between py-6">
				<div class="flex items-center">
					<button
						onclick={showGameList}
						class="text-foreground hover:text-primary flex items-center text-2xl font-bold transition-colors"
					>
						🎲 Tabletop Atlas
					</button>
				</div>
				<nav class="flex space-x-8">
					<button
						onclick={showGameList}
						class="text-muted-foreground hover:text-foreground transition-colors"
						class:text-foreground={currentView === 'list'}
						class:font-medium={currentView === 'list'}
					>
						Games
					</button>
					<button class="text-muted-foreground hover:text-foreground transition-colors">
						Upload Rules
					</button>
					<button class="text-muted-foreground hover:text-foreground transition-colors">
						Chat
					</button>
				</nav>
			</div>
		</div>
	</header>

	<!-- Main Content -->
	<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
		<!-- Hero Section (only show on list view) -->
		{#if currentView === 'list'}
			<div class="mb-12 text-center">
				<h1 class="text-foreground mb-4 text-4xl font-bold">Manage Your Board Game Collection</h1>
				<p class="text-muted-foreground mx-auto max-w-3xl text-xl">
					Organize your board games, upload rule books, create house rules, and get instant answers
					about gameplay through our AI-powered chat interface.
				</p>
			</div>

			<!-- Quick Actions -->
			<div class="mb-12 grid grid-cols-1 gap-6 md:grid-cols-3">
				<Card class="p-6 transition-shadow hover:shadow-lg">
					<div class="text-primary mb-4">
						<svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 4v16m8-8H4"
							></path>
						</svg>
					</div>
					<h3 class="text-card-foreground mb-2 text-lg font-semibold">Add New Game</h3>
					<p class="text-muted-foreground mb-4">
						Add a new board game to your collection with detailed information and metadata.
					</p>
					<Button onclick={showAddGame} class="w-full">Add Game</Button>
				</Card>

				<Card class="p-6 transition-shadow hover:shadow-lg">
					<div class="text-primary mb-4">
						<svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C20.832 18.477 19.246 18 17.5 18c-1.746 0-3.332.477-4.5 1.253"
							></path>
						</svg>
					</div>
					<h3 class="text-card-foreground mb-2 text-lg font-semibold">Upload Rules</h3>
					<p class="text-muted-foreground mb-4">
						Upload PDF rule books and we'll extract and index the content for easy searching.
					</p>
					<Button variant="outline" class="w-full">Upload PDF</Button>
				</Card>

				<Card class="p-6 transition-shadow hover:shadow-lg">
					<div class="text-primary mb-4">
						<svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
							></path>
						</svg>
					</div>
					<h3 class="text-card-foreground mb-2 text-lg font-semibold">Ask Questions</h3>
					<p class="text-muted-foreground mb-4">
						Get instant answers about game rules using our AI-powered chat interface.
					</p>
					<Button variant="outline" class="w-full">Start Chat</Button>
				</Card>
			</div>
		{/if}

		<!-- Content Area -->
		<div class="w-full">
			{#if currentView === 'list'}
				<!-- Game List View -->
				<div class="mb-6 flex items-center justify-between">
					<div></div>
					<Button onclick={showAddGame}>Add New Game</Button>
				</div>

				<GameList
					{refreshTrigger}
					onEdit={showEditGame}
					onView={showGameDetail}
					onDelete={handleGameDeleted}
				/>
			{:else if currentView === 'add'}
				<!-- Add Game View -->
				<div class="mb-6">
					<Button
						variant="ghost"
						onclick={showGameList}
						class="text-muted-foreground hover:text-foreground"
					>
						← Back to Games
					</Button>
				</div>

				<GameForm onSubmit={handleGameCreated} onCancel={showGameList} />
			{:else if currentView === 'edit' && selectedGame}
				<!-- Edit Game View -->
				<div class="mb-6">
					<Button
						variant="ghost"
						onclick={showGameList}
						class="text-muted-foreground hover:text-foreground"
					>
						← Back to Games
					</Button>
				</div>

				<GameForm game={selectedGame} onSubmit={handleGameUpdated} onCancel={showGameList} />
			{:else if currentView === 'detail' && selectedGame}
				<!-- Game Detail View -->
				<GameDetail
					gameId={selectedGame.id}
					onEdit={handleEditFromDetail}
					onDelete={handleGameDeleted}
					onBack={showGameList}
				/>
			{/if}
		</div>
	</main>

	<!-- Footer -->
	<footer class="bg-card mt-16 border-t">
		<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
			<div class="flex items-center justify-between">
				<div class="text-muted-foreground text-sm">
					© 2024 Tabletop Atlas. Made with ♥ for board game enthusiasts.
				</div>
				<div class="text-muted-foreground flex space-x-6 text-sm">
					<a href="#" class="hover:text-foreground transition-colors">About</a>
					<a href="#" class="hover:text-foreground transition-colors">Help</a>
					<a href="#" class="hover:text-foreground transition-colors">GitHub</a>
				</div>
			</div>
		</div>
	</footer>
</div>
