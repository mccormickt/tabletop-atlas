// place files you want to import through the `$lib` alias in this folder.

import { Api } from '../api/Api';
import { API_BASE_URL } from './utils';

// Create API client instance
export const api = new Api({
	host: API_BASE_URL,
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
	ChatMessage,
	ChatHistory,
	CreateChatSessionRequest,
	PaginatedResponse_for_GameSummary,
	PaginatedResponse_for_HouseRule,
	PaginatedResponse_for_ChatSessionSummary
} from '../api/Api.js';

// Re-export utilities
export { cn, formatDate, formatDateTime, API_BASE_URL } from './utils.js';
