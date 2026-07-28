---
name: adding-sveltekit-routes
description: "Adds and debugs SvelteKit routes served by the embedded Rust frontend. Use when a new route fails prerendering, works in Vite but returns 404 from Dropshot, or contains a dynamic segment."
---

# Adding SvelteKit routes

Account for both SvelteKit's static build and Dropshot's explicit SPA route handlers.

## Workflow

1. Create the route under `frontend/src/routes/` and inspect sibling route configuration.
2. If the route contains a dynamic segment such as `[id]`, add a sibling `+page.ts`:

   ```typescript
   export const prerender = false;
   ```

   The root `frontend/src/routes/+layout.ts` enables prerendering. SvelteKit cannot enumerate arbitrary parameter values, so dynamic pages in this repository opt out explicitly.

3. Check whether `backend/src/handlers/static_files.rs` already serves the route's top-level prefix. Existing wildcard handlers cover `/games`, `/search`, `/upload`, `/chat`, `/auth`, `/challenges`, `/tools`, and `/admin`; `/collection` and `/` have exact handlers.
4. For a new top-level prefix, add a matching SPA handler in `static_files.rs` and register it in `create_api_description()` in `backend/src/main.rs`. A Svelte route alone is not reachable from the embedded frontend when Dropshot has no matching endpoint.
5. Build from the repository root:

   ```bash
   pnpm --prefix frontend run build
   cargo build -p backend
   ```

   Alternatively, `pnpm run build` performs the release Rust build, whose build script builds the frontend. The backend embeds `frontend/build` at compile time, so rebuild and restart it after frontend changes.

## Verification

- Run `pnpm run check:frontend`.
- Confirm `pnpm --prefix frontend run build` succeeds without a prerender error.
- Exercise the route in Vite for frontend behavior.
- Exercise the rebuilt Rust server for embedded routing, especially when adding a new top-level prefix.

Do not expect a `frontend/build/<route>/index.html` file for a page with `prerender = false`; it is served through the static adapter's SPA fallback.
