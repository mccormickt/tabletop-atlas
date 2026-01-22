<script lang="ts">
	import { cn } from '$lib/utils';
	import { Button } from '$lib/components/ui';

	let {
		currentPage = 1,
		totalPages = 1,
		onPageChange,
		class: className
	}: {
		currentPage?: number;
		totalPages?: number;
		onPageChange?: (page: number) => void;
		class?: string;
	} = $props();

	// Calculate which page numbers to show
	function getVisiblePages(): (number | 'ellipsis')[] {
		const pages: (number | 'ellipsis')[] = [];
		const maxVisible = 7;

		if (totalPages <= maxVisible) {
			// Show all pages
			for (let i = 1; i <= totalPages; i++) {
				pages.push(i);
			}
		} else {
			// Always show first page
			pages.push(1);

			if (currentPage <= 3) {
				// Near start: 1 2 3 4 5 ... last
				for (let i = 2; i <= 5; i++) {
					pages.push(i);
				}
				pages.push('ellipsis');
				pages.push(totalPages);
			} else if (currentPage >= totalPages - 2) {
				// Near end: 1 ... last-4 last-3 last-2 last-1 last
				pages.push('ellipsis');
				for (let i = totalPages - 4; i <= totalPages; i++) {
					pages.push(i);
				}
			} else {
				// Middle: 1 ... current-1 current current+1 ... last
				pages.push('ellipsis');
				for (let i = currentPage - 1; i <= currentPage + 1; i++) {
					pages.push(i);
				}
				pages.push('ellipsis');
				pages.push(totalPages);
			}
		}

		return pages;
	}

	let visiblePages = $derived(getVisiblePages());

	function goToPage(page: number) {
		if (page >= 1 && page <= totalPages && page !== currentPage) {
			onPageChange?.(page);
		}
	}
</script>

<nav class={cn('flex items-center gap-2', className)} aria-label="Pagination">
	<Button
		variant="game-secondary"
		size="sm"
		onclick={() => goToPage(currentPage - 1)}
		disabled={currentPage <= 1}
	>
		Previous
	</Button>

	<!-- Page numbers (hidden on mobile) -->
	<div class="hidden items-center gap-1 sm:flex">
		{#each visiblePages as page, i (i)}
			{#if page === 'ellipsis'}
				<span class="text-muted-foreground font-display px-2">...</span>
			{:else}
				<button
					onclick={() => goToPage(page)}
					class={cn(
						'font-display h-8 min-w-8 rounded-md border-2 px-2 text-sm font-semibold transition-all',
						page === currentPage
							? 'bg-game-blue border-game-blue text-white shadow-md'
							: 'border-wood-dark bg-parchment text-foreground hover:bg-parchment-dark hover:shadow-sm'
					)}
					aria-current={page === currentPage ? 'page' : undefined}
				>
					{page}
				</button>
			{/if}
		{/each}
	</div>

	<!-- Mobile page indicator -->
	<span class="text-muted-foreground font-display text-sm sm:hidden">
		{currentPage} / {totalPages}
	</span>

	<Button
		variant="game-secondary"
		size="sm"
		onclick={() => goToPage(currentPage + 1)}
		disabled={currentPage >= totalPages}
	>
		Next
	</Button>
</nav>
