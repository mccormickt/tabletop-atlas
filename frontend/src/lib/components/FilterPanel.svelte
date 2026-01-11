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
			<Label class="text-sm font-semibold text-foreground mb-2 block">Player Count</Label>
			<div class="flex items-center gap-2">
				<Input
					type="number"
					placeholder="Min"
					min="1"
					max="99"
					bind:value={filters.minPlayers}
					class="w-16 text-center h-8 text-sm"
				/>
				<span class="text-muted-foreground">to</span>
				<Input
					type="number"
					placeholder="Max"
					min="1"
					max="99"
					bind:value={filters.maxPlayers}
					class="w-16 text-center h-8 text-sm"
				/>
			</div>
		</ComponentTraySection>

		<!-- Complexity -->
		<ComponentTraySection>
			<Label class="text-sm font-semibold text-foreground mb-2 block">Complexity</Label>
			<div class="flex items-center gap-2">
				<Input
					type="number"
					placeholder="Min"
					min="1"
					max="5"
					step="0.5"
					bind:value={filters.minComplexity}
					class="w-16 text-center h-8 text-sm"
				/>
				<span class="text-muted-foreground">to</span>
				<Input
					type="number"
					placeholder="Max"
					min="1"
					max="5"
					step="0.5"
					bind:value={filters.maxComplexity}
					class="w-16 text-center h-8 text-sm"
				/>
			</div>
		</ComponentTraySection>

		<!-- Toggles -->
		<ComponentTraySection>
			<Label class="text-sm font-semibold text-foreground mb-2 block">Content</Label>
			<div class="space-y-2">
				<label class="flex items-center gap-2 cursor-pointer">
					<input
						type="checkbox"
						bind:checked={filters.hasRules}
						class="w-4 h-4 rounded border-border"
					/>
					<span class="text-sm">Has PDF Rules</span>
				</label>
				<label class="flex items-center gap-2 cursor-pointer">
					<input
						type="checkbox"
						bind:checked={filters.hasHouseRules}
						class="w-4 h-4 rounded border-border"
					/>
					<span class="text-sm">Has House Rules</span>
				</label>
			</div>
		</ComponentTraySection>

		<!-- Actions -->
		<div class="flex gap-2 pt-2">
			<Button variant="game-primary" size="sm" class="flex-1" onclick={applyFilters}>
				Apply
			</Button>
			<Button variant="game-secondary" size="sm" class="flex-1" onclick={clearFilters}>
				Clear
			</Button>
		</div>
	</div>
</ComponentTray>
