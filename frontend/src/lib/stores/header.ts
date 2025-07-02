import { getContext, setContext } from 'svelte';
import { writable, type Writable } from 'svelte/store';
import type { Game } from '$lib';

export interface HeaderConfig {
	currentGame?: Game | null;
	showSearch?: boolean;
	title?: string;
}

const HEADER_CONTEXT_KEY = Symbol('header');

export function createHeaderStore(initialConfig: HeaderConfig = {}) {
	const store = writable<HeaderConfig>({
		currentGame: null,
		showSearch: true,
		title: undefined,
		...initialConfig
	});

	return {
		subscribe: store.subscribe,
		set: store.set,
		update: store.update,
		setCurrentGame: (game: Game | null) => {
			store.update((config) => ({ ...config, currentGame: game }));
		},
		setShowSearch: (show: boolean) => {
			store.update((config) => ({ ...config, showSearch: show }));
		},
		setTitle: (title: string | undefined) => {
			store.update((config) => ({ ...config, title }));
		},
		configure: (config: Partial<HeaderConfig>) => {
			store.update((current) => ({ ...current, ...config }));
		}
	};
}

export type HeaderStore = ReturnType<typeof createHeaderStore>;

export function setHeaderContext(store: HeaderStore) {
	setContext<HeaderStore>(HEADER_CONTEXT_KEY, store);
	return store;
}

export function getHeaderContext(): HeaderStore {
	const store = getContext<HeaderStore>(HEADER_CONTEXT_KEY);
	if (!store) {
		throw new Error(
			'Header context not found. Make sure to call setHeaderContext in a parent component.'
		);
	}
	return store;
}

// Utility function for pages to easily configure the header
export function useHeader() {
	try {
		return getHeaderContext();
	} catch {
		// If no context is available, return a mock store for development
		console.warn('Header context not available - using mock store');
		return createHeaderStore();
	}
}
