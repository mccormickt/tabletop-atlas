<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import Header from '$lib/components/Header.svelte';
	import MobileNav from '$lib/components/MobileNav.svelte';
	import { createHeaderStore, setHeaderContext } from '$lib/stores/header';
	import { createAuthStore, setAuthContext } from '$lib/stores/auth';
	import { createAuthState } from '$lib/stores/auth.svelte';

	let { children } = $props();

	// Public routes that don't require authentication
	const PUBLIC_ROUTES = ['/auth/login', '/auth/callback'];

	function isPublicRoute(pathname: string): boolean {
		return PUBLIC_ROUTES.some((route) => pathname.startsWith(route));
	}

	// Create and provide header context
	const headerStore = createHeaderStore();
	setHeaderContext(headerStore);

	// Create and provide auth context
	const authStore = createAuthStore();
	setAuthContext(authStore);

	// Subscribe to auth state
	const auth = createAuthState();

	async function initMocking() {
		if (
			typeof window !== 'undefined' &&
			(window as unknown as Record<string, unknown>).__MSW_ENABLED__
		) {
			const { worker } = await import('../mocks/browser');
			await worker.start({ onUnhandledRequest: 'bypass', quiet: true });
		}
	}

	// Check auth on mount
	let initialized = $state(false);

	$effect(() => {
		if (!initialized) {
			initialized = true;
			initMocking().then(() => authStore.checkAuth());
		}
	});

	// Redirect to login if not authenticated and not on a public route
	$effect(() => {
		const pathname = page.url.pathname;
		if (!auth.isLoading && !auth.user && !isPublicRoute(pathname)) {
			goto(resolve('/auth/login'));
		}
	});
</script>

{#if auth.isLoading && !isPublicRoute(page.url.pathname)}
	<!-- Show loading state while checking auth for protected routes -->
	<div class="bg-background flex min-h-screen items-center justify-center">
		<div class="text-center">
			<div
				class="border-game-blue mx-auto h-12 w-12 animate-spin rounded-full border-4 border-t-transparent"
			></div>
			<p class="text-muted-foreground mt-4">Loading...</p>
		</div>
	</div>
{:else}
	<div class="bg-background flex min-h-screen flex-col">
		<!-- Global Header -->
		<Header user={auth.user} isAuthLoading={auth.isLoading} />

		<!-- Page Content with bottom padding for mobile nav -->
		<main class="flex-1 pb-20 md:pb-0">
			{@render children()}
		</main>

		<!-- Mobile Bottom Navigation -->
		<MobileNav />
	</div>
{/if}
