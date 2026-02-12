/**
 * Mock data for MSW handlers.
 * All data uses snake_case keys to match backend JSON responses.
 * The generated API client converts snake_case → camelCase via processResponseBody.
 * Raw fetch() calls (challenges, collection, admin, tools) expect snake_case directly.
 */

// --- Users ---

export const mockUser = {
	id: 1,
	email: 'test@example.com',
	display_name: 'Test User',
	picture_url: null,
	role: 'user'
};

export const mockAdminUser = {
	id: 2,
	email: 'admin@test.com',
	display_name: 'Admin User',
	picture_url: null,
	role: 'admin'
};

// --- Games ---

const mockGameItems = [
	{
		id: 1,
		name: 'Catan',
		publisher: 'Kosmos',
		year_published: 1995,
		min_players: 3,
		max_players: 4,
		complexity_rating: 2.3,
		has_rules_pdf: true,
		house_rules_count: 2
	},
	{
		id: 2,
		name: 'Carcassonne',
		publisher: 'Hans im Glück',
		year_published: 2000,
		min_players: 2,
		max_players: 5,
		complexity_rating: 1.9,
		has_rules_pdf: true,
		house_rules_count: 1
	},
	{
		id: 3,
		name: 'Pandemic',
		publisher: 'Z-Man Games',
		year_published: 2008,
		min_players: 2,
		max_players: 4,
		complexity_rating: 2.4,
		has_rules_pdf: false,
		house_rules_count: 0
	}
];

export const mockGamesPage = {
	items: mockGameItems,
	page: 1,
	limit: 24,
	total: 3,
	total_pages: 1
};

export function mockFilteredGamesPage(search: string) {
	const filtered = mockGameItems.filter((g) => g.name.toLowerCase().includes(search.toLowerCase()));
	return {
		items: filtered,
		page: 1,
		limit: 24,
		total: filtered.length,
		total_pages: 1
	};
}

export const mockEmptyPage = {
	items: [],
	page: 1,
	limit: 24,
	total: 0,
	total_pages: 0
};

export const mockGameDetail = {
	id: 1,
	name: 'Catan',
	description: 'Trade, build, and settle the island of Catan.',
	publisher: 'Kosmos',
	year_published: 1995,
	min_players: 3,
	max_players: 4,
	play_time_minutes: 90,
	complexity_rating: 2.3,
	bgg_id: 13,
	rules_pdf_path: '/rules/catan.pdf',
	rules_text: 'Setup: Place the hex tiles...',
	created_at: '2024-01-01T00:00:00Z',
	updated_at: '2024-06-01T00:00:00Z'
};

// --- Rules Info ---

export const mockRulesInfo = {
	game_id: 1,
	game_name: 'Catan',
	has_rules_pdf: true,
	rules_pdf_path: '/rules/catan.pdf',
	chunk_count: 42,
	text_length: 15000,
	last_processed: '2024-06-01T00:00:00Z'
};

export const mockNoRulesInfo = {
	game_id: 3,
	game_name: 'Pandemic',
	has_rules_pdf: false,
	rules_pdf_path: null,
	chunk_count: 0,
	text_length: null,
	last_processed: null
};

// --- House Rules ---

export const mockHouseRulesPage = {
	items: [
		{
			id: 1,
			game_id: 1,
			title: 'Friendly Robber',
			description: 'The robber cannot be placed on a hex with fewer than 3 settlements.',
			category: 'Setup',
			is_active: true,
			created_at: '2024-01-15T00:00:00Z',
			updated_at: '2024-01-15T00:00:00Z'
		}
	],
	page: 1,
	limit: 10,
	total: 1,
	total_pages: 1
};

// --- Chat ---

export const mockChatSessions = {
	items: [
		{
			id: 1,
			game_id: 1,
			title: 'Chat about Catan',
			include_house_rules: true,
			message_count: 4,
			created_at: '2024-06-01T10:00:00Z',
			last_message_at: '2024-06-01T11:00:00Z'
		}
	],
	page: 1,
	limit: 50,
	total: 1,
	total_pages: 1
};

export const mockChatHistory = {
	session: {
		id: 1,
		game_id: 1,
		title: 'Chat about Catan',
		include_house_rules: true,
		created_at: '2024-06-01T10:00:00Z',
		updated_at: '2024-06-01T11:00:00Z'
	},
	messages: [
		{
			id: 1,
			session_id: 1,
			role: 'user',
			content: 'How do I win at Catan?',
			context_chunks: null,
			created_at: '2024-06-01T10:00:00Z'
		},
		{
			id: 2,
			session_id: 1,
			role: 'assistant',
			content: 'To win at Catan, you need to be the first player to reach **10 victory points**.',
			context_chunks: [1, 2],
			created_at: '2024-06-01T10:00:05Z'
		}
	]
};

export const mockChatResponse = {
	user_message: {
		id: 3,
		session_id: 1,
		role: 'user',
		content: 'What about longest road?',
		context_chunks: null,
		created_at: '2024-06-01T10:01:00Z'
	},
	assistant_message: {
		id: 4,
		session_id: 1,
		role: 'assistant',
		content: 'The **Longest Road** card is worth 2 victory points.',
		context_chunks: [3],
		created_at: '2024-06-01T10:01:05Z'
	},
	context_sources: [
		{
			embedding_id: 3,
			chunk_text: 'The longest road bonus gives 2 VP...',
			similarity_score: 0.92,
			metadata: 'Page 5',
			source_type: 'rules'
		}
	]
};

export const mockNewSession = {
	id: 2,
	game_id: 1,
	title: 'Chat about Catan',
	include_house_rules: true,
	created_at: '2024-06-01T12:00:00Z',
	updated_at: '2024-06-01T12:00:00Z'
};

// --- Collection ---

export const mockCollectionPage = {
	items: [
		{
			id: 1,
			master_game_id: 1,
			game_name: 'Catan',
			rating: 8,
			play_count: 12,
			notes: 'Family favorite',
			added_at: '2024-03-01T00:00:00Z'
		},
		{
			id: 2,
			master_game_id: 2,
			game_name: 'Carcassonne',
			rating: 7,
			play_count: 5,
			notes: null,
			added_at: '2024-04-01T00:00:00Z'
		}
	],
	page: 1,
	limit: 24,
	total: 2,
	total_pages: 1
};

// --- Custom Games ---

export const mockCustomGamesPage = {
	items: [
		{
			id: 1,
			name: 'My Custom Card Game',
			publisher: null,
			year_published: 2024,
			min_players: 2,
			max_players: 6,
			complexity_rating: null,
			has_rules_pdf: false,
			is_public: false,
			user_id: 1
		}
	],
	page: 1,
	limit: 24,
	total: 1,
	total_pages: 1
};

// --- Challenges ---

export const mockChallengesPage = {
	items: [
		{
			id: 1,
			name: 'Board Game Night 2024',
			description: 'Weekly challenge',
			status: 'active',
			owner_id: 1,
			grid_rows: 8,
			grid_cols: 8,
			participant_count: 3,
			completion_percentage: 25.0,
			start_date: '2024-01-01',
			end_date: null,
			created_at: '2024-01-01T00:00:00Z'
		}
	],
	page: 1,
	limit: 24,
	total: 1,
	total_pages: 1
};

export const mockChallengeGridView = {
	challenge: {
		id: 1,
		name: 'Board Game Night 2024',
		description: 'Weekly challenge',
		status: 'active',
		owner_id: 1,
		grid_rows: 8,
		grid_cols: 8,
		created_at: '2024-01-01T00:00:00Z',
		updated_at: '2024-01-01T00:00:00Z',
		start_date: '2024-01-01',
		end_date: null
	},
	games: [
		{
			id: 1,
			challenge_id: 1,
			row_index: 0,
			game_type: 'master',
			game_id: 1,
			display_name: 'Catan'
		}
	],
	participants: [
		{
			id: 1,
			challenge_id: 1,
			user_id: 1,
			display_name: 'Test User',
			picture_url: null,
			role: 'owner',
			joined_at: '2024-01-01T00:00:00Z'
		},
		{
			id: 2,
			challenge_id: 1,
			user_id: 3,
			display_name: 'Player Two',
			picture_url: null,
			role: 'participant',
			joined_at: '2024-01-02T00:00:00Z'
		}
	],
	plays: [],
	stats: {
		completed_cells: 16,
		total_cells: 64,
		completion_percentage: 25.0,
		leaderboard: [
			{
				user_id: 1,
				display_name: 'Test User',
				picture_url: null,
				total_plays: 10,
				wins: 6,
				win_percentage: 60.0
			},
			{
				user_id: 3,
				display_name: 'Player Two',
				picture_url: null,
				total_plays: 8,
				wins: 4,
				win_percentage: 50.0
			}
		]
	}
};

export const mockChallengeStats = {
	completed_cells: 16,
	total_cells: 64,
	completion_percentage: 25.0,
	leaderboard: [
		{
			user_id: 1,
			display_name: 'Test User',
			picture_url: null,
			total_plays: 10,
			wins: 6,
			win_percentage: 60.0
		},
		{
			user_id: 3,
			display_name: 'Player Two',
			picture_url: null,
			total_plays: 8,
			wins: 4,
			win_percentage: 50.0
		}
	]
};

export const mockChallengeDetail = {
	id: 1,
	name: 'Board Game Night 2024',
	description: 'Weekly challenge',
	status: 'active',
	owner_id: 1,
	grid_rows: 8,
	grid_cols: 8,
	created_at: '2024-01-01T00:00:00Z',
	updated_at: '2024-01-01T00:00:00Z',
	start_date: '2024-01-01',
	end_date: null
};

// --- Tools ---
// Note: Tool models use #[serde(rename_all = "camelCase")] in the backend,
// so the real API returns camelCase. Both raw fetch() and the generated client see camelCase.

export const mockToolsList = [
	{
		id: '7wonders',
		displayName: '7 Wonders Score Calculator',
		toolType: 'score_calculator',
		playerRange: { min: 3, max: 7 }
	},
	{
		id: 'carcassonne',
		displayName: 'Carcassonne Score Calculator',
		toolType: 'score_calculator',
		playerRange: { min: 2, max: 5 }
	}
];

export const mockToolDetail = {
	id: '7wonders',
	displayName: '7 Wonders Score Calculator',
	toolType: 'score_calculator',
	playerRange: { min: 3, max: 7 },
	schema: {
		categories: [
			{
				id: 'military',
				displayName: 'Military Conflicts',
				inputType: 'integer',
				rule: 'direct',
				min: -6,
				max: 18,
				step: null,
				requiresExpansion: null
			},
			{
				id: 'treasury',
				displayName: 'Treasury',
				inputType: 'integer',
				rule: { multiplier: { factor: 0.333 } },
				min: 0,
				max: null,
				step: null,
				requiresExpansion: null
			}
		],
		expansions: []
	}
};

export const mockScoreOutput = {
	players: [
		{
			name: 'Alice',
			total: 52,
			categoryScores: { military: 10, treasury: 3 }
		},
		{
			name: 'Bob',
			total: 48,
			categoryScores: { military: 6, treasury: 5 }
		}
	],
	winnerIndex: 0
};

// --- Search ---

export const mockSearchResults = {
	game_id: 1,
	query: 'victory points',
	results: [
		{
			chunk_id: 1,
			chunk_index: 5,
			chunk_text:
				'Victory points are earned through settlements (1 VP), cities (2 VP), development cards, and special bonuses.',
			metadata: 'Page 3',
			similarity_score: 0.95
		},
		{
			chunk_id: 2,
			chunk_index: 12,
			chunk_text: 'The first player to reach 10 victory points during their turn wins the game.',
			metadata: 'Page 7',
			similarity_score: 0.88
		}
	],
	total_results: 2
};

// --- Admin ---

export const mockAdminStats = {
	master_games_count: 150
};

export const mockEnrichmentStats = {
	total_with_bgg_id: 120,
	missing_any: 25,
	missing_year: 10,
	missing_players: 8,
	missing_play_time: 15,
	missing_complexity: 20,
	missing_description: 5
};
