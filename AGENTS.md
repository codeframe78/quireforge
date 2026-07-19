# QuireForge Repository Guidance

These instructions apply to the entire repository. A more deeply nested
`AGENTS.md` may add or narrow requirements for its subtree.

## Project state

QuireForge is an early-stage, unofficial native Linux workspace for Codex. The
repository currently contains discovery, architecture, governance, and vector
brand sources. Do not claim that an application, package, integration, website,
or release exists until its milestone has produced and verified it.

QuireForge is not made, endorsed, supported, or distributed by OpenAI. Preserve
accurate uses of the official names OpenAI, ChatGPT, Codex, official commands,
protocol fields, and third-party integration identifiers.

## Non-negotiable boundaries

- Work against attached directories in place; never substitute copied content.
- Never silently change a task's working directory or expand writable roots.
- Keep QuireForge metadata separate from Codex authentication, configuration,
  sessions, and connector credentials.
- Use documented Codex interfaces. Do not scrape the TUI or ChatGPT website,
  use private endpoints, capture browser tokens, or fabricate capabilities.
- Treat plugins, marketplaces, hooks, connectors, and MCP servers as
  supply-chain-sensitive.
- Keep detach/remove/archive operations separate from filesystem deletion.
- Never commit credentials, tokens, personal Codex data, hosting identifiers,
  private diagnostics, `.env` files, or generated support bundles.

## Change workflow

1. Read the relevant architecture decision, roadmap entry, and subsystem docs.
2. Preserve pre-existing changes and confirm the branch before editing.
3. Keep changes within the active milestone and use focused commits.
4. Update tests, README status, `docs/ROADMAP.md`, and `CHANGELOG.md` when a
   milestone changes user-visible behavior or project status.
5. Run `python3 scripts/validate_repository.py` and applicable subsystem checks.
6. Review staged content for secrets and unrelated changes before committing.

Do not push, merge, publish, deploy, install integrations, authorize
connectors, mutate GitHub/Cloudflare settings, or delete user data without the
specific approval required for that action.

## Implementation direction

- Desktop: Tauri 2, Rust, Tokio, React, TypeScript, and Vite.
- Website: Astro static output on Cloudflare Pages.
- Metadata: migrated SQLite owned by QuireForge; never a credential store.
- IPC: small typed commands and normalized events; the frontend must not consume
  raw Codex protocol messages or spawn arbitrary processes.
- Tests: deterministic mocks and sanitized fixtures; routine tests must not
  require billable model calls or real connector authorization.

Use `rg` for repository searches and `apply_patch` for deliberate file edits.
Avoid destructive Git recovery commands and broad cleanup operations.
