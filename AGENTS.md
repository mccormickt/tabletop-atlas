# AGENTS.md

This file provides guidance to coding agents working in this repository.

## Project Architecture

Tabletop Atlas is a board game rules management platform with AI-powered chat, built with:

- **Backend**: Rust with Dropshot web framework, SQLite database
- **Framework**: Svelte 5 using sveltekit with svelte-shadcn/ui components
- **Structure**: pnpm workspace monorepo (`pnpm-workspace.yaml` declares `frontend` as a workspace package)

### Key Directories

- `backend/`: Dropshot API server with handlers, models, and database layer as well as serving the frontend's static assets
- `frontend/`: Svelte 5 app
- `migrations/`: SQLite database migrations
- `docs/`: Feature architecture reference — see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- Root workspace manages both projects

## Development Commands

**IMPORTANT**: All `pnpm run` commands must be run from the **project root directory** (`/home/jan0ski/git/tabletop-atlas`), not from subdirectories like `frontend/` or `backend/`. The root `package.json` defines scripts that delegate to the appropriate workspace.

```bash
# Start both backend and frontend in development mode
pnpm run dev

# Run backend only
pnpm run backend
# or
cargo run -p backend

# Run frontend only
pnpm run frontend
# or
pnpm --prefix frontend run dev

# Build everything
pnpm run build

# Run tests (unit + e2e)
pnpm run test

# Run E2E tests only
pnpm --prefix frontend run test:e2e

# Run E2E tests for a specific file
pnpm --prefix frontend exec playwright test auth.test.ts

# Run E2E tests for a specific project (desktop/mobile)
pnpm --prefix frontend exec playwright test --project=desktop-chrome
pnpm --prefix frontend exec playwright test --project=mobile-chrome

# Install Playwright browsers (first time or after upgrade)
pnpm --prefix frontend run test:e2e:install

# Lint all code (backend + frontend)
pnpm run lint

# Lint backend only (Rust clippy)
pnpm run lint:backend

# Lint frontend only (ESLint + Prettier)
pnpm run lint:frontend

# Format all code
pnpm run format

# Format backend only (cargo fmt)
pnpm run format:backend

# Format frontend only (Prettier)
pnpm run format:frontend

# Check formatting without making changes
pnpm run format:check

# Type-check frontend (svelte-check)
pnpm run check:frontend

# Lint + type-check everything
pnpm run check

# Regenerate OpenAPI spec and TypeScript client after backend API changes
pnpm run generate
```

## Linting

### Backend (Rust)

- Uses **Clippy** for Rust linting with warnings treated as errors (`-D warnings`)
- Uses **cargo fmt** for code formatting
- Run `cargo clippy -- -D warnings` to check for issues (matches CI behavior)
- Run `cargo fmt` to format code

### Frontend (TypeScript/Svelte)

- Uses **ESLint** with TypeScript and Svelte plugins
- Uses **Prettier** for code formatting
- Configuration in `frontend/eslint.config.js` and `frontend/.prettierrc`
- The `src/api/` directory is excluded from linting (auto-generated code)
- Run `pnpm run lint:frontend` to check
- Run `pnpm run format:frontend` to fix formatting

### Svelte-specific Linting Notes

- All `{#each}` blocks should include a key: `{#each items as item (item.id)}`
- Avoid `any` types - use proper TypeScript types or `unknown`
- When using `{@html}`, add eslint-disable comment if the content is safe:
  ```svelte
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  {@html safeContent}
  ```
- The ESLint rule `svelte/no-navigation-without-resolve` is disabled globally (redundant with svelte-check typed routes). `resolve()` usage is enforced at the type level instead.

## Type Checking

### Frontend (svelte-check)

- Uses **svelte-check** with TypeScript for full type validation of Svelte components
- Run `pnpm run check:frontend` to check for type errors (matches CI behavior)
- Run `pnpm run check` to lint + type-check everything
- This catches errors that ESLint does not: missing imports, wrong prop types, invalid typed routes, etc.
- CI runs `pnpm --prefix frontend run check` which invokes `svelte-kit sync && svelte-check --tsconfig ./tsconfig.json`
- **Always run after making significant changes** — IDE may not catch all Svelte-specific type errors

### Common svelte-check Error Patterns

- **`resolve()` typed routes**: SvelteKit enforces typed route strings. Dynamic query params must be appended separately:
  ```typescript
  // Good
  goto(resolve('/chat') + `?game_id=${id}`);
  // Bad — typed routes don't accept query strings
  goto(resolve(`/chat?game_id=${id}`));
  ```
- **Missing type exports from `$lib`**: All API types used in components must be re-exported from `frontend/src/lib/index.ts`
- **`$derived` vs `$derived.by`**: Use `$derived.by()` when the derivation needs a function body (statements), use `$derived()` for simple expressions
- **Svelte 5 event handling**: Use callback props (`onUploaded`, `onDeleted`) not Svelte 4 `on:event` syntax
- **ErrorResult type**: Access HTTP status via `result.response.status`, not `result.statusCode`

## Backend Architecture

- **Framework**: Dropshot for type-safe HTTP APIs with automatic OpenAPI generation
- **Database**: SQLite with rusqlite and rusqlite_migration for schema management
- **Structure**: Handlers organized by resource, separate models and database layers
- **Key files**:
  - `backend/src/main.rs`: Server startup and configuration
  - `backend/src/handlers/`: API endpoint implementations
  - `backend/src/models/`: Data structures and validation
  - `backend/src/db/`: Database connection and query logic
  - `backend/src/llm.rs`: LLM client (gpt-oss via Ollama)
  - `backend/src/embeddings.rs`: Embedding generation (nomic-embed-text)
  - `backend/src/pdf.rs`: PDF text extraction and chunking
  - `backend/src/bgg.rs`: BoardGameGeek API client
  - `backend/src/auth/`: OAuth/OIDC authentication module
  - `backend/src/tools/`: Scoring calculators (7 Wonders, Carcassonne)

## Frontend Architecture

- **Framework**: Svelte 5 using sveltekit
- **Components**: huntabyte/shadcn-svelte design system with Tailwind CSS
- **Key files**:
  - `frontend/src/routes/`: Route component implementations
  - `frontend/src/lib/components/`: Shared Svelte components
  - `frontend/src/lib/`: Utilities, API client, and stores

### Svelte 5 Component Patterns

#### Presentational vs Container Components

**Reusable components in `lib/components/` should be presentational:**
- Receive data via props, not fetch their own data
- Use callback props (`onSaved`, `onDelete`, `onPageChange`) instead of `createEventDispatcher`
- Keep only UI-related state local (form visibility, editing state, etc.)
- Let parent pages/components manage data fetching and state

**Good example (presentational):**
```typescript
let {
  items,
  isLoading,
  error,
  onDelete,
  onSaved
}: {
  items: Item[];
  isLoading?: boolean;
  error?: string | null;
  onDelete?: (item: Item) => Promise<void>;
  onSaved?: (item: Item) => void;
} = $props();

// Only UI state is local
let showForm = $state(false);
let editingItem = $state<Item | null>(null);
```

**Avoid (container component anti-pattern):**
```typescript
let { itemId }: { itemId: number } = $props();

// Don't fetch data internally in reusable components
let items = $state<Item[]>([]);
let isLoading = $state(true);
let error = $state<string | null>(null);

$effect(() => {
  fetchItems(); // Anti-pattern for lib/components
});
```

**Page components (`routes/`) are expected to:**
- Manage data fetching and state
- Pass data down to presentational components
- Handle callbacks from child components

#### Svelte 5 Runes Best Practices

1. **Use `$effect` instead of `onMount`** for initialization:
   ```typescript
   let initialized = $state(false);

   $effect(() => {
     if (!initialized) {
       initialized = true;
       initialize();
     }
   });
   ```

2. **Use callback props instead of `createEventDispatcher`:**
   ```typescript
   // Good
   let { onSaved }: { onSaved?: (item: Item) => void } = $props();
   onSaved?.(item);

   // Avoid
   const dispatch = createEventDispatcher();
   dispatch('saved', item);
   ```

3. **Use `$effect` with cleanup for subscriptions:**
   ```typescript
   $effect(() => {
     const unsubscribe = store.subscribe((value) => {
       // handle value
     });
     return unsubscribe;
   });
   ```

4. **Use `$lib` utilities instead of local functions:**
   - Import `formatDate`, `formatDateTime` from `$lib`
   - Avoid duplicating utility functions in components

5. **Consolidate related `$effect` blocks** when they have the same dependencies

### Search and Filtering Patterns

**CRITICAL: Always use backend filtering, never client-side filtering for paginated data.**

Client-side filtering on paginated data is a recurring bug pattern in this codebase. When you filter data client-side after fetching a page, users only see filtered results from that page—items matching the filter on other pages are invisible.

**Anti-pattern (DO NOT DO THIS):**
```typescript
// BAD: Fetches page 1, then filters client-side
const result = await api.methods.listGames({ query: { page: 1, limit: 50 } });
const filtered = result.data.items.filter(g => g.hasRulesPdf); // Games with PDFs on page 2+ are lost!
```

**Correct pattern:**
```typescript
// GOOD: Filter on the backend using API parameters
const result = await api.methods.listGames({
  query: {
    page: 1,
    limit: 50,
    hasRulesPdf: true  // Backend filter parameter
  }
});
```

**Search implementation pattern for game lists:**
1. Add debounced search input (300ms delay)
2. Call backend API with `search` parameter on input change
3. Use backend filter parameters for default filters (e.g., `hasRulesPdf: true`)
4. When user searches, relax filters to show broader results (e.g., `hasRulesPdf: undefined`)

**Example: Game selector with search and PDF filter:**
```typescript
async function loadGames(search?: string) {
  // When searching, show all games; otherwise only show games with PDFs
  const hasRulesPdf = search ? undefined : true;

  const result = await api.methods.listGames({
    query: {
      page: 1,
      limit: 50,
      search: search || undefined,
      hasRulesPdf
    }
  });
  // ... handle result
}
```

**When adding new filters:**
1. Add the filter parameter to the backend API (`GameSearchParams` struct)
2. Update the database query to handle the filter
3. Regenerate the API client (`pnpm run generate`)
4. Use the filter parameter in frontend API calls

## Feature Areas

The application covers: games & collections, PDF rules & embeddings, AI chat (RAG), rules search, authentication (OIDC/JWT), challenges (multiplayer grids), tools & scoring calculators, admin (BGG import/enrichment), and house rules. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for detailed API endpoints, file mappings, and configuration values for each area.

## E2E Testing

The frontend has a comprehensive Playwright E2E test suite covering all pages in both desktop and mobile viewports.

### Architecture

- **Test framework**: Playwright with custom fixtures (`frontend/e2e/fixtures.ts`)
- **API mocking**: Single `page.route('**/api/**')` catch-all handler that dispatches by pathname and HTTP method
- **Mock data**: `frontend/src/mocks/data.ts` — all mock API responses (snake_case matching backend format, camelCase for tools endpoints)
- **MSW handlers**: `frontend/src/mocks/handlers.ts` — MSW handler definitions (used in dev, not in E2E tests)
- **Per-test overrides**: `overrideHandler()` in `frontend/e2e/helpers.ts` pushes to a shared override array checked before defaults
- **Projects**: Desktop Chrome + Mobile Chrome (Pixel 5) — 63 tests per project

### Key Files

- `frontend/e2e/fixtures.ts` — Playwright fixture with catch-all API route handler + override mechanism
- `frontend/e2e/helpers.ts` — `overrideHandler()`, `setupUnauthenticated()`, `setupAdmin()`
- `frontend/src/mocks/data.ts` — Mock data for all API endpoints
- `frontend/playwright.config.ts` — Playwright config (both desktop and mobile projects)

### Writing New E2E Tests

1. Import `{ test, expect }` from `'./fixtures'` (not from `@playwright/test`)
2. Default state: authenticated user with populated mock data
3. Override endpoints per-test using `overrideHandler(page, method, path, { status?, body })`
4. For unauthenticated tests: call `setupUnauthenticated(page)` before `page.goto()`
5. For admin tests: call `setupAdmin(page)` before `page.goto()`
6. Handle mobile viewports: use `page.viewportSize().width < 768` for responsive checks

### Adding New API Endpoints to Mocks

1. Add mock data to `frontend/src/mocks/data.ts`
2. Add route handler in `frontend/e2e/fixtures.ts` (in the `setupDefaultRoutes` catch-all)
3. Add MSW handler in `frontend/src/mocks/handlers.ts` (for dev mode consistency)

## Database

- SQLite with sqlite-vec extension for vector similarity search
- Migrations in `migrations/` directory
- Migration files follow `V001__description.sql` pattern
- Database initialization and migration running handled in backend startup

## Version Control: Jujutsu (JJ)

This project uses **Jujutsu (JJ)** for version control instead of Git. JJ is a Git-compatible VCS that provides a simpler and more powerful workflow.

### Common JJ Commands

#### Repository & Remote Operations
```bash
# Initialize a JJ repository with Git backend
jj git init --colocate

# Clone a repository
jj git clone <source>

# Fetch from remote
jj git fetch

# Push all bookmarks (branches)
jj git push --all
```

#### Daily Workflow
```bash
# Check status
jj st
# or
jj status

# View log/history
jj log

# Show diff of current change
jj diff

# Describe (add commit message to) current change
jj describe -m "Your message here"

# Create a new change (like committing in git)
jj new

# Commit with message and start new change
jj commit -m "Your message here"
```

#### Branching (Bookmarks in JJ)
```bash
# Create a bookmark (branch) at current revision
jj bookmark set my-branch

# Create bookmark at specific revision
jj bookmark set -r @ 'feat/branch'

# Push to remote with new bookmark
jj git push -r @ --allow-new --remote origin

# Track a remote bookmark
jj bookmark track main@origin
```

#### GitHub CLI (gh) with JJ

When using `gh` commands with JJ, note that JJ doesn't maintain a traditional Git branch checkout. This causes issues with commands like `gh pr create` which expect to detect the current branch automatically.

**Problem:** `gh pr create` fails with "not on any branch" error.

**Solution:** Always specify `--head` and `--base` explicitly:
```bash
# Instead of just: gh pr create
# Use:
gh pr create --head <bookmark-name> --base main --title "..." --body "..."
```

**Full PR workflow with JJ:**
```bash
# 1. Describe your change
jj describe -m "feat: your feature description"

# 2. Create a bookmark for the change
jj bookmark set -r @ 'feat/your-feature'

# 3. Push to remote
jj git push -r @ --allow-new --remote origin

# 4. Create PR with explicit head/base
gh pr create --head feat/your-feature --base main --title "feat: your feature" --body "..."
```

#### Advanced Operations
```bash
# Undo last jj command
jj undo

# Split current change into two
jj split

# Squash changes together
jj squash
jj squash file  # squash specific file
jj squash -i    # interactive selection

# Rebase changes
jj rebase -s <source> -d <destination>

# View operation history
jj op log

# Edit a previous change
jj edit <change-id>

# Annotate file (like git blame)
jj file annotate <filename>
```

### Git to JJ Command Reference

| Git Command | JJ Equivalent | Notes |
|-------------|---------------|-------|
| `git status` | `jj st` | Show working copy status |
| `git log` | `jj log` | View history |
| `git diff` | `jj diff` | Show changes |
| `git commit -a` | `jj commit` | Commit changes |
| `git add -p; git commit` | `jj split` | Partial commits |
| `git checkout -b <branch>` | `jj bookmark create <branch>` | Create branch |
| `git switch <branch>` | `jj edit <change>` | Switch to change |
| `git rebase` | `jj rebase` | Rebase changes |
| `git cherry-pick` | `jj duplicate` | Copy changes |
| `git blame` | `jj file annotate` | Show line authors |
| `git fetch` | `jj git fetch` | Fetch from remote |
| `git push` | `jj git push` | Push to remote |
| `git restore` | `jj restore` | Restore files |
| N/A | `jj undo` | Undo last operation |
| N/A | `jj squash` | Combine changes |

### JJ Resources

- [Official Jujutsu Documentation](http://docs.jj-vcs.dev/latest/)
- [Git Command Comparison](http://docs.jj-vcs.dev/latest/git-comparison/)
- [Jujutsu Tutorial by Steve Klabnik](https://steveklabnik.github.io/jujutsu-tutorial/)
- [Comprehensive Cheat Sheet](https://www.rahuljuliato.com/posts/jj-cheat-sheet)

## API Client Generation

The frontend uses a generated TypeScript client to communicate with the backend API. When you make changes to backend API endpoints, you must regenerate the client.

```bash
pnpm run generate
```

This command:
1. Runs the backend with `--openapi` flag to output the OpenAPI spec to `openapi.json` (root directory)
2. Uses `@oxide/openapi-gen-ts` to generate TypeScript client files in `frontend/src/api/`

**Generated files** (do not edit manually):
- `openapi.json` - OpenAPI specification (root directory)
- `frontend/src/api/Api.ts` - Generated API client class
- `frontend/src/api/http-client.ts` - HTTP client utilities
- `frontend/src/api/util.ts` - Utility functions

**Important**: Only the root `openapi.json` is the source of truth. Do not commit any `openapi.json` to `frontend/src/api/`.

## Development Workflow

- Both frontend and backend can be developed simultaneously
- After backend API changes, run `pnpm run generate` to update the TypeScript client
- shadcn components can be added via `npx shadcn@latest add [component]`
- Database schema changes require new migration files
- Use JJ (Jujutsu) for all version control operations instead of Git

### Auto-Formatting

**Formatting is handled automatically.** A coding agent hook runs `cargo fmt` (Rust) and `prettier` (frontend) after each file edit. You don't need to run format commands manually during development.

### When to Run Lint/Check

**Before opening a PR**, run the full check to catch issues CI will flag:

```bash
pnpm run check  # Lint + type-check everything
```

This runs:
- `cargo clippy -- -D warnings` (catches logical issues, not just formatting)
- `eslint` (code quality beyond formatting)
- `svelte-check` (type errors IDE may miss)

**Don't run lint/format after every small change** — the auto-format hook handles formatting, and running full checks constantly is slow. Save the full `pnpm run check` for:
- Before creating a PR
- After significant refactoring
- When you suspect type issues
