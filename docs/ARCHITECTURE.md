# Tabletop Atlas Architecture

## System Overview

Tabletop Atlas is a board game rules management platform with AI-powered chat. Users upload game rulebooks as PDFs, create house rules, and ask questions about gameplay via a RAG-based chat interface.

**Tech stack:** Rust (Dropshot) backend, Svelte 5 (SvelteKit) frontend, SQLite + sqlite-vec database, Ollama for LLM and embeddings.

## AI / LLM Setup

### Prerequisites

- [Ollama](https://ollama.com/) running locally on port 11434
- Models pulled:
  - `gpt-oss:latest` — chat completions
  - `nomic-embed-text:latest` — text embeddings (768 dimensions)

### Chat Pipeline

1. User sends a message in a chat session tied to a specific game
2. Message text is embedded via `nomic-embed-text`
3. sqlite-vec KNN search finds relevant rule chunks and (optionally) house rules
4. Matched chunks are prepended to the system prompt as context
5. Full conversation + context sent to `gpt-oss` via OpenAI-compatible API
6. Response stored with references to the context chunks used

### Configuration

| Setting | Value | Location |
|---------|-------|----------|
| LLM model | `gpt-oss:latest` | `backend/src/llm.rs:13` |
| Embedding model | `nomic-embed-text:latest` | `backend/src/embeddings.rs:4` |
| Embedding dimensions | 768 | `migrations/V003__create_embeddings_table.sql` |
| API base URL | `http://localhost:11434/v1` | `backend/src/llm.rs`, `backend/src/embeddings.rs` |
| Chat temperature | 0.7 | `backend/src/llm.rs:193` |
| PDF chunk size | 1000 chars | `backend/src/pdf.rs` |
| PDF chunk overlap | 300 chars | `backend/src/pdf.rs` |

## Feature Areas

### Games & Collections

Master game catalog with BGG integration, personal user collections, and custom/homebrew game definitions.

- Master library: name, description, publisher, year, player counts, complexity, BGG ID
- Collections: per-user with quantity, status (owned/playing/traded/sold), notes
- Custom games: user-created with public/private visibility

**Key files:** `handlers/games.rs`, `handlers/collections.rs`, `handlers/custom_games.rs`, `db/games.rs`, `db/collections.rs`, `db/custom_games.rs`

### PDF Rules & Embeddings

Upload rulebook PDFs, extract text, chunk with sentence-aware boundaries, embed, and store for similarity search.

**Upload flow:** file validation → storage in `uploads/` → text extraction (`pdf_extract` crate) → sentence-aware chunking (1000 char target, 300 char overlap) → embedding via `nomic-embed-text` → storage in sqlite-vec.

**API endpoints:**
- `POST /api/games/{id}/rules-upload` — upload PDF
- `GET /api/games/{id}/rules` — get rules info
- `DELETE /api/games/{id}/rules` — delete rules

**Key files:** `pdf.rs`, `embeddings.rs`, `handlers/upload.rs`, `db/embeddings.rs`

### AI Chat

Session-based chat with RAG context from rules PDFs and house rules.

**API endpoints:**
- `GET /api/chat/sessions` — list sessions
- `GET /api/chat/sessions/{id}` — get session with history
- `POST /api/chat/sessions` — create session
- `PATCH /api/chat/sessions/{id}` — toggle house rules inclusion
- `POST /api/chat/{id}/message` — send message, get AI response

**Key files:** `llm.rs`, `handlers/chat.rs`, `db/chat.rs`

### Rules Search

Vector similarity search over embedded rule chunks and house rules.

**API endpoints:**
- `POST /api/games/{id}/search` — similarity search on rules/house rules
- `GET /api/chat/search` — search rules (alias: `GET /api/search`)

**Frontend components:**
- `RulesSearch.svelte` — presentational search with similarity scores and result display
- `HeaderSearch.svelte` — global game search (Cmd/Ctrl+K shortcut, 200ms debounce)
- `search/+page.svelte` — full-page search with game picker

**Key files:** `db/embeddings.rs`, `handlers/chat.rs`

### Authentication

OIDC provider integration with JWT session management.

- Login redirects to configured OIDC provider
- Callback exchanges code for tokens, creates local JWT pair (access + refresh)
- Access tokens carry user_id, email, role; signed with HMAC-SHA256
- Middleware: `require_auth()`, `require_admin()`, `extract_auth()` (optional)

**API endpoints:**
- `GET /api/auth/login` — initiate OIDC flow
- `POST /api/auth/callback` — token exchange
- `GET /api/auth/me` — current user info
- `POST /api/auth/logout` — clear session
- `POST /api/auth/refresh` — refresh access token

**Key files:** `auth/jwt.rs`, `auth/config.rs`, `auth/middleware.rs`, `auth/oidc.rs`, `handlers/auth.rs`, `db/users.rs`, `db/sessions.rs`

### Challenges

Multiplayer gaming challenge grids for tracking plays across participants and games.

- Configurable grid size (1-10 rows/columns)
- Participants, game assignments to grid cells, play recording (winner, date, notes)
- Statistics: wins, plays, streaks, leaderboards
- Statuses: draft, active, completed, archived

**API endpoints:**
- `GET /api/challenges` — list challenges
- `POST /api/challenges` — create challenge
- `GET /api/challenges/{id}` — get challenge details
- `PUT /api/challenges/{id}` — update challenge
- `DELETE /api/challenges/{id}` — delete challenge
- `GET /api/challenges/{id}/grid` — grid view
- `POST /api/challenges/{id}/participants` — add participant
- `DELETE /api/challenges/{id}/participants/{user_id}` — remove participant
- `POST /api/challenges/{id}/games` — assign game to grid
- `DELETE /api/challenges/{id}/games/{game_id}` — remove game from grid
- `POST /api/challenges/{id}/plays` — record play
- `PATCH /api/challenges/{id}/plays/{play_id}` — update play
- `DELETE /api/challenges/{id}/plays/{play_id}` — delete play
- `GET /api/challenges/{id}/stats` — challenge statistics

**Key files:** `handlers/challenges.rs`, `db/challenges.rs`

### Tools & Scoring

Game-specific scoring calculators with a trait-based plugin system.

- `GameTool` trait with `inventory` crate for auto-registration
- Schema-driven: categories, expansions, input types (integer, counter, checkbox, select, science_symbols)
- Scoring rules: Direct, Multiplier, Threshold, Custom formula
- Implemented calculators: 7 Wonders, Carcassonne

**API endpoints:**
- `GET /api/tools` — list available tools
- `GET /api/tools/{toolId}` — tool details with scoring schema
- `POST /api/tools/{toolId}/calculate` — calculate scores

**Key files:** `tools/mod.rs`, `tools/scoring.rs`, `tools/calculators/seven_wonders.rs`, `tools/calculators/carcassonne.rs`, `handlers/tools.rs`

### Admin

Dashboard, BGG CSV import, and game data enrichment from BoardGameGeek API.

- Import games from BGG CSV exports with preview
- Enrich existing games with BGG metadata (single or bulk)
- Rate-limited BGG API calls (500ms between requests)

**API endpoints:**
- `GET /api/admin/stats` — dashboard statistics
- `POST /api/admin/games/import/preview` — preview CSV import
- `POST /api/admin/games/import/execute` — execute import
- `GET /api/admin/enrichment/stats` — enrichment statistics
- `POST /api/admin/enrichment/preview` — preview enrichment
- `POST /api/admin/enrichment/execute` — execute enrichment
- `POST /api/admin/bulk-enrich/preview` — preview bulk enrichment
- `POST /api/admin/bulk-enrich/execute` — execute bulk enrichment

**Key files:** `bgg.rs`, `handlers/admin.rs`, `db/admin.rs`

### House Rules

User-created per-game rules that are embedded into the vector database for inclusion in AI chat context.

- Title, description, category, active/inactive toggle
- Auto-embedded on creation/update for RAG search
- Per-session toggle to include/exclude from chat context
- Deletion cascades to embedding cleanup

**API endpoints:**
- `GET /api/games/{id}/house-rules` — list house rules for game
- `GET /api/house-rules/{id}` — get house rule
- `POST /api/games/{id}/house-rules` — create house rule
- `PUT /api/house-rules/{id}` — update house rule
- `DELETE /api/house-rules/{id}` — delete house rule

**Key files:** `handlers/house_rules.rs`, `db/house_rules.rs`

## Database

SQLite with sqlite-vec extension for vector operations. Database file: `atlas.db`.

### Migrations

14 migrations in `migrations/`, following `V001__description.sql` naming:

| # | Migration | Purpose |
|---|-----------|---------|
| V001 | create_games_table | Master games with BGG fields |
| V002 | create_house_rules_table | House rules |
| V003 | create_embeddings_table | Embeddings + sqlite-vec 768-dim KNN table |
| V004 | seed_games_data | Initial game data |
| V005 | add_house_rules_toggle_to_chat_sessions | include_house_rules flag |
| V006 | create_users_table | User accounts with roles |
| V007 | create_sessions_table | JWT session tracking |
| V008 | rename_games_to_master_games | Schema refactor |
| V009 | create_user_collections_table | User collections |
| V010 | create_custom_games_table | Custom games |
| V011 | update_house_rules_for_multitenancy | User scoping |
| V012 | update_embeddings_for_multitenancy | User scoping |
| V013 | update_chat_sessions_for_multitenancy | User scoping |
| V014 | create_challenges_tables | Challenge grids, participants, plays |

### Key Tables

- `games` — master game library
- `embeddings` + `vec_embeddings` — text chunks with 768-dim float32 vectors
- `chat_sessions` / `chat_messages` — conversation history with context chunk references
- `house_rules` — user house rules with active toggle
- `users` / `sessions` — accounts and JWT sessions
- `user_collections` — personal game collections
- `custom_games` — homebrew games
- `challenges` / `challenge_participants` / `challenge_games` / `challenge_plays` — challenge system
