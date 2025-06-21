import { writable } from 'svelte/store';
import type { SearchResult, Game } from '$lib';

// Search history item
export interface SearchHistoryItem {
	id: string;
	query: string;
	gameId: number;
	gameName: string;
	timestamp: Date;
	resultCount: number;
}

// Search state
export interface SearchState {
	isModalOpen: boolean;
	recentSearches: SearchHistoryItem[];
	favoriteResults: SearchResult[];
	currentGame: Game | null;
}

// Initial state
const initialState: SearchState = {
	isModalOpen: false,
	recentSearches: [],
	favoriteResults: [],
	currentGame: null
};

// Create the store
export const searchStore = writable<SearchState>(initialState);

// Search utilities
export const searchUtils = {
	// Open search modal
	openModal() {
		searchStore.update((state) => ({
			...state,
			isModalOpen: true
		}));
	},

	// Close search modal
	closeModal() {
		searchStore.update((state) => ({
			...state,
			isModalOpen: false
		}));
	},

	// Add search to history
	addToHistory(query: string, game: Game, resultCount: number) {
		const historyItem: SearchHistoryItem = {
			id: `${Date.now()}-${Math.random()}`,
			query,
			gameId: game.id,
			gameName: game.name,
			timestamp: new Date(),
			resultCount
		};

		searchStore.update((state) => {
			// Remove duplicate searches for the same game and query
			const filtered = state.recentSearches.filter(
				(item) => !(item.query === query && item.gameId === game.id)
			);

			// Add new search to the beginning and limit to 20 items
			const updated = [historyItem, ...filtered].slice(0, 20);

			return {
				...state,
				recentSearches: updated
			};
		});

		// Persist to localStorage
		if (typeof window !== 'undefined') {
			try {
				const searches = JSON.stringify([historyItem, ...getRecentSearches()].slice(0, 20));
				localStorage.setItem('tabletop-atlas-search-history', searches);
			} catch (e) {
				console.warn('Failed to save search history to localStorage:', e);
			}
		}
	},

	// Clear search history
	clearHistory() {
		searchStore.update((state) => ({
			...state,
			recentSearches: []
		}));

		if (typeof window !== 'undefined') {
			localStorage.removeItem('tabletop-atlas-search-history');
		}
	},

	// Add result to favorites
	addToFavorites(result: SearchResult, game: Game) {
		const favoriteResult = {
			...result,
			metadata: result.metadata || `${game.name} - ${new Date().toLocaleDateString()}`
		};

		searchStore.update((state) => {
			// Check if already in favorites
			const exists = state.favoriteResults.some((fav) => fav.chunkId === result.chunkId);
			if (exists) return state;

			const updated = [favoriteResult, ...state.favoriteResults].slice(0, 50);
			return {
				...state,
				favoriteResults: updated
			};
		});

		// Persist to localStorage
		if (typeof window !== 'undefined') {
			try {
				const favorites = JSON.stringify([favoriteResult, ...getFavoriteResults()].slice(0, 50));
				localStorage.setItem('tabletop-atlas-favorite-results', favorites);
			} catch (e) {
				console.warn('Failed to save favorite results to localStorage:', e);
			}
		}
	},

	// Remove result from favorites
	removeFromFavorites(chunkId: number) {
		searchStore.update((state) => ({
			...state,
			favoriteResults: state.favoriteResults.filter((fav) => fav.chunkId !== chunkId)
		}));

		// Update localStorage
		if (typeof window !== 'undefined') {
			try {
				const favorites = JSON.stringify(
					getFavoriteResults().filter((fav) => fav.chunkId !== chunkId)
				);
				localStorage.setItem('tabletop-atlas-favorite-results', favorites);
			} catch (e) {
				console.warn('Failed to update favorite results in localStorage:', e);
			}
		}
	},

	// Set current game context
	setCurrentGame(game: Game | null) {
		searchStore.update((state) => ({
			...state,
			currentGame: game
		}));
	},

	// Load persisted data
	loadPersistedData() {
		if (typeof window === 'undefined') return;

		try {
			// Load search history
			const historyData = localStorage.getItem('tabletop-atlas-search-history');
			if (historyData) {
				const searches = JSON.parse(historyData).map((item: any) => ({
					...item,
					timestamp: new Date(item.timestamp)
				}));

				searchStore.update((state) => ({
					...state,
					recentSearches: searches
				}));
			}

			// Load favorite results
			const favoritesData = localStorage.getItem('tabletop-atlas-favorite-results');
			if (favoritesData) {
				const favorites = JSON.parse(favoritesData);
				searchStore.update((state) => ({
					...state,
					favoriteResults: favorites
				}));
			}
		} catch (e) {
			console.warn('Failed to load persisted search data:', e);
		}
	}
};

// Helper functions to get current state values
export function getRecentSearches(): SearchHistoryItem[] {
	let searches: SearchHistoryItem[] = [];
	searchStore.subscribe((state) => {
		searches = state.recentSearches;
	})();
	return searches;
}

export function getFavoriteResults(): SearchResult[] {
	let favorites: SearchResult[] = [];
	searchStore.subscribe((state) => {
		favorites = state.favoriteResults;
	})();
	return favorites;
}

export function getCurrentGame(): Game | null {
	let game: Game | null = null;
	searchStore.subscribe((state) => {
		game = state.currentGame;
	})();
	return game;
}

// Keyboard shortcut handler
export function initSearchShortcuts() {
	if (typeof window === 'undefined') return;

	function handleKeydown(event: KeyboardEvent) {
		// Cmd/Ctrl + K to open search
		if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
			event.preventDefault();
			searchUtils.openModal();
		}

		// Escape to close search modal
		if (event.key === 'Escape') {
			searchUtils.closeModal();
		}
	}

	document.addEventListener('keydown', handleKeydown);

	// Return cleanup function
	return () => {
		document.removeEventListener('keydown', handleKeydown);
	};
}

// Search query suggestions based on common board game queries
export const searchSuggestions = [
	'How do I win the game?',
	'What happens during setup?',
	'How does combat work?',
	'What are the turn phases?',
	'How do I score points?',
	'What happens when the game ends?',
	'How many players can play?',
	'What is the goal of the game?',
	'How do special abilities work?',
	'What happens on my turn?',
	'How do I move pieces?',
	'What are the victory conditions?',
	'How long does a game take?',
	'What happens if I run out of cards?',
	'How do I resolve conflicts?',
	'What are the different actions I can take?',
	'How do resources work?',
	'What happens during the end phase?',
	'How do I use special cards?',
	'What are the rules for trading?'
];

// Search tips for users
export const searchTips = [
	{
		title: 'Ask Natural Questions',
		description:
			"Instead of searching for keywords, ask questions like 'How do I win?' or 'What happens during combat?'",
		icon: '💬'
	},
	{
		title: 'Be Specific',
		description:
			"Include specific game elements like 'combat', 'setup', 'scoring', or 'victory conditions' in your queries.",
		icon: '🎯'
	},
	{
		title: 'Use Different Phrasings',
		description:
			"If you don't find what you're looking for, try rephrasing your question using different words.",
		icon: '🔄'
	},
	{
		title: 'Semantic Search',
		description:
			"Our AI understands meaning, so you can find relevant information even if exact words don't match.",
		icon: '🧠'
	}
];
