/* eslint-disable */

    /**
     * This Source Code Form is subject to the terms of the Mozilla Public
     * License, v. 2.0. If a copy of the MPL was not distributed with this
     * file, you can obtain one at https://mozilla.org/MPL/2.0/.
     *
     * Copyright Oxide Computer Company
     */

    import type { FetchParams } from './http-client'
    import { HttpClient, toQueryString } from './http-client'

    export type { ApiConfig, ApiResult, ErrorBody, ErrorResult, } from './http-client'
    
export type MessageRole =
"user"
| "assistant"
| "system"
;

export type ChatMessage =
{"content": string,"contextChunks"?: (number)[] | null,"createdAt": Date,"id": number,"role": MessageRole,"sessionId": number,};

export type ChatSession =
{"createdAt": Date,"gameId": number,"id": number,"title"?: string | null,"updatedAt": Date,};

export type ChatHistory =
{"messages": (ChatMessage)[],"session": ChatSession,};

export type ChatRequest =
{"message": string,"sessionId": number,};

export type ContextSource =
{"chunkText": string,"embeddingId": number,"metadata"?: string | null,"similarityScore": number,"sourceType": string,};

export type ChatResponse =
{"contextSources": (ContextSource)[],"message": ChatMessage,};

export type ChatSessionSummary =
{"createdAt": Date,"gameId": number,"id": number,"lastMessageAt"?: Date | null,"messageCount": number,"title"?: string | null,};

export type CreateChatSessionRequest =
{"gameId": number,"title"?: string | null,};

export type CreateGameRequest =
{"bggId"?: number | null,"complexityRating"?: number | null,"description"?: string | null,"maxPlayers"?: number | null,"minPlayers"?: number | null,"name": string,"playTimeMinutes"?: number | null,"publisher"?: string | null,"yearPublished"?: number | null,};

export type CreateHouseRuleRequest =
{"category"?: string | null,"description": string,"gameId": number,"isActive"?: boolean,"title": string,};

export type Game =
{"bggId"?: number | null,"complexityRating"?: number | null,"createdAt": Date,"description"?: string | null,"id": number,"maxPlayers"?: number | null,"minPlayers"?: number | null,"name": string,"playTimeMinutes"?: number | null,"publisher"?: string | null,"rulesPdfPath"?: string | null,"rulesText"?: string | null,"updatedAt": Date,"yearPublished"?: number | null,};

export type GameSummary =
{"complexityRating"?: number | null,"hasRulesPdf": boolean,"houseRulesCount": number,"id": number,"maxPlayers"?: number | null,"minPlayers"?: number | null,"name": string,"publisher"?: string | null,"yearPublished"?: number | null,};

export type HouseRule =
{"category"?: string | null,"createdAt": Date,"description": string,"gameId": number,"id": number,"isActive": boolean,"title": string,"updatedAt": Date,};

export type PaginatedResponse_for_ChatSessionSummary =
{"items": (ChatSessionSummary)[],"limit": number,"page": number,"total": number,"totalPages": number,};

export type PaginatedResponse_for_GameSummary =
{"items": (GameSummary)[],"limit": number,"page": number,"total": number,"totalPages": number,};

export type PaginatedResponse_for_HouseRule =
{"items": (HouseRule)[],"limit": number,"page": number,"total": number,"totalPages": number,};

export type UpdateGameRequest =
{"bggId"?: number | null,"complexityRating"?: number | null,"description"?: string | null,"maxPlayers"?: number | null,"minPlayers"?: number | null,"name"?: string | null,"playTimeMinutes"?: number | null,"publisher"?: string | null,"yearPublished"?: number | null,};

export type UpdateHouseRuleRequest =
{"category"?: string | null,"description"?: string | null,"isActive"?: boolean | null,"title"?: string | null,};

export type UploadResponse =
{"chunksProcessed"?: number | null,"filePath"?: string | null,"message": string,};

export interface ListChatSessionsQueryParams {
  gameId: number,
  limit?: number,
  page?: number,
}

export interface GetChatSessionPathParams {
  id:number,
}

export interface ListGamesQueryParams {
  limit?: number,
  page?: number,
}

export interface GetGamePathParams {
  id:number,
}

export interface UpdateGamePathParams {
  id:number,
}

export interface DeleteGamePathParams {
  id:number,
}

export interface UploadRulesPdfPathParams {
  id:number,
}

export interface ListHouseRulesQueryParams {
  gameId: number,
  limit?: number,
  page?: number,
}

export interface GetHouseRulePathParams {
  id:number,
}

export interface UpdateHouseRulePathParams {
  id:number,
}

export interface DeleteHouseRulePathParams {
  id:number,
}

type EmptyObj = Record<string, never>;
export class Api extends HttpClient {
       methods = {
/**
* Send a message and get AI response
 */
chatWithRules: ({ 
body, }: {body: ChatRequest,
},
params: FetchParams = {}) => {
         return this.request<ChatResponse>({
           path: `/api/chat/message`,
           method: "POST",
  body,
  ...params,
         })
      },
/**
* List chat sessions for a specific game
 */
listChatSessions: ({ 
query, }: {query: ListChatSessionsQueryParams,
},
params: FetchParams = {}) => {
         return this.request<PaginatedResponse_for_ChatSessionSummary>({
           path: `/api/chat/sessions`,
           method: "GET",
  query,
  ...params,
         })
      },
/**
* Create a new chat session
 */
createChatSession: ({ 
body, }: {body: CreateChatSessionRequest,
},
params: FetchParams = {}) => {
         return this.request<ChatSession>({
           path: `/api/chat/sessions`,
           method: "POST",
  body,
  ...params,
         })
      },
/**
* Get a specific chat session with its message history
 */
getChatSession: ({ 
path, }: {path: GetChatSessionPathParams,
},
params: FetchParams = {}) => {
         return this.request<ChatHistory>({
           path: `/api/chat/sessions/${path.id}`,
           method: "GET",
  ...params,
         })
      },
/**
* List all games with pagination
 */
listGames: ({ 
query = {}, }: {query?: ListGamesQueryParams,
},
params: FetchParams = {}) => {
         return this.request<PaginatedResponse_for_GameSummary>({
           path: `/api/games`,
           method: "GET",
  query,
  ...params,
         })
      },
/**
* Create a new game
 */
createGame: ({ 
body, }: {body: CreateGameRequest,
},
params: FetchParams = {}) => {
         return this.request<Game>({
           path: `/api/games`,
           method: "POST",
  body,
  ...params,
         })
      },
/**
* Get a specific game by ID
 */
getGame: ({ 
path, }: {path: GetGamePathParams,
},
params: FetchParams = {}) => {
         return this.request<Game>({
           path: `/api/games/${path.id}`,
           method: "GET",
  ...params,
         })
      },
/**
* Update an existing game
 */
updateGame: ({ 
path, body, }: {path: UpdateGamePathParams,
body: UpdateGameRequest,
},
params: FetchParams = {}) => {
         return this.request<Game>({
           path: `/api/games/${path.id}`,
           method: "PUT",
  body,
  ...params,
         })
      },
/**
* Delete a game
 */
deleteGame: ({ 
path, }: {path: DeleteGamePathParams,
},
params: FetchParams = {}) => {
         return this.request<void>({
           path: `/api/games/${path.id}`,
           method: "DELETE",
  ...params,
         })
      },
/**
* Upload a PDF rules document for a game
 */
uploadRulesPdf: ({ 
path, }: {path: UploadRulesPdfPathParams,
},
params: FetchParams = {}) => {
         return this.request<UploadResponse>({
           path: `/api/games/${path.id}/upload-rules`,
           method: "POST",
  ...params,
         })
      },
/**
* List house rules for a specific game
 */
listHouseRules: ({ 
query, }: {query: ListHouseRulesQueryParams,
},
params: FetchParams = {}) => {
         return this.request<PaginatedResponse_for_HouseRule>({
           path: `/api/house-rules`,
           method: "GET",
  query,
  ...params,
         })
      },
/**
* Create a new house rule
 */
createHouseRule: ({ 
body, }: {body: CreateHouseRuleRequest,
},
params: FetchParams = {}) => {
         return this.request<HouseRule>({
           path: `/api/house-rules`,
           method: "POST",
  body,
  ...params,
         })
      },
/**
* Get a specific house rule by ID
 */
getHouseRule: ({ 
path, }: {path: GetHouseRulePathParams,
},
params: FetchParams = {}) => {
         return this.request<HouseRule>({
           path: `/api/house-rules/${path.id}`,
           method: "GET",
  ...params,
         })
      },
/**
* Update an existing house rule
 */
updateHouseRule: ({ 
path, body, }: {path: UpdateHouseRulePathParams,
body: UpdateHouseRuleRequest,
},
params: FetchParams = {}) => {
         return this.request<HouseRule>({
           path: `/api/house-rules/${path.id}`,
           method: "PUT",
  body,
  ...params,
         })
      },
/**
* Delete a house rule
 */
deleteHouseRule: ({ 
path, }: {path: DeleteHouseRulePathParams,
},
params: FetchParams = {}) => {
         return this.request<void>({
           path: `/api/house-rules/${path.id}`,
           method: "DELETE",
  ...params,
         })
      },
}
     ws = {
  }
     }

   export default Api;
