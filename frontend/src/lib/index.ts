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
	Game,
	GameSummary,
	CreateGameRequest,
	UpdateGameRequest,
	HouseRule,
	CreateHouseRuleRequest,
	UpdateHouseRuleRequest,
	ChatSession,
	ChatSessionSummary,
	ChatMessage,
	ChatHistory,
	ChatRequest,
	ChatResponse,
	ContextSource,
	RulesInfoResponse,
	UploadResponse,
	CreateChatSessionRequest,
	UpdateChatSessionRequest,
	PaginatedResponse_for_GameSummary,
	PaginatedResponse_for_HouseRule,
	PaginatedResponse_for_ChatSessionSummary,
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
	PaginatedResponse_for_CustomGameSummary
} from '../api/Api';

// Re-export utilities
export { cn, formatDate, formatDateTime, getStatusColor } from './utils';
