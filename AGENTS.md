# Repository guidance

Tabletop Atlas is a board-game rules platform with a Rust/Dropshot backend and a SvelteKit 5 frontend. The backend stores data in SQLite, calls an Ollama-compatible API for chat and embeddings, and embeds the production frontend in the Rust binary.

## Repository map

- `backend/src/handlers/`: Dropshot endpoints
- `backend/src/models/`: API and domain types
- `backend/src/db/`: SQLite queries
- `backend/src/auth/`: OIDC and JWT support
- `backend/src/agents/`, `embeddings.rs`, `pdf.rs`: rules ingestion and AI flows
- `frontend/src/routes/`: SvelteKit pages
- `frontend/src/lib/components/`: reusable UI components
- `frontend/src/api/`: generated TypeScript API client; do not edit manually
- `frontend/e2e/`: Playwright tests and API fixtures
- `migrations/`: ordered SQLite migrations
- `docs/ARCHITECTURE.md`: feature-level architecture reference

The root is a pnpm workspace containing `frontend`. Run the documented commands from the repository root; use `pnpm --prefix frontend ...` for frontend-only scripts.

## Toolchain and local configuration

- CI uses Node.js 22, pnpm 9, and stable Rust with `rustfmt` and `clippy`.
- `.agents/setup` installs the orb toolchain, JavaScript dependencies, `cargo-watch`, and Playwright Chromium.
- `.env` is ignored. Start from `.env.example`; real Google OAuth, JWT, BGG, or Ollama credentials must not be committed.
- SQLite is embedded and migrations run at backend startup. The default database is `atlas.db` in the repository root.
- Ollama is not required for compilation or most tests. The server performs a best-effort connection check at startup; AI features require the configured models.

## Commands

| Task                                            | Command                   |
| ----------------------------------------------- | ------------------------- |
| Watched backend with embedded frontend rebuilds | `pnpm run dev`            |
| Backend only                                    | `pnpm run backend`        |
| Vite frontend only                              | `pnpm run frontend`       |
| Release build                                   | `pnpm run build`          |
| Rust + frontend unit and E2E tests              | `pnpm run test`           |
| Full lint and frontend type-check               | `pnpm run check`          |
| Backend lint                                    | `pnpm run lint:backend`   |
| Frontend Prettier check + ESLint                | `pnpm run lint:frontend`  |
| Frontend type-check                             | `pnpm run check:frontend` |
| Format Rust and frontend                        | `pnpm run format`         |
| Regenerate OpenAPI and TypeScript client        | `pnpm run generate`       |

Useful focused checks:

```bash
cargo fmt --all -- --check
NO_BUILD_FRONTEND=1 cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
pnpm --prefix frontend run test:unit -- --run
pnpm --prefix frontend exec playwright test auth.test.ts --project=desktop-chrome
```

`pnpm run format:check` also runs the frontend lint script, so it is broader than a formatting-only check. Formatting is not automatic: format touched files or run the appropriate format command when needed.

## Backend conventions

- Keep endpoint registration in `create_api_description()` in `backend/src/main.rs`; it is also the source for OpenAPI generation.
- Keep HTTP handling, models, and database operations in their existing layers rather than putting SQL in handlers.
- Add schema changes as a new `migrations/VNNN__description.sql` file. Do not rewrite existing migrations.
- The production frontend is built from `backend/build.rs` and embedded with `include_dir!`. Set `NO_BUILD_FRONTEND=1` for Rust-only checks that do not need fresh assets.
- Backend startup requires `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, and `JWT_SECRET`; the placeholders from `.env.example` are sufficient for compilation and non-OAuth smoke tests.

## Generated API client

After changing an endpoint path, request/response type, or OpenAPI-visible schema, run:

```bash
pnpm run generate
```

Commit the resulting root `openapi.json` and generated files under `frontend/src/api/`. Do not hand-edit `Api.ts`, `http-client.ts`, or `util.ts`. Frontend code normally imports the client instance, shared utilities, and re-exported API types through `frontend/src/lib/index.ts`; add a re-export there when a component needs a newly generated type.

## Frontend conventions

- Use Svelte 5 runes and callback props; follow nearby components rather than introducing Svelte 4 `createEventDispatcher` patterns.
- Keep reusable components primarily presentational. Route pages should generally own API loading and pass data and callbacks down.
- Reuse utilities such as `formatDate`, `formatDateTime`, `unwrapResult`, and `createDebouncedAction` from `$lib`.
- For generated API failures, HTTP status is at `result.response.status`.
- Use SvelteKit's `resolve()` for typed route paths. Append dynamic query strings after resolving the pathname, for example `resolve('/chat') + '?game_id=1'`.
- `frontend/src/api/**` and `frontend/static/**` are intentionally excluded from ESLint. Generated API files still need to type-check.
- Key `{#each}` blocks when items have stable identities. Avoid `any` when a precise or `unknown` type is practical.

### Paginated filtering

Never filter a fetched page client-side when the filter is meant to apply to the whole result set. Add or use a backend query parameter, apply it in the database query, regenerate the client, and pass it from the frontend. Client-side filtering is appropriate only for explicitly local, non-paginated data.

The game selector pattern in `frontend/src/routes/chat/+page.svelte` defaults to `hasRulesPdf: true` and removes that restriction while searching. Preserve that behavior unless the product requirement changes.

## SvelteKit routes and embedded assets

The root layout sets `prerender = true`, while parameterized pages opt out with their own `+page.ts`. The static adapter emits an SPA fallback, but Dropshot serves only registered route prefixes. Load the `adding-sveltekit-routes` skill when adding a route or diagnosing a route that works in Vite but returns 404 from the Rust backend.

## E2E and mocks

- Import `test` and `expect` from `frontend/e2e/fixtures`, not directly from Playwright.
- Tests default to an authenticated happy path and run in desktop Chrome and Pixel 5 projects.
- `frontend/e2e/fixtures.ts` owns the single `**/api/**` catch-all route and default responses.
- Use `overrideHandler()` from `frontend/e2e/helpers.ts` for per-test API behavior; later overrides win.
- Call `setupUnauthenticated()` or `setupAdmin()` before `page.goto()` when needed.
- Keep `frontend/src/mocks/data.ts`, the E2E fixture, and `frontend/src/mocks/handlers.ts` consistent when an endpoint should exist in both Playwright and MSW mocks.

## Verification

Choose checks by scope:

- Documentation-only edits: formatting or link/content inspection as appropriate.
- Rust-only edits: `cargo fmt --all -- --check` plus a focused test or clippy command.
- Frontend edits: `pnpm run check:frontend` and the narrowest relevant unit/E2E test; run `pnpm run lint:frontend` when formatting or lint-sensitive files changed.
- API contract edits: regenerate the client, then check both backend and frontend.
- Cross-cutting or release-ready changes: `pnpm run check` and relevant tests.

This checkout uses Git. Do not assume Jujutsu is installed or that a `.jj` workspace exists.
