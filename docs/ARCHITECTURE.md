# Architecture

Status: Milestone 0 proposal. Interfaces are subject to contract validation in
their implementation milestones.

QuireForge is an unofficial native Linux workspace for Codex. It is not made,
endorsed, supported, or distributed by OpenAI.

## Goals

- Provide a native Linux graphical client around supported Codex interfaces.
- Work in user-selected directories in place.
- Keep Codex authoritative for sessions and integration state, Git authoritative
  for repositories, and application SQLite authoritative only for app metadata.
- Degrade honestly when capabilities are absent, experimental, or policy-blocked.
- Make security boundaries visible in the product rather than hiding them in a
  generic permissions toggle.

## Product identity and owned paths

The permanent display identity is **QuireForge**, with the tagline “Build
boldly. Work locally.” Technical components use the identity map in
[ADR 0003](DECISIONS/0003-permanent-quireforge-identity.md).

Application-owned configuration, data, cache, and state follow the XDG base
directories with the `quireforge` leaf name. The application must resolve XDG
environment overrides through platform APIs rather than hardcoding `$HOME`.
The documented home-relative paths describe defaults, not an instruction to
ignore `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_CACHE_HOME`, or
`XDG_STATE_HOME`.

Codex remains the owner of Codex authentication, configuration, integrations,
and session data. A QuireForge rename or uninstall must not move, copy, or
delete those locations.

## Process layout

```text
React UI
  │ typed Tauri commands + normalized events
  ▼
Rust application core
  ├── project/directory services ── SQLite metadata
  ├── Git and PTY services ──────── git / shell processes
  ├── Codex compatibility layer ─── app-server stdio
  │                              └─ CLI JSON fallbacks
  └── integration services ──────── supported Codex interfaces only
```

The frontend never executes arbitrary shell commands or consumes raw Codex
protocol messages. Tauri capabilities expose a small, typed command surface.

## Application layers

### Frontend

React and TypeScript render semantic, keyboard-accessible views. State is split
between persisted application state, short-lived UI state, and normalized
backend event streams. The frontend cannot directly open filesystem paths,
spawn processes, mutate Git, or edit Codex configuration.

### Native application core

Rust owns validation, persistence transactions, path identity, subprocesses,
redaction, and policy enforcement. Tokio manages concurrent Codex and terminal
processes. Long-running work is cancelable and keyed by stable IDs.

### Codex compatibility layer

`CodexBackend` is selected after runtime probing:

- `AppServerBackend`: JSONL over local stdio for rich interactive flows.
- `CliJsonBackend`: documented CLI commands with JSON output where available.
- `MockCodexBackend`: deterministic fixture-backed behavior.
- `UnavailableBackend`: structured diagnostics with no simulated success.

Each adapter declares method-level maturity and version constraints. Generated
schemas and sanitized fixtures drive contract tests.

## Required service boundaries

- `CodexBackend`: versioned conversation/auth/integration contract.
- `CodexProcessManager`: child lifecycle, restart, cancellation, stderr, and
  health.
- `CodexEventNormalizer`: raw messages to stable domain events.
- `SessionRepository`: Codex thread references and app grouping metadata.
- `ProjectRepository`: project/settings persistence.
- `DirectoryAttachmentService`: attach, confirm, detach, and relink workflow.
- `DirectoryIdentityService`: selected/resolved paths, stat identity, mounts,
  accessibility, and change detection.
- `GitService`: repository/worktree discovery, status, diff, branch, and safe
  mutations.
- `TerminalService`: independent PTY sessions rooted in verified directories.
- `ApprovalService`: request correlation, scope, decision validation, expiry.
- `PreviewService`: bounded MIME/type-aware previews.
- `SettingsService`: application settings without secret ownership.
- `CapabilityService`: Codex/runtime/version/policy capability map.
- `IntegrationCatalogService`: normalized category-preserving catalog.
- `PluginMarketplaceService`: source inspection and supported marketplace calls.
- `PluginInstallationService`: preview, confirmation, lifecycle, and result
  verification.
- `ConnectorAuthorizationService`: official URL handoff and status refresh.
- `McpServerService`: configuration/status/OAuth/health adapter.
- `IntegrationPolicyService`: workspace/admin and requested-permission rules.
- `IntegrationHealthService`: non-destructive status checks.

## Project and directory data model

Identifiers are UUIDv7 or equivalent stable, opaque IDs. Directory names and
paths are never database keys.

### `projects`

| Field | Purpose |
|---|---|
| `id` | Stable application project ID |
| `display_name` | User-facing name |
| `active_directory_association_id` | Explicit current working root |
| `archived_at` | Application organization only |
| `created_at`, `updated_at` | Metadata timestamps |

### `directory_associations`

| Field | Purpose |
|---|---|
| `id` | Stable association ID |
| `project_id` | Owning project |
| `selected_path` | Exact absolute path selected by the user |
| `resolved_path` | Last verified resolved absolute path |
| `display_path` | Home-relative presentation when appropriate |
| `role` | Primary, additional writable, read-only context, etc. |
| `is_primary` | Primary working-directory flag |
| `expected_access` | Read/write expectation |
| `device_id`, `inode` | Local identity evidence where supported |
| `filesystem_type`, `mount_id` | Mount/removable/network evidence |
| `git_common_dir`, `git_worktree_root` | Git/worktree identity |
| `last_verified_at` | Last complete verification |
| `accessibility_state` | Explicit state enum |

### Supporting records

- `project_settings`: model preference, sandbox/approval defaults, terminal and
  editor preferences; no secrets.
- `project_threads`: project ID to authoritative Codex thread ID and cwd
  association.
- `integration_references`: normalized identifiers, scope, and display cache;
  Codex remains authoritative for installed/auth state.
- `terminal_sessions`: recoverable presentation/process metadata, never shell
  history containing secrets by default.
- `schema_migrations`: ordered, transactional migration history.

The schema supports multiple roots from the beginning even if the first UI
offers only one primary root.

## Directory attachment lifecycle

```text
Native picker
  → lexical + resolved path validation
  → access, mount, symlink, Git/worktree, AGENTS.md, .codex, marketplace/MCP scan
  → confirmation preview
  → transactional association save
  → project workspace
```

Before every Codex turn or terminal launch:

1. Reload the association by stable ID.
2. Verify the selected path still exists and is a directory.
3. Resolve it and compare identity evidence.
4. Re-evaluate accessibility and mount state.
5. Refresh Git branch and dirty state.
6. Build the exact sandbox and writable-root policy.
7. Confirm the thread/project/cwd association matches.
8. Fail closed if any invariant is uncertain.

No fallback to the app directory, home, previous project, or similarly named
directory is permitted.

## Directory state model

- `connected_accessible`
- `connected_read_only`
- `missing_or_moved`
- `permission_denied`
- `removable_disconnected`
- `network_unavailable`
- `git_invalid`
- `sandbox_restricted`
- `identity_changed`
- `verification_unknown`

State transitions retain projects and Codex thread references. Relinking is a
new verification followed by an explicit association update.

## Event model

Normalized events include:

- Thread/turn lifecycle.
- Commentary and final messages.
- Commands and output deltas.
- File changes and diff updates.
- Plans and usage.
- Approval requests and resolutions.
- MCP/app tool calls and health changes.
- Authentication and policy changes.
- Adapter warnings, version mismatches, and unsupported capabilities.

Each event carries an application correlation ID, Codex thread/turn/item IDs
when available, timestamp, maturity source, and redaction status.

## Integration domain model

`IntegrationDescriptor` preserves category rather than flattening all tools:

- App/Connector
- Plugin
- Skill
- MCP Server
- Marketplace

It also represents publisher/source, version, install/update/auth/health state,
scope, compatibility evidence, policy state, requested permissions, bundled
components, external domains, and documentation/source links when supplied.

Unknown fields stay unknown. Missing metadata does not become a compatibility
or verification claim.

## Integrated terminal

The terminal uses a dedicated Rust PTY service, not TUI scraping and not the
experimental app-server `process/*` API. It starts only after the directory
preflight and inherits a controlled environment. GUI-launched Linux apps often
do not inherit shell dotfile `PATH`; Codex discovery and shell environment
handling must therefore use explicit, documented resolution rules.

## Git and worktrees

Git commands use argv arrays, explicit cwd, bounded output, and no shell
interpolation. Read operations are automatic. Mutating operations identify the
repository/worktree and preview effects. Destructive cleanup is separate from
closing/removing an application project.

Codex-managed sessions and user worktrees are never removed as a side effect of
detaching a directory or deleting app metadata.

## Website

The Astro static site is isolated under `apps/website`. It has no runtime
backend and receives only version-controlled public content. Screenshots and
compatibility data are curated release assets; local project, connector, and
account data never enter the site build.

Production is `https://quireforge.jamesjennison.net` on Cloudflare Pages.
GitHub owns source, validation, issues, and release binaries; GitHub Pages
remains disabled. Cloudflare is authoritative DNS while A2 retains the
main-site and mail origins unless separately changed. The deployment adapter
builds a static artifact, creates isolated previews, applies version-controlled
headers/redirects, and promotes only an approved production-branch deployment.
DNS cutover is independently approval-gated and recoverable.

## Testing seams

- Adapter contracts from generated schemas and sanitized fixtures.
- In-memory/mock process transport for event streams and approvals.
- Temporary Git repositories/worktrees.
- Temporary path fixtures for symlinks, mount-like state, missing paths, and
  permissions.
- SQLite migrations against temporary databases.
- Fake OAuth/browser handoff and policy results.
- Static-site production-origin builds and link/accessibility checks.

Most tests require neither model calls nor third-party authorization.

## Open architecture decisions

- Oldest supported Ubuntu packaging baseline.
- SQLite crate and migration mechanism.
- PTY implementation and terminal renderer.
- Exact frontend state/query libraries.
- Whether repository-scoped integration settings should be edited directly or
  only through Codex-supported configuration RPCs.
- Cloudflare Git integration versus protected GitHub Actions direct upload,
  pending project-level permission review.
- Functional validation that the selected Tauri, GTK, desktop-entry, D-Bus,
  and packaging toolchain versions preserve the canonical application ID and
  reverse-DNS desktop filename.
