<script lang="ts">
	import { goto } from '$app/navigation';
	import { api } from '$lib';
	import { Button } from '$lib/components/ui';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui';
	import { onMount } from 'svelte';

	// Navigation functions
	function navigateToGames() {
		goto('/games');
	}

	function navigateToAddGame() {
		goto('/games/add');
	}

	function navigateToUpload() {
		// TODO: Implement upload route
		console.log('Navigate to upload');
	}

	function navigateToChat() {
		// TODO: Implement chat route
		console.log('Navigate to chat');
	}

	async function countGames() {
		const result = await api.methods.listGames({});

		if (result.type === 'success') {
			return result.data.total;
		} else if (result.type === 'error') {
			return result.data.message || 'Failed to load games';
		} else if (result.type === 'client_error') {
			return result.error.message || 'Failed to load games';
		}
		return 0;
	}

	let totalGames = $state();
	onMount(async () => {
		totalGames = await countGames();
	});
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
					<a
						href="/"
						class="text-foreground hover:text-primary flex items-center text-2xl font-bold transition-colors"
					>
						🎲 Tabletop Atlas
					</a>
				</div>
				<nav class="flex space-x-8">
					<a href="/games" class="text-muted-foreground hover:text-foreground transition-colors">
						Games
					</a>
					<button
						onclick={navigateToUpload}
						class="text-muted-foreground hover:text-foreground transition-colors"
					>
						Upload Rules
					</button>
					<button
						onclick={navigateToChat}
						class="text-muted-foreground hover:text-foreground transition-colors"
					>
						Chat
					</button>
				</nav>
			</div>
		</div>
	</header>

	<!-- Main Content -->
	<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
		<!-- Hero Section -->
		<div class="mb-12 text-center">
			<h1 class="text-foreground mb-4 text-4xl font-bold">Manage Your Board Game Collection</h1>
			<p class="text-muted-foreground mx-auto max-w-3xl text-xl">
				Organize your board games, upload rule books, create house rules, and get instant answers
				about gameplay through our AI-powered chat interface.
			</p>
		</div>

		<!-- Quick Actions -->
		<div class="mb-12 grid grid-cols-1 gap-6 md:grid-cols-3">
			<Card class="transition-shadow hover:shadow-lg">
				<CardHeader>
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
					<CardTitle>Add New Game</CardTitle>
					<CardDescription>
						Add a new board game to your collection with detailed information and metadata.
					</CardDescription>
				</CardHeader>
				<CardContent>
					<Button onclick={navigateToAddGame} class="w-full">Add Game</Button>
				</CardContent>
			</Card>

			<Card class="transition-shadow hover:shadow-lg">
				<CardHeader>
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
					<CardTitle>Upload Rules</CardTitle>
					<CardDescription>
						Upload PDF rule books and we'll extract and index the content for easy searching.
					</CardDescription>
				</CardHeader>
				<CardContent>
					<Button variant="outline" onclick={navigateToUpload} class="w-full">Upload PDF</Button>
				</CardContent>
			</Card>

			<Card class="transition-shadow hover:shadow-lg">
				<CardHeader>
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
					<CardTitle>Ask Questions</CardTitle>
					<CardDescription>
						Get instant answers about game rules using our AI-powered chat interface.
					</CardDescription>
				</CardHeader>
				<CardContent>
					<Button variant="outline" onclick={navigateToChat} class="w-full">Start Chat</Button>
				</CardContent>
			</Card>
		</div>

		<!-- Recent Activity / Stats -->
		<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
			<Card>
				<CardHeader>
					<CardTitle>Your Collection</CardTitle>
					<CardDescription>Quick overview of your board game library</CardDescription>
				</CardHeader>
				<CardContent>
					<div class="flex items-center justify-between">
						<div>
							<p class="text-foreground text-2xl font-bold">{totalGames}</p>
							<p class="text-muted-foreground text-sm">Games in collection</p>
						</div>
						<Button variant="outline" onclick={navigateToGames}>View All Games</Button>
					</div>
				</CardContent>
			</Card>

			<Card>
				<CardHeader>
					<CardTitle>Quick Start</CardTitle>
					<CardDescription>Get started with Tabletop Atlas</CardDescription>
				</CardHeader>
				<CardContent class="space-y-3">
					<div class="flex items-center justify-between">
						<span class="text-muted-foreground text-sm">1. Add your first game</span>
						<Button size="sm" variant="outline" onclick={navigateToAddGame}>Add Game</Button>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-muted-foreground text-sm">2. Upload rule books</span>
						<Button size="sm" variant="outline" onclick={navigateToUpload}>Upload</Button>
					</div>
					<div class="flex items-center justify-between">
						<span class="text-muted-foreground text-sm">3. Start asking questions</span>
						<Button size="sm" variant="outline" onclick={navigateToChat}>Chat</Button>
					</div>
				</CardContent>
			</Card>
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
					<button class="hover:text-foreground transition-colors">About</button>
					<button class="hover:text-foreground transition-colors">Help</button>
					<button class="hover:text-foreground transition-colors">GitHub</button>
				</div>
			</div>
		</div>
	</footer>
</div>
