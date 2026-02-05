<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { api, type GameSummary } from '$lib';
	import { SearchGlass } from './icons';

	// State
	let query = $state('');
	let searchResults = $state<GameSummary[]>([]);
	let isOpen = $state(false);
	let isLoading = $state(false);
	let inputElement = $state<HTMLInputElement | null>(null);
	let selectedIndex = $state(-1);
	let searchTimeout: ReturnType<typeof setTimeout> | null = null;

	// Keyboard shortcut (Cmd/Ctrl + K)
	$effect(() => {
		function handleKeydown(event: KeyboardEvent) {
			if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
				event.preventDefault();
				isOpen = true;
				// Focus input after it renders
				setTimeout(() => inputElement?.focus(), 0);
			}
			// Escape to close
			if (event.key === 'Escape' && isOpen) {
				isOpen = false;
				query = '';
				inputElement?.blur();
			}
		}
		document.addEventListener('keydown', handleKeydown);
		return () => document.removeEventListener('keydown', handleKeydown);
	});

	// Debounced search when query changes
	$effect(() => {
		if (searchTimeout) {
			clearTimeout(searchTimeout);
		}

		if (!query.trim()) {
			searchResults = [];
			isLoading = false;
			return;
		}

		isLoading = true;
		searchTimeout = setTimeout(() => {
			searchGames(query.trim());
		}, 200);

		return () => {
			if (searchTimeout) {
				clearTimeout(searchTimeout);
			}
		};
	});

	// Reset selection when results change
	$effect(() => {
		void searchResults;
		selectedIndex = -1;
	});

	async function searchGames(searchQuery: string) {
		try {
			const result = await api.methods.listGames({
				query: { limit: 10, search: searchQuery }
			});
			if (result.type === 'success') {
				searchResults = result.data.items;
			}
		} catch {
			// Silently fail
			searchResults = [];
		} finally {
			isLoading = false;
		}
	}

	function handleBlur(event: FocusEvent) {
		// Delay to allow click events on dropdown items
		const relatedTarget = event.relatedTarget as HTMLElement;
		if (relatedTarget?.closest('.search-dropdown')) {
			return;
		}
		setTimeout(() => {
			isOpen = false;
			query = '';
		}, 150);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (!isOpen || searchResults.length === 0) return;

		if (event.key === 'ArrowDown' || (event.key === 'Tab' && !event.shiftKey)) {
			event.preventDefault();
			selectedIndex = Math.min(selectedIndex + 1, searchResults.length - 1);
		} else if (event.key === 'ArrowUp' || (event.key === 'Tab' && event.shiftKey)) {
			event.preventDefault();
			selectedIndex = Math.max(selectedIndex - 1, -1);
		} else if (event.key === 'Enter' && selectedIndex >= 0) {
			event.preventDefault();
			selectGame(searchResults[selectedIndex]);
		}
	}

	function selectGame(game: GameSummary) {
		goto(resolve(`/games/${game.id}`));
		query = '';
		isOpen = false;
		inputElement?.blur();
	}

	function getShortcutText(): string {
		if (typeof navigator !== 'undefined') {
			const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
			return isMac ? '⌘K' : 'Ctrl+K';
		}
		return 'Ctrl+K';
	}
</script>

<div class="relative">
	{#if isOpen}
		<!-- Input mode -->
		<div class="relative">
			<SearchGlass
				size={16}
				class="text-muted-foreground pointer-events-none absolute top-1/2 left-3 -translate-y-1/2"
			/>
			<input
				bind:this={inputElement}
				bind:value={query}
				onblur={handleBlur}
				onkeydown={handleKeydown}
				type="text"
				placeholder="Search games..."
				class="bg-card border-game-blue ring-game-blue/20 h-9 w-64 rounded-full border-2 py-1 pr-3 pl-9 text-sm shadow-lg ring-2 outline-none"
			/>
		</div>
	{:else}
		<!-- Button mode -->
		<button
			onclick={() => {
				isOpen = true;
				setTimeout(() => inputElement?.focus(), 0);
			}}
			class="bg-parchment hover:bg-parchment-dark border-border flex h-9 items-center gap-2 rounded-full border px-3 text-sm transition-colors"
		>
			<SearchGlass size={16} class="text-muted-foreground" />
			<span class="text-foreground">Search Games</span>
			<span class="bg-foreground/10 text-muted-foreground rounded px-1.5 py-0.5 text-xs"
				>{getShortcutText()}</span
			>
		</button>
	{/if}

	<!-- Dropdown -->
	{#if isOpen && query.trim()}
		<div
			class="search-dropdown border-border bg-card absolute top-10 right-0 z-50 mt-2 w-72 overflow-hidden rounded-lg border shadow-lg"
		>
			{#if isLoading}
				<div class="text-muted-foreground p-4 text-center text-sm">Searching...</div>
			{:else if searchResults.length > 0}
				<div class="max-h-80 overflow-y-auto py-1">
					{#each searchResults as game, index (game.id)}
						<button
							onmousedown={() => selectGame(game)}
							class="flex w-full items-center gap-3 px-3 py-2 text-left transition-colors
								{index === selectedIndex ? 'bg-primary/10' : 'hover:bg-muted'}"
						>
							<div
								class="bg-game-blue flex h-8 w-8 flex-shrink-0 items-center justify-center rounded text-xs font-bold text-white"
							>
								{game.name[0].toUpperCase()}
							</div>
							<div class="min-w-0 flex-1">
								<p class="text-foreground truncate text-sm font-medium">{game.name}</p>
								{#if game.publisher}
									<p class="text-muted-foreground truncate text-xs">{game.publisher}</p>
								{/if}
							</div>
							{#if game.hasRulesPdf}
								<span class="text-game-green text-xs">Has rules</span>
							{/if}
						</button>
					{/each}
				</div>
			{:else if query.trim()}
				<div class="p-4 text-center">
					<p class="text-muted-foreground text-sm">No games found</p>
					<a
						href={resolve('/games/add')}
						class="text-game-blue mt-1 inline-block text-xs hover:underline"
					>
						Add a new game
					</a>
				</div>
			{/if}
		</div>
	{/if}
</div>
