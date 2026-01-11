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

## Development Workflow

- Both frontend and backend can be developed simultaneously
- API changes should include OpenAPI schema updates
- shadcn components can be added via `npx shadcn@latest add [component]`
- Database schema changes require new migration files
- Use JJ (Jujutsu) for all version control operations instead of Git
