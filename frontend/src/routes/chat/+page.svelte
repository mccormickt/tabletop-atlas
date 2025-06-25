<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import {
		api,
		type GameSummary,
		type ChatSessionSummary,
		type ChatHistory,
		type ChatMessage
	} from '$lib';
	import { Button } from '$lib/components/ui/button';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Badge } from '$lib/components/ui/badge';
	import { LoadingSpinner, EmptyState } from '$lib/components/ui';

	// State
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

	// Load games on mount
	onMount(async () => {
		await loadGames();

		// Wait for games to be loaded, then process URL params
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
	});

	async function loadGames() {
		loadingGames = true;
		error = null;

		try {
			const result = await api.methods.listGames({
				query: { page: 1, limit: 100 }
			});

			if (result.type === 'success') {
				// Only show games that have rules PDF (and therefore embeddings)
				const gamesWithPdf = result.data.items.filter((game) => game.hasRulesPdf);

				// Use games with PDF rules, or fallback to all games for testing
				games = gamesWithPdf.length > 0 ? gamesWithPdf : result.data.items;
			} else {
				error = 'Failed to load games';
				// Error already set above
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			// Error already set above
		} finally {
			loadingGames = false;
		}
	}

	async function selectGame(game: GameSummary) {
		selectedGame = game;
		currentSession = null;
		await loadChatSessions(game.id);

		// Update URL
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
				// Reload sessions first, then load the new session
				await loadChatSessions(selectedGame.id);
				await loadChatSession(result.data.id);

				// Update URL
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

		try {
			const result = await api.methods.getChatSession({
				path: { id: sessionId }
			});

			if (result.type === 'success') {
				currentSession = result.data;
				console.log('Debug - Chat session data:', currentSession);
				console.log('Debug - Messages:', currentSession.messages);
				currentSession.messages.forEach((msg, idx) => {
					console.log(`Debug - Message ${idx}:`, {
						id: msg.id,
						role: msg.role,
						content: msg.content.substring(0, 50) + '...',
						createdAt: msg.createdAt,
						createdAtType: typeof msg.createdAt,
						createdAtValue: msg.createdAt?.toString()
					});
				});

				// Update URL
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
		newMessage = ''; // Clear input immediately

		try {
			const result = await api.methods.chatWithRules({
				body: {
					sessionId: currentSession.session.id,
					message: messageText
				}
			});

			if (result.type === 'success') {
				// Reload the chat session to get updated messages
				await loadChatSession(currentSession.session.id);
			} else {
				error = 'Failed to send message';
				newMessage = messageText; // Restore message on error
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'An unexpected error occurred';
			newMessage = messageText; // Restore message on error
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

	function formatTime(date: Date | string): string {
		try {
			const dateObj = typeof date === 'string' ? new Date(date) : date;
			if (isNaN(dateObj.getTime())) {
				return 'Invalid date';
			}
			return new Intl.DateTimeFormat('en-US', {
				hour: 'numeric',
				minute: '2-digit',
				hour12: true
			}).format(dateObj);
		} catch (error) {
			console.error('Date formatting error:', error, 'for date:', date);
			return 'Invalid date';
		}
	}

	function getRoleColor(role: string): string {
		switch (role) {
			case 'user':
				return 'bg-blue-500 text-white';
			case 'assistant':
				return 'bg-green-500 text-white';
			case 'system':
				return 'bg-gray-500 text-white';
			default:
				return 'bg-gray-400 text-white';
		}
	}
</script>

<svelte:head>
	<title>Chat - Tabletop Atlas</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 p-4">
	<div class="mx-auto max-w-7xl">
		<div class="mb-6">
			<h1 class="text-3xl font-bold text-gray-900">Game Rules Chat</h1>
			<p class="mt-2 text-gray-600">
				Ask questions about game rules and get AI-powered answers with context from the rulebook.
			</p>
		</div>

		{#if error}
			<div class="mb-6 rounded-md border border-red-200 bg-red-50 p-4">
				<p class="text-sm text-red-700">{error}</p>
			</div>
		{/if}

		<div class="grid grid-cols-1 gap-6 lg:grid-cols-4">
			<!-- Game Selection Sidebar -->
			<div class="lg:col-span-1">
				<Card>
					<CardHeader>
						<CardTitle>Select Game</CardTitle>
						<CardDescription>Choose a game with uploaded rules to start chatting</CardDescription>
					</CardHeader>
					<CardContent class="space-y-2">
						{#if loadingGames && games.length === 0}
							<LoadingSpinner text="Loading games..." />
						{:else if games.length === 0}
							<EmptyState
								icon="game"
								title="No games available"
								description="Upload PDF rules for games to enable chat"
								size="sm"
							/>
						{:else}
							{#each games as game}
								<button
									onclick={() => selectGame(game)}
									class="w-full rounded-lg border p-3 text-left transition-colors hover:bg-gray-50 {selectedGame?.id ===
									game.id
										? 'border-blue-500 bg-blue-50'
										: 'border-gray-200'}"
								>
									<div class="font-medium">{game.name}</div>
									{#if game.publisher}
										<div class="text-sm text-gray-600">{game.publisher}</div>
									{/if}
									<Badge variant="secondary" class="mt-1 text-xs">PDF Available</Badge>
								</button>
							{/each}
						{/if}
					</CardContent>
				</Card>

				<!-- Chat Sessions -->
				{#if selectedGame}
					<Card class="mt-4">
						<CardHeader>
							<CardTitle class="flex items-center justify-between">
								<span>Chat Sessions</span>
								<Button size="sm" onclick={createNewSession} disabled={loadingSessions}
									>New Chat</Button
								>
							</CardTitle>
							<CardDescription>Previous conversations about {selectedGame.name}</CardDescription>
						</CardHeader>
						<CardContent class="space-y-2">
							{#if loadingSessions}
								<LoadingSpinner text="Loading sessions..." />
							{:else if chatSessions.length === 0}
								<EmptyState
									icon="chat"
									title="No chat sessions"
									description="Create a new chat to get started"
									size="sm"
								/>
							{:else}
								{#each chatSessions as session (session.id)}
									<button
										onclick={() => loadChatSession(session.id)}
										class="w-full rounded-lg border p-3 text-left transition-colors hover:bg-gray-50 {currentSession
											?.session.id === session.id
											? 'border-blue-500 bg-blue-50'
											: 'border-gray-200'}"
									>
										<div class="font-medium">
											{session.title || `Chat ${session.id}`}
										</div>
										<div class="text-sm text-gray-600">
											{session.messageCount} message{session.messageCount === 1 ? '' : 's'}
											{#if session.lastMessageAt}
												• {session.lastMessageAt
													? formatTime(session.lastMessageAt)
													: 'Unknown time'}
											{/if}
										</div>
									</button>
								{/each}
							{/if}
						</CardContent>
					</Card>
				{/if}
			</div>

			<!-- Chat Interface -->
			<div class="lg:col-span-3">
				{#if !selectedGame}
					<Card class="h-[calc(100vh-12rem)]">
						<CardContent class="flex h-full items-center justify-center">
							<EmptyState
								icon="game"
								title="Select a game to start chatting"
								description="Choose a game from the sidebar to begin asking questions about its rules"
							/>
						</CardContent>
					</Card>
				{:else if !currentSession}
					<Card class="h-[calc(100vh-12rem)]">
						<CardContent class="flex h-full items-center justify-center">
							<EmptyState
								icon="chat"
								title="Start a new conversation"
								description="Create a new chat session to ask questions about {selectedGame.name}"
								actionText="New Chat"
								onAction={createNewSession}
							/>
						</CardContent>
					</Card>
				{:else}
					<Card class="flex h-[calc(100vh-12rem)] flex-col">
						<CardHeader class="flex-shrink-0">
							<CardTitle>
								{currentSession.session.title || `Chat about ${selectedGame.name}`}
							</CardTitle>
							<CardDescription>
								Ask questions about {selectedGame.name} rules and get AI-powered answers
							</CardDescription>
						</CardHeader>

						<!-- Messages -->
						<CardContent class="flex-1 space-y-4 overflow-y-auto">
							{#if currentSession.messages.length === 0}
								<EmptyState
									icon="chat"
									title="No messages yet"
									description="Start by asking a question about the game rules"
									size="sm"
								/>
							{:else}
								{#each currentSession.messages as message}
									<div class="flex items-start space-x-3">
										<div class="flex-shrink-0">
											<div
												class="flex h-8 w-8 items-center justify-center rounded-full {getRoleColor(
													message.role
												)}"
											>
												{#if message.role === 'user'}
													<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
														<path
															fill-rule="evenodd"
															d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z"
															clip-rule="evenodd"
														></path>
													</svg>
												{:else}
													<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
														<path
															fill-rule="evenodd"
															d="M2 5a2 2 0 012-2h8a2 2 0 012 2v10a2 2 0 002 2H4a2 2 0 01-2-2V5zm3 1h6v4H5V6zm6 6H5v2h6v-2z"
															clip-rule="evenodd"
														></path>
													</svg>
												{/if}
											</div>
										</div>
										<div class="min-w-0 flex-1">
											<div class="flex items-center space-x-2">
												<Badge variant="outline" class="text-xs capitalize">
													{message.role}
												</Badge>
												<span class="text-xs text-gray-500">
													{message.createdAt ? formatTime(message.createdAt) : 'Unknown time'}
												</span>
											</div>
											<div class="prose prose-sm mt-1 max-w-none">
												<p class="whitespace-pre-wrap">{message.content}</p>
											</div>
										</div>
									</div>
								{/each}
							{/if}
						</CardContent>

						<!-- Message Input -->
						<div class="flex-shrink-0 border-t p-4">
							<div class="flex space-x-2">
								<Input
									bind:value={newMessage}
									placeholder="Ask a question about the game rules..."
									disabled={sendingMessage}
									onkeydown={handleKeydown}
									class="flex-1"
								/>
								<Button
									onclick={sendMessage}
									disabled={!newMessage.trim() || sendingMessage}
									class="flex items-center"
								>
									{#if sendingMessage}
										<LoadingSpinner size="sm" class="mr-2" />
										Sending...
									{:else}
										<svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
											<path
												stroke-linecap="round"
												stroke-linejoin="round"
												stroke-width="2"
												d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"
											></path>
										</svg>
										Send
									{/if}
								</Button>
							</div>
						</div>
					</Card>
				{/if}
			</div>
		</div>
	</div>
</div>
