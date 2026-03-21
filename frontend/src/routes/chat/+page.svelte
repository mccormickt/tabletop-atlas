<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { browser } from '$app/environment';
	import {
		api,
		createDebouncedAction,
		type GameSummary,
		type ChatSessionSummary,
		type ChatHistory,
		type ChatMessage
	} from '$lib';
	import {
		Button,
		Input,
		Badge,
		GameBox,
		CardSleeve,
		LoadingSpinner,
		EmptyState
	} from '$lib/components/ui';
	import { ComponentTray, ComponentTraySection } from '$lib/components/ui';
	import { Meeple, Dice, GameBoxIcon, Rulebook, ChatBubble } from '$lib/components/icons';
	import { page } from '$app/state';
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';

	// Configure marked for synchronous parsing
	marked.setOptions({
		async: false,
		gfm: true, // GitHub Flavored Markdown
		breaks: true // Convert \n to <br>
	});

	// Wrap code blocks with a container for copy button functionality
	function wrapCodeBlocks(html: string): string {
		return html.replace(
			/<pre><code([^>]*)>([\s\S]*?)<\/code><\/pre>/g,
			`<div class="code-block-wrapper">
				<button class="copy-btn" onclick="window.copyCodeBlock(this)">Copy</button>
				<pre><code$1>$2</code></pre>
			</div>`
		);
	}

	function parseMarkdown(content: string): string {
		const rawHtml = marked.parse(content) as string;
		const sanitized = DOMPurify.sanitize(rawHtml, {
			ALLOWED_TAGS: [
				'p',
				'br',
				'strong',
				'em',
				'ul',
				'ol',
				'li',
				'h1',
				'h2',
				'h3',
				'h4',
				'h5',
				'h6',
				'blockquote',
				'pre',
				'code',
				'a',
				'table',
				'thead',
				'tbody',
				'tr',
				'th',
				'td',
				'hr',
				'del',
				'sup',
				'sub',
				'div',
				'button'
			],
			ALLOWED_ATTR: ['href', 'target', 'rel', 'class', 'onclick']
		});
		return wrapCodeBlocks(sanitized);
	}

	// Register copy function on window for onclick handlers in sanitized HTML
	if (browser) {
		(
			window as Window & { copyCodeBlock?: (button: HTMLButtonElement) => Promise<void> }
		).copyCodeBlock = async (button: HTMLButtonElement) => {
			const pre = button.closest('.code-block-wrapper')?.querySelector('pre');
			if (pre) {
				await navigator.clipboard.writeText(pre.textContent || '');
				button.textContent = 'Copied!';
				setTimeout(() => (button.textContent = 'Copy'), 2000);
			}
		};
	}

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
	let gameFilterQuery = $state('');
	const debouncedGameSearch = createDebouncedAction(() => loadGames(gameFilterQuery));

	let showGameDrawer = $state(false);
	let showSessionDrawer = $state(false);
	let chatMessagesContainer: HTMLDivElement | null = $state(null);

	let includeHouseRules = $derived(currentSession?.session.includeHouseRules ?? true);

	function scrollToBottom() {
		if (chatMessagesContainer) {
			// Use requestAnimationFrame to ensure DOM has updated
			requestAnimationFrame(() => {
				chatMessagesContainer?.scrollTo({
					top: chatMessagesContainer.scrollHeight,
					behavior: 'smooth'
				});
			});
		}
	}

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

	async function loadGames(search?: string) {
		loadingGames = true;
		error = null;

		try {
			// When searching, show all games; otherwise only show games with PDFs
			const hasRulesPdf = search ? undefined : true;

			const result = await api.methods.listGames({
				query: {
					page: 1,
					limit: 50,
					search: search || undefined,
					hasRulesPdf
				}
			});

			if (result.type === 'success') {
				games = result.data.items;
			} else {
				error = 'Failed to load games';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
		} finally {
			loadingGames = false;
		}
	}

	function handleGameSearchInput(event: Event) {
		gameFilterQuery = (event.target as HTMLInputElement).value;
		debouncedGameSearch.trigger();
	}

	function handleGameSearchKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			debouncedGameSearch.cancel();
			loadGames(gameFilterQuery);
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
		goto(resolve('/chat') + url.search, { replaceState: true });
	}

	async function loadChatSessions(gameId: number) {
		loadingSessions = true;
		error = null;

		try {
			const result = await api.methods.listChatSessions({
				query: { gameId: String(gameId), page: 1, limit: 50 }
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
				goto(resolve('/chat') + url.search, { replaceState: true });
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
				goto(resolve('/chat') + url.search, { replaceState: true });
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

		// Optimistically add user message to the UI immediately
		const optimisticUserMessage: ChatMessage = {
			id: -1, // Temporary ID
			sessionId: currentSession.session.id,
			role: 'user' as const,
			content: messageText,
			contextChunks: null,
			createdAt: new Date()
		};
		currentSession = {
			session: currentSession.session,
			messages: [...currentSession.messages, optimisticUserMessage]
		};
		scrollToBottom();

		try {
			const session = currentSession!;
			const result = await api.methods.chatWithRules({
				body: {
					sessionId: session.session.id,
					message: messageText
				}
			});

			if (result.type === 'success') {
				// Replace optimistic message with real user message and append assistant response
				const messagesWithoutOptimistic = session.messages.filter((m) => m.id !== -1);
				currentSession = {
					session: session.session,
					messages: [
						...messagesWithoutOptimistic,
						result.data.userMessage,
						result.data.assistantMessage
					]
				};
				scrollToBottom();
			} else {
				error = 'Failed to send message';
				// Remove optimistic message on error
				currentSession = {
					session: session.session,
					messages: session.messages.filter((m) => m.id !== -1)
				};
				newMessage = messageText;
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			// Remove optimistic message on error
			if (currentSession) {
				currentSession = {
					session: currentSession.session,
					messages: currentSession.messages.filter((m) => m.id !== -1)
				};
			}
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

<div class="bg-background min-h-screen">
	<div class="mx-auto max-w-7xl px-4 py-6 sm:px-6 lg:px-8">
		<!-- Page Header -->
		<div class="mb-6">
			<div class="rulebook-header">
				<h1 class="text-2xl md:text-3xl">Game Rules Chat</h1>
			</div>
			<p class="text-muted-foreground font-body text-center">
				Ask questions about game rules and get AI-powered answers
			</p>
		</div>

		{#if error}
			<div class="border-game-red bg-game-red/10 mb-4 rounded-lg border-2 p-4">
				<p class="text-game-red font-ui text-sm">{error}</p>
			</div>
		{/if}

		<!-- Mobile Action Buttons -->
		<div class="mb-4 flex gap-2 lg:hidden">
			<Button
				variant="game-secondary"
				size="sm"
				onclick={() => (showGameDrawer = true)}
				class="flex-1"
			>
				<GameBoxIcon size={16} class="mr-2" />
				{selectedGame ? selectedGame.name : 'Select Game'}
			</Button>
			{#if selectedGame}
				<Button
					variant="game-secondary"
					size="sm"
					onclick={() => (showSessionDrawer = true)}
					class="flex-1"
				>
					<ChatBubble size={16} class="mr-2" />
					Sessions
				</Button>
			{/if}
		</div>

		<div class="grid grid-cols-1 gap-6 lg:grid-cols-4">
			<!-- Game Selection Sidebar (Desktop) -->
			<div class="hidden space-y-4 lg:col-span-1 lg:block">
				<ComponentTray title="Select Game">
					<div class="mb-2">
						<Input
							value={gameFilterQuery}
							oninput={handleGameSearchInput}
							onkeydown={handleGameSearchKeydown}
							placeholder="Search games..."
							class="bg-parchment text-foreground placeholder:text-foreground/50 h-8 text-sm"
						/>
					</div>
					{#if loadingGames}
						<ComponentTraySection>
							<LoadingSpinner text="Searching..." />
						</ComponentTraySection>
					{:else if games.length === 0}
						<ComponentTraySection>
							{#if gameFilterQuery}
								<p class="text-parchment/70 py-2 text-center text-sm">No games match search</p>
							{:else}
								<EmptyState
									icon="game"
									title="No games available"
									description="Upload PDF rules for games to enable chat"
									size="sm"
								/>
							{/if}
						</ComponentTraySection>
					{:else}
						<div class="max-h-64 space-y-2 overflow-y-auto">
							{#each games as game (game.id)}
								<button
									onclick={() => selectGame(game)}
									class="w-full rounded-lg p-3 text-left transition-all
										{selectedGame?.id === game.id
										? 'bg-game-blue text-white'
										: 'bg-parchment hover:bg-parchment-dark text-foreground'}"
								>
									<div class="font-display text-sm font-medium">{game.name}</div>
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
							<Button
								variant="game-primary"
								size="sm"
								onclick={createNewSession}
								disabled={loadingSessions}
								class="w-full"
							>
								New Chat
							</Button>
						</div>
						{#if loadingSessions}
							<ComponentTraySection>
								<LoadingSpinner text="Loading..." />
							</ComponentTraySection>
						{:else if chatSessions.length === 0}
							<ComponentTraySection>
								<p class="text-parchment/70 text-center text-xs">No chat sessions yet</p>
							</ComponentTraySection>
						{:else}
							<div class="max-h-64 space-y-2 overflow-y-auto">
								{#each chatSessions as session (session.id)}
									<button
										onclick={() => loadChatSession(session.id)}
										class="w-full rounded p-2 text-left text-sm transition-all
											{currentSession?.session.id === session.id
											? 'bg-game-blue text-white'
											: 'bg-parchment/20 hover:bg-parchment/40 text-parchment'}"
									>
										<div class="truncate font-medium">{session.title || `Chat ${session.id}`}</div>
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
								<div
									class="bg-parchment-dark mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full"
								>
									<GameBoxIcon size={40} class="text-game-blue" />
								</div>
								<h3 class="font-display mb-2 text-lg font-semibold">Select a Game</h3>
								<p class="text-muted-foreground font-body max-w-sm text-sm">
									Choose a game from the sidebar to start asking questions about its rules
								</p>
							</div>
						</div>
					</GameBox>
				{:else if !currentSession}
					<GameBox variant="default" showCorners={true} class="h-[calc(100vh-16rem)]">
						<div class="flex h-full items-center justify-center">
							<div class="text-center">
								<div
									class="bg-parchment-dark mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-full"
								>
									<ChatBubble size={40} class="text-game-purple" />
								</div>
								<h3 class="font-display mb-2 text-lg font-semibold">Start a Conversation</h3>
								<p class="text-muted-foreground font-body mb-4 max-w-sm text-sm">
									Create a new chat session to ask questions about {selectedGame.name}
								</p>
								<Button variant="game-primary" onclick={createNewSession}>New Chat</Button>
							</div>
						</div>
					</GameBox>
				{:else}
					<div class="game-box-lid flex h-[calc(100vh-16rem)] flex-col">
						<!-- Chat Header -->
						<div class="border-wood-dark bg-parchment-dark/50 flex-shrink-0 border-b-2 p-4">
							<div class="flex items-center justify-between">
								<div>
									<h2 class="font-display font-semibold">
										{currentSession.session.title || `Chat about ${selectedGame.name}`}
									</h2>
									<p class="text-muted-foreground font-ui text-sm">
										{selectedGame.name}
									</p>
								</div>
								<div class="flex items-center gap-3">
									<label class="flex cursor-pointer items-center gap-2">
										<span class="font-ui text-muted-foreground hidden text-sm sm:inline"
											>House Rules</span
										>
										<button
											type="button"
											onclick={toggleHouseRules}
											disabled={togglingHouseRules}
											class="relative inline-flex h-6 w-11 flex-shrink-0 rounded-full border-2 transition-colors duration-200
												{includeHouseRules ? 'bg-game-green border-game-green' : 'bg-muted border-border'}
												disabled:cursor-not-allowed disabled:opacity-50"
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
						<div
							bind:this={chatMessagesContainer}
							class="paper-texture flex-1 space-y-4 overflow-y-auto p-4"
						>
							{#if loadingCurrentSession}
								<div class="flex items-center justify-center py-8">
									<LoadingSpinner text="Loading conversation..." />
								</div>
							{:else if currentSession.messages.length === 0}
								<div class="flex items-center justify-center py-8">
									<div class="text-center">
										<Dice size={32} value={6} class="text-muted-foreground mx-auto mb-2" />
										<p class="text-muted-foreground font-body text-sm">
											Start by asking a question about the game rules
										</p>
									</div>
								</div>
							{:else}
								{#each currentSession.messages as message (message.id)}
									<div
										class="flex items-start gap-3 {message.role === 'user'
											? 'flex-row-reverse'
											: ''}"
									>
										<!-- Avatar -->
										<div class="flex-shrink-0">
											{#if message.role === 'user'}
												<div
													class="bg-game-blue flex h-10 w-10 items-center justify-center rounded-full shadow-md"
												>
													<Meeple size={20} color="current" class="text-white" />
												</div>
											{:else}
												<div
													class="bg-game-purple flex h-10 w-10 items-center justify-center rounded-full shadow-md"
												>
													<Dice size={20} value={6} class="text-white" />
												</div>
											{/if}
										</div>

										<!-- Message Bubble -->
										<div class="max-w-[80%] min-w-0 flex-1">
											<div
												class="mb-1 flex items-center gap-2 {message.role === 'user'
													? 'flex-row-reverse'
													: ''}"
											>
												<Badge variant="outline" class="font-ui text-xs capitalize">
													{message.role === 'user' ? 'You' : 'Game Master'}
												</Badge>
												{#if message.createdAt}
													<span class="text-muted-foreground font-ui text-xs">
														{formatTime(message.createdAt)}
													</span>
												{/if}
											</div>
											<div
												class="rounded-lg p-3 shadow-sm
												{message.role === 'user'
													? 'bg-game-blue rounded-tr-none text-white'
													: 'bg-card border-border rounded-tl-none border-2'}"
											>
												{#if message.role === 'assistant'}
													<div
														class="prose prose-sm prose-chat font-body dark:prose-invert max-w-none overflow-x-auto"
													>
														<!-- eslint-disable-next-line svelte/no-at-html-tags -->
														{@html parseMarkdown(message.content)}
													</div>
												{:else}
													<p class="font-body text-sm whitespace-pre-wrap">{message.content}</p>
												{/if}
											</div>
										</div>
									</div>
								{/each}

								<!-- Typing indicator while waiting for AI response -->
								{#if sendingMessage}
									<div class="flex items-start gap-3">
										<div class="flex-shrink-0">
											<div
												class="bg-game-purple flex h-10 w-10 items-center justify-center rounded-full shadow-md"
											>
												<Dice size={20} value={6} class="text-white" />
											</div>
										</div>
										<div class="max-w-[80%] min-w-0 flex-1">
											<div class="mb-1 flex items-center gap-2">
												<Badge variant="outline" class="font-ui text-xs">Game Master</Badge>
											</div>
											<div
												class="bg-card border-border rounded-lg rounded-tl-none border-2 p-3 shadow-sm"
											>
												<div class="flex items-center gap-2">
													<div class="flex gap-1">
														<span
															class="bg-muted-foreground h-2 w-2 animate-bounce rounded-full [animation-delay:-0.3s]"
														></span>
														<span
															class="bg-muted-foreground h-2 w-2 animate-bounce rounded-full [animation-delay:-0.15s]"
														></span>
														<span class="bg-muted-foreground h-2 w-2 animate-bounce rounded-full"
														></span>
													</div>
													<span class="text-muted-foreground font-body text-sm">Thinking...</span>
												</div>
											</div>
										</div>
									</div>
								{/if}
							{/if}
						</div>

						<!-- Message Input -->
						<div class="border-wood-dark bg-parchment-dark/50 flex-shrink-0 border-t-2 p-4">
							<div class="flex gap-2">
								<Input
									bind:value={newMessage}
									placeholder="Ask about game rules..."
									disabled={sendingMessage}
									onkeydown={handleKeydown}
									class="bg-card flex-1"
								/>
								<Button
									variant="game-primary"
									onclick={sendMessage}
									disabled={!newMessage.trim() || sendingMessage}
								>
									{#if sendingMessage}
										<LoadingSpinner size="sm" class="mr-2" />
									{:else}
										<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
											/>
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
		<div class="absolute inset-0 bg-black/50" onclick={() => (showGameDrawer = false)}></div>
		<div class="bg-background absolute top-0 bottom-0 left-0 w-80 overflow-y-auto shadow-xl">
			<div class="border-wood-dark bg-parchment-dark sticky top-0 border-b-2 p-4">
				<div class="flex items-center justify-between">
					<h2 class="font-display font-semibold">Select Game</h2>
					<button onclick={() => (showGameDrawer = false)} class="hover:bg-muted rounded p-2">
						<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M6 18L18 6M6 6l12 12"
							/>
						</svg>
					</button>
				</div>
			</div>
			<div class="p-4">
				<div class="mb-3">
					<Input
						value={gameFilterQuery}
						oninput={handleGameSearchInput}
						onkeydown={handleGameSearchKeydown}
						placeholder="Search games..."
						class="h-8 text-sm"
					/>
				</div>
				{#if loadingGames}
					<p class="text-muted-foreground py-2 text-center text-sm">Searching...</p>
				{:else if games.length === 0}
					<p class="text-muted-foreground py-2 text-center text-sm">
						{gameFilterQuery ? 'No games match search' : 'No games found'}
					</p>
				{:else}
					<div class="space-y-2">
						{#each games as game (game.id)}
							<CardSleeve variant={selectedGame?.id === game.id ? 'highlighted' : 'default'}>
								<button onclick={() => selectGame(game)} class="w-full text-left">
									<div class="font-display font-medium">{game.name}</div>
									{#if game.publisher}
										<div class="text-muted-foreground text-sm">{game.publisher}</div>
									{/if}
								</button>
							</CardSleeve>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}

<!-- Mobile Session Drawer -->
{#if showSessionDrawer}
	<div class="fixed inset-0 z-50 lg:hidden">
		<div class="absolute inset-0 bg-black/50" onclick={() => (showSessionDrawer = false)}></div>
		<div class="bg-background absolute top-0 right-0 bottom-0 w-80 overflow-y-auto shadow-xl">
			<div class="border-wood-dark bg-parchment-dark sticky top-0 border-b-2 p-4">
				<div class="flex items-center justify-between">
					<h2 class="font-display font-semibold">Chat Sessions</h2>
					<button onclick={() => (showSessionDrawer = false)} class="hover:bg-muted rounded p-2">
						<svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M6 18L18 6M6 6l12 12"
							/>
						</svg>
					</button>
				</div>
			</div>
			<div class="p-4">
				<Button variant="game-primary" onclick={createNewSession} class="mb-4 w-full">
					New Chat
				</Button>
				<div class="space-y-2">
					{#each chatSessions as session (session.id)}
						<CardSleeve
							variant={currentSession?.session.id === session.id ? 'highlighted' : 'default'}
						>
							<button onclick={() => loadChatSession(session.id)} class="w-full text-left">
								<div class="font-display font-medium">{session.title || `Chat ${session.id}`}</div>
								<div class="text-muted-foreground text-sm">
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
