<script lang="ts">
	import { resolve } from '$app/paths';
	import { useAuth, type UserInfo } from '$lib/stores/auth';
	import { Button } from '$lib/components/ui/button';

	let {
		user,
		isLoading = false
	}: {
		user: UserInfo | null;
		isLoading?: boolean;
	} = $props();

	const auth = useAuth();

	let menuOpen = $state(false);

	function handleLogin() {
		auth.login();
	}

	async function handleLogout() {
		menuOpen = false;
		await auth.logout();
	}

	function toggleMenu() {
		menuOpen = !menuOpen;
	}

	function closeMenu() {
		menuOpen = false;
	}

	function getInitials(name: string | null, email: string): string {
		if (name) {
			return name
				.split(' ')
				.map((n) => n[0])
				.join('')
				.toUpperCase()
				.slice(0, 2);
		}
		return email[0].toUpperCase();
	}
</script>

<svelte:window onclick={closeMenu} />

{#if isLoading}
	<div class="bg-muted h-9 w-9 animate-pulse rounded-full"></div>
{:else if user}
	<div class="relative">
		<button
			onclick={(e) => {
				e.stopPropagation();
				toggleMenu();
			}}
			class="relative h-9 w-9 rounded-full focus:ring-2 focus:ring-offset-2 focus:outline-none"
		>
			{#if user.picture_url}
				<img
					src={user.picture_url}
					alt={user.display_name || user.email}
					class="h-9 w-9 rounded-full object-cover"
				/>
			{:else}
				<div
					class="bg-game-blue flex h-9 w-9 items-center justify-center rounded-full text-sm font-medium text-white"
				>
					{getInitials(user.display_name, user.email)}
				</div>
			{/if}
		</button>

		{#if menuOpen}
			<div
				class="bg-card border-border absolute right-0 z-50 mt-2 w-56 origin-top-right rounded-md border shadow-lg"
				onclick={(e) => e.stopPropagation()}
			>
				<div class="border-border border-b px-4 py-3">
					{#if user.display_name}
						<p class="text-foreground text-sm font-medium">{user.display_name}</p>
					{/if}
					<p class="text-muted-foreground truncate text-xs">{user.email}</p>
				</div>
				<div class="py-1">
					<a
						href={resolve('/collection')}
						class="text-foreground hover:bg-muted block px-4 py-2 text-sm"
						onclick={closeMenu}
					>
						My Collection
					</a>
					<a
						href={resolve('/games/custom')}
						class="text-foreground hover:bg-muted block px-4 py-2 text-sm"
						onclick={closeMenu}
					>
						My Custom Games
					</a>
					{#if user.role === 'admin'}
						<div class="border-border my-1 border-t"></div>
						<a
							href={resolve('/admin')}
							class="text-foreground hover:bg-muted block px-4 py-2 text-sm"
							onclick={closeMenu}
						>
							Admin Dashboard
						</a>
					{/if}
				</div>
				<div class="border-border border-t py-1">
					<button
						onclick={handleLogout}
						class="text-destructive hover:bg-muted block w-full px-4 py-2 text-left text-sm"
					>
						Sign Out
					</button>
				</div>
			</div>
		{/if}
	</div>
{:else}
	<Button variant="outline" size="sm" onclick={handleLogin}>Sign in with Google</Button>
{/if}
