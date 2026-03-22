import { useAuth, type AuthState } from './auth';

export function createAuthState() {
	const auth = useAuth();
	let state = $state<AuthState>({ user: null, isLoading: true, error: null });

	$effect(() => {
		const unsubscribe = auth.subscribe((s) => {
			state = s;
		});
		return unsubscribe;
	});

	return {
		get user() {
			return state.user;
		},
		get isLoading() {
			return state.isLoading;
		},
		get error() {
			return state.error;
		},
		get isAdmin() {
			return state.user?.role === 'admin';
		},
		get isAuthenticated() {
			return state.user !== null;
		}
	};
}
