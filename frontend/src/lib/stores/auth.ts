import { getContext, setContext } from 'svelte';
import { writable, derived, type Readable } from 'svelte/store';

export interface UserInfo {
	id: number;
	email: string;
	display_name: string | null;
	picture_url: string | null;
	role: string;
}

export interface AuthState {
	user: UserInfo | null;
	isLoading: boolean;
	error: string | null;
}

const AUTH_CONTEXT_KEY = Symbol('auth');

export function createAuthStore(initialState: Partial<AuthState> = {}) {
	const store = writable<AuthState>({
		user: null,
		isLoading: true,
		error: null,
		...initialState
	});

	async function checkAuth() {
		store.update((s) => ({ ...s, isLoading: true, error: null }));

		try {
			const response = await fetch('/api/auth/me', {
				credentials: 'include'
			});

			if (response.ok) {
				const data = await response.json();
				store.update((s) => ({ ...s, user: data.user, isLoading: false }));
				return data.user;
			} else if (response.status === 401) {
				// Try to refresh the token
				const refreshed = await refreshToken();
				if (!refreshed) {
					store.update((s) => ({ ...s, user: null, isLoading: false }));
				}
				return null;
			} else {
				store.update((s) => ({ ...s, user: null, isLoading: false }));
				return null;
			}
		} catch (error) {
			console.error('Auth check failed:', error);
			store.update((s) => ({
				...s,
				user: null,
				isLoading: false,
				error: 'Failed to check authentication'
			}));
			return null;
		}
	}

	async function refreshToken(): Promise<boolean> {
		try {
			const response = await fetch('/api/auth/refresh', {
				method: 'POST',
				credentials: 'include'
			});

			if (response.ok) {
				// Re-check auth after successful refresh
				const meResponse = await fetch('/api/auth/me', {
					credentials: 'include'
				});
				if (meResponse.ok) {
					const data = await meResponse.json();
					store.update((s) => ({ ...s, user: data.user, isLoading: false }));
					return true;
				}
			}
			return false;
		} catch (error) {
			console.error('Token refresh failed:', error);
			return false;
		}
	}

	function login() {
		// Redirect to login endpoint
		window.location.href = '/api/auth/login';
	}

	async function logout() {
		try {
			await fetch('/api/auth/logout', {
				method: 'POST',
				credentials: 'include'
			});
		} catch (error) {
			console.error('Logout failed:', error);
		}

		store.update((s) => ({ ...s, user: null, isLoading: false, error: null }));
		window.location.href = '/';
	}

	// Derived stores for convenience
	const isAuthenticated: Readable<boolean> = derived(store, ($store) => $store.user !== null);
	const isAdmin: Readable<boolean> = derived(store, ($store) => $store.user?.role === 'admin');

	return {
		subscribe: store.subscribe,
		set: store.set,
		update: store.update,
		checkAuth,
		login,
		logout,
		refreshToken,
		isAuthenticated,
		isAdmin
	};
}

export type AuthStore = ReturnType<typeof createAuthStore>;

export function setAuthContext(store: AuthStore) {
	setContext<AuthStore>(AUTH_CONTEXT_KEY, store);
	return store;
}

export function getAuthContext(): AuthStore {
	const store = getContext<AuthStore>(AUTH_CONTEXT_KEY);
	if (!store) {
		throw new Error(
			'Auth context not found. Make sure to call setAuthContext in a parent component.'
		);
	}
	return store;
}

export function useAuth() {
	try {
		return getAuthContext();
	} catch {
		console.warn('Auth context not available - using mock store');
		return createAuthStore();
	}
}
