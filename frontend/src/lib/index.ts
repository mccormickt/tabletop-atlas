// place files you want to import through the `$lib` alias in this folder.

import { Api, type UploadResponse } from '../api/Api';
import { handleResponse, type ApiResult } from '../api/http-client';

// Create API client instance
export const api = new Api({
	baseParams: {
		credentials: 'include',
		headers: {
			'Content-Type': 'application/json'
		}
	}
});

/**
 * Upload a PDF file for a game's rules.
 * This is a custom function because the generated API client doesn't support binary uploads.
 */
export async function uploadRulesPdf(
	gameId: number,
	file: File
): Promise<ApiResult<UploadResponse>> {
	const response = await fetch(`/api/games/${gameId}/rules-upload`, {
		method: 'POST',
		credentials: 'include',
		headers: {
			'Content-Type': 'application/octet-stream'
		},
		body: file
	});
	return handleResponse<UploadResponse>(response);
}

// Re-export types for convenience
export type {
	// Game types
	Game,
	GameSummary,
	GameType,
	CreateGameRequest,
	UpdateGameRequest,
	// House rules
	HouseRule,
	CreateHouseRuleRequest,
	UpdateHouseRuleRequest,
	// Chat types
	ChatSession,
	ChatSessionSummary,
	ChatMessage,
	ChatHistory,
	ChatRequest,
	ChatResponse,
	ContextSource,
	RulesInfoResponse,
	CreateChatSessionRequest,
	UpdateChatSessionRequest,
	MessageRole,
	// Upload
	UploadResponse,
	DeleteRulesResponse,
	// Search types
	SearchResult,
	RulesSearchResponse,
	SearchRulesQueryParams,
	// Collection types
	CollectionEntry,
	CollectionEntryWithGame,
	AddToCollectionRequest,
	UpdateCollectionRequest,
	PaginatedResponse_for_CollectionEntryWithGame,
	// Custom game types
	CustomGame,
	CustomGameSummary,
	CreateCustomGameRequest,
	UpdateCustomGameRequest,
	PaginatedResponse_for_CustomGameSummary,
	// Challenge types
	Challenge,
	ChallengeSummary,
	ChallengeStatus,
	ChallengeGame,
	ChallengeParticipant,
	ChallengePlayWithParticipants,
	ChallengeStats,
	ChallengeGridView,
	ParticipantRole,
	LeaderboardEntry,
	PlayParticipant,
	PlayParticipantInput,
	RecordPlayRequest,
	CreateChallengeRequest,
	AddParticipantRequest,
	AssignGameRequest,
	PaginatedResponse_for_ChallengeSummary,
	// Tool types
	ToolSummary,
	ToolDetails,
	ToolType,
	ScoreInput,
	ScoreOutput,
	ScoringCategory,
	ScoringSchema,
	ScoringRule,
	PlayerScoreInput,
	PlayerScoreResult,
	Expansion,
	InputType,
	SelectOption,
	ThresholdEntry,
	PlayerRange,
	// Admin / BGG types
	AdminDashboardStats,
	BggEnrichPreviewResponse,
	BggEnrichRequest,
	BggEnrichError,
	BggGameValues,
	BggGameEnrichPreview,
	FieldChange,
	BulkEnrichPreviewResponse,
	BulkEnrichRequest,
	BulkEnrichResponse,
	EnrichmentStats,
	BggGamePreview,
	BggGameUpdatePreview,
	BggParseError,
	BggImportPreviewResponse,
	BggImportResponse,
	// User management types
	UserListItem,
	UpdateUserRoleRequest,
	PaginatedResponse_for_UserListItem,
	// Pagination types
	PaginatedResponse_for_GameSummary,
	PaginatedResponse_for_HouseRule,
	PaginatedResponse_for_ChatSessionSummary
} from '../api/Api';

// Re-export utilities
export {
	cn,
	formatDate,
	formatDateTime,
	getStatusColor,
	unwrapResult,
	createDebouncedAction
} from './utils';
