# Codebase Consolidation Design

**Date:** 2026-03-21
**Goal:** Reduce boilerplate, consolidate shared logic, and improve maintainability across the backend and frontend without changing any user-facing behavior.

---

## 1. Backend Error Consolidation

### Problem

Error handling is scattered and inconsistent:
- ~30 instances of `.map_err(|e| internal_error(format!("Failed to ...: {}", e)))` across handlers
- Some handlers log with `slog::error!` before returning errors, others don't
- 5 error helper functions in `handlers/mod.rs` wrap every `HttpError` with CORS headers
- Direct `HttpError::for_client_error(...)` calls bypass the shared helpers in some places

### Design

Add `thiserror` dependency and create `backend/src/error.rs`:

```rust
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
```

The `Internal` variant covers non-database internal errors (file I/O in upload, LLM/embedding failures in chat, etc.).

**`From<AppError> for HttpError`** in the same file:
- Maps each variant to the appropriate HTTP status code
- Wraps all responses with CORS headers (using existing `add_cors_headers`)
- Logs `Db` errors via `eprintln!` (consistent with existing mutex-poison logging; Dropshot logs request failures independently)

**`DbResultExt` trait** on `SqliteResult<T>`:
```rust
pub trait DbResultExt<T> {
    fn db_context(self, ctx: &str) -> Result<T, AppError>;
}
```

**`OptionExt` trait** on `Option<T>`:
```rust
pub trait OptionExt<T> {
    fn or_not_found(self, msg: impl Into<String>) -> Result<T, AppError>;
}
```

**Migration path:** Handlers still return `Result<HttpOk<T>, HttpError>`. The `?` operator chains `AppError -> HttpError` via the `From` impl. Migrate handlers file-by-file from `.map_err(|e| internal_error(...))` to `.db_context("...")?` and from `.ok_or_else(|| not_found_error(...))` to `.or_not_found("...")?`.

Existing helpers in `handlers/mod.rs` (`internal_error`, `bad_request_error`, etc.) remain available for cases that don't fit the `AppError` pattern (e.g., validation logic that constructs errors directly).

### Files Changed

| File | Change |
|------|--------|
| `Cargo.toml` | Add `thiserror` dependency |
| `backend/src/error.rs` | New: `AppError` enum, `From` impl, extension traits |
| `backend/src/main.rs` | Add `mod error;` |
| `backend/src/handlers/games.rs` | Migrate to `.db_context()` / `.or_not_found()` |
| `backend/src/handlers/collections.rs` | Same migration |
| `backend/src/handlers/custom_games.rs` | Same migration |
| `backend/src/handlers/challenges.rs` | Same migration |
| `backend/src/handlers/house_rules.rs` | Same migration |
| `backend/src/handlers/chat.rs` | Same migration |
| `backend/src/handlers/upload.rs` | Same migration |
| `backend/src/handlers/admin.rs` | Same migration |
| `backend/src/handlers/tools.rs` | Same migration |
| `backend/src/handlers/auth.rs` | Same migration |

**Note:** `handlers/static_files.rs` has 4 direct `HttpError::for_internal_error(...)` calls but these are filesystem/build-time errors, not DB errors. Leave as-is — not worth adding `AppError` dependency for static file serving.

---

## 2. Backend Shared Helpers & Deduplication

### Problem

- `default_page()` / `default_limit()` defined identically in 3 places
- 6+ near-identical path param structs (`GamePathParam`, `UserIdPath`, `GameIdPath`, `ChallengePath`, `CollectionEntryPath`, `CustomGamePath`)
- Type aliases (`GameId`, `HouseRuleId`, `EmbeddingId`, etc.) are all `i64` with no type safety — they don't prevent misuse

### Design

**Pagination defaults:** Make existing `default_page()` / `default_limit()` in `models/mod.rs` public. Delete the local copies in `handlers/games.rs`, `handlers/admin.rs`, and `handlers/house_rules.rs`. Handler search param structs use `#[serde(default = "...")]` which requires the function to be in scope — update these to reference `crate::models::default_page` via a local `use` import at the top of each handler file.

**Path param consolidation:** Define a single struct in `handlers/mod.rs`:
```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct IdPath {
    pub id: i64,
}
```

Replace all handler-specific path param structs that have the same single `{ id: i64 }` shape, including `UploadPathParam`. Multi-field path structs like `ChallengePlayPath { id, play_id }` and `ChallengeParticipantPath { id, participant_id }` remain as-is since they have different shapes.

**Remove type aliases:** Delete all `i64` type aliases: `GameId`, `HouseRuleId`, `EmbeddingId`, `ChatSessionId`, `ChatMessageId` from `models/mod.rs`, and `ChallengeId`, `ChallengeGameId`, `ChallengePlayId` from `models/challenge.rs`. Replace all usages with `i64`. They provide no compile-time safety and add indirection. Keep `SessionId = String` in `models/user.rs` since it's a different type (`String`, not `i64`).

### Files Changed

| File | Change |
|------|--------|
| `backend/src/models/mod.rs` | Make `default_page`/`default_limit` pub, remove type aliases |
| `backend/src/handlers/mod.rs` | Add `IdPath` struct |
| `backend/src/handlers/games.rs` | Remove `GamePathParam`, `default_page`/`default_limit`; use `IdPath`, import from models |
| `backend/src/handlers/admin.rs` | Remove `GameIdPath`, `UserIdPath`, `default_page`/`default_limit`; use `IdPath`, import from models |
| `backend/src/handlers/challenges.rs` | Remove `ChallengePath`; use `IdPath` (keep `ChallengePlayPath`, `ChallengeParticipantPath`) |
| `backend/src/handlers/collections.rs` | Remove `CollectionEntryPath`; use `IdPath` |
| `backend/src/handlers/custom_games.rs` | Remove `CustomGamePath`; use `IdPath` |
| `backend/src/handlers/house_rules.rs` | Remove `HouseRulePathParam`, local `default_page`/`default_limit`; use `IdPath`, import from models |
| `backend/src/handlers/upload.rs` | Remove `UploadPathParam`; use `IdPath` |
| `backend/src/models/game.rs` | Replace `GameId` with `i64` |
| `backend/src/models/user.rs` | Replace `UserId` with `i64` |
| `backend/src/models/house_rule.rs` | Replace `HouseRuleId` with `i64` |
| `backend/src/models/embedding.rs` | Replace `EmbeddingId` with `i64` |
| `backend/src/models/chat.rs` | Replace `ChatSessionId`/`ChatMessageId` with `i64` |
| `backend/src/models/challenge.rs` | Replace `ChallengeId`/`ChallengeGameId`/`ChallengePlayId` with `i64` |
| All db/ files | Replace type alias usage with `i64` |

**Note:** `SessionId = String` in `models/user.rs` is kept — it's a different type, not `i64`. `ToolIdPath { tool_id: String }` in `handlers/tools.rs` is also kept — different field name and type.

---

## 3. Paginated Query Builder

### Problem

Every paginated list function (~6 in the codebase) repeats the same ~30-line pattern:
1. Build `Vec<String>` of WHERE conditions
2. Build `Vec<Box<dyn ToSql>>` of params
3. Conditionally push search/filter clauses
4. Join conditions into WHERE clause
5. Run COUNT query with those conditions
6. Run SELECT query with same conditions + LIMIT/OFFSET
7. Collect rows with a mapper function
8. Return `PaginatedResponse::new(items, total, page, limit)`

### Design

Add `PaginatedQuery` to `db/mod.rs`:

```rust
pub struct PaginatedQuery {
    conditions: Vec<String>,
    params: Vec<Box<dyn rusqlite::ToSql>>,
}

impl PaginatedQuery {
    pub fn new() -> Self;

    /// Add a WHERE condition with a parameter
    pub fn filter(&mut self, clause: &str, param: impl rusqlite::ToSql + 'static);

    /// Add a LIKE search condition with proper escaping
    pub fn filter_like(&mut self, column: &str, term: &str);

    /// Add a multi-column LIKE search (OR across columns)
    pub fn filter_like_any(&mut self, columns: &[&str], term: &str);

    /// Add a raw condition with no parameters (e.g., "rules_pdf_path IS NOT NULL")
    pub fn filter_raw(&mut self, clause: &str);

    /// Execute the paginated query: runs COUNT then SELECT with LIMIT/OFFSET.
    /// Uses `impl Fn` for mapper (not `fn` pointer) so closures that capture
    /// local state work (e.g., `list_user_challenges` computes derived fields).
    /// Params are passed to rusqlite as `&[&dyn ToSql]` via internal conversion
    /// from the `Vec<Box<dyn ToSql>>` storage.
    pub fn execute<T>(
        &self,
        conn: &rusqlite::Connection,
        count_from: &str,       // e.g., "master_games g"
        select_columns: &str,   // e.g., "g.id, g.name, ..."
        select_from: &str,      // e.g., "master_games g LEFT JOIN ..."
        order_by: &str,         // e.g., "g.name ASC"
        page: u32,
        limit: u32,
        mapper: impl Fn(&rusqlite::Row) -> rusqlite::Result<T>,
    ) -> rusqlite::Result<PaginatedResponse<T>>;
}
```

`filter_like` and `filter_like_any` use the existing `like_pattern()` helper and append `ESCAPE '\'` automatically.

The `count_from` vs `select_from` distinction handles cases like `list_games` where the SELECT joins with `house_rules` for a count column but the COUNT query only needs the base table.

### Files Changed

| File | Change |
|------|--------|
| `backend/src/db/mod.rs` | Add `PaginatedQuery` struct and impl |
| `backend/src/db/games.rs` | Refactor `list_games` to use `PaginatedQuery` |
| `backend/src/db/users.rs` | Refactor `list_users` to use `PaginatedQuery` |
| `backend/src/db/collections.rs` | Refactor `list_collection` to use `PaginatedQuery` |
| `backend/src/db/custom_games.rs` | Refactor `list_custom_games` to use `PaginatedQuery` |
| `backend/src/db/challenges.rs` | Refactor `list_user_challenges` to use `PaginatedQuery` |
| `backend/src/db/house_rules.rs` | Refactor `list_house_rules` to use `PaginatedQuery` |
| `backend/src/db/chat.rs` | Refactor `list_chat_sessions` to use `PaginatedQuery` (if paginated) |

---

## 4. Frontend API Result Helper

### Problem

14+ occurrences of the three-branch API result pattern across 10 pages:
```typescript
if (result.type === 'success') { ... }
else if (result.type === 'error') { error = result.data.message || 'Fallback'; }
else if (result.type === 'client_error') { error = result.error.message || 'Fallback'; }
```

Some pages only check `success` and use a bare `else`, missing `client_error` details.

### Design

Add to `$lib/utils.ts` (import `ApiResult` from `../api/http-client`):
```typescript
import type { ApiResult } from '../api/http-client';

export function unwrapResult<T>(
    result: ApiResult<T>,
    fallback: string
): { ok: true; data: T } | { ok: false; error: string } {
    if (result.type === 'success') return { ok: true, data: result.data };
    if (result.type === 'error') return { ok: false, error: result.data.message || fallback };
    // client_error is typically a JSON parse failure — Error.message would be
    // confusing to users, so always use the fallback
    return { ok: false, error: fallback };
}
```

Re-export from `$lib/index.ts`.

Usage in pages:
```typescript
const r = unwrapResult(await api.methods.listGames({ query }), 'Failed to load games');
if (!r.ok) { error = r.error; return; }
games = r.data.items;
```

### Files Changed

| File | Change |
|------|--------|
| `frontend/src/lib/utils.ts` | Add `unwrapResult()` |
| `frontend/src/lib/index.ts` | Re-export `unwrapResult` |
| All route `+page.svelte` files with API calls | Migrate to `unwrapResult()` |

---

## 5. Frontend Auth State Helper

### Problem

12 pages repeat the same auth subscription boilerplate:
```typescript
const auth = useAuth();
let authState = $state<AuthState>({ user: null, isLoading: true, error: null });
$effect(() => {
    const unsubscribe = auth.subscribe((state) => { authState = state; });
    return unsubscribe;
});
```

### Design

Create `frontend/src/lib/stores/auth.svelte.ts`:
```typescript
import { useAuth, type AuthState } from './auth';

export function createAuthState() {
    const auth = useAuth();
    let state = $state<AuthState>({ user: null, isLoading: true, error: null });

    $effect(() => {
        const unsubscribe = auth.subscribe((s) => { state = s; });
        return unsubscribe;
    });

    return {
        get user() { return state.user; },
        get isLoading() { return state.isLoading; },
        get error() { return state.error; },
        get isAdmin() { return state.user?.role === 'admin'; },
        get isAuthenticated() { return state.user !== null; }
    };
}
```

The `.svelte.ts` extension enables rune usage outside components. Getters ensure reads happen at render time for proper Svelte 5 reactivity.

Usage in pages:
```svelte
<script lang="ts">
    import { createAuthState } from '$lib/stores/auth.svelte';
    const auth = createAuthState();
    // auth.user, auth.isAdmin, auth.isLoading — all reactive
</script>
```

### Files Changed

| File | Change |
|------|--------|
| `frontend/src/lib/stores/auth.svelte.ts` | New file |
| All route files using auth subscription | Migrate to `createAuthState()` |

---

## 6. Frontend Debounce Helper

### Problem

4+ pages implement identical debounce logic for search inputs.

### Design

Add to `$lib/utils.ts`:
```typescript
export function createDebouncedAction(fn: () => void, delay = 300) {
    let timeout: ReturnType<typeof setTimeout>;
    return {
        trigger() { clearTimeout(timeout); timeout = setTimeout(fn, delay); },
        cancel() { clearTimeout(timeout); }
    };
}
```

Usage:
```typescript
const debouncedSearch = createDebouncedAction(() => loadUsers(1));

function handleSearchInput(value: string) {
    searchQuery = value;
    debouncedSearch.trigger();
}
```

### Files Changed

| File | Change |
|------|--------|
| `frontend/src/lib/utils.ts` | Add `createDebouncedAction()` |
| `frontend/src/lib/index.ts` | Re-export |
| Pages with debounced search | Migrate to helper |

---

## Verification

After all changes:

1. `cargo clippy -- -D warnings` passes
2. `pnpm run check` passes (lint + type-check)
3. `pnpm --prefix frontend run test:e2e` passes (all existing E2E tests)
4. No user-facing behavior changes
