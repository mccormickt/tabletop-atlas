# Codebase Consolidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce boilerplate and consolidate shared logic across the backend and frontend without changing user-facing behavior.

**Architecture:** Six independent workstreams: (1) structured error handling with `thiserror`, (2) shared helpers/dedup, (3) paginated query builder, (4) handler migration, (5) DB migration, (6) frontend utilities. Each workstream compiles and passes all tests independently.

**Tech Stack:** Rust (thiserror, rusqlite, Dropshot), SvelteKit 5, TypeScript

**Spec:** `docs/superpowers/specs/2026-03-21-codebase-consolidation-design.md`

---

## File Structure

### New Files
- `backend/src/error.rs` — `AppError` enum, `From<AppError> for HttpError`, `DbResultExt`, `OptionExt`
- `frontend/src/lib/stores/auth.svelte.ts` — `createAuthState()` reactive helper

### Modified Files (Backend)
- `backend/Cargo.toml` — add `thiserror`
- `backend/src/main.rs` — add `mod error`
- `backend/src/handlers/mod.rs` — add `IdPath`, re-export error traits
- `backend/src/models/mod.rs` — make `default_page`/`default_limit` pub, remove type aliases
- `backend/src/models/game.rs` — replace `GameId` with `i64`
- `backend/src/models/user.rs` — replace `UserId` with `i64` (keep `SessionId = String`)
- `backend/src/models/house_rule.rs` — replace `HouseRuleId` with `i64`
- `backend/src/models/embedding.rs` — replace `EmbeddingId` with `i64`
- `backend/src/models/chat.rs` — replace `ChatSessionId`/`ChatMessageId` with `i64`
- `backend/src/models/challenge.rs` — replace `ChallengeId`/`ChallengeGameId`/`ChallengePlayId` with `i64`
- `backend/src/db/mod.rs` — add `PaginatedQuery`
- `backend/src/db/games.rs` — replace type aliases, use `PaginatedQuery`
- `backend/src/db/users.rs` — replace type aliases, use `PaginatedQuery`
- `backend/src/db/collections.rs` — replace type aliases, use `PaginatedQuery`
- `backend/src/db/custom_games.rs` — replace type aliases, use `PaginatedQuery`
- `backend/src/db/challenges.rs` — replace type aliases, use `PaginatedQuery`
- `backend/src/db/house_rules.rs` — replace type aliases, use `PaginatedQuery`
- `backend/src/db/chat.rs` — replace type aliases, use `PaginatedQuery`
- `backend/src/db/sessions.rs` — replace type aliases
- `backend/src/db/embeddings.rs` — replace type aliases
- `backend/src/handlers/games.rs` — migrate to `AppError`, `IdPath`, remove local defaults
- `backend/src/handlers/collections.rs` — migrate to `AppError`, `IdPath`
- `backend/src/handlers/custom_games.rs` — migrate to `AppError`, `IdPath`
- `backend/src/handlers/challenges.rs` — migrate to `AppError`, `IdPath` (keep multi-field path structs)
- `backend/src/handlers/house_rules.rs` — migrate to `AppError`, `IdPath`, remove local defaults
- `backend/src/handlers/chat.rs` — migrate to `AppError`, remove `ChatSessionPathParam`
- `backend/src/handlers/upload.rs` — migrate to `AppError`, `IdPath`
- `backend/src/handlers/admin.rs` — migrate to `AppError`, `IdPath`, remove local defaults
- `backend/src/handlers/tools.rs` — migrate to `AppError` (keep `ToolIdPath` — has `String` field)
- `backend/src/handlers/auth.rs` — migrate to `AppError`

### Modified Files (Frontend)
- `frontend/src/lib/utils.ts` — add `unwrapResult()`, `createDebouncedAction()`
- `frontend/src/lib/index.ts` — re-export new utilities
- All `+page.svelte` and `+layout.svelte` files with API calls or auth subscriptions

---

## Task 1: Add `thiserror` and Create `AppError`

**Files:**
- Modify: `backend/Cargo.toml`
- Create: `backend/src/error.rs`
- Modify: `backend/src/main.rs`

- [ ] **Step 1: Add `thiserror` dependency**

In `backend/Cargo.toml`, add under `[dependencies]`:
```toml
thiserror = "2"
```

- [ ] **Step 2: Create `backend/src/error.rs`**

```rust
use crate::handlers::{
    bad_request_error, forbidden_error, internal_error, not_found_error, unauthorized_error,
};
use dropshot::HttpError;
use rusqlite::Result as SqliteResult;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Internal(String),

    #[error("{context}: {source}")]
    Db {
        #[source]
        source: rusqlite::Error,
        context: String,
    },
}

impl From<AppError> for HttpError {
    fn from(err: AppError) -> Self {
        match err {
            AppError::NotFound(msg) => not_found_error(msg),
            AppError::BadRequest(msg) => bad_request_error(msg),
            AppError::Forbidden(msg) => forbidden_error(msg),
            AppError::Unauthorized(msg) => unauthorized_error(msg),
            AppError::Internal(msg) => internal_error(msg),
            AppError::Db { ref context, ref source } => {
                eprintln!("ERROR: {}: {}", context, source);
                internal_error(context.clone())
            }
        }
    }
}

/// Extension trait for converting `SqliteResult<T>` to `Result<T, AppError>` with context.
pub trait DbResultExt<T> {
    fn db_context(self, ctx: &str) -> Result<T, AppError>;
}

impl<T> DbResultExt<T> for SqliteResult<T> {
    fn db_context(self, ctx: &str) -> Result<T, AppError> {
        self.map_err(|e| AppError::Db {
            source: e,
            context: ctx.to_string(),
        })
    }
}

/// Extension trait for converting `Option<T>` to `Result<T, AppError>`.
pub trait OptionExt<T> {
    fn or_not_found(self, msg: impl Into<String>) -> Result<T, AppError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_not_found(self, msg: impl Into<String>) -> Result<T, AppError> {
        self.ok_or_else(|| AppError::NotFound(msg.into()))
    }
}
```

- [ ] **Step 3: Register the module in `main.rs`**

Add `mod error;` after `mod embeddings;` (around line 18 in `backend/src/main.rs`).

- [ ] **Step 4: Verify compilation**

Run: `cargo clippy -- -D warnings`
Expected: Clean compilation. The new module is defined but not yet used by any handlers.

- [ ] **Step 5: Commit**

```
jj describe -m "refactor: add AppError with thiserror for structured error handling"
jj new
```

---

## Task 2: Deduplicate Pagination Defaults and Add `IdPath`

**Files:**
- Modify: `backend/src/models/mod.rs` (lines 43-48)
- Modify: `backend/src/handlers/mod.rs` (after line 90)
- Modify: `backend/src/handlers/games.rs` (lines 20-25)
- Modify: `backend/src/handlers/house_rules.rs` (lines 103-108)
- Modify: `backend/src/handlers/admin.rs` (lines 60-65)

- [ ] **Step 1: Make pagination defaults public in `models/mod.rs`**

Change `fn default_page()` to `pub fn default_page()` and `fn default_limit()` to `pub fn default_limit()` at lines 43-48.

- [ ] **Step 2: Add `IdPath` to `handlers/mod.rs`**

Add after the `deleted_response` function (around line 115):

```rust
/// Shared path parameter for endpoints that take a single `{id}`.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct IdPath {
    pub id: i64,
}
```

- [ ] **Step 3: Delete local `default_page`/`default_limit` from handler files**

In `handlers/games.rs`, delete lines 20-25 and add to the import block:
```rust
use crate::models::{default_page, default_limit};
```

Do the same in `handlers/house_rules.rs` (lines 103-108) and `handlers/admin.rs` (lines 60-65).

Note: The `#[serde(default = "default_page")]` attributes on search param structs will resolve correctly as long as `default_page` is imported into scope.

- [ ] **Step 4: Verify compilation**

Run: `cargo clippy -- -D warnings`
Expected: Clean compilation. `IdPath` is defined but not yet used.

- [ ] **Step 5: Commit**

```
jj describe -m "refactor: deduplicate pagination defaults and add shared IdPath"
jj new
```

---

## Task 3: Remove Type Aliases

**Files:**
- Modify: `backend/src/models/mod.rs` (lines 28-32)
- Modify: `backend/src/models/game.rs`
- Modify: `backend/src/models/user.rs`
- Modify: `backend/src/models/house_rule.rs`
- Modify: `backend/src/models/embedding.rs`
- Modify: `backend/src/models/chat.rs`
- Modify: `backend/src/models/challenge.rs` (lines 7-9)
- Modify: All `db/` files that import these aliases
- Modify: All `handlers/` files that import these aliases

- [ ] **Step 1: Delete type aliases from `models/mod.rs`**

Remove lines 28-32:
```rust
pub type GameId = i64;
pub type HouseRuleId = i64;
pub type EmbeddingId = i64;
pub type ChatSessionId = i64;
pub type ChatMessageId = i64;
```

- [ ] **Step 2: Delete type aliases from `models/challenge.rs`**

Remove lines 7-9:
```rust
pub type ChallengeId = i64;
pub type ChallengeGameId = i64;
pub type ChallengePlayId = i64;
```

- [ ] **Step 3: Replace all usages across model files**

In each model file, remove the `use super::GameId;` (or similar) import and replace the type alias usage with `i64` in struct fields. Keep `SessionId = String` in `models/user.rs` (line 6) and `pub type UserId = i64` in `models/user.rs` (line 5) — delete `UserId` too, it's `i64`.

Files to update:
- `models/game.rs`: Remove `use super::GameId;`, change `pub id: GameId` → `pub id: i64`
- `models/user.rs`: Remove `pub type UserId = i64;`, change all `UserId` → `i64`. Keep `pub type SessionId = String;`
- `models/house_rule.rs`: Remove `use super::HouseRuleId;`, change `HouseRuleId` → `i64`
- `models/embedding.rs`: Remove `use super::EmbeddingId;`, change `EmbeddingId` → `i64`
- `models/chat.rs`: Remove `use super::{ChatSessionId, ChatMessageId};`, change both → `i64`
- `models/challenge.rs`: Change `ChallengeId`, `ChallengeGameId`, `ChallengePlayId` → `i64`

- [ ] **Step 4: Fix all `db/` and `handlers/` files that imported these aliases**

Use the compiler to find them. Run `cargo check 2>&1 | head -50` and fix each "unresolved import" error by removing the import and replacing the type with `i64`. Repeat until clean.

Common pattern: `use crate::models::GameId;` → delete, then `game_id: GameId` → `game_id: i64`.

- [ ] **Step 5: Verify compilation**

Run: `cargo clippy -- -D warnings`
Expected: Clean compilation.

- [ ] **Step 6: Commit**

```
jj describe -m "refactor: remove i64 type aliases (GameId, UserId, etc.)"
jj new
```

---

## Task 4: Create `PaginatedQuery` Builder

**Files:**
- Modify: `backend/src/db/mod.rs`

- [ ] **Step 1: Add `PaginatedQuery` to `db/mod.rs`**

Add after the `query_row_optional` function (after line 132):

```rust
/// Builder for paginated list queries with dynamic WHERE clauses.
///
/// Handles the common pattern of: build conditions → COUNT query → SELECT with LIMIT/OFFSET.
#[derive(Default)]
pub struct PaginatedQuery {
    conditions: Vec<String>,
    params: Vec<Box<dyn rusqlite::ToSql>>,
}

impl PaginatedQuery {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            params: Vec::new(),
        }
    }

    /// Add a WHERE condition with a bound parameter (e.g., `"role = ?"`, role_value).
    pub fn filter(&mut self, clause: &str, param: impl rusqlite::ToSql + 'static) {
        self.conditions.push(clause.to_string());
        self.params.push(Box::new(param));
    }

    /// Add a LIKE search on a single column with proper wildcard escaping.
    pub fn filter_like(&mut self, column: &str, term: &str) {
        self.conditions
            .push(format!("{} LIKE ? ESCAPE '\\'", column));
        self.params.push(Box::new(like_pattern(term)));
    }

    /// Add a LIKE search across multiple columns (OR). Each column gets its own param.
    pub fn filter_like_any(&mut self, columns: &[&str], term: &str) {
        let pattern = like_pattern(term);
        let parts: Vec<String> = columns
            .iter()
            .map(|col| format!("{} LIKE ? ESCAPE '\\'", col))
            .collect();
        self.conditions.push(format!("({})", parts.join(" OR ")));
        for _ in columns {
            self.params.push(Box::new(pattern.clone()));
        }
    }

    /// Add a raw condition with no bound parameters (e.g., `"rules_pdf_path IS NOT NULL"`).
    pub fn filter_raw(&mut self, clause: &str) {
        self.conditions.push(clause.to_string());
    }

    /// Execute the query: COUNT for total, then SELECT with pagination.
    ///
    /// - `count_from`: table/join for COUNT (e.g., `"master_games g"`)
    /// - `select_columns`: columns for SELECT (e.g., `"g.id, g.name"`)
    /// - `select_from`: table/join for SELECT (may differ from count_from if JOINs add columns)
    /// - `order_by`: ORDER BY clause (e.g., `"g.name ASC"`)
    /// - `group_by`: optional GROUP BY clause (e.g., `"g.id, g.name"`)
    pub fn execute<T>(
        &self,
        conn: &Connection,
        count_from: &str,
        select_columns: &str,
        select_from: &str,
        order_by: &str,
        group_by: Option<&str>,
        page: u32,
        limit: u32,
        mapper: impl Fn(&rusqlite::Row) -> SqliteResult<T>,
    ) -> SqliteResult<crate::models::PaginatedResponse<T>> {
        let pagination = PaginationInfo::new(page, limit);

        let where_clause = if self.conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", self.conditions.join(" AND "))
        };

        let group_clause = group_by
            .map(|g| format!("GROUP BY {}", g))
            .unwrap_or_default();

        // Build param refs for the WHERE clause
        let where_refs: Vec<&dyn rusqlite::ToSql> =
            self.params.iter().map(|p| p.as_ref()).collect();

        // COUNT query
        let count_sql = format!("SELECT COUNT(*) FROM {} {}", count_from, where_clause);
        let total: u32 = conn.query_row(&count_sql, where_refs.as_slice(), |row| row.get(0))?;

        // SELECT query with pagination
        let select_sql = format!(
            "SELECT {} FROM {} {} {} ORDER BY {} LIMIT ? OFFSET ?",
            select_columns, select_from, where_clause, group_clause, order_by
        );

        // Build full param list: WHERE params + LIMIT + OFFSET
        let mut all_refs: Vec<&dyn rusqlite::ToSql> = where_refs;
        let limit_val = pagination.limit;
        let offset_val = pagination.offset;
        all_refs.push(&limit_val);
        all_refs.push(&offset_val);

        let mut stmt = conn.prepare(&select_sql)?;
        let items: Vec<T> = stmt
            .query_map(all_refs.as_slice(), |row| mapper(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(crate::models::PaginatedResponse::new(items, total, page, limit))
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo clippy -- -D warnings`
Expected: Clean compilation. `PaginatedQuery` is defined but not yet used.

- [ ] **Step 3: Commit**

```
jj describe -m "refactor: add PaginatedQuery builder for paginated list endpoints"
jj new
```

---

## Task 5: Migrate Handlers to `AppError` and `IdPath`

This task migrates all handler files to use the new `AppError` extension traits and `IdPath`. Each handler file follows the same mechanical pattern.

**Files:**
- Modify: All `backend/src/handlers/*.rs` files (except `mod.rs` and `static_files.rs`)

- [ ] **Step 1: Migrate `handlers/games.rs`**

Add to imports:
```rust
use crate::error::{DbResultExt, OptionExt};
use super::IdPath;
```

Remove `GamePathParam` struct (line 16). Replace `Path<GamePathParam>` with `Path<IdPath>` in all endpoints.

Replace patterns:
- `match games::list_games(...).await { Ok(result) => ..., Err(e) => { slog::error!(...); Err(internal_error(...)) } }`
  → `let result = games::list_games(...).await.db_context("Failed to list games")?; success_response(result)`
- `.ok_or_else(|| not_found_error(format!("Game with id {} not found", game_id)))`
  → `.or_not_found(format!("Game with id {} not found", game_id))?`
- Remove all `slog::error!` calls for DB errors (logging now happens in `From<AppError> for HttpError`).

- [ ] **Step 2: Migrate `handlers/collections.rs`**

Same pattern. Remove `CollectionEntryPath`, use `IdPath`. Replace `.map_err(|e| internal_error(...))` with `.db_context(...)`.

- [ ] **Step 3: Migrate `handlers/custom_games.rs`**

Same pattern. Remove `CustomGamePath`, use `IdPath`. Also fix the direct `HttpError::for_client_error(None, FORBIDDEN, ...)` call to use `AppError::Forbidden(...)`.

- [ ] **Step 4: Migrate `handlers/challenges.rs`**

Remove `ChallengePath` (line 59), use `IdPath`. **Keep** `ChallengeGamePath`, `ChallengeParticipantPath`, `ChallengePlayPath` (multi-field structs). Replace all `.map_err(|e| internal_error(...))` with `.db_context(...)`.

- [ ] **Step 5: Migrate `handlers/house_rules.rs`**

Remove `HouseRulePathParam`. Replace match-based error handling with `.db_context()`. Remove `slog::error!` calls for DB errors.

- [ ] **Step 6: Migrate `handlers/chat.rs`**

Remove `ChatSessionPathParam` (line 18), use `IdPath`. Replace error patterns.

- [ ] **Step 7: Migrate `handlers/upload.rs`**

Remove `UploadPathParam` (line 19), use `IdPath`. Replace `.map_err(|e| internal_error(...))` with `.db_context(...)`.

- [ ] **Step 8: Migrate `handlers/admin.rs`**

Remove `UserIdPath` (line 78), use `IdPath`. The `GameIdPath` was already in this file for BGG endpoints — remove it too. Keep the string-matching error conversion for the last-admin constraint in `update_user_role`.

- [ ] **Step 9: Migrate `handlers/tools.rs`**

Keep `ToolIdPath` (has `tool_id: String`, not `id: i64`). Only migrate error handling.

- [ ] **Step 10: Migrate `handlers/auth.rs`**

No path params to change. Migrate `.map_err()` patterns.

- [ ] **Step 11: Verify compilation and tests**

Run: `cargo clippy -- -D warnings`
Expected: Clean compilation.

Run: `pnpm run generate` (regenerate API client to confirm OpenAPI spec is unchanged — `IdPath` uses the same `id` field name as all existing path structs, so no diff expected)
Then: `pnpm run check:frontend` to verify the generated client still type-checks.

- [ ] **Step 12: Commit**

```
jj describe -m "refactor: migrate all handlers to AppError and IdPath"
jj new
```

---

## Task 6: Migrate DB List Functions to `PaginatedQuery`

**Files:**
- Modify: `backend/src/db/games.rs` (lines 44-121)
- Modify: `backend/src/db/users.rs` (lines 89-150)
- Modify: `backend/src/db/collections.rs`
- Modify: `backend/src/db/custom_games.rs`
- Modify: `backend/src/db/challenges.rs`
- Modify: `backend/src/db/house_rules.rs`
- Modify: `backend/src/db/chat.rs`

- [ ] **Step 1: Migrate `db/games.rs::list_games`**

Replace the manual WHERE-building + COUNT + SELECT pattern with:

```rust
pub async fn list_games(
    db: &Database,
    page: u32,
    limit: u32,
    search: Option<&str>,
    has_rules_pdf: Option<bool>,
) -> SqliteResult<PaginatedResponse<GameSummary>> {
    db.with_connection(|conn| {
        let mut q = PaginatedQuery::new();

        if let Some(term) = search {
            q.filter_like("LOWER(g.name)", term);
        }
        if let Some(true) = has_rules_pdf {
            q.filter_raw("g.rules_pdf_path IS NOT NULL");
        } else if let Some(false) = has_rules_pdf {
            q.filter_raw("g.rules_pdf_path IS NULL");
        }

        q.execute(
            conn,
            "master_games g",
            "g.id, g.name, g.publisher, g.year_published, g.min_players, g.max_players, g.complexity_rating, g.rules_pdf_path, COUNT(hr.id) as house_rules_count",
            "master_games g LEFT JOIN house_rules hr ON g.id = hr.game_id AND hr.is_active = TRUE",
            "g.name ASC",
            Some("g.id, g.name, g.publisher, g.year_published, g.min_players, g.max_players, g.complexity_rating, g.rules_pdf_path"),
            page,
            limit,
            row_to_game_summary,
        )
    })
}
```

- [ ] **Step 2: Migrate `db/users.rs::list_users`**

```rust
pub async fn list_users(
    db: &Database,
    page: u32,
    limit: u32,
    search: Option<&str>,
    role: Option<&str>,
) -> SqliteResult<PaginatedResponse<UserListItem>> {
    db.with_connection(|conn| {
        let mut q = PaginatedQuery::new();

        if let Some(term) = search {
            q.filter_like_any(&["LOWER(email)", "LOWER(display_name)"], term);
        }
        if let Some(r) = role {
            q.filter("role = ?", r.to_string());
        }

        q.execute(
            conn,
            "users",
            "id, email, display_name, role, created_at",
            "users",
            "created_at DESC",
            None,
            page,
            limit,
            row_to_user_list_item,
        )
    })
}
```

- [ ] **Step 3: Migrate remaining DB list functions**

Apply the same pattern to:
- `db/collections.rs` — `list_collection` (or `list_user_collection`)
- `db/custom_games.rs` — `list_user_custom_games`, `list_public_custom_games`
- `db/challenges.rs` — `list_user_challenges`
- `db/house_rules.rs` — `list_house_rules`
- `db/chat.rs` — `list_chat_sessions`

For `list_user_challenges` which uses a closure mapper (computes `completion_percentage`), the `impl Fn` parameter handles this.

- [ ] **Step 4: Verify compilation and tests**

Run: `cargo clippy -- -D warnings`
Expected: Clean compilation.

Run: `pnpm --prefix frontend run test:e2e`
Expected: All E2E tests pass (API behavior unchanged).

- [ ] **Step 5: Commit**

```
jj describe -m "refactor: migrate paginated list functions to PaginatedQuery builder"
jj new
```

---

## Task 7: Frontend Utilities

**Files:**
- Modify: `frontend/src/lib/utils.ts`
- Modify: `frontend/src/lib/index.ts`
- Create: `frontend/src/lib/stores/auth.svelte.ts`

- [ ] **Step 1: Add `unwrapResult` to `utils.ts`**

Add at the end of `frontend/src/lib/utils.ts`:

```typescript
import type { ApiResult } from '../api/http-client';

export function unwrapResult<T>(
	result: ApiResult<T>,
	fallback: string
): { ok: true; data: T } | { ok: false; error: string } {
	if (result.type === 'success') return { ok: true, data: result.data };
	if (result.type === 'error') return { ok: false, error: result.data.message || fallback };
	// client_error is typically a JSON parse failure — use fallback
	return { ok: false, error: fallback };
}
```

- [ ] **Step 2: Add `createDebouncedAction` to `utils.ts`**

Add after `unwrapResult`:

```typescript
export function createDebouncedAction(fn: () => void, delay = 300) {
	let timeout: ReturnType<typeof setTimeout>;
	return {
		trigger() {
			clearTimeout(timeout);
			timeout = setTimeout(fn, delay);
		},
		cancel() {
			clearTimeout(timeout);
		}
	};
}
```

- [ ] **Step 3: Re-export from `index.ts`**

In `frontend/src/lib/index.ts`, add to the re-export line:
```typescript
export { cn, formatDate, formatDateTime, getStatusColor, unwrapResult, createDebouncedAction } from './utils';
```

- [ ] **Step 4: Create `auth.svelte.ts`**

Create `frontend/src/lib/stores/auth.svelte.ts`:

```typescript
import { useAuth, type AuthState } from './auth';

export function createAuthState() {
	const auth = useAuth();
	let state = $state<AuthState>({ user: null, isLoading: true, error: null });

	$effect(() => {
		const unsubscribe = auth.subscribe((s) => {
			state = s;
		});
		return unsubscribe;
	});

	return {
		get user() {
			return state.user;
		},
		get isLoading() {
			return state.isLoading;
		},
		get error() {
			return state.error;
		},
		get isAdmin() {
			return state.user?.role === 'admin';
		},
		get isAuthenticated() {
			return state.user !== null;
		}
	};
}
```

- [ ] **Step 5: Verify**

Run: `pnpm run check:frontend`
Expected: Clean (0 errors). New utilities are defined but not yet consumed.

- [ ] **Step 6: Commit**

```
jj describe -m "refactor: add frontend utilities (unwrapResult, createDebouncedAction, createAuthState)"
jj new
```

---

## Task 8: Migrate Frontend Pages

This is a mechanical migration. For each page, replace the auth subscription boilerplate, API result branching, and debounce patterns with the new utilities. The approach is the same for every file.

**Files:** All route `+page.svelte` and `+layout.svelte` files listed below.

- [ ] **Step 1: Migrate auth subscriptions**

For each file that has the `useAuth()` + `$effect` + `subscribe` pattern, replace with:

```typescript
import { createAuthState } from '$lib/stores/auth.svelte';
const auth = createAuthState();
```

Then replace `authState.user` with `auth.user`, `authState.isLoading` with `auth.isLoading`, etc.

Files (check each for the pattern):
- `routes/+layout.svelte`
- `routes/+page.svelte`
- `routes/admin/+layout.svelte`
- `routes/admin/users/+page.svelte`
- `routes/games/+page.svelte`
- `routes/games/[id]/+page.svelte`
- `routes/games/custom/add/+page.svelte`
- `routes/challenges/+page.svelte`
- `routes/challenges/[id]/+page.svelte`
- `routes/challenges/[id]/stats/+page.svelte`
- `routes/challenges/new/+page.svelte`
- `routes/collection/+page.svelte`
- `routes/chat/+page.svelte`
- `routes/search/+page.svelte`
- `routes/auth/login/+page.svelte`
- `routes/auth/callback/+page.svelte`

- [ ] **Step 2: Migrate API result handling**

For each file with the three-branch pattern, replace with `unwrapResult`. Example transformation:

Before:
```typescript
const result = await api.methods.listGames({ query: params });
if (result.type === 'success') {
    games = result.data.items;
} else if (result.type === 'error') {
    error = result.data.message || 'Failed to load games';
} else if (result.type === 'client_error') {
    error = result.error.message || 'Failed to load games';
}
```

After:
```typescript
import { unwrapResult } from '$lib';

const r = unwrapResult(await api.methods.listGames({ query: params }), 'Failed to load games');
if (!r.ok) { error = r.error; return; }
games = r.data.items;
```

Files (14+ occurrences across ~12 files — see spec for full list).

- [ ] **Step 3: Migrate debounce patterns**

For each file with `clearTimeout`/`setTimeout` debounce logic, replace with `createDebouncedAction`.

Before:
```typescript
let searchTimeout: ReturnType<typeof setTimeout>;
function handleSearchInput(value: string) {
    searchQuery = value;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => { loadUsers(1); }, 300);
}
```

After:
```typescript
import { createDebouncedAction } from '$lib';

const debouncedSearch = createDebouncedAction(() => loadUsers(1));
function handleSearchInput(value: string) {
    searchQuery = value;
    debouncedSearch.trigger();
}
```

Files:
- `routes/admin/users/+page.svelte`
- `routes/admin/upload/+page.svelte`
- `routes/chat/+page.svelte`
- `routes/search/+page.svelte`

- [ ] **Step 4: Verify**

Run: `pnpm run check` (full lint + type-check)
Expected: 0 errors.

Run: `pnpm --prefix frontend run test:e2e`
Expected: All E2E tests pass.

- [ ] **Step 5: Commit**

```
jj describe -m "refactor: migrate frontend pages to shared utilities"
jj new
```

---

## Task 9: Final Verification

- [ ] **Step 1: Full backend check**

Run: `cargo clippy -- -D warnings`
Expected: Clean.

- [ ] **Step 2: Regenerate API client**

Run: `pnpm run generate`
Then: `pnpm run check:frontend`
Expected: Clean.

- [ ] **Step 3: Full E2E test suite**

Run: `pnpm --prefix frontend run test:e2e`
Expected: All tests pass.

- [ ] **Step 4: Squash spec commit into first task**

```
jj rebase -s <spec-commit> -d <first-task-parent>
jj squash
```

This keeps the spec doc alongside the implementation commits.
