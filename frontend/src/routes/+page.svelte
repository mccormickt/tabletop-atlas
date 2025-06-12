<script lang="ts">
	import { onMount } from 'svelte';

	let games: any[] = [];
	let loading = true;
	let error = '';

	onMount(async () => {
		try {
			const response = await fetch('http://127.0.0.1:8080/api/games');
			if (response.ok) {
				const data = await response.json();
				games = data.items || [];
			} else {
				error = 'Failed to load games';
			}
		} catch (e) {
			error = 'Unable to connect to backend';
		} finally {
			loading = false;
		}
	});
</script>

<svelte:head>
	<title>Tabletop Atlas</title>
	<meta name="description" content="Comprehensive board game rules management" />
</svelte:head>

<div class="min-h-screen bg-gray-50">
	<!-- Header -->
	<header class="border-b bg-white shadow-sm">
		<div class="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
			<div class="flex items-center justify-between py-6">
				<div class="flex items-center">
					<h1 class="text-3xl font-bold text-gray-900">🎲 Tabletop Atlas</h1>
				</div>
				<nav class="flex space-x-8">
					<a href="/" class="text-gray-600 hover:text-gray-900">Games</a>
					<a href="/upload" class="text-gray-600 hover:text-gray-900">Upload Rules</a>
					<a href="/chat" class="text-gray-600 hover:text-gray-900">Chat</a>
				</nav>
			</div>
		</div>
	</header>

	<!-- Main Content -->
	<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
		<!-- Hero Section -->
		<div class="mb-12 text-center">
			<h2 class="mb-4 text-4xl font-bold text-gray-900">Manage Your Board Game Rules</h2>
			<p class="mx-auto max-w-3xl text-xl text-gray-600">
				Upload PDF rule books, create house rules, and ask questions about gameplay through our
				AI-powered chat interface.
			</p>
		</div>

		<!-- Quick Actions -->
		<div class="mb-12 grid grid-cols-1 gap-6 md:grid-cols-3">
			<div class="rounded-lg bg-white p-6 shadow-md transition-shadow hover:shadow-lg">
				<div class="mb-4 text-blue-600">
					<svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C20.832 18.477 19.246 18 17.5 18c-1.746 0-3.332.477-4.5 1.253"
						></path>
					</svg>
				</div>
				<h3 class="mb-2 text-lg font-semibold text-gray-900">Upload Rules</h3>
				<p class="mb-4 text-gray-600">
					Upload PDF rule books and we'll extract and index the content for easy searching.
				</p>
				<button
					class="w-full rounded bg-blue-600 px-4 py-2 text-white transition-colors hover:bg-blue-700"
				>
					Upload PDF
				</button>
			</div>

			<div class="rounded-lg bg-white p-6 shadow-md transition-shadow hover:shadow-lg">
				<div class="mb-4 text-green-600">
					<svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
						></path>
					</svg>
				</div>
				<h3 class="mb-2 text-lg font-semibold text-gray-900">Add House Rules</h3>
				<p class="mb-4 text-gray-600">
					Create custom house rules and variations that complement the official rules.
				</p>
				<button
					class="w-full rounded bg-green-600 px-4 py-2 text-white transition-colors hover:bg-green-700"
				>
					Add House Rule
				</button>
			</div>

			<div class="rounded-lg bg-white p-6 shadow-md transition-shadow hover:shadow-lg">
				<div class="mb-4 text-purple-600">
					<svg class="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
						></path>
					</svg>
				</div>
				<h3 class="mb-2 text-lg font-semibold text-gray-900">Ask Questions</h3>
				<p class="mb-4 text-gray-600">
					Get instant answers about game rules using our AI-powered chat interface.
				</p>
				<button
					class="w-full rounded bg-purple-600 px-4 py-2 text-white transition-colors hover:bg-purple-700"
				>
					Start Chat
				</button>
			</div>
		</div>

		<!-- Games Library -->
		<section>
			<div class="mb-6 flex items-center justify-between">
				<h3 class="text-2xl font-bold text-gray-900">Your Game Library</h3>
				<button
					class="rounded bg-blue-600 px-4 py-2 text-white transition-colors hover:bg-blue-700"
				>
					Add Game
				</button>
			</div>

			{#if loading}
				<div class="rounded-lg bg-white p-8 text-center shadow-md">
					<div
						class="mx-auto mb-4 h-12 w-12 animate-spin rounded-full border-b-2 border-blue-600"
					></div>
					<p class="text-gray-600">Loading your games...</p>
				</div>
			{:else if error}
				<div class="rounded-lg bg-white p-8 text-center shadow-md">
					<div class="mb-4 text-red-600">
						<svg class="mx-auto h-12 w-12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
							></path>
						</svg>
					</div>
					<h4 class="mb-2 text-lg font-semibold text-gray-900">Unable to Load Games</h4>
					<p class="mb-4 text-gray-600">{error}</p>
					<button
						class="rounded bg-blue-600 px-4 py-2 text-white transition-colors hover:bg-blue-700"
						on:click={() => location.reload()}
					>
						Try Again
					</button>
				</div>
			{:else if games.length === 0}
				<div class="rounded-lg bg-white p-8 text-center shadow-md">
					<div class="mb-4 text-gray-400">
						<svg class="mx-auto h-16 w-16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
							></path>
						</svg>
					</div>
					<h4 class="mb-2 text-lg font-semibold text-gray-900">No Games Yet</h4>
					<p class="mb-4 text-gray-600">
						Start building your board game library by adding your first game.
					</p>
					<button
						class="rounded bg-blue-600 px-4 py-2 text-white transition-colors hover:bg-blue-700"
					>
						Add Your First Game
					</button>
				</div>
			{:else}
				<div class="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
					{#each games as game (game.id)}
						<div class="rounded-lg bg-white shadow-md transition-shadow hover:shadow-lg">
							<div class="p-6">
								<h4 class="mb-2 text-lg font-semibold text-gray-900">{game.name}</h4>
								<div class="mb-4 space-y-1 text-sm text-gray-600">
									{#if game.publisher}
										<p><span class="font-medium">Publisher:</span> {game.publisher}</p>
									{/if}
									{#if game.year_published}
										<p><span class="font-medium">Year:</span> {game.year_published}</p>
									{/if}
									{#if game.min_players && game.max_players}
										<p>
											<span class="font-medium">Players:</span>
											{game.min_players}-{game.max_players}
										</p>
									{/if}
									{#if game.complexity_rating}
										<p><span class="font-medium">Complexity:</span> {game.complexity_rating}/5</p>
									{/if}
								</div>
								<div class="flex items-center justify-between">
									<div class="flex space-x-2">
										{#if game.has_rules_pdf}
											<span
												class="inline-flex items-center rounded-full bg-green-100 px-2.5 py-0.5 text-xs font-medium text-green-800"
											>
												PDF Rules
											</span>
										{/if}
										{#if game.house_rules_count > 0}
											<span
												class="inline-flex items-center rounded-full bg-blue-100 px-2.5 py-0.5 text-xs font-medium text-blue-800"
											>
												{game.house_rules_count} House Rules
											</span>
										{/if}
									</div>
									<button class="text-sm font-medium text-blue-600 hover:text-blue-800">
										View Details
									</button>
								</div>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</section>
	</main>
</div>
