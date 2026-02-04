# Tabletop Atlas

A board game rules management platform with AI-powered chat. Upload rulebooks, create house rules, track your collection, and ask questions about gameplay through an LLM-powered interface.

## Features

- **Rules Management** — Upload PDF rulebooks. Text is extracted, chunked, and embedded for semantic search.
- **AI Chat** — Ask questions about any game's rules. Answers are grounded in the actual rulebook via RAG (retrieval-augmented generation), with optional house rules context.
- **House Rules** — Create per-game house rules that are automatically embedded and available as chat context.
- **Game Library** — Browse a master catalog of games with data imported and enriched from BoardGameGeek.
- **Collections** — Track which games you own, are playing, or have traded.
- **Custom Games** — Define homebrew games with public or private visibility.
- **Challenges** — Create multiplayer challenge grids to track plays, record winners, and view leaderboards.
- **Scoring Tools** — Built-in scoring calculators for games like 7 Wonders and Carcassonne, with a plugin system for adding more.
- **Admin** — Import games from BGG CSV exports and enrich metadata from the BGG API.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust with [Dropshot](https://github.com/oxidecomputer/dropshot) |
| Frontend | [Svelte 5](https://svelte.dev/) with SvelteKit, [shadcn-svelte](https://www.shadcn-svelte.com/) |
| Database | SQLite + [sqlite-vec](https://github.com/asg017/sqlite-vec) for vector search |
| LLM | [Ollama](https://ollama.com/) (gpt-oss for chat, nomic-embed-text for embeddings) |
| Auth | OpenID Connect with JWT sessions |
| API Client | Auto-generated TypeScript client from OpenAPI spec |

## Prerequisites

- Rust toolchain (stable)
- Node.js + pnpm
- Ollama with models pulled:
  ```bash
  ollama pull gpt-oss:latest
  ollama pull nomic-embed-text:latest
  ```

## Getting Started

```bash
# Install dependencies
pnpm install

# Start both backend and frontend in dev mode
pnpm run dev
```

The backend serves on `http://localhost:4035` and proxies frontend dev requests.

## Development

```bash
pnpm run dev          # Start backend + frontend
pnpm run build        # Production build
pnpm run test         # Run tests
pnpm run lint         # Lint backend (clippy) + frontend (eslint)
pnpm run format       # Format backend (cargo fmt) + frontend (prettier)
pnpm run generate     # Regenerate OpenAPI spec + TypeScript client
```

All `pnpm run` commands should be run from the project root.

## Project Structure

```
backend/           Rust API server (Dropshot)
  src/
    handlers/      API endpoint implementations
    models/        Data structures
    db/            Database queries
    auth/          OIDC + JWT authentication
    tools/         Scoring calculators
    llm.rs         LLM client
    embeddings.rs  Embedding service
    pdf.rs         PDF extraction + chunking
    bgg.rs         BoardGameGeek API client
frontend/          Svelte 5 app (SvelteKit)
  src/
    routes/        Page components
    lib/
      components/  Reusable UI components
      api/         Auto-generated API client (do not edit)
migrations/        SQLite schema migrations (V001-V014)
docs/              Architecture documentation
```

## Documentation

- [CLAUDE.md](CLAUDE.md) — Development conventions and patterns for AI-assisted coding
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — Feature areas, API endpoints, database schema, and configuration reference
