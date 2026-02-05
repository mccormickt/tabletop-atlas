<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { browser } from '$app/environment';
	import type { GameSummary, CollectionEntryWithGame, CustomGameSummary } from '$lib';
	import { SvelteSet } from 'svelte/reactivity';
	import { Button, Badge, CardSleeve, Pagination } from '$lib/components/ui';
	import { Dice } from './icons';

	type ViewMode = 'table' | 'cards' | 'compact';
	type SortField = 'name' | 'yearPublished' | 'complexityRating' | 'minPlayers';
	type SortDirection = 'asc' | 'desc';
	type DashboardMode = 'library' | 'collection' | 'custom';

	let {
		mode = 'library' as DashboardMode,
		games = [] as GameSummary[],
		collectionItems = [] as CollectionEntryWithGame[],
		customGames = [] as CustomGameSummary[],
		currentPage = 1,
		totalPages = 1,
		total = 0,
		isAdmin = false,
		selectedIds = new Set<string>(),
		onPageChange,
		onDelete,
		onSelectionChange
	}: {
		mode?: DashboardMode;
		games?: GameSummary[];
		collectionItems?: CollectionEntryWithGame[];
		customGames?: CustomGameSummary[];
		currentPage?: number;
		totalPages?: number;
		total?: number;
		isAdmin?: boolean;
		selectedIds?: Set<string>;
		onPageChange?: (page: number) => void;
		onDelete?: (game: GameSummary) => void;
		onSelectionChange?: (selection: Set<string>) => void;
	} = $props();

	// Initialize view mode from localStorage
	function getInitialViewMode(): ViewMode {
		if (browser) {
			const stored = localStorage.getItem('gamesViewMode');
			if (stored === 'table' || stored === 'cards' || stored === 'compact') {
				return stored;
			}
		}
		return 'cards';
	}

	let viewMode = $state<ViewMode>(getInitialViewMode());
	let sortField = $state<SortField>('name');
	let sortDirection = $state<SortDirection>('asc');

	// Persist view mode to localStorage when it changes
	$effect(() => {
		if (browser) {
			localStorage.setItem('gamesViewMode', viewMode);
		}
	});

	// Unified item type for rendering
	type DisplayItem = {
		id: string;
		name: string;
		publisher?: string | null;
		yearPublished?: number | null;
		minPlayers?: number | null;
		maxPlayers?: number | null;
		complexityRating?: number | null;
		hasRulesPdf?: boolean;
		houseRulesCount?: number;
		// Collection-specific
		rating?: number | null;
		notes?: string | null;
		playCount?: number;
		masterGameId?: number;
		// Custom game-specific
		isPublic?: boolean;
	};

	// Convert data to unified display items based on mode
	let displayItems = $derived.by<DisplayItem[]>(() => {
		if (mode === 'library') {
			return games.map((g) => ({
				id: String(g.id),
				name: g.name,
				publisher: g.publisher,
				yearPublished: g.yearPublished,
				minPlayers: g.minPlayers,
				maxPlayers: g.maxPlayers,
				complexityRating: g.complexityRating,
				hasRulesPdf: g.hasRulesPdf,
				houseRulesCount: g.houseRulesCount
			}));
		} else if (mode === 'collection') {
			return collectionItems.map((item) => ({
				id: String(item.id),
				name: item.gameName,
				masterGameId: item.masterGameId,
				rating: item.rating,
				notes: item.notes,
				playCount: item.playCount
			}));
		} else {
			return customGames.map((g) => ({
				id: String(g.id),
				name: g.name,
				yearPublished: g.yearPublished,
				minPlayers: g.minPlayers,
				maxPlayers: g.maxPlayers,
				complexityRating: g.complexityRating,
				hasRulesPdf: g.hasRulesPdf,
				isPublic: g.isPublic
			}));
		}
	});

	const sortedItems = $derived.by(() => {
		const items = displayItems;
		return [...items].sort((a, b) => {
			let aVal: string | number | undefined | null = a[sortField as keyof DisplayItem] as
				| string
				| number
				| undefined
				| null;
			let bVal: string | number | undefined | null = b[sortField as keyof DisplayItem] as
				| string
				| number
				| undefined
				| null;

			if (aVal === undefined || aVal === null) return 1;
			if (bVal === undefined || bVal === null) return -1;

			if (typeof aVal === 'string') {
				aVal = aVal.toLowerCase();
				bVal = (bVal as string).toLowerCase();
			}

			if (aVal < bVal) return sortDirection === 'asc' ? -1 : 1;
			if (aVal > bVal) return sortDirection === 'asc' ? 1 : -1;
			return 0;
		});
	});

	function toggleSort(field: SortField) {
		if (sortField === field) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortField = field;
			sortDirection = 'asc';
		}
	}

	function handleView(item: DisplayItem) {
		if (mode === 'collection' && item.masterGameId) {
			goto(resolve(`/games/${item.masterGameId}`));
		} else if (mode === 'custom') {
			goto(resolve(`/games/custom/${item.id}`));
		} else {
			goto(resolve(`/games/${item.id}`));
		}
	}

	function handleEdit(item: DisplayItem) {
		if (mode === 'custom') {
			goto(resolve(`/games/custom/${item.id}/edit`));
		} else if (mode === 'collection' && item.masterGameId) {
			goto(resolve(`/games/${item.masterGameId}/edit`));
		} else {
			goto(resolve(`/games/${item.id}/edit`));
		}
	}

	function toggleSelect(itemId: string) {
		const newSelection = new SvelteSet(selectedIds);
		if (newSelection.has(itemId)) {
			newSelection.delete(itemId);
		} else {
			newSelection.add(itemId);
		}
		onSelectionChange?.(newSelection);
	}

	function selectAll() {
		const items = displayItems;
		if (selectedIds.size === items.length) {
			onSelectionChange?.(new Set());
		} else {
			onSelectionChange?.(new Set(items.map((i) => i.id)));
		}
	}

	function formatPlayers(min?: number | null, max?: number | null): string {
		if (!min && !max) return '-';
		if (min === max) return String(min);
		return `${min || '?'}-${max || '?'}`;
	}

	function getComplexityColor(rating?: number | null): string {
		if (!rating) return 'bg-muted';
		if (rating < 2) return 'bg-game-green';
		if (rating < 3) return 'bg-game-yellow';
		if (rating < 4) return 'bg-game-orange';
		return 'bg-game-red';
	}

	// Check if we should show actions for this mode
	let showEditButton = $derived(mode === 'library' ? isAdmin : mode === 'custom');
	let showDeleteButton = $derived(mode === 'library' ? isAdmin && !!onDelete : mode === 'custom');
	let showSelectionCheckboxes = $derived(
		mode === 'library' || mode === 'collection' || mode === 'custom'
	);
</script>

<div class="collection-dashboard">
	<!-- Toolbar -->
	<div class="mb-6 flex flex-wrap items-center justify-between gap-4">
		<div class="flex items-center gap-2">
			<span class="text-muted-foreground font-ui text-sm">
				{total}
				{mode === 'collection'
					? 'in collection'
					: mode === 'custom'
						? 'custom game'
						: 'game'}{total === 1 ? '' : 's'}
			</span>
			{#if selectedIds.size > 0}
				<Badge variant="secondary" class="text-xs">
					{selectedIds.size} selected
				</Badge>
			{/if}
		</div>

		<!-- View Mode Toggle -->
		<div class="bg-secondary flex items-center gap-1 rounded-lg p-1">
			<button
				onclick={() => (viewMode = 'table')}
				class="font-ui rounded px-3 py-1 text-sm transition-colors
					{viewMode === 'table'
					? 'bg-card text-foreground shadow-sm'
					: 'text-muted-foreground hover:text-foreground'}"
			>
				List
			</button>
			<button
				onclick={() => (viewMode = 'cards')}
				class="font-ui rounded px-3 py-1 text-sm transition-colors
					{viewMode === 'cards'
					? 'bg-card text-foreground shadow-sm'
					: 'text-muted-foreground hover:text-foreground'}"
			>
				Cards
			</button>
			<button
				onclick={() => (viewMode = 'compact')}
				class="font-ui rounded px-3 py-1 text-sm transition-colors
					{viewMode === 'compact'
					? 'bg-card text-foreground shadow-sm'
					: 'text-muted-foreground hover:text-foreground'}"
			>
				Compact
			</button>
		</div>
	</div>

	<!-- Table View -->
	{#if viewMode === 'table'}
		<div class="game-box-lid overflow-hidden">
			<div class="overflow-x-auto">
				<table class="w-full">
					<thead class="bg-parchment-dark border-wood-dark border-b-2">
						<tr>
							{#if showSelectionCheckboxes}
								<th class="px-4 py-3 text-left">
									<input
										type="checkbox"
										checked={selectedIds.size === displayItems.length && displayItems.length > 0}
										onchange={selectAll}
										class="h-4 w-4 rounded"
									/>
								</th>
							{/if}
							<th class="px-4 py-3 text-left">
								<button
									onclick={() => toggleSort('name')}
									class="font-display hover:text-game-blue flex items-center gap-1 text-sm font-semibold"
								>
									Game
									{#if sortField === 'name'}
										<span class="text-game-blue">{sortDirection === 'asc' ? '↑' : '↓'}</span>
									{/if}
								</button>
							</th>
							{#if mode === 'collection'}
								<th class="hidden px-4 py-3 text-center sm:table-cell">
									<span class="font-display text-sm font-semibold">Rating</span>
								</th>
								<th class="hidden px-4 py-3 text-center md:table-cell">
									<span class="font-display text-sm font-semibold">Plays</span>
								</th>
							{:else}
								<th class="hidden px-4 py-3 text-left sm:table-cell">
									<button
										onclick={() => toggleSort('yearPublished')}
										class="font-display hover:text-game-blue flex items-center gap-1 text-sm font-semibold"
									>
										Year
										{#if sortField === 'yearPublished'}
											<span class="text-game-blue">{sortDirection === 'asc' ? '↑' : '↓'}</span>
										{/if}
									</button>
								</th>
								<th class="hidden px-4 py-3 text-center md:table-cell">
									<button
										onclick={() => toggleSort('minPlayers')}
										class="font-display hover:text-game-blue flex items-center gap-1 text-sm font-semibold"
									>
										Players
										{#if sortField === 'minPlayers'}
											<span class="text-game-blue">{sortDirection === 'asc' ? '↑' : '↓'}</span>
										{/if}
									</button>
								</th>
								<th class="hidden px-4 py-3 text-center lg:table-cell">
									<button
										onclick={() => toggleSort('complexityRating')}
										class="font-display hover:text-game-blue flex items-center gap-1 text-sm font-semibold"
									>
										Complexity
										{#if sortField === 'complexityRating'}
											<span class="text-game-blue">{sortDirection === 'asc' ? '↑' : '↓'}</span>
										{/if}
									</button>
								</th>
							{/if}
							<th class="hidden px-4 py-3 text-center lg:table-cell">Status</th>
							<th class="px-4 py-3 text-right">Actions</th>
						</tr>
					</thead>
					<tbody class="divide-border divide-y">
						{#each sortedItems as item (item.id)}
							<tr class="hover:bg-parchment-dark/50 transition-colors">
								{#if showSelectionCheckboxes}
									<td class="px-4 py-3">
										<input
											type="checkbox"
											checked={selectedIds.has(item.id)}
											onchange={() => toggleSelect(item.id)}
											class="h-4 w-4 rounded"
										/>
									</td>
								{/if}
								<td class="px-4 py-3">
									<button
										onclick={() => handleView(item)}
										class="hover:text-game-blue text-left transition-colors"
									>
										<div class="font-display font-semibold">{item.name}</div>
										{#if item.publisher}
											<div class="text-muted-foreground text-sm">{item.publisher}</div>
										{/if}
										{#if mode === 'collection' && item.notes}
											<div class="text-muted-foreground mt-1 line-clamp-1 text-xs italic">
												{item.notes}
											</div>
										{/if}
									</button>
								</td>
								{#if mode === 'collection'}
									<td class="hidden px-4 py-3 text-center sm:table-cell">
										{#if item.rating}
											<span class="text-game-yellow">{'★'.repeat(item.rating)}</span>
										{:else}
											<span class="text-muted-foreground">-</span>
										{/if}
									</td>
									<td class="hidden px-4 py-3 text-center md:table-cell">
										{item.playCount || 0}
									</td>
								{:else}
									<td class="text-muted-foreground hidden px-4 py-3 sm:table-cell">
										{item.yearPublished || '-'}
									</td>
									<td class="hidden px-4 py-3 text-center md:table-cell">
										{formatPlayers(item.minPlayers, item.maxPlayers)}
									</td>
									<td class="hidden px-4 py-3 lg:table-cell">
										<div class="flex items-center justify-center gap-1">
											<div class="dice-rating text-sm {getComplexityColor(item.complexityRating)}">
												{item.complexityRating?.toFixed(1) || '-'}
											</div>
										</div>
									</td>
								{/if}
								<td class="hidden px-4 py-3 lg:table-cell">
									<div class="flex items-center justify-center gap-1">
										{#if item.hasRulesPdf}
											<Badge variant="secondary" class="text-xs">PDF</Badge>
										{/if}
										{#if item.houseRulesCount && item.houseRulesCount > 0}
											<Badge variant="outline" class="text-xs">{item.houseRulesCount}HR</Badge>
										{/if}
										{#if mode === 'custom' && item.isPublic !== undefined}
											<Badge variant={item.isPublic ? 'default' : 'secondary'} class="text-xs">
												{item.isPublic ? 'Public' : 'Private'}
											</Badge>
										{/if}
									</div>
								</td>
								<td class="px-4 py-3 text-right">
									<div class="flex items-center justify-end gap-1">
										<Button variant="ghost" size="sm" onclick={() => handleView(item)}>View</Button>
										{#if showEditButton}
											<Button variant="ghost" size="sm" onclick={() => handleEdit(item)}
												>Edit</Button
											>
										{/if}
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}

	<!-- Card View -->
	{#if viewMode === 'cards'}
		<div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each sortedItems as item (item.id)}
				<CardSleeve variant="default" class="p-0">
					<div class="relative">
						{#if showSelectionCheckboxes}
							<div class="absolute top-3 left-3 z-10">
								<input
									type="checkbox"
									checked={selectedIds.has(item.id)}
									onchange={() => toggleSelect(item.id)}
									class="h-4 w-4 rounded"
								/>
							</div>
						{/if}
						<button onclick={() => handleView(item)} class="w-full p-4 text-left">
							<div class="mb-3 flex items-start justify-between">
								<div class={showSelectionCheckboxes ? 'ml-6' : ''}>
									<h3 class="font-display text-lg font-semibold">{item.name}</h3>
									{#if item.publisher}
										<p class="text-muted-foreground text-sm">{item.publisher}</p>
									{/if}
								</div>
								{#if item.complexityRating}
									<div class="dice-rating text-sm {getComplexityColor(item.complexityRating)}">
										{item.complexityRating.toFixed(1)}
									</div>
								{/if}
							</div>

							{#if mode === 'collection'}
								<div class="mb-3 flex items-center gap-3">
									{#if item.rating}
										<span class="text-game-yellow text-sm">
											{'★'.repeat(item.rating)}{'☆'.repeat(5 - item.rating)}
										</span>
									{/if}
									{#if item.playCount && item.playCount > 0}
										<span class="text-muted-foreground text-sm">
											{item.playCount} play{item.playCount === 1 ? '' : 's'}
										</span>
									{/if}
								</div>
								{#if item.notes}
									<p class="text-muted-foreground mb-3 line-clamp-2 text-sm italic">
										{item.notes}
									</p>
								{/if}
							{:else}
								<div class="text-muted-foreground mb-3 flex flex-wrap items-center gap-3 text-sm">
									{#if item.yearPublished}
										<span>{item.yearPublished}</span>
									{/if}
									{#if item.minPlayers || item.maxPlayers}
										<span class="flex items-center gap-1">
											<Dice size={14} value={1} />
											{formatPlayers(item.minPlayers, item.maxPlayers)} players
										</span>
									{/if}
								</div>
							{/if}

							<div class="flex items-center gap-2">
								{#if item.hasRulesPdf}
									<Badge variant="secondary" class="text-xs">PDF Rules</Badge>
								{/if}
								{#if item.houseRulesCount && item.houseRulesCount > 0}
									<Badge variant="outline" class="text-xs">
										{item.houseRulesCount} House Rule{item.houseRulesCount === 1 ? '' : 's'}
									</Badge>
								{/if}
								{#if mode === 'custom' && item.isPublic !== undefined}
									<Badge variant={item.isPublic ? 'default' : 'secondary'} class="text-xs">
										{item.isPublic ? 'Public' : 'Private'}
									</Badge>
								{/if}
							</div>
						</button>
					</div>

					{#if showEditButton || showDeleteButton}
						<div class="border-border flex items-center justify-end gap-2 border-t px-4 pt-3 pb-4">
							{#if showEditButton}
								<Button variant="ghost" size="sm" onclick={() => handleEdit(item)}>Edit</Button>
							{/if}
							{#if showDeleteButton && mode === 'library' && onDelete}
								<Button
									variant="ghost"
									size="sm"
									class="text-destructive hover:text-destructive"
									onclick={() =>
										onDelete(games.find((g) => String(g.id) === item.id) as GameSummary)}
								>
									Delete
								</Button>
							{/if}
						</div>
					{/if}
				</CardSleeve>
			{/each}
		</div>
	{/if}

	<!-- Compact View -->
	{#if viewMode === 'compact'}
		<div class="game-box-lid p-2">
			<div class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each sortedItems as item (item.id)}
					<div class="flex items-center gap-2">
						{#if showSelectionCheckboxes}
							<input
								type="checkbox"
								checked={selectedIds.has(item.id)}
								onchange={() => toggleSelect(item.id)}
								class="h-4 w-4 flex-shrink-0 rounded"
							/>
						{/if}
						<button
							onclick={() => handleView(item)}
							class="hover:bg-parchment-dark flex flex-1 items-center gap-3 rounded-lg p-3 text-left transition-colors"
						>
							<div
								class="bg-game-blue flex h-8 w-8 flex-shrink-0 items-center justify-center rounded"
							>
								<span class="font-display text-sm font-bold text-white">
									{item.name.charAt(0).toUpperCase()}
								</span>
							</div>
							<div class="min-w-0 flex-1">
								<div class="font-display truncate font-medium">{item.name}</div>
								<div class="text-muted-foreground text-xs">
									{#if mode === 'collection'}
										{#if item.rating}
											<span class="text-game-yellow">{'★'.repeat(item.rating)}</span>
										{:else}
											No rating
										{/if}
										{#if item.playCount && item.playCount > 0}
											· {item.playCount} plays
										{/if}
									{:else}
										{item.yearPublished || 'N/A'} · {formatPlayers(
											item.minPlayers,
											item.maxPlayers
										)}p
									{/if}
								</div>
							</div>
							{#if item.hasRulesPdf}
								<div class="bg-game-green h-2 w-2 flex-shrink-0 rounded-full" title="Has PDF"></div>
							{/if}
						</button>
					</div>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Pagination -->
	{#if totalPages > 1}
		<div class="mt-6 flex flex-col items-center justify-between gap-4 sm:flex-row">
			<div class="text-muted-foreground font-ui text-sm">
				Page {currentPage} of {totalPages} ({total}
				{mode === 'collection' ? 'in collection' : mode === 'custom' ? 'custom games' : 'games'})
			</div>

			<Pagination {currentPage} {totalPages} {onPageChange} />
		</div>
	{/if}
</div>
