<script lang="ts">
	import { ComponentTray, ComponentTraySection } from '$lib/components/ui';
	import { Button, Input, Label } from '$lib/components/ui';

	type FilterState = {
		minPlayers?: number;
		maxPlayers?: number;
		minComplexity?: number;
		maxComplexity?: number;
		hasRules?: boolean;
		hasHouseRules?: boolean;
	};

	let {
		filters = $bindable<FilterState>({}),
		onApply,
		onClear
	}: {
		filters?: FilterState;
		onApply?: () => void;
		onClear?: () => void;
	} = $props();

	function clearFilters() {
		filters = {};
		onClear?.();
	}

	function applyFilters() {
		onApply?.();
	}
</script>

<ComponentTray title="Filters" class="w-full">
	<div class="space-y-4">
		<!-- Player Count -->
		<ComponentTraySection>
			<Label class="text-foreground mb-2 block text-sm font-semibold">Player Count</Label>
			<div class="flex items-center gap-2">
				<Input
					type="number"
					placeholder="Min"
					min="1"
					max="99"
					bind:value={filters.minPlayers}
					class="h-8 w-16 text-center text-sm"
				/>
				<span class="text-muted-foreground">to</span>
				<Input
					type="number"
					placeholder="Max"
					min="1"
					max="99"
					bind:value={filters.maxPlayers}
					class="h-8 w-16 text-center text-sm"
				/>
			</div>
		</ComponentTraySection>

		<!-- Complexity -->
		<ComponentTraySection>
			<Label class="text-foreground mb-2 block text-sm font-semibold">Complexity</Label>
			<div class="flex items-center gap-2">
				<Input
					type="number"
					placeholder="Min"
					min="1"
					max="5"
					step="0.5"
					bind:value={filters.minComplexity}
					class="h-8 w-16 text-center text-sm"
				/>
				<span class="text-muted-foreground">to</span>
				<Input
					type="number"
					placeholder="Max"
					min="1"
					max="5"
					step="0.5"
					bind:value={filters.maxComplexity}
					class="h-8 w-16 text-center text-sm"
				/>
			</div>
		</ComponentTraySection>

		<!-- Toggles -->
		<ComponentTraySection>
			<Label class="text-foreground mb-2 block text-sm font-semibold">Content</Label>
			<div class="space-y-2">
				<label class="flex cursor-pointer items-center gap-2">
					<input
						type="checkbox"
						bind:checked={filters.hasRules}
						class="border-border h-4 w-4 rounded"
					/>
					<span class="text-sm">Has PDF Rules</span>
				</label>
				<label class="flex cursor-pointer items-center gap-2">
					<input
						type="checkbox"
						bind:checked={filters.hasHouseRules}
						class="border-border h-4 w-4 rounded"
					/>
					<span class="text-sm">Has House Rules</span>
				</label>
			</div>
		</ComponentTraySection>

		<!-- Actions -->
		<div class="flex gap-2 pt-2">
			<Button variant="game-primary" size="sm" class="flex-1" onclick={applyFilters}>Apply</Button>
			<Button variant="game-secondary" size="sm" class="flex-1" onclick={clearFilters}>
				Clear
			</Button>
		</div>
	</div>
</ComponentTray>
