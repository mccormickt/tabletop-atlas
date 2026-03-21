<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import {
		api,
		unwrapResult,
		createDebouncedAction,
		type GameSummary,
		type RulesInfoResponse,
		type UploadResponse
	} from '$lib';
	import { Button, GameBox, LoadingSpinner } from '$lib/components/ui';
	import { ComponentTray, ComponentTraySection } from '$lib/components/ui';
	import {
		GameBoxIcon,
		Rulebook,
		Upload as UploadIcon,
		ChatBubble,
		SearchGlass
	} from '$lib/components/icons';
	import PDFUpload from '$lib/components/PDFUpload.svelte';
	import { useHeader } from '$lib/stores/header';

	const header = useHeader();
	header.configure({
		showSearch: true,
		currentGame: null
	});

	let games = $state<GameSummary[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let selectedGameId = $state<number | null>(null);
	let selectedGame = $state<GameSummary | null>(null);
	let rulesInfo = $state<RulesInfoResponse | null>(null);
	let uploadSuccess = $state(false);
	let searchQuery = $state('');
	const debouncedGameSearch = createDebouncedAction(() => loadGames(searchQuery));

	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			loadGames();
		}
	});

	async function loadGames(search?: string) {
		loading = true;
		error = null;

		try {
			const r = unwrapResult(
				await api.methods.listGames({
					query: {
						limit: 50,
						search: search || undefined
					}
				}),
				'Failed to load games'
			);
			if (!r.ok) {
				error = r.error;
				return;
			}
			games = r.data.items;
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			loading = false;
		}
	}

	function handleSearchInput(event: Event) {
		searchQuery = (event.target as HTMLInputElement).value;
		debouncedGameSearch.trigger();
	}

	function handleSearchKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			debouncedGameSearch.cancel();
			loadGames(searchQuery);
		}
	}

	async function selectGame(gameId: number) {
		selectedGameId = gameId;
		selectedGame = games.find((g) => g.id === gameId) || null;
		uploadSuccess = false;

		try {
			const result = await api.methods.getRulesInfo({
				path: { id: gameId }
			});

			if (result.type === 'success') {
				rulesInfo = result.data;
			} else {
				rulesInfo = null;
			}
		} catch {
			rulesInfo = null;
			error = 'Failed to load rules info';
		}
	}

	// eslint-disable-next-line @typescript-eslint/no-unused-vars
	function handleUploadSuccess(_response: UploadResponse) {
		uploadSuccess = true;
		if (selectedGameId) {
			selectGame(selectedGameId);
		}
	}

	function handleUploadDeleted() {
		if (selectedGameId) {
			selectGame(selectedGameId);
		}
	}

	function handleUploadError(errorMsg: string) {
		error = errorMsg;
	}

	function goToGame(gameId: number) {
		goto(resolve(`/games/${gameId}`));
	}

	function goToChat() {
		if (selectedGame) {
			goto(resolve('/chat') + `?game_id=${selectedGame.id}`);
		}
	}

	function formatPlayerCount(min?: number | null, max?: number | null): string {
		if (!min && !max) return 'Not specified';
		if (min && max) {
			return min === max ? `${min}` : `${min}-${max}`;
		}
		if (min) return `${min}+`;
		if (max) return `Up to ${max}`;
		return '?';
	}
</script>

<svelte:head>
	<title>Upload Rules - Tabletop Atlas</title>
	<meta name="description" content="Upload PDF rule books for your board games" />
</svelte:head>

<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Page Header -->
	<div class="mb-8">
		<div class="rulebook-header">
			<h1 class="text-2xl md:text-3xl">Upload Rules</h1>
		</div>
		<p class="text-muted-foreground font-body text-center">
			Upload PDF rule books to enable AI-powered search and chat
		</p>
	</div>

	<div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
		<!-- Game Selection -->
		<div class="lg:col-span-1">
			<ComponentTray title="Select Game">
				{#if error && games.length === 0 && !searchQuery}
					<ComponentTraySection>
						<p class="text-parchment/70 mb-2 text-center text-sm">{error}</p>
						<Button variant="game-primary" onclick={() => loadGames()} size="sm" class="w-full"
							>Try Again</Button
						>
					</ComponentTraySection>
				{:else}
					<!-- Search Input -->
					<ComponentTraySection class="mb-3">
						<div class="relative">
							<SearchGlass
								size={16}
								class="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2"
							/>
							<input
								type="text"
								placeholder="Search games..."
								value={searchQuery}
								oninput={handleSearchInput}
								onkeydown={handleSearchKeydown}
								class="bg-parchment border-wood-dark/30 placeholder:text-muted-foreground h-9 w-full rounded-md border pr-3 pl-8 text-sm focus:ring-2 focus:ring-blue-500 focus:outline-none"
							/>
						</div>
					</ComponentTraySection>
					{#if loading}
						<ComponentTraySection>
							<LoadingSpinner text="Searching..." />
						</ComponentTraySection>
					{:else if games.length === 0}
						<ComponentTraySection>
							<p class="text-muted-foreground py-4 text-center text-sm">
								{searchQuery ? `No games match "${searchQuery}"` : 'No games found'}
							</p>
							{#if !searchQuery}
								<Button variant="game-primary" href="/games/add" size="sm" class="w-full"
									>Add Game</Button
								>
							{/if}
						</ComponentTraySection>
					{:else}
						<div class="max-h-80 space-y-2 overflow-y-auto">
							{#each games as game (game.id)}
								<button
									onclick={() => selectGame(game.id)}
									class="w-full rounded-lg p-3 text-left transition-all
										{selectedGameId === game.id
										? 'bg-game-blue text-white'
										: 'bg-parchment hover:bg-parchment-dark text-foreground'}"
								>
									<div class="flex items-start justify-between">
										<div class="min-w-0 flex-1">
											<div class="font-display truncate text-sm font-medium">{game.name}</div>
											{#if game.publisher}
												<div class="truncate text-xs opacity-70">{game.publisher}</div>
											{/if}
										</div>
										{#if game.hasRulesPdf}
											<Rulebook size={16} class="ml-2 flex-shrink-0 opacity-70" />
										{/if}
									</div>
								</button>
							{/each}
						</div>
					{/if}
				{/if}
			</ComponentTray>

			{#if selectedGame}
				<ComponentTray title="Game Info" class="mt-4">
					<ComponentTraySection>
						<div class="mb-3 flex items-center gap-3">
							<div
								class="bg-game-blue flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-lg"
							>
								<GameBoxIcon size={24} class="text-white" />
							</div>
							<div class="min-w-0 flex-1">
								<p class="font-display truncate font-semibold">{selectedGame.name}</p>
								{#if selectedGame.publisher}
									<p class="text-xs opacity-70">{selectedGame.publisher}</p>
								{/if}
							</div>
						</div>

						<div class="mb-3 grid grid-cols-2 gap-2 text-xs">
							{#if selectedGame.yearPublished}
								<div class="bg-parchment/30 rounded p-2 text-center">
									<p class="opacity-70">Year</p>
									<p class="font-semibold">{selectedGame.yearPublished}</p>
								</div>
							{/if}
							<div class="bg-parchment/30 rounded p-2 text-center">
								<p class="opacity-70">Players</p>
								<p class="font-semibold">
									{formatPlayerCount(selectedGame.minPlayers, selectedGame.maxPlayers)}
								</p>
							</div>
						</div>

						<Button
							variant="ghost"
							size="sm"
							onclick={() => selectedGame && goToGame(selectedGame.id)}
							class="w-full"
						>
							View Details
						</Button>
					</ComponentTraySection>
				</ComponentTray>
			{/if}
		</div>

		<!-- Upload Section -->
		<div class="lg:col-span-2">
			{#if selectedGame}
				<PDFUpload
					gameId={selectedGame.id}
					gameName={selectedGame.name}
					existingRulesInfo={rulesInfo}
					onUploaded={handleUploadSuccess}
					onDeleted={handleUploadDeleted}
					onError={handleUploadError}
				/>

				{#if uploadSuccess}
					<GameBox variant="featured" showCorners={true} class="mt-6">
						<div class="py-6 text-center">
							<div
								class="bg-game-green/20 mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full"
							>
								<svg
									class="text-game-green h-8 w-8"
									fill="none"
									stroke="currentColor"
									viewBox="0 0 24 24"
								>
									<path
										stroke-linecap="round"
										stroke-linejoin="round"
										stroke-width="2"
										d="M5 13l4 4L19 7"
									/>
								</svg>
							</div>
							<h3 class="font-display text-game-green mb-2 text-lg font-semibold">
								Upload Complete!
							</h3>
							<p class="text-muted-foreground font-body mb-6 text-sm">
								Your PDF has been processed and indexed. You can now search and chat about {selectedGame.name}.
							</p>
							<div class="flex flex-wrap justify-center gap-3">
								<Button variant="game-primary" onclick={goToChat} class="gap-2">
									<ChatBubble size={18} />
									Start Chat
								</Button>
								<Button
									variant="game-secondary"
									href="/search?gameId={selectedGame.id}"
									class="gap-2"
								>
									<SearchGlass size={18} />
									Search Rules
								</Button>
							</div>
						</div>
					</GameBox>
				{/if}
			{:else}
				<!-- No Game Selected - Open Box Style -->
				<GameBox
					variant="default"
					showCorners={true}
					class="flex min-h-96 items-center justify-center"
				>
					<div class="py-12 text-center">
						<div class="relative mx-auto mb-6 h-24 w-24">
							<!-- Open box illustration -->
							<div class="bg-wood-light absolute inset-0 rotate-2 transform rounded-lg"></div>
							<div
								class="bg-parchment absolute inset-2 flex -rotate-1 transform items-center justify-center rounded-lg"
							>
								<UploadIcon size={40} class="text-game-blue" />
							</div>
						</div>
						<h3 class="font-display mb-2 text-xl font-semibold">Select a Game</h3>
						<p class="text-muted-foreground font-body mx-auto max-w-sm">
							Choose a game from the list to upload its rule book PDF
						</p>
					</div>
				</GameBox>
			{/if}
		</div>
	</div>

	<!-- Features Section -->
	<div class="mt-12">
		<h2 class="font-display mb-6 text-center text-xl font-semibold">What Happens After Upload?</h2>
		<div class="grid grid-cols-1 gap-4 md:grid-cols-3">
			<GameBox variant="default" class="p-6 text-center">
				<div
					class="bg-game-blue/10 mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full"
				>
					<Rulebook size={24} class="text-game-blue" />
				</div>
				<h3 class="font-display mb-2 font-semibold">Text Extraction</h3>
				<p class="text-muted-foreground font-body text-sm">
					We extract and index all text content from your PDF for searching
				</p>
			</GameBox>

			<GameBox variant="default" class="p-6 text-center">
				<div
					class="bg-game-green/10 mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full"
				>
					<SearchGlass size={24} class="text-game-green" />
				</div>
				<h3 class="font-display mb-2 font-semibold">AI Processing</h3>
				<p class="text-muted-foreground font-body text-sm">
					Content is processed for semantic search and question answering
				</p>
			</GameBox>

			<GameBox variant="default" class="p-6 text-center">
				<div
					class="bg-game-purple/10 mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full"
				>
					<ChatBubble size={24} class="text-game-purple" />
				</div>
				<h3 class="font-display mb-2 font-semibold">Chat Ready</h3>
				<p class="text-muted-foreground font-body text-sm">
					Ask questions about rules and get instant, accurate answers
				</p>
			</GameBox>
		</div>
	</div>
</main>
