<script lang="ts">
	import { page } from '$app/state';
	import { Home, GameBoxIcon, Upload, SearchGlass, ChatBubble } from './icons';

	let currentPath = $derived(page.url.pathname);

	function isActivePath(path: string): boolean {
		return currentPath === path || currentPath.startsWith(path + '/');
	}

	const navItems = [
		{ path: '/', label: 'Home', icon: Home },
		{ path: '/games', label: 'Games', icon: GameBoxIcon },
		{ path: '/upload', label: 'Upload', icon: Upload },
		{ path: '/search', label: 'Search', icon: SearchGlass },
		{ path: '/chat', label: 'Chat', icon: ChatBubble }
	] as const;
</script>

<nav class="mobile-nav md:hidden fixed bottom-0 left-0 right-0 z-50 border-t-2 border-wood-dark bg-parchment safe-area-inset-bottom">
	<div class="flex items-stretch justify-around">
		{#each navItems as item}
			<a
				href={item.path}
				class="mobile-nav-item flex flex-1 flex-col items-center justify-center py-2 px-1 transition-all
					{isActivePath(item.path)
						? 'text-game-blue bg-parchment-dark'
						: 'text-muted-foreground hover:text-foreground hover:bg-parchment-dark/50'}"
				aria-current={isActivePath(item.path) ? 'page' : undefined}
			>
				<div class="relative">
					<svelte:component this={item.icon} size={24} class={isActivePath(item.path) ? 'text-game-blue' : ''} />
					{#if isActivePath(item.path)}
						<div class="absolute -bottom-1 left-1/2 -translate-x-1/2 w-1 h-1 rounded-full bg-game-blue"></div>
					{/if}
				</div>
				<span class="text-xs mt-1 font-ui font-medium">{item.label}</span>
			</a>
		{/each}
	</div>
</nav>

<style>
	.safe-area-inset-bottom {
		padding-bottom: env(safe-area-inset-bottom, 0);
	}

	.mobile-nav-item {
		min-height: 56px;
		min-width: 44px;
	}
</style>
