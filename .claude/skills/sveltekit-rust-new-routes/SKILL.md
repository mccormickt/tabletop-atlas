---
name: sveltekit-rust-new-routes
description: |
  Fix 404 errors when adding new SvelteKit routes to tabletop-atlas. Use when:
  (1) New route shows "404 Not Found" after creating +page.svelte,
  (2) SvelteKit console shows "Not found: /path/to/route",
  (3) Build fails with "routes were marked as prerenderable, but were not prerendered",
  (4) Dynamic [id] routes return 404 in production build.
  Covers the SvelteKit prerender settings and Rust backend rebuild requirements.
author: Claude Code
version: 1.0.0
date: 2026-01-21
---

# Adding New Routes to SvelteKit + Rust Backend

## Problem
New SvelteKit routes return 404 even after creating the `+page.svelte` file. This happens because:
1. Dynamic routes need `prerender = false` to avoid build errors
2. The Rust backend embeds the frontend at compile time via `include_dir!`

## Context / Trigger Conditions
- Console shows: `Not found: /challenges/1/stats` (or similar path)
- Build fails with: `The following routes were marked as prerenderable, but were not prerendered because they were not found while crawling your app`
- New page works in `vite dev` but not when served by the Rust backend
- Route contains dynamic segments like `[id]`

## Solution

### Step 1: Create the route files
```
frontend/src/routes/your-route/+page.svelte  # Your component
frontend/src/routes/your-route/+page.ts      # Prerender config
```

### Step 2: Disable prerendering for dynamic routes
Create `+page.ts` alongside your `+page.svelte`:
```typescript
export const prerender = false;
```

### Step 3: Rebuild BOTH frontend and backend
```bash
# Option 1: Full rebuild
pnpm run build

# Option 2: Manual steps
cd frontend && pnpm run build
cd .. && cargo build -p backend
```

### Step 4: Restart the dev server
The backend must be restarted to load the new embedded frontend.

## Verification
1. Navigate to the new route in browser
2. Page should render without 404
3. Check that the route appears in `frontend/build/` directory

## Example
Adding `/challenges/[id]/stats` route:

1. Create `frontend/src/routes/challenges/[id]/stats/+page.svelte`
2. Create `frontend/src/routes/challenges/[id]/stats/+page.ts`:
   ```typescript
   export const prerender = false;
   ```
3. Run `pnpm run build`
4. Restart the backend server

## Notes
- The backend uses `include_dir!("$CARGO_MANIFEST_DIR/../frontend/build")` to embed assets at compile time
- Static routes (no `[id]` segments) may work without `+page.ts` if they can be crawled during prerender
- The backend already has wildcard handlers for `/challenges/{path:.*}`, `/games/{path:.*}`, etc. in `static_files.rs`
- Check existing sibling routes for the pattern - if `[id]/+page.ts` exists, child routes need their own

## Related
- Backend static file handling: `backend/src/handlers/static_files.rs`
- SvelteKit adapter config: `frontend/svelte.config.js`
