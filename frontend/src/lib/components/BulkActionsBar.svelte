<script lang="ts">
	import { Button } from '$lib/components/ui';

	type TabMode = 'library' | 'collection' | 'custom';

	let {
		selectedCount = 0,
		mode = 'library' as TabMode,
		onAddToCollection,
		onRemoveFromCollection,
		onDelete,
		onClearSelection
	}: {
		selectedCount?: number;
		mode?: TabMode;
		onAddToCollection?: () => Promise<void>;
		onRemoveFromCollection?: () => Promise<void>;
		onDelete?: () => Promise<void>;
		onClearSelection?: () => void;
	} = $props();

	let isProcessing = $state(false);

	async function handleAction(action: (() => Promise<void>) | undefined) {
		if (!action || isProcessing) return;
		isProcessing = true;
		try {
			await action();
		} finally {
			isProcessing = false;
		}
	}
</script>

{#if selectedCount > 0}
	<div
		class="bg-card border-border fixed right-4 bottom-24 left-4 z-50 rounded-lg border p-4 shadow-lg md:right-auto md:bottom-8 md:left-1/2 md:-translate-x-1/2"
	>
		<div class="flex flex-wrap items-center justify-between gap-3">
			<span class="text-foreground font-ui text-sm font-medium">
				{selectedCount} game{selectedCount === 1 ? '' : 's'} selected
			</span>

			<div class="flex items-center gap-2">
				{#if mode === 'library' && onAddToCollection}
					<Button
						variant="game-primary"
						size="sm"
						onclick={() => handleAction(onAddToCollection)}
						disabled={isProcessing}
					>
						{isProcessing ? 'Adding...' : 'Add to Collection'}
					</Button>
				{/if}

				{#if mode === 'collection' && onRemoveFromCollection}
					<Button
						variant="destructive"
						size="sm"
						onclick={() => handleAction(onRemoveFromCollection)}
						disabled={isProcessing}
					>
						{isProcessing ? 'Removing...' : 'Remove from Collection'}
					</Button>
				{/if}

				{#if mode === 'custom' && onDelete}
					<Button
						variant="destructive"
						size="sm"
						onclick={() => handleAction(onDelete)}
						disabled={isProcessing}
					>
						{isProcessing ? 'Deleting...' : 'Delete'}
					</Button>
				{/if}

				{#if onClearSelection}
					<Button variant="ghost" size="sm" onclick={onClearSelection} disabled={isProcessing}>
						Clear
					</Button>
				{/if}
			</div>
		</div>
	</div>
{/if}
