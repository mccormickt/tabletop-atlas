<script lang="ts">
	import { goto } from '$app/navigation';
	import {
		api,
		type GameSummary,
		type ChatSessionSummary,
		type ChatHistory
	} from '$lib';
	import { Button, Input, Badge, GameBox, CardSleeve, LoadingSpinner, EmptyState } from '$lib/components/ui';
	import { ComponentTray, ComponentTraySection } from '$lib/components/ui';
	import { Meeple, Dice, GameBoxIcon, Rulebook, ChatBubble } from '$lib/components/icons';
	import { page } from '$app/state';

	let games = $state<GameSummary[]>([]);
	let selectedGame = $state<GameSummary | null>(null);
	let chatSessions = $state<ChatSessionSummary[]>([]);
	let currentSession = $state<ChatHistory | null>(null);
	let newMessage = $state('');
	let loadingGames = $state(false);
	let loadingSessions = $state(false);
	let loadingCurrentSession = $state(false);
	let sendingMessage = $state(false);
	let error = $state<string | null>(null);
	let togglingHouseRules = $state(false);

	let showGameDrawer = $state(false);
	let showSessionDrawer = $state(false);

	let includeHouseRules = $derived(currentSession?.session.includeHouseRules ?? true);

	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			initialize();
		}
	});

	async function initialize() {
		await loadGames();

		if (games.length > 0) {
			const gameIdParam = page.url.searchParams.get('game_id');
			const sessionIdParam = page.url.searchParams.get('session_id');

			if (gameIdParam) {
				const gameId = parseInt(gameIdParam);
				const game = games.find((g) => g.id === gameId);
				if (game) {
					selectedGame = game;
					await loadChatSessions(gameId);

					if (sessionIdParam) {
						const sessionId = parseInt(sessionIdParam);
						await loadChatSession(sessionId);
					}
				}
			}
		}
	}

	async function loadGames() {
		loadingGames = true;
		error = null;

		try {
			const result = await api.methods.listGames({
				query: { page: 1, limit: 100 }
			});

			if (result.type === 'success') {
				const gamesWithPdf = result.data.items.filter((game) => game.hasRulesPdf);
				games = gamesWithPdf.length > 0 ? gamesWithPdf : result.data.items;
			} else {
				error = 'Failed to load games';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			loadingGames = false;
		}
	}

	async function selectGame(game: GameSummary) {
		selectedGame = game;
		currentSession = null;
		showGameDrawer = false;
		await loadChatSessions(game.id);

		const url = new URL(window.location.href);
		url.searchParams.set('game_id', game.id.toString());
		url.searchParams.delete('session_id');
		goto(url.toString(), { replaceState: true });
	}

	async function loadChatSessions(gameId: number) {
		loadingSessions = true;
		error = null;

		try {
			const result = await api.methods.listChatSessions({
				query: { gameId, page: 1, limit: 50 }
			});

			if (result.type === 'success') {
				chatSessions = result.data.items;
			} else {
				error = 'Failed to load chat sessions';
				chatSessions = [];
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			chatSessions = [];
		} finally {
			loadingSessions = false;
		}
	}

	async function createNewSession() {
		if (!selectedGame) return;

		loadingSessions = true;
		error = null;

		try {
			const result = await api.methods.createChatSession({
				body: {
					gameId: selectedGame.id,
					title: `Chat about ${selectedGame.name}`
				}
			});

			if (result.type === 'success') {
				await loadChatSessions(selectedGame.id);
				await loadChatSession(result.data.id);
				showSessionDrawer = false;

				const url = new URL(window.location.href);
				url.searchParams.set('session_id', result.data.id.toString());
				goto(url.toString(), { replaceState: true });
			} else {
				error = 'Failed to create chat session';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			loadingSessions = false;
		}
	}

	async function loadChatSession(sessionId: number) {
		loadingCurrentSession = true;
		error = null;
		showSessionDrawer = false;

		try {
			const result = await api.methods.getChatSession({
				path: { id: sessionId }
			});

			if (result.type === 'success') {
				currentSession = result.data;

				const url = new URL(window.location.href);
				url.searchParams.set('session_id', sessionId.toString());
				goto(url.toString(), { replaceState: true });
			} else {
				error = 'Failed to load chat session';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			loadingCurrentSession = false;
		}
	}

	async function sendMessage() {
		if (!currentSession || !newMessage.trim()) return;

		sendingMessage = true;
		error = null;

		const messageText = newMessage.trim();
		newMessage = '';

		try {
			const result = await api.methods.chatWithRules({
				body: {
					sessionId: currentSession.session.id,
					message: messageText
				}
			});

			if (result.type === 'success') {
				await loadChatSession(currentSession.session.id);
			} else {
				error = 'Failed to send message';
				newMessage = messageText;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			newMessage = messageText;
		} finally {
			sendingMessage = false;
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey) {
			event.preventDefault();
			sendMessage();
		}
	}

	async function toggleHouseRules() {
		if (!currentSession) return;

		togglingHouseRules = true;
		try {
			const result = await api.methods.updateChatSession({
				path: { id: currentSession.session.id },
				body: {
					includeHouseRules: !includeHouseRules
				}
			});

			if (result.type === 'success') {
				currentSession = {
					...currentSession,
					session: {
						...currentSession.session,
						includeHouseRules: result.data.includeHouseRules
					}
				};
			} else {
				error = 'Failed to update session settings';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			togglingHouseRules = false;
		}
	}

	function formatTime(date: Date | string): string {
		try {
			const dateObj = typeof date === 'string' ? new Date(date) : date;
			if (isNaN(dateObj.getTime())) {
				return '';
			}
			return new Intl.DateTimeFormat('en-US', {
				hour: 'numeric',
				minute: '2-digit',
				hour12: true
			}).format(dateObj);
		} catch {
			return '';
		}
	}
</script>

<svelte:head>
	<title>Chat - Tabletop Atlas</title>
</svelte:head>

<div class="min-h-screen bg-background">
	<div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
		<!-- Page Header -->
		<div class="mb-6">
			<div class="rulebook-header">
				<h1 class="text-2xl md:text-3xl">Game Rules Chat</h1>
			</div>
			<p class="text-center text-muted-foreground font-body">
				Ask questions about game rules and get AI-powered answers
			</p>
		</div>

		{#if error}
			<div class="mb-4 rounded-lg border-2 border-game-red bg-game-red/10 p-4">
				<p class="text-sm text-game-red font-ui">{error}</p>
			</div>
		{/if}

		<!-- Mobile Action Buttons -->
		<div class="flex gap-2 mb-4 lg:hidden">
			<Button variant="game-secondary" size="sm" onclick={() => showGameDrawer = true} class="flex-1">
				<GameBoxIcon size={16} class="mr-2" />
				{selectedGame ? selectedGame.name : 'Select Game'}
			</Button>
			{#if selectedGame}
				<Button variant="game-secondary" size="sm" onclick={() => showSessionDrawer = true} class="flex-1">
					<ChatBubble size={16} class="mr-2" />
					Sessions
				</Button>
			{/if}
		</div>

		<div class="grid grid-cols-1 gap-6 lg:grid-cols-4">
			<!-- Game Selection Sidebar (Desktop) -->
			<div class="hidden lg:block lg:col-span-1 space-y-4">
				<ComponentTray title="Select Game">
					{#if loadingGames && games.length === 0}
						<ComponentTraySection>
							<LoadingSpinner text="Loading games..." />
						</ComponentTraySection>
					{:else if games.length === 0}
						<ComponentTraySection>
							<EmptyState
								icon="game"
								title="No games available"
								description="Upload PDF rules for games to enable chat"
								size="sm"
							/>
						</ComponentTraySection>
					{:else}
						<div class="space-y-2">
							{#each games as game}
								<button
									onclick={() => selectGame(game)}
									class="w-full text-left p-3 rounded-lg transition-all
										{selectedGame?.id === game.id
											? 'bg-game-blue text-white'
											: 'bg-parchment hover:bg-parchment-dark text-foreground'}"
								>
									<div class="font-display font-medium text-sm">{game.name}</div>
									{#if game.publisher}
										<div class="text-xs opacity-70">{game.publisher}</div>
									{/if}
									{#if game.hasRulesPdf}
										<div class="mt-1">
											<Rulebook size={12} class="inline opacity-60" />
										</div>
									{/if}
								</button>
							{/each}
						</div>
					{/if}
				</ComponentTray>

				{#if selectedGame}
					<ComponentTray title="Chat Sessions">
						<div class="mb-3">
							<Button variant="game-primary" size="sm" onclick={createNewSession} disabled={loadingSessions} class="w-full">
								New Chat
							</Button>
						</div>
						{#if loadingSessions}
							<ComponentTraySection>
								<LoadingSpinner text="Loading..." />
							</ComponentTraySection>
						{:else if chatSessions.length === 0}
							<ComponentTraySection>
								<p class="text-xs text-parchment/70 text-center">No chat sessions yet</p>
							</ComponentTraySection>
						{:else}
							<div class="space-y-2 max-h-64 overflow-y-auto">
								{#each chatSessions as session (session.id)}
									<button
										onclick={() => loadChatSession(session.id)}
										class="w-full text-left p-2 rounded transition-all text-sm
											{currentSession?.session.id === session.id
												? 'bg-game-blue text-white'
												: 'bg-parchment/20 hover:bg-parchment/40 text-parchment'}"
									>
										<div class="font-medium truncate">{session.title || `Chat ${session.id}`}</div>
										<div class="text-xs opacity-70">
											{session.messageCount} msg{session.messageCount === 1 ? '' : 's'}
										</div>
									</button>
								{/each}
							</div>
						{/if}
					</ComponentTray>
				{/if}
			</div>

			<!-- Chat Interface -->
			<div class="lg:col-span-3">
				{#if !selectedGame}
					<GameBox variant="default" showCorners={true} class="h-[calc(100vh-16rem)]">
						<div class="flex h-full items-center justify-center">
							<div class="text-center">
								<div class="mx-auto w-20 h-20 rounded-full bg-parchment-dark flex items-center justify-center mb-4">
									<GameBoxIcon size={40} class="text-game-blue" />
								</div>
								<h3 class="font-display font-semibold text-lg mb-2">Select a Game</h3>
								<p class="text-muted-foreground font-body text-sm max-w-sm">
									Choose a game from the sidebar to start asking questions about its rules
								</p>
							</div>
						</div>
					</GameBox>
				{:else if !currentSession}
					<GameBox variant="default" showCorners={true} class="h-[calc(100vh-16rem)]">
						<div class="flex h-full items-center justify-center">
							<div class="text-center">
								<div class="mx-auto w-20 h-20 rounded-full bg-parchment-dark flex items-center justify-center mb-4">
									<ChatBubble size={40} class="text-game-purple" />
								</div>
								<h3 class="font-display font-semibold text-lg mb-2">Start a Conversation</h3>
								<p class="text-muted-foreground font-body text-sm max-w-sm mb-4">
									Create a new chat session to ask questions about {selectedGame.name}
								</p>
								<Button variant="game-primary" onclick={createNewSession}>
									New Chat
								</Button>
							</div>
						</div>
					</GameBox>
				{:else}
					<div class="game-box-lid flex h-[calc(100vh-16rem)] flex-col">
						<!-- Chat Header -->
						<div class="flex-shrink-0 p-4 border-b-2 border-wood-dark bg-parchment-dark/50">
							<div class="flex items-center justify-between">
								<div>
									<h2 class="font-display font-semibold">
										{currentSession.session.title || `Chat about ${selectedGame.name}`}
									</h2>
									<p class="text-sm text-muted-foreground font-ui">
										{selectedGame.name}
									</p>
								</div>
								<div class="flex items-center gap-3">
									<label class="flex items-center gap-2 cursor-pointer">
										<span class="text-sm font-ui text-muted-foreground hidden sm:inline">House Rules</span>
										<button
											type="button"
											onclick={toggleHouseRules}
											disabled={togglingHouseRules}
											class="relative inline-flex h-6 w-11 flex-shrink-0 rounded-full border-2 transition-colors duration-200
												{includeHouseRules ? 'bg-game-green border-game-green' : 'bg-muted border-border'}
												disabled:opacity-50 disabled:cursor-not-allowed"
											role="switch"
											aria-checked={includeHouseRules}
										>
											<span
												class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow transition duration-200
													{includeHouseRules ? 'translate-x-5' : 'translate-x-0'}"
											></span>
										</button>
									</label>
								</div>
							</div>
						</div>

						<!-- Messages - Parchment Scroll Style -->
						<div class="flex-1 overflow-y-auto p-4 space-y-4 paper-texture">
							{#if loadingCurrentSession}
								<div class="flex items-center justify-center py-8">
									<LoadingSpinner text="Loading conversation..." />
								</div>
							{:else if currentSession.messages.length === 0}
								<div class="flex items-center justify-center py-8">
									<div class="text-center">
										<Dice size={32} value={6} class="mx-auto mb-2 text-muted-foreground" />
										<p class="text-muted-foreground font-body text-sm">
											Start by asking a question about the game rules
										</p>
									</div>
								</div>
							{:else}
								{#each currentSession.messages as message}
									<div class="flex items-start gap-3 {message.role === 'user' ? 'flex-row-reverse' : ''}">
										<!-- Avatar -->
										<div class="flex-shrink-0">
											{#if message.role === 'user'}
												<div class="w-10 h-10 rounded-full bg-game-blue flex items-center justify-center shadow-md">
													<Meeple size={20} color="current" class="text-white" />
												</div>
											{:else}
												<div class="w-10 h-10 rounded-full bg-game-purple flex items-center justify-center shadow-md">
													<Dice size={20} value={6} class="text-white" />
												</div>
											{/if}
										</div>

										<!-- Message Bubble -->
										<div class="flex-1 min-w-0 max-w-[80%]">
											<div class="flex items-center gap-2 mb-1 {message.role === 'user' ? 'flex-row-reverse' : ''}">
												<Badge variant="outline" class="text-xs capitalize font-ui">
													{message.role === 'user' ? 'You' : 'Game Master'}
												</Badge>
												{#if message.createdAt}
													<span class="text-xs text-muted-foreground font-ui">
														{formatTime(message.createdAt)}
													</span>
												{/if}
											</div>
											<div class="rounded-lg p-3 shadow-sm
												{message.role === 'user'
													? 'bg-game-blue text-white rounded-tr-none'
													: 'bg-card border-2 border-border rounded-tl-none'}">
												<p class="whitespace-pre-wrap font-body text-sm">{message.content}</p>
											</div>
										</div>
									</div>
								{/each}
							{/if}
						</div>

						<!-- Message Input -->
						<div class="flex-shrink-0 p-4 border-t-2 border-wood-dark bg-parchment-dark/50">
							<div class="flex gap-2">
								<Input
									bind:value={newMessage}
									placeholder="Ask about game rules..."
									disabled={sendingMessage}
									onkeydown={handleKeydown}
									class="flex-1 bg-card"
								/>
								<Button
									variant="game-primary"
									onclick={sendMessage}
									disabled={!newMessage.trim() || sendingMessage}
								>
									{#if sendingMessage}
										<LoadingSpinner size="sm" class="mr-2" />
									{:else}
										<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
										</svg>
									{/if}
								</Button>
							</div>
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>

<!-- Mobile Game Drawer -->
{#if showGameDrawer}
	<div class="fixed inset-0 z-50 lg:hidden">
		<div class="absolute inset-0 bg-black/50" onclick={() => showGameDrawer = false}></div>
		<div class="absolute left-0 top-0 bottom-0 w-80 bg-background shadow-xl overflow-y-auto">
			<div class="p-4 border-b-2 border-wood-dark bg-parchment-dark sticky top-0">
				<div class="flex items-center justify-between">
					<h2 class="font-display font-semibold">Select Game</h2>
					<button onclick={() => showGameDrawer = false} class="p-2 hover:bg-muted rounded">
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
						</svg>
					</button>
				</div>
			</div>
			<div class="p-4 space-y-2">
				{#each games as game}
					<CardSleeve variant={selectedGame?.id === game.id ? 'highlighted' : 'default'}>
						<button onclick={() => selectGame(game)} class="w-full text-left">
							<div class="font-display font-medium">{game.name}</div>
							{#if game.publisher}
								<div class="text-sm text-muted-foreground">{game.publisher}</div>
							{/if}
						</button>
					</CardSleeve>
				{/each}
			</div>
		</div>
	</div>
{/if}

<!-- Mobile Session Drawer -->
{#if showSessionDrawer}
	<div class="fixed inset-0 z-50 lg:hidden">
		<div class="absolute inset-0 bg-black/50" onclick={() => showSessionDrawer = false}></div>
		<div class="absolute right-0 top-0 bottom-0 w-80 bg-background shadow-xl overflow-y-auto">
			<div class="p-4 border-b-2 border-wood-dark bg-parchment-dark sticky top-0">
				<div class="flex items-center justify-between">
					<h2 class="font-display font-semibold">Chat Sessions</h2>
					<button onclick={() => showSessionDrawer = false} class="p-2 hover:bg-muted rounded">
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
						</svg>
					</button>
				</div>
			</div>
			<div class="p-4">
				<Button variant="game-primary" onclick={createNewSession} class="w-full mb-4">
					New Chat
				</Button>
				<div class="space-y-2">
					{#each chatSessions as session (session.id)}
						<CardSleeve variant={currentSession?.session.id === session.id ? 'highlighted' : 'default'}>
							<button onclick={() => loadChatSession(session.id)} class="w-full text-left">
								<div class="font-display font-medium">{session.title || `Chat ${session.id}`}</div>
								<div class="text-sm text-muted-foreground">
									{session.messageCount} message{session.messageCount === 1 ? '' : 's'}
								</div>
							</button>
						</CardSleeve>
					{/each}
				</div>
			</div>
		</div>
	</div>
{/if}
