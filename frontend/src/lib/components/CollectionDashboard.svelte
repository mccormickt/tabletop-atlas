<script lang="ts">
	import { goto } from '$app/navigation';
	import type { GameSummary } from '$lib';
	import { Button, Badge, CardSleeve, ScoreTrack } from '$lib/components/ui';
	import { Dice } from './icons';

	type ViewMode = 'table' | 'cards' | 'compact';
	type SortField = 'name' | 'yearPublished' | 'complexityRating' | 'minPlayers';
	type SortDirection = 'asc' | 'desc';

	let {
		games,
		loading = false,
		currentPage = 1,
		totalPages = 1,
		total = 0,
		onPageChange,
		onDelete
	}: {
		games: GameSummary[];
		loading?: boolean;
		currentPage?: number;
		totalPages?: number;
		total?: number;
		onPageChange?: (page: number) => void;
		onDelete?: (game: GameSummary) => void;
	} = $props();

	let viewMode = $state<ViewMode>('cards');
	let sortField = $state<SortField>('name');
	let sortDirection = $state<SortDirection>('asc');
	let selectedGames = $state<Set<string>>(new Set());

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
	<div class="flex flex-wrap items-center justify-between gap-4 mb-6">
		<div class="flex items-center gap-2">
			<span class="text-sm text-muted-foreground font-ui">
				{total} game{total === 1 ? '' : 's'}
			</span>
			{#if selectedGames.size > 0}
				<Badge variant="secondary" class="text-xs">
					{selectedGames.size} selected
				</Badge>
			{/if}
		</div>

		<!-- View Mode Toggle -->
		<div class="flex items-center gap-1 bg-secondary rounded-lg p-1">
			<button
				onclick={() => (viewMode = 'table')}
				class="px-3 py-1 rounded text-sm font-ui transition-colors
					{viewMode === 'table' ? 'bg-card shadow-sm text-foreground' : 'text-muted-foreground hover:text-foreground'}"
			>
				List
			</button>
			<button
				onclick={() => (viewMode = 'cards')}
				class="px-3 py-1 rounded text-sm font-ui transition-colors
					{viewMode === 'cards' ? 'bg-card shadow-sm text-foreground' : 'text-muted-foreground hover:text-foreground'}"
			>
				Cards
			</button>
			<button
				onclick={() => (viewMode = 'compact')}
				class="px-3 py-1 rounded text-sm font-ui transition-colors
					{viewMode === 'compact' ? 'bg-card shadow-sm text-foreground' : 'text-muted-foreground hover:text-foreground'}"
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
					<thead class="bg-parchment-dark border-b-2 border-wood-dark">
						<tr>
							<th class="px-4 py-3 text-left">
								<input
									type="checkbox"
									checked={selectedGames.size === games.length && games.length > 0}
									onchange={selectAll}
									class="w-4 h-4 rounded"
								/>
							</th>
							<th class="px-4 py-3 text-left">
								<button
									onclick={() => toggleSort('name')}
									class="flex items-center gap-1 font-display font-semibold text-sm hover:text-game-blue"
								>
									Game
									{#if sortField === 'name'}
										<span class="text-game-blue">{sortDirection === 'asc' ? '↑' : '↓'}</span>
									{/if}
								</button>
							</th>
							<th class="px-4 py-3 text-left hidden sm:table-cell">
								<button
									onclick={() => toggleSort('yearPublished')}
									class="flex items-center gap-1 font-display font-semibold text-sm hover:text-game-blue"
								>
									Year
									{#if sortField === 'yearPublished'}
										<span class="text-game-blue">{sortDirection === 'asc' ? '↑' : '↓'}</span>
									{/if}
								</button>
							</th>
							<th class="px-4 py-3 text-center hidden md:table-cell">
								<button
									onclick={() => toggleSort('minPlayers')}
									class="flex items-center gap-1 font-display font-semibold text-sm hover:text-game-blue"
								>
									Players
									{#if sortField === 'minPlayers'}
										<span class="text-game-blue">{sortDirection === 'asc' ? '↑' : '↓'}</span>
									{/if}
								</button>
							</th>
							<th class="px-4 py-3 text-center hidden lg:table-cell">
								<button
									onclick={() => toggleSort('complexityRating')}
									class="flex items-center gap-1 font-display font-semibold text-sm hover:text-game-blue"
								>
									Complexity
									{#if sortField === 'complexityRating'}
										<span class="text-game-blue">{sortDirection === 'asc' ? '↑' : '↓'}</span>
									{/if}
								</button>
							</th>
							<th class="px-4 py-3 text-center hidden lg:table-cell">Status</th>
							<th class="px-4 py-3 text-right">Actions</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-border">
						{#each sortedGames() as game (game.id)}
							<tr class="hover:bg-parchment-dark/50 transition-colors">
								<td class="px-4 py-3">
									<input
										type="checkbox"
										checked={selectedGames.has(game.id)}
										onchange={() => toggleSelect(game.id)}
										class="w-4 h-4 rounded"
									/>
								</td>
								<td class="px-4 py-3">
									<button onclick={() => handleView(game)} class="text-left hover:text-game-blue transition-colors">
										<div class="font-display font-semibold">{game.name}</div>
										{#if game.publisher}
											<div class="text-sm text-muted-foreground">{game.publisher}</div>
										{/if}
									</button>
								</td>
								<td class="px-4 py-3 text-muted-foreground hidden sm:table-cell">
									{game.yearPublished || '-'}
								</td>
								<td class="px-4 py-3 text-center hidden md:table-cell">
									{formatPlayers(game.minPlayers, game.maxPlayers)}
								</td>
								<td class="px-4 py-3 hidden lg:table-cell">
									<div class="flex items-center justify-center gap-1">
										<div class="dice-rating text-sm {getComplexityColor(game.complexityRating)}">
											{game.complexityRating?.toFixed(1) || '-'}
										</div>
									</div>
								</td>
								<td class="px-4 py-3 hidden lg:table-cell">
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
										<Button variant="ghost" size="sm" onclick={() => handleView(game)}>
											View
										</Button>
										<Button variant="ghost" size="sm" onclick={() => handleEdit(game)}>
											Edit
										</Button>
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
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			{#each sortedGames() as game (game.id)}
				<CardSleeve variant="default" class="p-0">
					<button onclick={() => handleView(game)} class="w-full text-left p-4">
						<div class="flex items-start justify-between mb-3">
							<div>
								<h3 class="font-display font-semibold text-lg">{game.name}</h3>
								{#if game.publisher}
									<p class="text-sm text-muted-foreground">{game.publisher}</p>
								{/if}
							</div>
							{#if game.complexityRating}
								<div class="dice-rating text-sm {getComplexityColor(game.complexityRating)}">
									{game.complexityRating.toFixed(1)}
								</div>
							{/if}
						</div>

						<div class="flex flex-wrap items-center gap-3 text-sm text-muted-foreground mb-3">
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

					<div class="flex items-center justify-end gap-2 px-4 pb-4 border-t border-border pt-3">
						<Button variant="ghost" size="sm" onclick={() => handleEdit(game)}>Edit</Button>
						{#if onDelete}
							<Button variant="ghost" size="sm" class="text-destructive hover:text-destructive" onclick={() => onDelete(game)}>
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
			<div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2">
				{#each sortedGames() as game (game.id)}
					<button
						onclick={() => handleView(game)}
						class="flex items-center gap-3 p-3 rounded-lg hover:bg-parchment-dark transition-colors text-left"
					>
						<div class="flex-shrink-0 w-8 h-8 rounded bg-game-blue flex items-center justify-center">
							<span class="text-white font-display font-bold text-sm">
								{game.name.charAt(0).toUpperCase()}
							</span>
						</div>
						<div class="flex-1 min-w-0">
							<div class="font-display font-medium truncate">{game.name}</div>
							<div class="text-xs text-muted-foreground">
								{game.yearPublished || 'N/A'} · {formatPlayers(game.minPlayers, game.maxPlayers)}p
							</div>
						</div>
						{#if game.hasRulesPdf}
							<div class="flex-shrink-0 w-2 h-2 rounded-full bg-game-green" title="Has PDF"></div>
						{/if}
					</button>
				{/each}
			</div>
		</div>
	{/if}

	<!-- Pagination using Score Track -->
	{#if totalPages > 1}
		<div class="mt-6 flex flex-col sm:flex-row items-center justify-between gap-4">
			<div class="text-sm text-muted-foreground font-ui">
				Page {currentPage} of {totalPages}
			</div>

			<div class="flex items-center gap-2">
				<Button
					variant="game-secondary"
					size="sm"
					onclick={() => onPageChange?.(currentPage - 1)}
					disabled={currentPage <= 1}
				>
					Previous
				</Button>

				<!-- Score Track Pagination (hidden on mobile) -->
				<div class="hidden sm:block">
					<ScoreTrack
						total={Math.min(totalPages, 7)}
						current={Math.min(currentPage, 7)}
						showNumbers={true}
					/>
				</div>

				<Button
					variant="game-secondary"
					size="sm"
					onclick={() => onPageChange?.(currentPage + 1)}
					disabled={currentPage >= totalPages}
				>
					Next
				</Button>
			</div>
		</div>
	{/if}
</div>
