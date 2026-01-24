/* eslint-disable */

import type { FetchParams, FullParams, ApiResult } from './http-client';
import { dateReplacer, handleResponse, mergeParams, toQueryString } from './http-client';
import { snakeify } from './util';

export type { ApiResult, ErrorBody, ErrorResult } from './http-client';

export type AddParticipantRequest = { userId: number };

export type AddToCollectionRequest = {
	masterGameId: number;
	notes?: string | null;
	rating?: number | null;
};

/**
 * Admin dashboard stats
 */
export type AdminDashboardStats = { masterGamesCount: number };

export type GameType = 'master' | 'custom' | 'collection';

export type AssignGameRequest = {
	displayName?: string | null;
	gameId: number;
	gameType: GameType;
	rowIndex: number;
};

export type UserInfo = {
	displayName?: string | null;
	email: string;
	id: number;
	pictureUrl?: string | null;
	role: string;
};

export type AuthResponse = { user: UserInfo };

/**
 * Error that occurred while enriching a game from BGG
 */
export type BggEnrichError = {
	/** BGG ID */
	bggId: number;
	/** Database game ID */
	gameId: number;
	/** Error message */
	message: string;
};

/**
 * Values for comparing current game data vs BGG data
 */
export type BggGameValues = {
	/** Average weight/complexity (1.0-5.0) */
	complexityRating?: number | null;
	/** Game description */
	description?: string | null;
	/** Maximum players */
	maxPlayers?: number | null;
	/** Minimum players */
	minPlayers?: number | null;
	/** Game name */
	name: string;
	/** Playing time in minutes */
	playTimeMinutes?: number | null;
	/** Year published */
	yearPublished?: number | null;
};

/**
 * A field that will be changed during update
 */
export type FieldChange = {
	/** Field name */
	field: string;
	/** New value (as string for display) */
	newValue?: string | null;
	/** Current value (as string for display) */
	oldValue?: string | null;
};

/**
 * Response for single game BGG enrichment preview
 */
export type BggEnrichPreviewResponse = {
	/** BGG ID */
	bggId: number;
	/** Values from BGG API */
	bggValues: BggGameValues;
	/** List of fields that differ */
	changes: FieldChange[];
	/** Current values in our database */
	currentValues: BggGameValues;
	/** Database game ID */
	gameId: number;
};

/**
 * Request to execute single game BGG enrichment
 */
export type BggEnrichRequest = {
	/** Which fields to update from BGG data */
	fieldsToUpdate: string[];
};

/**
 * Preview of a game that will be enriched from BGG
 */
export type BggGameEnrichPreview = {
	/** BGG ID */
	bggId: number;
	/** Fields that will change */
	changes: FieldChange[];
	/** Database game ID */
	gameId: number;
	/** Game name */
	name: string;
};

/**
 * Preview of a game to be inserted from BGG CSV
 */
export type BggGamePreview = {
	/** BGG object ID */
	bggId: number;
	/** Average weight/complexity (1.0-5.0) */
	complexityRating?: number | null;
	/** Maximum players */
	maxPlayers?: number | null;
	/** Minimum players */
	minPlayers?: number | null;
	/** Game name from BGG */
	name: string;
	/** Playing time in minutes */
	playTimeMinutes?: number | null;
	/** Row number in CSV (1-indexed) */
	row: number;
	/** Year published */
	yearPublished?: number | null;
};

/**
 * Preview of a game that will be updated
 */
export type BggGameUpdatePreview = {
	/** BGG object ID */
	bggId: number;
	/** Fields that will change */
	changes: FieldChange[];
	/** Existing database ID */
	existingId: number;
	/** Game name */
	name: string;
	/** Row number in CSV (1-indexed) */
	row: number;
};

/**
 * Error that occurred while parsing a row
 */
export type BggParseError = {
	/** Error message */
	message: string;
	/** Row number in CSV (1-indexed) */
	row: number;
};

/**
 * Response for BGG import preview
 */
export type BggImportPreviewResponse = {
	/** Parsing errors encountered */
	errors: BggParseError[];
	/** Games that will be inserted (new) */
	gamesToInsert: BggGamePreview[];
	/** Games that will be updated (existing by bgg_id) */
	gamesToUpdate: BggGameUpdatePreview[];
	/** Total rows in the CSV */
	totalRows: number;
};

/**
 * Response for BGG import execution
 */
export type BggImportResponse = {
	/** Errors that occurred during import */
	errors: BggParseError[];
	/** Number of games inserted */
	insertedCount: number;
	/** Number of games updated */
	updatedCount: number;
};

/**
 * Response for bulk BGG enrichment preview
 */
export type BulkEnrichPreviewResponse = {
	/** Errors encountered while fetching from BGG */
	errors: BggEnrichError[];
	/** Games that will be updated */
	gamesToUpdate: BggGameEnrichPreview[];
	/** Total games fetched from BGG */
	totalFetched: number;
};

/**
 * Request for bulk BGG enrichment
 */
export type BulkEnrichRequest = {
	/** Which fields to enrich (e.g., ["year_published", "min_players"]) */
	fieldsToEnrich: string[];
	/** Maximum number of games to process (default 50) */
	limit?: number | null;
};

/**
 * Response for bulk BGG enrichment execution
 */
export type BulkEnrichResponse = {
	/** Errors encountered during update */
	errors: BggEnrichError[];
	/** Number of games updated */
	updatedCount: number;
};

export type ChallengeStatus = 'draft' | 'active' | 'completed' | 'archived';

export type Challenge = {
	createdAt: Date;
	description?: string | null;
	endDate?: string | null;
	gridCols: number;
	gridRows: number;
	id: number;
	name: string;
	ownerId: number;
	startDate?: string | null;
	status: ChallengeStatus;
	updatedAt: Date;
};

export type ChallengeGame = {
	challengeId: number;
	createdAt: Date;
	displayName?: string | null;
	gameId: number;
	gameType: GameType;
	id: number;
	rowIndex: number;
};

export type ParticipantRole = 'owner' | 'participant';

export type ChallengeParticipant = {
	challengeId: number;
	displayName?: string | null;
	id: number;
	joinedAt: Date;
	pictureUrl?: string | null;
	role: ParticipantRole;
	userId: number;
};

export type PlayParticipant = {
	challengePlayId: number;
	displayName?: string | null;
	id: number;
	isWinner: boolean;
	score?: number | null;
	userId: number;
};

export type ChallengePlayWithParticipants = {
	challengeGameId: number;
	challengeId: number;
	colIndex: number;
	createdAt: Date;
	id: number;
	notes?: string | null;
	participants: PlayParticipant[];
	playedAt: string;
	updatedAt: Date;
};

export type LeaderboardEntry = {
	displayName?: string | null;
	pictureUrl?: string | null;
	totalPlays: number;
	userId: number;
	winPercentage: number;
	wins: number;
};

export type ChallengeStats = {
	completedCells: number;
	completionPercentage: number;
	leaderboard: LeaderboardEntry[];
	totalCells: number;
};

export type ChallengeGridView = {
	challenge: Challenge;
	games: ChallengeGame[];
	participants: ChallengeParticipant[];
	plays: ChallengePlayWithParticipants[];
	stats: ChallengeStats;
};

export type ChallengeSummary = {
	completionPercentage: number;
	createdAt: Date;
	description?: string | null;
	endDate?: string | null;
	gridCols: number;
	gridRows: number;
	id: number;
	name: string;
	ownerId: number;
	participantCount: number;
	startDate?: string | null;
	status: ChallengeStatus;
};

export type MessageRole = 'user' | 'assistant' | 'system';

export type ChatMessage = {
	content: string;
	contextChunks?: number[] | null;
	createdAt: Date;
	id: number;
	role: MessageRole;
	sessionId: number;
};

export type ChatSession = {
	createdAt: Date;
	gameId: number;
	id: number;
	includeHouseRules: boolean;
	title?: string | null;
	updatedAt: Date;
};

export type ChatHistory = { messages: ChatMessage[]; session: ChatSession };

export type ChatRequest = { message: string; sessionId: number };

export type ContextSource = {
	chunkText: string;
	embeddingId: number;
	metadata?: string | null;
	similarityScore: number;
	sourceType: string;
};

export type ChatResponse = {
	assistantMessage: ChatMessage;
	contextSources: ContextSource[];
	userMessage: ChatMessage;
};

export type ChatSessionSummary = {
	createdAt: Date;
	gameId: number;
	id: number;
	includeHouseRules: boolean;
	lastMessageAt?: Date | null;
	messageCount: number;
	title?: string | null;
};

export type CollectionEntry = {
	addedAt: Date;
	id: number;
	masterGameId: number;
	notes?: string | null;
	playCount: number;
	rating?: number | null;
	userId: number;
};

export type CollectionEntryWithGame = {
	addedAt: Date;
	gameName: string;
	id: number;
	masterGameId: number;
	notes?: string | null;
	playCount: number;
	rating?: number | null;
};

export type CreateChallengeRequest = {
	description?: string | null;
	endDate?: string | null;
	gridCols?: number;
	gridRows?: number;
	name: string;
	startDate?: string | null;
};

export type CreateChatSessionRequest = {
	gameId: number;
	includeHouseRules?: boolean;
	title?: string | null;
};

export type CreateCustomGameRequest = {
	complexityRating?: number | null;
	description?: string | null;
	isPublic?: boolean | null;
	maxPlayers?: number | null;
	minPlayers?: number | null;
	name: string;
	playTimeMinutes?: number | null;
	publisher?: string | null;
	yearPublished?: number | null;
};

export type CreateGameRequest = {
	bggId?: number | null;
	complexityRating?: number | null;
	description?: string | null;
	maxPlayers?: number | null;
	minPlayers?: number | null;
	name: string;
	playTimeMinutes?: number | null;
	publisher?: string | null;
	yearPublished?: number | null;
};

export type CreateHouseRuleRequest = {
	category?: string | null;
	description: string;
	gameId: number;
	isActive?: boolean;
	title: string;
};

export type CustomGame = {
	complexityRating?: number | null;
	createdAt: Date;
	description?: string | null;
	id: number;
	isPublic: boolean;
	maxPlayers?: number | null;
	minPlayers?: number | null;
	name: string;
	playTimeMinutes?: number | null;
	publisher?: string | null;
	rulesPdfPath?: string | null;
	rulesText?: string | null;
	updatedAt: Date;
	userId: number;
	yearPublished?: number | null;
};

export type CustomGameSummary = {
	complexityRating?: number | null;
	hasRulesPdf: boolean;
	id: number;
	isPublic: boolean;
	maxPlayers?: number | null;
	minPlayers?: number | null;
	name: string;
	publisher?: string | null;
	userId: number;
	yearPublished?: number | null;
};

export type DeleteRulesResponse = {
	embeddingsDeleted: number;
	fileDeleted: boolean;
	message: string;
};

/**
 * Statistics about games needing enrichment
 */
export type EnrichmentStats = {
	/** Games missing at least one field */
	missingAny: number;
	/** Games missing complexity_rating */
	missingComplexity: number;
	/** Games missing description */
	missingDescription: number;
	/** Games missing play_time_minutes */
	missingPlayTime: number;
	/** Games missing player counts (min or max) */
	missingPlayers: number;
	/** Games missing year_published */
	missingYear: number;
	/** Total games with a BGG ID */
	totalWithBggId: number;
};

export type Expansion = { displayName: string; id: string };

export type Game = {
	bggId?: number | null;
	complexityRating?: number | null;
	createdAt: Date;
	description?: string | null;
	id: number;
	maxPlayers?: number | null;
	minPlayers?: number | null;
	name: string;
	playTimeMinutes?: number | null;
	publisher?: string | null;
	rulesPdfPath?: string | null;
	rulesText?: string | null;
	updatedAt: Date;
	yearPublished?: number | null;
};

export type GameSummary = {
	complexityRating?: number | null;
	hasRulesPdf: boolean;
	houseRulesCount: number;
	id: number;
	maxPlayers?: number | null;
	minPlayers?: number | null;
	name: string;
	publisher?: string | null;
	yearPublished?: number | null;
};

export type HouseRule = {
	category?: string | null;
	createdAt: Date;
	description: string;
	gameId: number;
	id: number;
	isActive: boolean;
	title: string;
	updatedAt: Date;
};

export type SelectOption = { label: string; value: number };

export type InputType =
	| 'integer'
	| 'counter'
	| 'checkbox'
	| 'science_symbols'
	| { select: { options: SelectOption[] } };

export type PaginatedResponse_for_ChallengeSummary = {
	items: ChallengeSummary[];
	limit: number;
	page: number;
	total: number;
	totalPages: number;
};

export type PaginatedResponse_for_ChatSessionSummary = {
	items: ChatSessionSummary[];
	limit: number;
	page: number;
	total: number;
	totalPages: number;
};

export type PaginatedResponse_for_CollectionEntryWithGame = {
	items: CollectionEntryWithGame[];
	limit: number;
	page: number;
	total: number;
	totalPages: number;
};

export type PaginatedResponse_for_CustomGameSummary = {
	items: CustomGameSummary[];
	limit: number;
	page: number;
	total: number;
	totalPages: number;
};

export type PaginatedResponse_for_GameSummary = {
	items: GameSummary[];
	limit: number;
	page: number;
	total: number;
	totalPages: number;
};

export type PaginatedResponse_for_HouseRule = {
	items: HouseRule[];
	limit: number;
	page: number;
	total: number;
	totalPages: number;
};

export type PlayParticipantInput = { isWinner: boolean; score?: number | null; userId: number };

export type PlayerRange = { max: number; min: number };

export type PlayerScoreInput = { name: string; scores: Record<string, number> };

export type PlayerScoreResult = {
	categoryScores: Record<string, number>;
	name: string;
	total: number;
};

export type RecordPlayRequest = {
	challengeGameId: number;
	colIndex: number;
	notes?: string | null;
	participants: PlayParticipantInput[];
	playedAt: string;
};

export type RulesInfoResponse = {
	chunkCount: number;
	gameId: number;
	gameName: string;
	hasRulesPdf: boolean;
	lastProcessed?: string | null;
	rulesPdfPath?: string | null;
	textLength?: number | null;
};

export type SearchResult = {
	chunkId: number;
	chunkIndex: number;
	chunkText: string;
	metadata: string;
	similarityScore: number;
};

export type RulesSearchResponse = {
	gameId: number;
	query: string;
	results: SearchResult[];
	totalResults: number;
};

export type ScoreInput = { enabledExpansions: string[]; players: PlayerScoreInput[] };

export type ScoreOutput = { players: PlayerScoreResult[]; winnerIndex?: number | null };

export type ThresholdEntry = { max: number; min: number; score: number };

export type ScoringRule =
	| 'direct'
	| { multiplier: { factor: number } }
	| { threshold: { thresholds: ThresholdEntry[] } }
	| { custom: { formula: string } };

export type ScoringCategory = {
	displayName: string;
	id: string;
	inputType: InputType;
	max?: number | null;
	min?: number | null;
	/** If set, this category only appears when this expansion is enabled */
	requiresExpansion?: string | null;
	rule: ScoringRule;
	step?: number | null;
};

export type ScoringSchema = { categories: ScoringCategory[]; expansions: Expansion[] };

export type ToolType = 'score_calculator' | 'timer' | 'dice_roller' | 'randomizer';

export type ToolDetails = {
	displayName: string;
	id: string;
	playerRange: PlayerRange;
	schema: ScoringSchema;
	toolType: ToolType;
};

export type ToolSummary = {
	displayName: string;
	id: string;
	playerRange: PlayerRange;
	toolType: ToolType;
};

export type UpdateChallengeRequest = {
	description?: string | null;
	endDate?: string | null;
	name?: string | null;
	startDate?: string | null;
	status?: ChallengeStatus | null;
};

export type UpdateChatSessionRequest = {
	includeHouseRules?: boolean | null;
	title?: string | null;
};

export type UpdateCollectionRequest = {
	notes?: string | null;
	playCount?: number | null;
	rating?: number | null;
};

export type UpdateCustomGameRequest = {
	complexityRating?: number | null;
	description?: string | null;
	isPublic?: boolean | null;
	maxPlayers?: number | null;
	minPlayers?: number | null;
	name?: string | null;
	playTimeMinutes?: number | null;
	publisher?: string | null;
	yearPublished?: number | null;
};

export type UpdateGameRequest = {
	bggId?: number | null;
	complexityRating?: number | null;
	description?: string | null;
	maxPlayers?: number | null;
	minPlayers?: number | null;
	name?: string | null;
	playTimeMinutes?: number | null;
	publisher?: string | null;
	yearPublished?: number | null;
};

export type UpdateHouseRuleRequest = {
	category?: string | null;
	description?: string | null;
	isActive?: boolean | null;
	title?: string | null;
};

export type UpdatePlayRequest = {
	notes?: string | null;
	participants?: PlayParticipantInput[] | null;
	playedAt?: string | null;
};

export type UploadResponse = {
	chunksProcessed?: number | null;
	filePath?: string | null;
	message: string;
	textLength?: number | null;
};

export interface ExecuteBggEnrichPathParams {
	id: number;
}

export interface PreviewBggEnrichPathParams {
	id: number;
}

export interface CallbackQueryParams {
	code: string;
	state?: string | null;
}

export interface ListChallengesQueryParams {
	limit?: number;
	page?: number;
}

export interface GetChallengePathParams {
	id: number;
}

export interface DeleteChallengePathParams {
	id: number;
}

export interface UpdateChallengePathParams {
	id: number;
}

export interface AssignGamePathParams {
	id: number;
}

export interface RemoveGamePathParams {
	gameId: number;
	id: number;
}

export interface GetChallengeGridPathParams {
	id: number;
}

export interface AddParticipantPathParams {
	id: number;
}

export interface RemoveParticipantPathParams {
	id: number;
	userId: number;
}

export interface RecordPlayPathParams {
	id: number;
}

export interface DeletePlayPathParams {
	id: number;
	playId: number;
}

export interface UpdatePlayPathParams {
	id: number;
	playId: number;
}

export interface GetChallengeStatsPathParams {
	id: number;
}

export interface SearchRulesQueryParams {
	gameId: string;
	limit?: number | null;
	query: string;
}

export interface ListChatSessionsQueryParams {
	gameId: string;
	limit: number;
	page: number;
}

export interface GetChatSessionPathParams {
	id: number;
}

export interface UpdateChatSessionPathParams {
	id: number;
}

export interface ListCollectionQueryParams {
	limit?: number;
	page?: number;
}

export interface RemoveFromCollectionPathParams {
	id: number;
}

export interface UpdateCollectionEntryPathParams {
	id: number;
}

export interface ListCustomGamesQueryParams {
	limit?: number;
	page?: number;
}

export interface GetCustomGamePathParams {
	id: number;
}

export interface DeleteCustomGamePathParams {
	id: number;
}

export interface UpdateCustomGamePathParams {
	id: number;
}

export interface ListGamesQueryParams {
	hasRulesPdf?: boolean | null;
	limit?: number;
	page?: number;
	search?: string | null;
}

export interface GetGamePathParams {
	id: number;
}

export interface DeleteGamePathParams {
	id: number;
}

export interface UpdateGamePathParams {
	id: number;
}

export interface DeleteRulesPathParams {
	id: number;
}

export interface GetRulesInfoPathParams {
	id: number;
}

export interface UploadRulesPdfPathParams {
	id: number;
}

export interface ListHouseRulesQueryParams {
	gameId: number;
	limit?: number;
	page?: number;
}

export interface GetHouseRulePathParams {
	id: number;
}

export interface DeleteHouseRulePathParams {
	id: number;
}

export interface UpdateHouseRulePathParams {
	id: number;
}

export interface ListPublicCustomGamesQueryParams {
	limit?: number;
	page?: number;
}

export interface GetToolPathParams {
	toolId: string;
}

export interface CalculateScoresPathParams {
	toolId: string;
}

type EmptyObj = Record<string, never>;
export interface ApiConfig {
	/**
	 * No host means requests will be sent to the current host. This is used in
	 * the web console.
	 */
	host?: string;
	token?: string;
	baseParams?: FetchParams;
}

export class Api {
	host: string;
	token?: string;
	baseParams: FetchParams;
	/**
	 * Pulled from info.version in the OpenAPI schema. Sent in the
	 * `api-version` header on all requests.
	 */
	apiVersion = '1.0.0';

	constructor({ host = '', baseParams = {}, token }: ApiConfig = {}) {
		this.host = host;
		this.token = token;

		const headers = new Headers({
			'Content-Type': 'application/json',
			'api-version': this.apiVersion
		});

		if (token) headers.append('Authorization', `Bearer ${token}`);

		this.baseParams = mergeParams({ headers }, baseParams);
	}

	public async request<Data>({
		body,
		path,
		query,
		host,
		...fetchParams
	}: FullParams): Promise<ApiResult<Data>> {
		const url = (host || this.host) + path + toQueryString(query);
		const init = {
			...mergeParams(this.baseParams, fetchParams),
			body: JSON.stringify(snakeify(body), dateReplacer)
		};
		return handleResponse(await fetch(url, init));
	}

	methods = {
		/**
		 * Execute bulk BGG enrichment
		 */
		executeBulkEnrich: ({ body }: { body: BulkEnrichRequest }, params: FetchParams = {}) => {
			return this.request<BulkEnrichResponse>({
				path: `/api/admin/bgg/bulk`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Preview bulk BGG enrichment
		 */
		previewBulkEnrich: ({ body }: { body: BulkEnrichRequest }, params: FetchParams = {}) => {
			return this.request<BulkEnrichPreviewResponse>({
				path: `/api/admin/bgg/bulk/preview`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Execute BGG enrichment for a single game
		 */
		executeBggEnrich: (
			{ path, body }: { path: ExecuteBggEnrichPathParams; body: BggEnrichRequest },
			params: FetchParams = {}
		) => {
			return this.request<Game>({
				path: `/api/admin/bgg/game/${path.id}`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Preview BGG enrichment for a single game
		 */
		previewBggEnrich: (
			{ path }: { path: PreviewBggEnrichPathParams },
			params: FetchParams = {}
		) => {
			return this.request<BggEnrichPreviewResponse>({
				path: `/api/admin/bgg/game/${path.id}/preview`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Get enrichment statistics - how many games are missing data
		 */
		getEnrichmentStats: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<EnrichmentStats>({
				path: `/api/admin/bgg/stats`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Execute BGG CSV import
		 */
		executeBggImport: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<BggImportResponse>({
				path: `/api/admin/games/import`,
				method: 'POST',
				...params
			});
		},
		/**
		 * Preview BGG CSV import (shows what will be inserted/updated)
		 */
		previewBggImport: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<BggImportPreviewResponse>({
				path: `/api/admin/games/import/preview`,
				method: 'POST',
				...params
			});
		},
		/**
		 * Get admin dashboard stats
		 */
		getAdminStats: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<AdminDashboardStats>({
				path: `/api/admin/stats`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Handle Google OAuth callback
		 */
		callback: ({ query }: { query: CallbackQueryParams }, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/api/auth/callback`,
				method: 'GET',
				query,
				...params
			});
		},
		/**
		 * Initiate Google OAuth login - redirects to Google
		 */
		login: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/api/auth/login`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Logout and clear cookies
		 */
		logout: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/api/auth/logout`,
				method: 'POST',
				...params
			});
		},
		/**
		 * Get current user info
		 */
		getMe: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<AuthResponse>({
				path: `/api/auth/me`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Refresh access token
		 */
		refresh: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/api/auth/refresh`,
				method: 'POST',
				...params
			});
		},
		/**
		 * List current user's challenges
		 */
		listChallenges: (
			{ query = {} }: { query?: ListChallengesQueryParams },
			params: FetchParams = {}
		) => {
			return this.request<PaginatedResponse_for_ChallengeSummary>({
				path: `/api/challenges`,
				method: 'GET',
				query,
				...params
			});
		},
		/**
		 * Create a new challenge
		 */
		createChallenge: ({ body }: { body: CreateChallengeRequest }, params: FetchParams = {}) => {
			return this.request<Challenge>({
				path: `/api/challenges`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Get a challenge by ID
		 */
		getChallenge: ({ path }: { path: GetChallengePathParams }, params: FetchParams = {}) => {
			return this.request<Challenge>({
				path: `/api/challenges/${path.id}`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Delete a challenge
		 */
		deleteChallenge: ({ path }: { path: DeleteChallengePathParams }, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/api/challenges/${path.id}`,
				method: 'DELETE',
				...params
			});
		},
		/**
		 * Update a challenge
		 */
		updateChallenge: (
			{ path, body }: { path: UpdateChallengePathParams; body: UpdateChallengeRequest },
			params: FetchParams = {}
		) => {
			return this.request<Challenge>({
				path: `/api/challenges/${path.id}`,
				method: 'PATCH',
				body,
				...params
			});
		},
		/**
		 * Assign a game to a challenge row
		 */
		assignGame: (
			{ path, body }: { path: AssignGamePathParams; body: AssignGameRequest },
			params: FetchParams = {}
		) => {
			return this.request<ChallengeGame>({
				path: `/api/challenges/${path.id}/games`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Remove a game from a challenge
		 */
		removeGame: ({ path }: { path: RemoveGamePathParams }, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/api/challenges/${path.id}/games/${path.gameId}`,
				method: 'DELETE',
				...params
			});
		},
		/**
		 * Get full grid view of a challenge
		 */
		getChallengeGrid: (
			{ path }: { path: GetChallengeGridPathParams },
			params: FetchParams = {}
		) => {
			return this.request<ChallengeGridView>({
				path: `/api/challenges/${path.id}/grid`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Add a participant to a challenge
		 */
		addParticipant: (
			{ path, body }: { path: AddParticipantPathParams; body: AddParticipantRequest },
			params: FetchParams = {}
		) => {
			return this.request<ChallengeParticipant>({
				path: `/api/challenges/${path.id}/participants`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Remove a participant from a challenge
		 */
		removeParticipant: (
			{ path }: { path: RemoveParticipantPathParams },
			params: FetchParams = {}
		) => {
			return this.request<void>({
				path: `/api/challenges/${path.id}/participants/${path.userId}`,
				method: 'DELETE',
				...params
			});
		},
		/**
		 * Record a play in a challenge
		 */
		recordPlay: (
			{ path, body }: { path: RecordPlayPathParams; body: RecordPlayRequest },
			params: FetchParams = {}
		) => {
			return this.request<ChallengePlayWithParticipants>({
				path: `/api/challenges/${path.id}/plays`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Delete a play
		 */
		deletePlay: ({ path }: { path: DeletePlayPathParams }, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/api/challenges/${path.id}/plays/${path.playId}`,
				method: 'DELETE',
				...params
			});
		},
		/**
		 * Update a play
		 */
		updatePlay: (
			{ path, body }: { path: UpdatePlayPathParams; body: UpdatePlayRequest },
			params: FetchParams = {}
		) => {
			return this.request<ChallengePlayWithParticipants>({
				path: `/api/challenges/${path.id}/plays/${path.playId}`,
				method: 'PATCH',
				body,
				...params
			});
		},
		/**
		 * Get challenge stats and leaderboard
		 */
		getChallengeStats: (
			{ path }: { path: GetChallengeStatsPathParams },
			params: FetchParams = {}
		) => {
			return this.request<ChallengeStats>({
				path: `/api/challenges/${path.id}/stats`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Send a message and get AI response
		 */
		chatWithRules: ({ body }: { body: ChatRequest }, params: FetchParams = {}) => {
			return this.request<ChatResponse>({
				path: `/api/chat/message`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Search rules text for a specific game using embedding similarity
		 */
		searchRules: ({ query }: { query: SearchRulesQueryParams }, params: FetchParams = {}) => {
			return this.request<RulesSearchResponse>({
				path: `/api/chat/search-rules`,
				method: 'GET',
				query,
				...params
			});
		},
		/**
		 * List chat sessions for a specific game
		 */
		listChatSessions: (
			{ query }: { query: ListChatSessionsQueryParams },
			params: FetchParams = {}
		) => {
			return this.request<PaginatedResponse_for_ChatSessionSummary>({
				path: `/api/chat/sessions`,
				method: 'GET',
				query,
				...params
			});
		},
		/**
		 * Create a new chat session
		 */
		createChatSession: ({ body }: { body: CreateChatSessionRequest }, params: FetchParams = {}) => {
			return this.request<ChatSession>({
				path: `/api/chat/sessions`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Get a specific chat session with its message history
		 */
		getChatSession: ({ path }: { path: GetChatSessionPathParams }, params: FetchParams = {}) => {
			return this.request<ChatHistory>({
				path: `/api/chat/sessions/${path.id}`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Update a chat session (e.g., toggle include_house_rules)
		 */
		updateChatSession: (
			{ path, body }: { path: UpdateChatSessionPathParams; body: UpdateChatSessionRequest },
			params: FetchParams = {}
		) => {
			return this.request<ChatSession>({
				path: `/api/chat/sessions/${path.id}`,
				method: 'PATCH',
				body,
				...params
			});
		},
		/**
		 * List current user's game collection
		 */
		listCollection: (
			{ query = {} }: { query?: ListCollectionQueryParams },
			params: FetchParams = {}
		) => {
			return this.request<PaginatedResponse_for_CollectionEntryWithGame>({
				path: `/api/collection`,
				method: 'GET',
				query,
				...params
			});
		},
		/**
		 * Add a game to current user's collection
		 */
		addToCollection: ({ body }: { body: AddToCollectionRequest }, params: FetchParams = {}) => {
			return this.request<CollectionEntry>({
				path: `/api/collection`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Remove a game from collection
		 */
		removeFromCollection: (
			{ path }: { path: RemoveFromCollectionPathParams },
			params: FetchParams = {}
		) => {
			return this.request<void>({
				path: `/api/collection/${path.id}`,
				method: 'DELETE',
				...params
			});
		},
		/**
		 * Update a collection entry
		 */
		updateCollectionEntry: (
			{ path, body }: { path: UpdateCollectionEntryPathParams; body: UpdateCollectionRequest },
			params: FetchParams = {}
		) => {
			return this.request<CollectionEntry>({
				path: `/api/collection/${path.id}`,
				method: 'PATCH',
				body,
				...params
			});
		},
		/**
		 * List current user's custom games
		 */
		listCustomGames: (
			{ query = {} }: { query?: ListCustomGamesQueryParams },
			params: FetchParams = {}
		) => {
			return this.request<PaginatedResponse_for_CustomGameSummary>({
				path: `/api/custom-games`,
				method: 'GET',
				query,
				...params
			});
		},
		/**
		 * Create a custom game
		 */
		createCustomGame: ({ body }: { body: CreateCustomGameRequest }, params: FetchParams = {}) => {
			return this.request<CustomGame>({
				path: `/api/custom-games`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Get a custom game (public games visible to all, private only to owner)
		 */
		getCustomGame: ({ path }: { path: GetCustomGamePathParams }, params: FetchParams = {}) => {
			return this.request<CustomGame>({
				path: `/api/custom-games/${path.id}`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Delete a custom game (owner only)
		 */
		deleteCustomGame: (
			{ path }: { path: DeleteCustomGamePathParams },
			params: FetchParams = {}
		) => {
			return this.request<void>({
				path: `/api/custom-games/${path.id}`,
				method: 'DELETE',
				...params
			});
		},
		/**
		 * Update a custom game (owner only)
		 */
		updateCustomGame: (
			{ path, body }: { path: UpdateCustomGamePathParams; body: UpdateCustomGameRequest },
			params: FetchParams = {}
		) => {
			return this.request<CustomGame>({
				path: `/api/custom-games/${path.id}`,
				method: 'PATCH',
				body,
				...params
			});
		},
		/**
		 * List all games with pagination and optional search
		 */
		listGames: ({ query = {} }: { query?: ListGamesQueryParams }, params: FetchParams = {}) => {
			return this.request<PaginatedResponse_for_GameSummary>({
				path: `/api/games`,
				method: 'GET',
				query,
				...params
			});
		},
		/**
		 * Create a new game (admin only)
		 */
		createGame: ({ body }: { body: CreateGameRequest }, params: FetchParams = {}) => {
			return this.request<Game>({
				path: `/api/games`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Get a specific game by ID
		 */
		getGame: ({ path }: { path: GetGamePathParams }, params: FetchParams = {}) => {
			return this.request<Game>({
				path: `/api/games/${path.id}`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Delete a game (admin only)
		 */
		deleteGame: ({ path }: { path: DeleteGamePathParams }, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/api/games/${path.id}`,
				method: 'DELETE',
				...params
			});
		},
		/**
		 * Update an existing game (admin only)
		 */
		updateGame: (
			{ path, body }: { path: UpdateGamePathParams; body: UpdateGameRequest },
			params: FetchParams = {}
		) => {
			return this.request<Game>({
				path: `/api/games/${path.id}`,
				method: 'PATCH',
				body,
				...params
			});
		},
		/**
		 * Delete uploaded rules for a game
		 */
		deleteRules: ({ path }: { path: DeleteRulesPathParams }, params: FetchParams = {}) => {
			return this.request<DeleteRulesResponse>({
				path: `/api/games/${path.id}/rules`,
				method: 'DELETE',
				...params
			});
		},
		/**
		 * Get information about uploaded rules for a game
		 */
		getRulesInfo: ({ path }: { path: GetRulesInfoPathParams }, params: FetchParams = {}) => {
			return this.request<RulesInfoResponse>({
				path: `/api/games/${path.id}/rules-info`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Upload a PDF rules document for a game
		 */
		uploadRulesPdf: ({ path }: { path: UploadRulesPdfPathParams }, params: FetchParams = {}) => {
			return this.request<UploadResponse>({
				path: `/api/games/${path.id}/rules-upload`,
				method: 'POST',
				...params
			});
		},
		/**
		 * List house rules for a specific game
		 */
		listHouseRules: ({ query }: { query: ListHouseRulesQueryParams }, params: FetchParams = {}) => {
			return this.request<PaginatedResponse_for_HouseRule>({
				path: `/api/house-rules`,
				method: 'GET',
				query,
				...params
			});
		},
		/**
		 * Create a new house rule
		 */
		createHouseRule: ({ body }: { body: CreateHouseRuleRequest }, params: FetchParams = {}) => {
			return this.request<HouseRule>({
				path: `/api/house-rules`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Get a specific house rule by ID
		 */
		getHouseRule: ({ path }: { path: GetHouseRulePathParams }, params: FetchParams = {}) => {
			return this.request<HouseRule>({
				path: `/api/house-rules/${path.id}`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Delete a house rule
		 */
		deleteHouseRule: ({ path }: { path: DeleteHouseRulePathParams }, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/api/house-rules/${path.id}`,
				method: 'DELETE',
				...params
			});
		},
		/**
		 * Update an existing house rule
		 */
		updateHouseRule: (
			{ path, body }: { path: UpdateHouseRulePathParams; body: UpdateHouseRuleRequest },
			params: FetchParams = {}
		) => {
			return this.request<HouseRule>({
				path: `/api/house-rules/${path.id}`,
				method: 'PATCH',
				body,
				...params
			});
		},
		/**
		 * List public custom games (browsable by anyone)
		 */
		listPublicCustomGames: (
			{ query = {} }: { query?: ListPublicCustomGamesQueryParams },
			params: FetchParams = {}
		) => {
			return this.request<PaginatedResponse_for_CustomGameSummary>({
				path: `/api/public-games`,
				method: 'GET',
				query,
				...params
			});
		},
		/**
		 * List all available tools
		 */
		listTools: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<ToolSummary[]>({
				path: `/api/tools`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Get tool details including scoring schema
		 */
		getTool: ({ path }: { path: GetToolPathParams }, params: FetchParams = {}) => {
			return this.request<ToolDetails>({
				path: `/api/tools/${path.toolId}`,
				method: 'GET',
				...params
			});
		},
		/**
		 * Calculate scores for a tool (stateless)
		 */
		calculateScores: (
			{ path, body }: { path: CalculateScoresPathParams; body: ScoreInput },
			params: FetchParams = {}
		) => {
			return this.request<ScoreOutput>({
				path: `/api/tools/${path.toolId}/calculate`,
				method: 'POST',
				body,
				...params
			});
		},
		/**
		 * Health check endpoint
		 */
		healthCheck: (_: EmptyObj, params: FetchParams = {}) => {
			return this.request<void>({
				path: `/health`,
				method: 'GET',
				...params
			});
		}
	};
	ws = {};
}

export default Api;
