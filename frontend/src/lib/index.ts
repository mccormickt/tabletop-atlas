// place files you want to import through the `$lib` alias in this folder.

import { Api } from '../api/Api';

// Create API client instance
export const api = new Api({
	baseParams: {
		credentials: 'include',
		headers: {
			'Content-Type': 'application/json'
		}
	}
});

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
	PaginatedResponse_for_GameSummary,
	PaginatedResponse_for_HouseRule,
	PaginatedResponse_for_ChatSessionSummary,
	SearchResult,
	RulesSearchResponse,
	SearchRulesQueryParams
} from '../api/Api';

// Re-export utilities
export { cn, formatDate, formatDateTime } from './utils';
