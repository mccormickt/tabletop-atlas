<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { browser } from '$app/environment';
	import type { GameSummary } from '$lib';
	import { Button, Badge, CardSleeve, Pagination } from '$lib/components/ui';
	import { Dice } from './icons';

	type ViewMode = 'table' | 'cards' | 'compact';
	type SortField = 'name' | 'yearPublished' | 'complexityRating' | 'minPlayers';
	type SortDirection = 'asc' | 'desc';

	let {
		games,
		currentPage = 1,
		totalPages = 1,
		total = 0,
		onPageChange,
		onDelete
	}: {
		games: GameSummary[];
		currentPage?: number;
		totalPages?: number;
		total?: number;
		onPageChange?: (page: number) => void;
		onDelete?: (game: GameSummary) => void;
	} = $props();

	// Initialize view mode from localStorage or URL param
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
	let selectedGames = $state<Set<string>>(new Set());

	// Persist view mode to localStorage when it changes
	$effect(() => {
		if (browser) {
			localStorage.setItem('gamesViewMode', viewMode);
		}
	});

	const sortedGames = $derived(() => {
		return [...games].sort((a, b) => {
			let aVal = a[sortField];
			let bVal = b[sortField];

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

	function handleView(game: GameSummary) {
		goto(`/games/${game.id}`);
	}

	function handleEdit(game: GameSummary) {
		goto(`/games/${game.id}/edit`);
	}

	function toggleSelect(gameId: string) {
		if (selectedGames.has(gameId)) {
			selectedGames.delete(gameId);
		} else {
			selectedGames.add(gameId);
		}
		selectedGames = new Set(selectedGames);
	}

	function selectAll() {
		if (selectedGames.size === games.length) {
			selectedGames = new Set();
		} else {
			selectedGames = new Set(games.map((g) => g.id));
		}
	}

	function formatPlayers(min?: number, max?: number): string {
		if (!min && !max) return '-';
		if (min === max) return String(min);
		return `${min || '?'}-${max || '?'}`;
	}

	function getComplexityColor(rating?: number): string {
		if (!rating) return 'bg-muted';
		if (rating < 2) return 'bg-game-green';
		if (rating < 3) return 'bg-game-yellow';
		if (rating < 4) return 'bg-game-orange';
		return 'bg-game-red';
	}
</script>

<div class="collection-dashboard">
	<!-- Toolbar -->
	<div class="mb-6 flex flex-wrap items-center justify-between gap-4">
		<div class="flex items-center gap-2">
			<span class="text-muted-foreground font-ui text-sm">
				{total} game{total === 1 ? '' : 's'}
			</span>
			{#if selectedGames.size > 0}
				<Badge variant="secondary" class="text-xs">
					{selectedGames.size} selected
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
							<th class="px-4 py-3 text-left">
								<input
									type="checkbox"
									checked={selectedGames.size === games.length && games.length > 0}
									onchange={selectAll}
									class="h-4 w-4 rounded"
								/>
							</th>
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
							<th class="hidden px-4 py-3 text-center lg:table-cell">Status</th>
							<th class="px-4 py-3 text-right">Actions</th>
						</tr>
					</thead>
					<tbody class="divide-border divide-y">
						{#each sortedGames() as game (game.id)}
							<tr class="hover:bg-parchment-dark/50 transition-colors">
								<td class="px-4 py-3">
									<input
										type="checkbox"
										checked={selectedGames.has(game.id)}
										onchange={() => toggleSelect(game.id)}
										class="h-4 w-4 rounded"
									/>
								</td>
								<td class="px-4 py-3">
									<button
										onclick={() => handleView(game)}
										class="hover:text-game-blue text-left transition-colors"
									>
										<div class="font-display font-semibold">{game.name}</div>
										{#if game.publisher}
											<div class="text-muted-foreground text-sm">{game.publisher}</div>
										{/if}
									</button>
								</td>
								<td class="text-muted-foreground hidden px-4 py-3 sm:table-cell">
									{game.yearPublished || '-'}
								</td>
								<td class="hidden px-4 py-3 text-center md:table-cell">
									{formatPlayers(game.minPlayers, game.maxPlayers)}
								</td>
								<td class="hidden px-4 py-3 lg:table-cell">
									<div class="flex items-center justify-center gap-1">
										<div class="dice-rating text-sm {getComplexityColor(game.complexityRating)}">
											{game.complexityRating?.toFixed(1) || '-'}
										</div>
									</div>
								</td>
								<td class="hidden px-4 py-3 lg:table-cell">
									<div class="flex items-center justify-center gap-1">
										{#if game.hasRulesPdf}
											<Badge variant="secondary" class="text-xs">PDF</Badge>
										{/if}
										{#if game.houseRulesCount > 0}
											<Badge variant="outline" class="text-xs">{game.houseRulesCount}HR</Badge>
										{/if}
									</div>
								</td>
								<td class="px-4 py-3 text-right">
									<div class="flex items-center justify-end gap-1">
										<Button variant="ghost" size="sm" onclick={() => handleView(game)}>View</Button>
										<Button variant="ghost" size="sm" onclick={() => handleEdit(game)}>Edit</Button>
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
			{#each sortedGames() as game (game.id)}
				<CardSleeve variant="default" class="p-0">
					<button onclick={() => handleView(game)} class="w-full p-4 text-left">
						<div class="mb-3 flex items-start justify-between">
							<div>
								<h3 class="font-display text-lg font-semibold">{game.name}</h3>
								{#if game.publisher}
									<p class="text-muted-foreground text-sm">{game.publisher}</p>
								{/if}
							</div>
							{#if game.complexityRating}
								<div class="dice-rating text-sm {getComplexityColor(game.complexityRating)}">
									{game.complexityRating.toFixed(1)}
								</div>
							{/if}
						</div>

						<div class="text-muted-foreground mb-3 flex flex-wrap items-center gap-3 text-sm">
							{#if game.yearPublished}
								<span>{game.yearPublished}</span>
							{/if}
							{#if game.minPlayers || game.maxPlayers}
								<span class="flex items-center gap-1">
									<Dice size={14} value={1} />
									{formatPlayers(game.minPlayers, game.maxPlayers)} players
								</span>
							{/if}
						</div>

						<div class="flex items-center gap-2">
							{#if game.hasRulesPdf}
								<Badge variant="secondary" class="text-xs">PDF Rules</Badge>
							{/if}
							{#if game.houseRulesCount > 0}
								<Badge variant="outline" class="text-xs">
									{game.houseRulesCount} House Rule{game.houseRulesCount === 1 ? '' : 's'}
								</Badge>
							{/if}
						</div>
					</button>

					<div class="border-border flex items-center justify-end gap-2 border-t px-4 pt-3 pb-4">
						<Button variant="ghost" size="sm" onclick={() => handleEdit(game)}>Edit</Button>
						{#if onDelete}
							<Button
								variant="ghost"
								size="sm"
								class="text-destructive hover:text-destructive"
								onclick={() => onDelete(game)}
							>
								Delete
							</Button>
						{/if}
					</div>
				</CardSleeve>
			{/each}
		</div>
	{/if}

	<!-- Compact View -->
	{#if viewMode === 'compact'}
		<div class="game-box-lid p-2">
			<div class="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each sortedGames() as game (game.id)}
					<button
						onclick={() => handleView(game)}
						class="hover:bg-parchment-dark flex items-center gap-3 rounded-lg p-3 text-left transition-colors"
					>
						<div
							class="bg-game-blue flex h-8 w-8 flex-shrink-0 items-center justify-center rounded"
						>
							<span class="font-display text-sm font-bold text-white">
								{game.name.charAt(0).toUpperCase()}
							</span>
						</div>
						<div class="min-w-0 flex-1">
							<div class="font-display truncate font-medium">{game.name}</div>
							<div class="text-muted-foreground text-xs">
								{game.yearPublished || 'N/A'} · {formatPlayers(game.minPlayers, game.maxPlayers)}p
							</div>
						</div>
						{#if game.hasRulesPdf}
							<div class="bg-game-green h-2 w-2 flex-shrink-0 rounded-full" title="Has PDF"></div>
						{/if}
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Pagination -->
	{#if totalPages > 1}
		<div class="mt-6 flex flex-col items-center justify-between gap-4 sm:flex-row">
			<div class="text-muted-foreground font-ui text-sm">
				Page {currentPage} of {totalPages} ({total} games)
			</div>

			<Pagination {currentPage} {totalPages} {onPageChange} />
		</div>
	{/if}
</div>
