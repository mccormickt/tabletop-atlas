<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { api } from '$lib';
	import { Button, GameBox } from '$lib/components/ui';
	import { ComponentTray, ComponentTraySection } from '$lib/components/ui';
	import {
		Meeple,
		Dice,
		GameBoxIcon,
		Rulebook,
		SearchGlass,
		ChatBubble,
		Trophy
	} from '$lib/components/icons';
	import { useHeader } from '$lib/stores/header';

	const header = useHeader();
	header.configure({
		showSearch: true,
		currentGame: null
	});

	function navigateToGames() {
		goto(resolve('/games'));
	}

	function navigateToAddGame() {
		goto(resolve('/games/add'));
	}

	function navigateToUpload() {
		goto(resolve('/upload'));
	}

	function navigateToSearch() {
		goto(resolve('/search'));
	}

	function navigateToChat() {
		goto(resolve('/chat'));
	}

	function navigateToChallenges() {
		goto(resolve('/challenges'));
	}

	async function countGames() {
		const result = await api.methods.listGames({});

		if (result.type === 'success') {
			return result.data.total;
		}
		return 0;
	}

	let totalGames = $state(0);
	let loading = $state(true);
	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			countGames().then((count) => {
				totalGames = count;
				loading = false;
			});
		}
	});
</script>

<svelte:head>
	<title>Tabletop Atlas - Board Game Management</title>
	<meta name="description" content="Comprehensive board game rules management system" />
</svelte:head>

<main class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
	<!-- Hero Section - Game Table Theme -->
	<div class="relative mb-12 text-center">
		<!-- Decorative scattered game pieces -->
		<div class="pointer-events-none absolute inset-0 hidden overflow-hidden lg:block">
			<Meeple size={32} color="red" class="absolute top-4 left-16 rotate-12 opacity-20" />
			<Meeple size={24} color="blue" class="absolute top-12 right-24 -rotate-6 opacity-20" />
			<Meeple size={28} color="green" class="absolute bottom-4 left-32 rotate-45 opacity-20" />
			<Dice size={24} value={4} class="absolute top-8 right-40 rotate-12 opacity-20" />
			<Dice size={20} value={2} class="absolute right-16 bottom-8 -rotate-12 opacity-20" />
		</div>

		<div class="relative z-10">
			<div
				class="bg-parchment-dark border-wood-dark mb-4 inline-flex items-center gap-2 rounded-full border px-4 py-2"
			>
				<Dice size={18} value={6} class="text-game-blue" />
				<span class="font-ui text-muted-foreground text-sm">Your tabletop companion</span>
			</div>

			<h1 class="font-display text-foreground mb-6 text-4xl font-bold md:text-5xl lg:text-6xl">
				Manage Your<br />
				<span class="text-game-blue">Board Game</span> Collection
			</h1>

			<p class="text-muted-foreground font-body mx-auto mb-8 max-w-2xl text-lg">
				Organize your board games, upload rule books, create house rules, and get instant answers
				about gameplay through our AI-powered chat interface.
			</p>

			<div class="flex flex-wrap items-center justify-center gap-4">
				<Button variant="game-primary" size="lg" onclick={navigateToAddGame} class="gap-2">
					<GameBoxIcon size={20} />
					Add Your First Game
				</Button>
				<Button variant="game-secondary" size="lg" onclick={navigateToGames} class="gap-2">
					Browse Collection
				</Button>
			</div>
		</div>
	</div>

	<!-- Quick Actions - Game Box Lids -->
	<div class="mb-12 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
		<button onclick={navigateToAddGame} class="group text-left">
			<GameBox
				variant="default"
				class="h-full transition-all group-hover:-translate-y-1 group-hover:shadow-xl"
			>
				<div class="flex flex-col items-center p-4 text-center">
					<div
						class="bg-game-blue/10 group-hover:bg-game-blue/20 mb-4 flex h-16 w-16 items-center justify-center rounded-full transition-colors"
					>
						<GameBoxIcon size={32} class="text-game-blue" />
					</div>
					<h3 class="font-display mb-2 text-lg font-semibold">Add New Game</h3>
					<p class="text-muted-foreground font-body text-sm">
						Add a new board game to your collection
					</p>
				</div>
			</GameBox>
		</button>

		<button onclick={navigateToUpload} class="group text-left">
			<GameBox
				variant="default"
				class="h-full transition-all group-hover:-translate-y-1 group-hover:shadow-xl"
			>
				<div class="flex flex-col items-center p-4 text-center">
					<div
						class="bg-game-orange/10 group-hover:bg-game-orange/20 mb-4 flex h-16 w-16 items-center justify-center rounded-full transition-colors"
					>
						<Rulebook size={32} class="text-game-orange" />
					</div>
					<h3 class="font-display mb-2 text-lg font-semibold">Upload Rules</h3>
					<p class="text-muted-foreground font-body text-sm">
						Upload PDF rule books for easy access
					</p>
				</div>
			</GameBox>
		</button>

		<button onclick={navigateToSearch} class="group text-left">
			<GameBox
				variant="default"
				class="h-full transition-all group-hover:-translate-y-1 group-hover:shadow-xl"
			>
				<div class="flex flex-col items-center p-4 text-center">
					<div
						class="bg-game-green/10 group-hover:bg-game-green/20 mb-4 flex h-16 w-16 items-center justify-center rounded-full transition-colors"
					>
						<SearchGlass size={32} class="text-game-green" />
					</div>
					<h3 class="font-display mb-2 text-lg font-semibold">Search Rules</h3>
					<p class="text-muted-foreground font-body text-sm">
						Find specific rules with AI-powered search
					</p>
				</div>
			</GameBox>
		</button>

		<button onclick={navigateToChat} class="group text-left">
			<GameBox
				variant="default"
				class="h-full transition-all group-hover:-translate-y-1 group-hover:shadow-xl"
			>
				<div class="flex flex-col items-center p-4 text-center">
					<div
						class="bg-game-purple/10 group-hover:bg-game-purple/20 mb-4 flex h-16 w-16 items-center justify-center rounded-full transition-colors"
					>
						<ChatBubble size={32} class="text-game-purple" />
					</div>
					<h3 class="font-display mb-2 text-lg font-semibold">Ask Questions</h3>
					<p class="text-muted-foreground font-body text-sm">
						Get instant answers about game rules
					</p>
				</div>
			</GameBox>
		</button>

		<button onclick={navigateToChallenges} class="group text-left">
			<GameBox
				variant="default"
				class="h-full transition-all group-hover:-translate-y-1 group-hover:shadow-xl"
			>
				<div class="flex flex-col items-center p-4 text-center">
					<div
						class="bg-game-red/10 group-hover:bg-game-red/20 mb-4 flex h-16 w-16 items-center justify-center rounded-full transition-colors"
					>
						<Trophy size={32} class="text-game-red" />
					</div>
					<h3 class="font-display mb-2 text-lg font-semibold">8x8 Challenge</h3>
					<p class="text-muted-foreground font-body text-sm">Track game sessions with friends</p>
				</div>
			</GameBox>
		</button>
	</div>

	<!-- Stats & Quick Start Row -->
	<div class="grid grid-cols-1 gap-6 lg:grid-cols-2">
		<!-- Collection Stats - Component Tray -->
		<ComponentTray title="Your Collection">
			<ComponentTraySection>
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-4">
						<div class="bg-game-blue flex h-12 w-12 items-center justify-center rounded-lg">
							<GameBoxIcon size={24} class="text-white" />
						</div>
						<div>
							{#if loading}
								<p class="font-display text-parchment text-2xl font-bold">...</p>
							{:else}
								<p class="font-display text-parchment text-2xl font-bold">{totalGames}</p>
							{/if}
							<p class="text-parchment/70 text-sm">Games in collection</p>
						</div>
					</div>
					<Button variant="game-secondary" size="sm" onclick={navigateToGames}>View All</Button>
				</div>
			</ComponentTraySection>

			<div class="mt-4 grid grid-cols-3 gap-2">
				<div class="bg-parchment/10 rounded p-2 text-center">
					<Meeple size={20} color="red" class="mx-auto mb-1 opacity-60" />
					<p class="text-parchment/70 text-xs">Ready to play</p>
				</div>
				<div class="bg-parchment/10 rounded p-2 text-center">
					<Rulebook size={20} class="text-parchment mx-auto mb-1 opacity-60" />
					<p class="text-parchment/70 text-xs">With PDF rules</p>
				</div>
				<div class="bg-parchment/10 rounded p-2 text-center">
					<Dice size={20} value={5} class="text-parchment mx-auto mb-1 opacity-60" />
					<p class="text-parchment/70 text-xs">House rules</p>
				</div>
			</div>
		</ComponentTray>

		<!-- Quick Start - Rulebook Style -->
		<GameBox variant="default" title="Getting Started" showCorners={true}>
			<div class="space-y-4">
				<div
					class="bg-parchment-dark/50 border-border flex items-center gap-4 rounded-lg border p-3"
				>
					<div
						class="bg-game-blue flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full"
					>
						<span class="font-display text-sm font-bold text-white">1</span>
					</div>
					<div class="min-w-0 flex-1">
						<p class="font-display font-medium">Add your first game</p>
						<p class="text-muted-foreground text-sm">Enter game details and metadata</p>
					</div>
					<Button variant="ghost" size="sm" onclick={navigateToAddGame}>Start</Button>
				</div>

				<div
					class="bg-parchment-dark/50 border-border flex items-center gap-4 rounded-lg border p-3"
				>
					<div
						class="bg-game-orange flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full"
					>
						<span class="font-display text-sm font-bold text-white">2</span>
					</div>
					<div class="min-w-0 flex-1">
						<p class="font-display font-medium">Upload rule books</p>
						<p class="text-muted-foreground text-sm">We'll index the content for you</p>
					</div>
					<Button variant="ghost" size="sm" onclick={navigateToUpload}>Upload</Button>
				</div>

				<div
					class="bg-parchment-dark/50 border-border flex items-center gap-4 rounded-lg border p-3"
				>
					<div
						class="bg-game-green flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full"
					>
						<span class="font-display text-sm font-bold text-white">3</span>
					</div>
					<div class="min-w-0 flex-1">
						<p class="font-display font-medium">Search or ask questions</p>
						<p class="text-muted-foreground text-sm">AI-powered rule lookups</p>
					</div>
					<Button variant="ghost" size="sm" onclick={navigateToSearch}>Search</Button>
				</div>
			</div>
		</GameBox>
	</div>
</main>

<!-- Footer -->
<footer class="bg-wood-light/30 border-wood-dark mt-16 border-t-2">
	<div class="mx-auto max-w-7xl px-4 py-8 sm:px-6 lg:px-8">
		<div class="flex flex-col items-center justify-between gap-4 sm:flex-row">
			<div class="text-muted-foreground font-ui flex items-center gap-2 text-sm">
				<Meeple size={16} color="blue" />
				<span>© 2024 Tabletop Atlas. Made with ♥ for board game enthusiasts.</span>
			</div>
			<div class="font-ui flex items-center gap-6 text-sm">
				<a
					href="https://github.com/mccormickt/tabletop-atlas"
					class="text-muted-foreground hover:text-foreground flex items-center gap-1 transition-colors"
				>
					<svg class="h-4 w-4" fill="currentColor" viewBox="0 0 24 24">
						<path
							d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"
						/>
					</svg>
					GitHub
				</a>
			</div>
		</div>
	</div>
</footer>
