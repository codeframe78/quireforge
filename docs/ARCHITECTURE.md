# Architecture

Status: Milestone 0 application proposal with the Milestone 2 website foundation
and Milestones 3–7A desktop scaffold, Codex process adapter, authentication,
project attachment, and native conversation boundary implemented locally. Git,
terminal, conversation UI, and integration interfaces remain subject to
validation in their implementation milestones.

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

### Milestone 3 implementation boundary

The initial desktop package lives under `apps/desktop`. React and strict
TypeScript render the shell; Rust and Tauri own the native process. One command,
`desktop_bootstrap`, returns a versioned product/capability snapshot. The Rust
serializer and TypeScript runtime schema consume the same sanitized JSON
fixture in their contract tests.

The main-window capability grants no plugin permissions. No filesystem, shell,
opener, process, Codex, project, Git, database, or integration command exists in
the Milestone 3 scaffold. Browser preview mode reports that native IPC is
unavailable instead of simulating success. The window enables the accepted GTK
application ID, and a local GNOME Wayland launch verified ownership of
`io.github.codeframe78.QuireForge` on the session bus.

### Milestone 4 implementation boundary

The native core now exposes the fixed-purpose `codex_runtime_probe` command. It
accepts no user input, paths, commands, environment values, or credentials. A
serialized `CodexRuntimeService` caches one normalized result per application
run so React development effects cannot create duplicate app-server children.

The adapter invokes only `codex --version`, starts `codex app-server --listen
stdio://`, initializes with truthful QuireForge client metadata, and requests
`model/list`. JSONL messages are size- and time-bounded, responses are
correlated by numeric request ID, server requests fail closed, and unrelated
notification payloads are discarded. User-agent, Codex-home, account,
installation, remote-control, and raw error payloads are never retained or
returned to the frontend. Closing or killing the owned child is followed by a
wait, including timeout and early-exit paths.

Milestone 4 committed only initialize and `model/list` portions of the installed
CLI's generated schema with hashes; later milestones add reviewed account and
conversation subsets. Runtime models and reasoning efforts always come from the
live supported catalog; sanitized fixtures are deterministic test contracts,
not hardcoded production state. The Milestone 4 probe itself still excludes
threads, turns, approvals, project working directories, Codex configuration
writes, and session persistence.

### Milestone 5 implementation boundary

The native core adds fixed-purpose account status/refresh, browser or device
login start, active-login cancellation, explicit logout, and native browser
handoff commands. Only login method is accepted as frontend input; it is a
closed enum. The browser command accepts no URL from React and opens only the
current native-owned handoff after Rust and TypeScript independently require
HTTPS, no embedded credentials, bounded length, and an OpenAI or ChatGPT host.
The Tauri main-window capability still grants no direct plugin permission.

`CodexAuthService` caches normalized status to prevent duplicate development
probes. A pending login moves its app-server process into one background owner
task, which correlates the exact bounded `loginId`, handles account/completion
notifications, serializes cancellation, clears handoff data at terminal state,
and shuts down and waits for the child on every ordinary path. Dropping the
service closes the control channel so the owner also performs shutdown; process
drop retains kill-on-drop as a final fallback.

Account email, plan, IDs, raw RPC errors, tokens, API keys, and Codex credential
storage never enter the normalized snapshot. A browser/device URL and optional
one-time code exist only in the in-memory pending snapshot because they are
required for user handoff; they disappear after completion or cancellation.
Logout requires a second explicit UI action. QuireForge does not read Codex
credential files or browser storage and creates no authentication database.

### Milestone 6 implementation boundary

The native `ProjectService` owns migrated QuireForge SQLite metadata and native
directory selection. Attachment confirmation binds selected and resolved path
identity, mount and accessibility evidence, Git/worktree identity, and detected
project instructions. Every execution preflight reloads and revalidates that
evidence; uncertainty fails closed without falling back to another directory.
Detach, archive, and relink mutate only application metadata and never delete
source content or Codex-owned state.

### Milestone 7A implementation boundary

`ConversationService` is the serialized native owner of one MVP conversation.
It reserves the project against detach, archive, and relink races, revalidates
the attached directory immediately before execution, discovers the live model
catalog on the owned app-server process, and sends fixed `thread/start` and
`turn/start` requests with the exact verified cwd and explicit model, reasoning,
sandbox, and approval settings.

The native boundary correlates UUIDv7 thread and turn IDs, emits only bounded
normalized lifecycle, agent-message, reasoning-summary, plan, coarse-activity,
and stable-error events, and rejects raw protocol fields or mismatched stream
identities. `turn/interrupt` uses only the native-owned thread and turn IDs.
Approval server requests are not guessed or auto-approved: the conversation
enters a blocked state and its child is closed and waited. SQLite stores only
Codex reference IDs, selected controls, and lifecycle status—never prompts,
message text, reasoning, command output, diffs, or credentials. The strict
frontend contract can start, poll, inspect, and interrupt by an application ID;
the user-facing conversation view remains Milestone 7B.

### Milestone 8A implementation boundary

The same serialized `ConversationService` owns session lifecycle mutations so
resume, fork, archive, restore, and reconciliation cannot race an active turn.
The frontend supplies only an app conversation UUIDv7 and, for resume/fork, a
bounded prompt. Native code reloads the reference-only row, revalidates the
attached directory and exact cwd, reads the owned Codex thread, and invokes the
reviewed lifecycle method with stored controls. Rollout paths, provided history,
configuration objects, runtime workspace roots, Codex IDs, and cwd values never
cross the webview boundary.

Codex remains authoritative. Bounded `thread/list` requests batch exact verified
cwd filters for current and archived threads, then match only IDs already owned
by QuireForge; unrelated threads are not imported. A missing thread becomes a
stable normalized state. SQLite schema version 3 stores only optional parent app
lineage and an archive timestamp in addition to the existing reference fields.
Database open converts stale starting/running/stopping rows to interrupted and
clears active-turn IDs because subprocess ownership cannot survive a restart.
No lifecycle command deletes source content, Codex history, or a QuireForge
reference.

### Milestone 8B presentation boundary

Title search cannot replace complete reconciliation. Native code first obtains
the bounded current/archived authoritative set for exact verified cwds, then
runs an optional bounded `searchTerm` projection on the same supervised
process. Filtered IDs must exist in the complete set and are intersected with
QuireForge-owned references. Only a normalized optional title of at most 256
characters crosses IPC; it is never stored in SQLite. Previews, transcripts,
paths, Codex IDs, and raw thread records remain native-only.

React groups summaries with app-owned project and parent references. Open tabs,
selection, and unsent continuation prompts are temporary UI state. Resume,
fork, archive, and restore continue to use fixed IPC commands and exact app
IDs. Browser preview cannot simulate a session operation. Detailed real-time
command/tool/process disclosure remains Milestone 9 and requires its own
redacted normalized event contract.

### Milestone 9 approval and activity boundary

The serialized conversation owner now handles only three reviewed stable
server-request methods: command execution, file change, and permissions. It
correlates native thread, turn, request, and item identity, then replaces the
native request/item identity with app-owned approval and activity UUIDv7 values
before IPC. React can submit only an app conversation ID, app approval ID, and
approve/decline/cancel enum through the fixed
`conversation_approval_decide` command.

Command session acceptance and execution/network policy amendments are not
offered. Additional per-command permission and network context is strictly
parsed and summarized. Permission approval is always turn-scoped; decline and
cancel grant nothing. File requests containing the unstable session write-root
grant can only be declined or canceled. Cancel resolves the pending request
before `turn/interrupt` is sent with native-held IDs.

Activity schema version 2 exposes stable app activity IDs, normalized titles,
sanitized details, status, exit code, bounded command-output lines, MCP progress,
and approval lifecycle events. Rust removes terminal/bidirectional controls,
redacts credential-shaped values, presents only safe project-relative paths,
and discards raw arguments and diffs. Command output waits for a line boundary
so split secret assignments are redacted together; incomplete oversized lines
and final unterminated tails are omitted.

The pending approval and output buffer are ephemeral. SQLite continues to store
only the active reference/status, so restart recovery marks the lost turn
interrupted instead of persisting or replaying consent.

The frontend derives a bounded view of at most 64 activities from the already
bounded event stream and retains at most 32 KiB of display output per activity.
A semantic button expands each activity in place while keeping raw protocol
data out of React. The approval card renders only advertised choices and sends
the exact app conversation/approval IDs through the fixed decision command.
Decision submission is single-flight, and App pauses its poll loop while any
conversation action is in progress so a stale response cannot overwrite the
decision result.

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

The implemented compatibility boundary consists of:

- `CodexBackend`: the versioned asynchronous normalized-snapshot contract.
- `SystemCodexBackend`: fixed CLI detection followed by local app-server JSONL
  probing, with a CLI-only degraded result if app-server is unavailable.
- `MockCodexBackend`: deterministic fixture-backed test behavior.
- `CodexRuntimeSnapshot::unavailable`: structured diagnostics with no simulated
  success when the CLI is missing or invalid.
- `CodexAuthService`: serialized status cache plus one owned pending-login task
  with bounded handoff data and stable diagnostics.
- `CodexAuthSnapshot`: strict account-kind and lifecycle state without account
  identity or secret fields.
- `ConversationService`: serialized app-server ownership, project reservation,
  exact turn correlation and interruption, and reference-only persistence.
- `ConversationSnapshot`: strict application-ID state plus bounded normalized
  events; native Codex IDs and cwd never cross IPC.

Later milestones extend recovery, approvals, and presentation without bypassing
this normalization layer. Generated schemas and sanitized fixtures drive
contract tests.

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
- `conversation_references`: project ID to authoritative Codex thread ID,
  active-turn reference, selected controls, and lifecycle status; no transcript
  or task content.
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
  → access, mount, symlink, Git/worktree, AGENTS.md, and .codex detection
  → confirmation preview
  → transactional association save
  → project workspace
```

Marketplace, MCP, plugin, and connector discovery is deliberately deferred to
the supply-chain-sensitive integration milestones; attaching a directory does
not install or authorize anything it references.

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

Native normalization correlates Codex thread/turn/item IDs when available, but
the frontend event contract carries an application conversation ID and ordered
sequence only. Raw Codex IDs, cwd, timestamps from untrusted payloads, and
unreviewed event fields do not cross IPC.

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

The Astro static site is isolated under `apps/website` in the root pnpm
workspace. It has no runtime backend and receives only version-controlled public
content. Central typed content drives static routes; reusable Astro components
consume layered CSS tokens and approved vector/raster brand exports. Screenshots
and compatibility data are curated release assets; local project, connector,
and account data never enter the site build.

Production is `https://quireforge.jamesjennison.net` on Cloudflare Pages.
GitHub owns source, validation, issues, and release binaries; GitHub Pages
remains disabled. Cloudflare is authoritative DNS. The deployment adapter
builds a static artifact, creates isolated previews, applies version-controlled
headers/redirects, and promotes only an approved production-branch deployment.
DNS cutover is independently approval-gated and recoverable. The generated
artifact is checked for routes, links, assets, canonical metadata, disclaimers,
inline code, and version-controlled headers before browser tests exercise its
desktop/mobile structure, themes, overflow, and accessibility baseline.

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
- PTY implementation and terminal renderer.
- Exact frontend state/query libraries.
- Whether repository-scoped integration settings should be edited directly or
  only through Codex-supported configuration RPCs.
- Cloudflare Git integration versus protected GitHub Actions direct upload,
  pending project-level permission review.
- Functional validation that the selected Tauri, GTK, desktop-entry, D-Bus,
  and packaging toolchain versions preserve the canonical application ID and
  reverse-DNS desktop filename.
