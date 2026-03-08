# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/mccormickt/tabletop-atlas/compare/v0.1.3...v0.1.4) - 2026-03-08

### Added

- add Ticket to Ride and Wingspan score calculators ([#51](https://github.com/mccormickt/tabletop-atlas/pull/51))
- migrate from async-openai to Rig framework ([#47](https://github.com/mccormickt/tabletop-atlas/pull/47))
- move PDF upload to admin panel and add auth guards ([#41](https://github.com/mccormickt/tabletop-atlas/pull/41))
- consolidate all env vars into clap CLI args ([#42](https://github.com/mccormickt/tabletop-atlas/pull/42))
- consolidate all env vars into clap CLI args ([#40](https://github.com/mccormickt/tabletop-atlas/pull/40))

### Other

- *(deps)* update typescript and cargo packages ([#50](https://github.com/mccormickt/tabletop-atlas/pull/50))

## [0.1.3](https://github.com/mccormickt/tabletop-atlas/compare/v0.1.2...v0.1.3) - 2026-02-11

### Added

- parallelize APK builds with native ARM64 runners ([#36](https://github.com/mccormickt/tabletop-atlas/pull/36))
- *(llm)* Use gpt-oss as the default model
- Enrich games with BGG API
- admin interface for game rules
- markdown formatting in chat
- add loading state and optimistic UI to chat feature
- add game tools feature with score calculators
- add 8x8 Challenge feature for tracking game sessions
- add Google OIDC authentication with multi-tenancy
- integrate house rules into chat context with toggle
- allow selection of listen addr and port
- chat with extracted game rules
- better pdf text chunking
- embeddings with ollama
- pdf uploads with fake embeddings
- add cli arg for bind address
- serve static assets and use buildscript to build frontend
- add frontend components and game create page
- CORS headers
- cors headers on responses

### Fixed

- correctly search using backend queries
- correct query string parsing for house rules
- Correct default games table name
- improve code quality and safety across backend and frontend
- *(challenges)* address security vulnerabilities and code quality issues
- *(auth)* address critical security issues in authentication flow
- *(chat)* remove theshhold on similarity search
- lower similarity to get more results
- properly search embeddings with sqlite-vec
- direct navigation for search and upload routes
- serve frontend routes properly to support direct links

### Other

- remove unused dependencies, integrate dropshot logger, and modernize deps ([#35](https://github.com/mccormickt/tabletop-atlas/pull/35))
- release v0.1.2 ([#27](https://github.com/mccormickt/tabletop-atlas/pull/27))
- release v0.1.1 ([#26](https://github.com/mccormickt/tabletop-atlas/pull/26))
- Add CI and migrate to pnpm workspace monorepo  ([#23](https://github.com/mccormickt/tabletop-atlas/pull/23))
- Replace 6 outdated AI-generated docs with consolidated ARCHITECTURE.md ([#22](https://github.com/mccormickt/tabletop-atlas/pull/22))
- *(games)* Clearly deliniate games, collections, and custom games
- change PUT update endpoints to PATCH for AIP-134 alignment
- regen api client
- fix clippy lints, add lint/format scripts, and remove dead code
- base search feature around keywords instead of natural language
- organize logic
- use shadcn/ui components and implement client-side navigation
- Initial app scaffolding

## [0.1.2](https://github.com/mccormickt/tabletop-atlas/compare/v0.1.1...v0.1.2) - 2026-02-05

### Other

- update Cargo.toml dependencies

## [0.1.1](https://github.com/mccormickt/tabletop-atlas/compare/v0.1.0...v0.1.1) - 2026-02-05

### Added

- *(llm)* Use gpt-oss as the default model
- Enrich games with BGG API
- admin interface for game rules
- markdown formatting in chat
- add loading state and optimistic UI to chat feature
- add game tools feature with score calculators
- add 8x8 Challenge feature for tracking game sessions
- add Google OIDC authentication with multi-tenancy
- integrate house rules into chat context with toggle
- allow selection of listen addr and port
- chat with extracted game rules
- better pdf text chunking
- embeddings with ollama
- pdf uploads with fake embeddings
- add cli arg for bind address
- serve static assets and use buildscript to build frontend
- add frontend components and game create page
- CORS headers
- cors headers on responses

### Fixed

- correctly search using backend queries
- correct query string parsing for house rules
- Correct default games table name
- improve code quality and safety across backend and frontend
- *(challenges)* address security vulnerabilities and code quality issues
- *(auth)* address critical security issues in authentication flow
- *(chat)* remove theshhold on similarity search
- lower similarity to get more results
- properly search embeddings with sqlite-vec
- direct navigation for search and upload routes
- serve frontend routes properly to support direct links

### Other

- Add CI and migrate to pnpm workspace monorepo  ([#23](https://github.com/mccormickt/tabletop-atlas/pull/23))
- Replace 6 outdated AI-generated docs with consolidated ARCHITECTURE.md ([#22](https://github.com/mccormickt/tabletop-atlas/pull/22))
- *(games)* Clearly deliniate games, collections, and custom games
- change PUT update endpoints to PATCH for AIP-134 alignment
- regen api client
- fix clippy lints, add lint/format scripts, and remove dead code
- base search feature around keywords instead of natural language
- organize logic
- use shadcn/ui components and implement client-side navigation
- Initial app scaffolding
