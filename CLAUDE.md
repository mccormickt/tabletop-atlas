# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Architecture

This is a full-stack blog application inspired by the famous Ruby on Rails 15-minute blog demo, built with:

- **Backend**: Rust with Dropshot web framework, SQLite database
- **Framework**: Svelte 5 using sveltekit with svelte-shadcn/ui components
- **Structure**: Monorepo with workspace setup

### Key Directories

- `backend/`: Dropshot API server with handlers, models, and database layer as well as serving the frontend's static assets
- `frontend/`: Svelte 5 app
- `migrations/`: SQLite database migrations
- Root workspace manages both projects

## Development Commands

```bash
# Start both backend and frontend in development mode
npm run dev

# Run backend only
npm run backend
# or
cargo run -p backend

# Run frontend only
npm run frontend
# or (from frontend directory)
npm run dev

# Build everything
npm run build

# Run tests
npm run test
```

## Backend Architecture

- **Framework**: Dropshot for type-safe HTTP APIs with automatic OpenAPI generation
- **Database**: SQLite with rusqlite and rusqlite_migration for schema management
- **Structure**: Handlers organized by resource (posts, comments), separate models and database layers
- **Key files**:
  - `backend/src/main.rs`: Server startup and configuration
  - `backend/src/handlers/`: API endpoint implementations
  - `backend/src/models/`: Data structures and validation
  - `backend/src/db/`: Database connection and query logic

## Frontend Architecture

- **Framework**: Svelte 5 using sveltekit
- **Components**: huntabyte/shadcn-svelte design system with Tailwind CSS
- **Key files**:
  - `app/routes/`: Route component implementations
  - `app/components/`: Shared Svelte components
  - `app/lib/`: Utilities and API client

## Database

- SQLite with migrations in `migrations/` directory
- Migration files follow `V001__description.sql` pattern
- Database initialization and migration running handled in backend startup

## Development Workflow

- Both frontend and backend can be developed simultaneously
- API changes should include OpenAPI schema updates
- shadcn components can be added via `npx shadcn@latest add [component]`
- Database schema changes require new migration files
