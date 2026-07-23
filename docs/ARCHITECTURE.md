# Architecture

Status: desktop implementation is locally verified through Milestone 19 and
the static website is complete through production Milestone 16 with Milestone
19 accessibility hardening applied locally. Packaging, release publication,
and unsupported integration-management expansion remain subject to separately
gated work.

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
  ├── file preview service ──────── bounded project files
  ├── attachment staging service ── private conversation images
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

### Milestone 21A product-readiness boundary

The desktop startup path renders only the local bootstrap, Codex runtime probe,
and Codex-owned authentication state until access is granted. Project,
conversation, active-task, session, terminal, integration, Git, worktree, and
usage readers do not start behind the sign-in gate. The accepted account state
is normalized by Codex: ChatGPT browser or device login remains the preferred
OpenAI flow, while an already configured API-key or managed provider can report
authenticated or no-additional-login-required state without QuireForge reading
or storing its credential.

The fixed `codex_usage_status` and `codex_usage_refresh` commands invoke only
the documented `account/rateLimits/read` method after app-server
initialization. Rust normalizes at most eight named meters and two windows per
meter into integer used/remaining percentages, bounded durations, bounded Unix
reset times, and a coarse limit-reached boolean. It discards plan type, credit
balance, spend controls, account metadata, reset-credit inventory and IDs, and
all unreviewed fields. Unknown enums, malformed labels/identifiers, impossible
percentages, and invalid timestamps fail closed. The UI never predicts quota
or offers reset-credit redemption.

The authenticated React shell is a presentation hierarchy, not a new backend
capability: Home summarizes existing project and reference-only session state,
then links to the established fixed-purpose workspaces. Internal roadmap
milestones remain repository metadata and are not rendered as product
navigation.

### Milestone 6 implementation boundary

The native `ProjectService` owns migrated QuireForge SQLite metadata and native
directory selection. Attachment confirmation binds selected and resolved path
identity, mount and accessibility evidence, Git/worktree identity, and detected
project instructions. Every execution preflight reloads and revalidates that
evidence; uncertainty fails closed without falling back to another directory.
Detach, archive, and relink mutate only application metadata and never delete
source content or Codex-owned state.

### Milestone 7A implementation boundary

`ConversationService` was introduced as the serialized native owner of one MVP
conversation.
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

### Milestone 10A read-only Git boundary

`GitService` accepts only an app-owned project ID and closed status/diff/open
requests. It reuses the attachment identity check but permits a verified
read-only directory. Each operation reloads and reinspects the attachment; no
frontend cwd, absolute path, executable, Git argument, revision, or environment
value is accepted.

The service runs fixed Git argv arrays with an explicit attached-directory cwd,
cleared environment, prompts and optional locks disabled, external diff and
text conversion disabled, and bounded output and lifetime. Porcelain-v2 object
IDs and raw patch headers are discarded. Paths must be UTF-8, relative,
component-safe, control-safe, directional-formatting-safe, and present in a
fresh status snapshot. Worktree content and editor handoff additionally require
a non-symlink regular file whose canonical path remains inside the attachment.

React receives a strict normalized branch/status projection and bounded diff
line records only. It cannot supply Git options or open an arbitrary file.
Browser preview does not simulate repository data, and no status or diff content
is stored. See [ADR 0012](DECISIONS/0012-read-only-git-review-boundary.md).

### Milestone 10B reviewed Git mutation boundary

`GitService` adds fixed preview, confirm, and recover commands without exposing
Git argv, cwd, revisions, absolute paths, object IDs, or reference names. A
preview contains a closed operation and either one current-status path or one
bounded commit message. Native code requires a writable revalidated attachment,
reserves the project against an active Codex turn, builds an exact plan, and
retains it behind an expiring process-local UUIDv7. Confirmation consumes only
that token, reacquires project ownership, and rechecks root and Git evidence.

Stage and unstage snapshot exact index entries and verify final status, with
index-entry rollback on an unexpected postcondition. Revert accepts only a
tracked regular-file worktree modification no larger than one MiB, snapshots
bytes and ordinary mode, restores only the worktree from the index, and returns
a 30-minute one-use in-memory recovery token. Recovery refuses newer worktree
changes and atomically restores the snapshot. It is not a persistent backup.

Commit requires every staged path and rename source to remain in the exact
attachment. It rejects conflicts, submodules, active merge/cherry-pick/revert
state, missing repository-local identity, unscannable staged blobs, and
high-confidence secrets in staged content, sensitive filenames, or the commit
message. Git plumbing writes the reviewed tree, locks and revalidates the index,
creates a hookless/unsigned commit, and updates `HEAD` only from the reviewed old
value. It checks final reference and index state and attempts expected-value
rollback on an unexpected postcondition. See
[ADR 0013](DECISIONS/0013-reviewed-git-mutation-boundary.md).

### Milestone 11A managed-worktree boundary

`WorktreeService` discovers the repository group from an app-owned project and
runs a bounded, shell-free `git worktree list --porcelain -z`. React receives
only display paths, normalized branch names, ownership/state enums, and optional
QuireForge project IDs; object IDs, raw output, stderr, configuration, Git
directories, and common-directory paths stay native.

Create preview accepts an app-owned project ID and one bounded new branch name.
The service generates a destination beneath private app storage, captures the
source repository identity and HEAD internally, verifies branch absence, and
stores the plan behind a five-minute one-use UUIDv7. Attach preview obtains the
path only through the native picker and binds exact directory/Git identity. On
confirmation, all app-owned projects sharing the source repository are reserved
and the complete evidence is revalidated. Git uses fixed argument arrays, no
shell, no prompts/global/system configuration, a timeout, bounded output,
disabled hooks, and overrides for configured checkout filters.

Schema migration 4 adds only the source project, worktree project, managed or
attached ownership, and optional normalized branch. Every linked worktree is an
ordinary project, so existing conversation cwd and Git safeguards are reused.
If Git creation succeeds but the transaction fails, QuireForge leaves the
worktree untouched and reports its display path for future explicit recovery.
No remove, prune, filesystem cleanup, concurrent execution, or conflict action
exists in 11A. See
[ADR 0014](DECISIONS/0014-managed-worktree-foundation.md).

### Milestone 11B bounded parallel-execution boundary

`ConversationService` now owns a registry of at most four active or starting
tasks keyed by app-owned conversation UUIDv7. A short-lived registry mutex
protects capacity and membership; each active process has a separate mutex, so
polling or approval I/O for one task does not serialize unrelated worktree
tasks. Project reservations continue to reject duplicate execution in one
project and block metadata or reviewed Git mutation races.

Start and resume reserve capacity before process creation. Poll, approval, and
interrupt first resolve the exact app conversation ID to its native slot. Each
terminal path marks the slot finished once, closes and waits for its child,
releases its exact project, removes only the same registered slot, and retains
only a bounded event-free recent snapshot. Active-task app-server I/O never
holds the registry lock; existing all-session reconciliation remains serialized
only while no task is active.

The fixed `conversation_active` command returns schema version 1, literal
capacity four, and at most four active normalized `ConversationSnapshot`
records with empty event batches. Codex thread/turn/request IDs, cwd, process
identity, arguments, environment, and raw protocol messages remain native.
Startup recovery remains conservative: process ownership is not reconstructable
after application exit, so persisted stale active rows become interrupted.

React stores bounded events independently per project, recovers the active
registry after webview refresh, and polls each task by its app conversation ID.
Per-task action generations discard a stale response only for the task whose
approval or interruption changed. The worktree monitor filters tasks through
the current native inventory and combines their normalized lifecycle with
read-only `GitService` changed-file and conflict counts. Selecting a monitor
row opens the existing expandable live activity view. No raw Git output,
automatic conflict resolution, Git mutation, worktree cleanup, or durable task
recovery was added. See
[ADR 0015](DECISIONS/0015-bounded-parallel-worktree-execution.md).

### Milestone 11C managed cleanup and recovery boundary

Worktree IPC schema version 2 adds no frontend path or Git-command input.
Native inventory may attach an expiring opaque recovery ID to an otherwise
external entry only when its canonical linked-worktree directory occupies the
exact QuireForge-managed slot for that source project. Recovery consumes the ID,
rebinds complete directory/repository identity behind a fresh confirmation, and
registers the existing checkout without changing its files or branch.

Removal preview accepts the selected app-owned project ID and one related
worktree project ID. The stored relation must still be `managed`; source,
selected, attached, external, locked, and prunable worktrees are excluded.
Confirmation reserves the repository's complete app-owned project group and
revalidates the relation, private-storage shape, canonical identity, common Git
directory, inventory entry, branch, `HEAD`, and clean tracked/untracked/
submodule state. Repository-configured filters are replaced with a fixed
identity transform for both explicit status and Git's internal removal check.

The fixed `git worktree remove` call never uses force and never deletes the
branch. The native service verifies directory absence, inventory absence, and
branch retention before transactionally detaching and archiving the project
metadata. A post-Git metadata failure remains visible as a missing managed
entry; a separately reviewed, non-destructive confirmation can finalize only
the metadata while the directory and inventory entry remain absent. Generic
prune is unavailable because Git cannot scope it to one app-owned target. See
[ADR 0016](DECISIONS/0016-safe-managed-worktree-cleanup.md).

### Milestone 15A safe file-preview boundary

`FilePreviewService` is stateless and accepts one app-owned project UUIDv7 plus
the path returned directly by the native picker. Before any content crosses
IPC, Rust reloads the attachment, revalidates directory identity and readable
accessibility, canonicalizes the selection, requires attachment containment,
rejects symlinks/non-regular files, retains an identity-checked root directory
descriptor, opens the relative target through that descriptor with
`O_NOFOLLOW`, and rechecks the opened path/device/inode. Absolute paths never
enter React.

The closed schema distinguishes normalized UTF-8 text, bounded PNG/JPEG image,
and metadata-only PDF presentation. Text is capped at 128 KiB/2,000 lines and
has controls and bidi overrides replaced. Images are capped at 4 MiB, 8,192
pixels per dimension, and 16 million pixels; type/dimensions are checked before
and after the full read, and APNG is refused. PDF bytes, unknown binary content,
and active documents never enter the privileged webview. HTML/SVG markup can
appear only as inert normalized text, never as active markup. Source files are
capped at 8 MiB and preview state is never persisted.

The production CSP permits `data:` only for image sources so the two bounded
image types can render. Browser preview cannot select or read a local file.
Conversation attachments use the separate 15B boundary below. Notifications,
reviewed open-with behavior, and Linux verification use the 15C boundary below.
See [ADR 0021](DECISIONS/0021-safe-project-file-previews.md).

### Milestone 15B conversation-image boundary

`ConversationAttachmentService` accepts explicit paths returned by the native
picker, bounded PNG/JPEG bytes from an HTML drag/drop event, or a one-use Linux
file-manager drop captured by GTK. Tauri's default file-drop handling remains
disabled so the webview never receives native filesystem paths. Picker paths
remain in Rust; dropped browser `File` objects are read only after an explicit
user gesture and carry bytes, a safe display name, and a declared image type to
the fixed staging command. When WebKitGTK supplies an empty HTML `FileList`, a
Linux-only GTK signal retains at most five file URIs (four allowed plus one
overflow sentinel) in process memory for 30 seconds. The drop zone invokes a
separate path-free claim command, after which Rust consumes the slot and applies
the normal file/content limits.

Rust independently validates the real PNG/JPEG structure, dimensions, MIME,
4 MiB per-file and four-file/16 MiB aggregate limits, refuses symlinks and
unsafe names, and writes app-owned copies beneath a mode-`0700` staging root as
mode-`0600` UUIDv7 files. React receives only opaque draft IDs and normalized
name/type/size/dimension metadata. Draft IDs are project-bound, expire after
15 minutes, are consumed once by explicit start/resume/fork, and never enter
SQLite.

Immediately before a turn, native code reopens and revalidates each staged
file's device, inode, length, type, and dimensions. It then constructs only the
documented Codex `localImage` input from the private path. Because
`turn/start` returns the initial turn before streamed work completes and the
official contract gives no earlier consumption guarantee, claimed copies stay
native-owned until the normalized turn is completed, interrupted, blocked, or
failed. Cancellation, failed sends, expiry, and startup reconciliation provide
the other cleanup paths. Generic files remain unsupported because the reviewed
Codex 0.145.0 turn schema has no generic local-file input. See
[ADR 0022](DECISIONS/0022-bounded-conversation-image-attachments.md).

### Milestone 15C desktop-integration boundary

Each ready safe preview now registers one process-local, five-minute UUIDv7
open action. React can review the attachment-relative file name and the closed
`System default application` destination, but it cannot provide a path,
application, executable, argument, MIME type, URL, or working directory. Claim
is one-use; native code reloads the attachment and revalidates canonical
containment, regular non-symlink state, descriptor identity, and the previewed
device/inode before calling Tauri's opener. Replacement, clear, expiry, and
successful claim remove the action, and at most 16 actions exist.

Background conversation notifications accept only an app-owned conversation
UUIDv7 and re-resolve it against the native recent-state registry. Pending
approval, completion, block, and failure select fixed privacy-safe copy;
interrupted and other states are ineligible. The main window's native focus
state suppresses foreground alerts, and approval/terminal identity provides
bounded deduplication. Project names, prompts, paths, model/account data,
outputs, diagnostics, and raw protocol fields never enter the notification.
The official Rust notification plugin is initialized without granting its
commands to the webview; the main capability list remains empty. See
[ADR 0023](DECISIONS/0023-reviewed-desktop-handoffs-and-notifications.md).

## Application layers

### Frontend

React and TypeScript render semantic, keyboard-accessible views. State is split
between persisted application state, short-lived UI state, and normalized
backend event streams. The frontend cannot directly open filesystem paths,
spawn processes, execute Git, or edit Codex configuration. Its editor action
supplies only an app-owned project ID and normalized changed-file path. A Git
mutation preview supplies only a closed operation and path/message; confirmation
supplies only the native-held plan token. A worktree create preview supplies one
bounded branch name; existing paths come from the native picker, and every
worktree confirmation supplies only its native-held token. Recovery and cleanup
previews add only opaque app-owned recovery/project IDs; React still cannot
submit a worktree path, branch for removal, cwd, executable, or Git argument.
File preview adds only an opaque project ID; the native picker owns the path,
and React can consume only the strict bounded snapshot. Its optional desktop
handoff adds one native-held open-action ID with a fixed default-application
destination; React still cannot submit a path or application.
Conversation attachments add only opaque project/attachment IDs plus bounded
dragged image bytes; no source path, staged path, generic file handle, or
arbitrary read operation crosses the boundary.
Notifications add only an app-owned conversation ID and return a closed
delivery status; native code owns eligibility, focus suppression, copy, and
deduplication.

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
  four-task bounded registry, exact process routing and interruption, project
  reservation, and reference-only persistence.
- `ConversationSnapshot`: strict application-ID state plus bounded normalized
  events; native Codex IDs and cwd never cross IPC.
- `ConversationRegistrySnapshot`: strict active-only capacity/task projection
  with no event replay or native identity.

Milestone 18 implements `ModelSelectionService` inside this boundary using the
Milestone 13 validated app-server lifecycle. The service owns the
normalized catalog, current effective selection, one pending next-turn
selection, selector-ownership policy, and bounded provenance. A Codex request
can only stage a change after native revalidation; the current turn continues
on its existing model, and the pending value is applied only when constructing
the next `turn/start`. React cannot submit an unadvertised model, private model
identifier, raw protocol payload, or direct configuration edit.

Registration rejection retries the thread without the dynamic tool and exposes
recommendation-only availability. Conversation and session schema version 3
carry a schema-versioned selector snapshot, while `model_selection_update`
accepts only an opaque conversation ID, reviewed choice, closed policy, and
pending action. Later milestones extend hardening without bypassing this
normalization layer. Generated schemas and sanitized fixtures drive contract
tests.

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
- `GitService`: implemented bounded status, branch, diff, changed-file editor
  handoff, stage, unstage, bounded revert/recovery, and commit; remote
  operations remain later work.
- `WorktreeService`: implemented bounded inventory, managed creation, native-
  picker attachment, retained-worktree recovery, clean managed removal,
  metadata-only cleanup finalization, expiring confirmation, and project
  registration; React composes its inventory with bounded conversation/Git
  snapshots.
- `TerminalService`: independent PTY sessions rooted in verified directories.
- `ApprovalService`: request correlation, scope, decision validation, expiry.
- `PreviewService`: implemented project-contained normalized text, bounded
  PNG/JPEG, and metadata-only PDF previews; no general read operation.
- `ConversationAttachmentService`: implemented private PNG/JPEG staging,
  one-use project-bound drafts, native `localImage` construction, and terminal-
  turn cleanup; no generic local-file input or credential storage.
- `SettingsService`: application settings without secret ownership.
- `CapabilityService`: Codex/runtime/version/policy capability map.
- `ModelSelectionService`: live catalog validation, Manual/Recommend/Automatic
  ownership policy, one staged next-turn selection, user-lock precedence,
  bounded rationale/provenance, and oscillation/cost-ceiling enforcement.
- `IntegrationCatalogService`: normalized category-preserving catalog.
- `PluginMarketplaceService`: source inspection and supported marketplace calls.
- `PluginInstallationService`: preview, confirmation, lifecycle, and result
  verification.
- `ConnectorAuthorizationService`: official URL handoff and status refresh.
- `McpServerService`: configuration/status/OAuth/health adapter.
- `IntegrationPolicyService`: workspace/admin and requested-permission rules.
- `IntegrationHealthService`: non-destructive status checks.

Milestone 13A fixes the shared `codex-integration-v1` boundary before those
services become runtime IPC. Capabilities carry domain, operation, official
route, method stability, upstream availability, independent QuireForge
implementation state, mutation/confirmation policy, and stable diagnostics.
Entries retain their connector, plugin, marketplace, skill, or MCP-server
category and expose only bounded display metadata plus normalized scope,
source, installation, enablement, authentication, permissions, requirements,
policy, and health. Raw paths, URLs, configuration, managed requirements,
account identity, credentials, tool arguments, and protocol payloads remain
native-only.

Milestone 13B implements `IntegrationCatalogService` as a serialized native
owner with one fixed-purpose `integration_catalog_read` command. The service
gates the installed CLI to the reviewed 0.145.x minor, owns and reaps one
app-server process, and caches only a validated normalized snapshot. It uses
`app/list` plus `app/installed` for connector state, `skills/list` from a neutral
temporary cwd, `mcpServerStatus/list` with bounded detail, and effective policy
reads. Plugin and marketplace discovery run through fixed stable CLI JSON
commands with bounded stdout, timeout, null stdin/stderr, and a neutral `/`
cwd; experimental plugin-management RPCs are not production routes.

App, skill, MCP-startup, configuration-warning, and account-change
notifications become closed refresh-reason enums; their raw payloads are
discarded. Refreshes replace only the affected category, while independent CLI
plugin/marketplace failures preserve healthy app-server categories. Absolute
paths, URLs, config contents, marketplace roots, endpoint/tool definitions,
account identity, and upstream error text remain native-only and are reduced
to stable diagnostics.

Milestone 14A implements only `plugin.install`, `plugin.remove`, and
`marketplace.configure` through `IntegrationMutationService`. Two fixed Tauri
commands expose a closed preview/confirm state machine; no generic integration
command exists. Preview forces a fresh catalog/policy build and stores exact
CLI source evidence behind a five-minute, one-use native UUIDv7. Confirmation
serializes mutations, rechecks the CLI minor, policy, capability, normalized
entry, and exact evidence, then runs one fixed stable CLI command and verifies
both its closed JSON result and a fresh catalog postcondition. Repository
marketplace adds require a pinned hexadecimal reference; remote marketplace
upgrades disclose their mutable-source risk. Bounded permissions, warnings,
opaque entry IDs, and stable diagnostics are the only mutation data crossing
IPC. See [ADR 0019](DECISIONS/0019-confirmed-integration-mutations.md).

Milestone 14B implements the React Integration Center over only the normalized
13B catalog and fixed 14A mutation contracts. The view preserves integration
categories, searches bounded display metadata, filters category and health,
and displays normalized source, scope, installation, enablement,
authentication, policy, publisher, version, permission, requirement, and
health state. Mutation controls render only when upstream availability and
QuireForge implementation are both ready. Preview and confirmation preserve
the fixed operation ID, disclose normalized permissions, warnings, destructive
status, and separate hook trust, and refresh the catalog after an applied
result. Browser preview uses sanitized deterministic fixtures and never claims
live native state.

Milestone 14C adds `IntegrationControlService` as the serialized native owner
for four closed controls: connector authorization, MCP authorization, skill
enable, and skill disable. Preview resolves one opaque catalog ID to fresh
native evidence and places it behind a five-minute one-use UUIDv7. Confirmation
revalidates the ready capability, catalog state, and exact evidence before it
uses a Codex-returned connector URL, `mcpServer/oauth/login`, or
`skills/config/write`. Skill results require an exact effective-state response
and fresh list postcondition. Authorization URLs remain native-only and are
opened through Tauri from a ten-minute opaque action; MCP completion must match
the exact native server name. The corresponding fixed IPC surface is
`integration_catalog_refresh`, `integration_control_preview`,
`integration_control_confirm`, `integration_control_open_browser`, and
`integration_control_status`. See
[ADR 0020](DECISIONS/0020-confirmed-integration-authorization-and-controls.md).

Conversation start can include at most eight unique normalized connector entry
IDs. The control service re-resolves each against `app/list` and
`app/installed`, requires accessible, enabled, and callable state, and returns
native-held display/path evidence to the conversation service. Only native code
constructs the documented `mention` item and `app://` path. Neither the raw app
ID/path nor the authorization URL is persisted or serialized to React.

Explicit refresh is non-destructive and rebuilds the normalized catalog and
health state. Plugin enable/disable, generic connector installation or
configuration, MCP add/remove/logout/configuration, arbitrary health repair,
generic config editing, and the app-owned dynamic tool remain unimplemented.

Milestone 17A advances the catalog boundary to `codex-integration-v2` and adds
`scheduledTasks` without adding a second IPC surface. The service derives a
bounded native-only lookup list from installed, enabled plugin rows and exact
marketplace roots, then calls stable `plugin/read` on the already-owned
app-server process. It accepts only the reviewed plugin detail and schedule
shapes, binds every task to an existing normalized plugin entry, bounds global
and per-plugin counts, collapses prompts to inert single-line previews, and
removes unsafe control/directional characters. Raw paths, plugin loader
metadata, and protocol errors never cross IPC.

The Scheduled React workspace consumes only schema-v2 task metadata and has no
mutation bridge or action control. A degraded task response does not erase
unrelated catalog categories. QuireForge does not persist or execute task
prompts and implements no local or hosted scheduler. See
[ADR 0025](DECISIONS/0025-read-only-scheduled-task-catalog.md).

The implemented app-owned dynamic-tool boundary registers a closed schema through
`thread/start`, accepts only the correlated `item/tool/call` server request,
and returns a bounded response. Registration, invocation, and response IDs are
native-owned. Milestone 13A established the lifecycle contract; Milestone 18
uses it to stage a policy-valid model choice only after successful turn
completion and to apply it after a fresh catalog check on the next turn, never
to replace the executing turn's model. See
[ADR 0026](DECISIONS/0026-policy-bounded-next-turn-selection.md).

## Project and directory data model

Identifiers are UUIDv7 or equivalent stable, opaque IDs. Directory names and
paths are never database keys.

### `projects`

| Field                             | Purpose                       |
| --------------------------------- | ----------------------------- |
| `id`                              | Stable application project ID |
| `display_name`                    | User-facing name              |
| `active_directory_association_id` | Explicit current working root |
| `archived_at`                     | Application organization only |
| `created_at`, `updated_at`        | Metadata timestamps           |

### `directory_associations`

| Field                                 | Purpose                                               |
| ------------------------------------- | ----------------------------------------------------- |
| `id`                                  | Stable association ID                                 |
| `project_id`                          | Owning project                                        |
| `selected_path`                       | Exact absolute path selected by the user              |
| `resolved_path`                       | Last verified resolved absolute path                  |
| `display_path`                        | Home-relative presentation when appropriate           |
| `role`                                | Primary, additional writable, read-only context, etc. |
| `is_primary`                          | Primary working-directory flag                        |
| `expected_access`                     | Read/write expectation                                |
| `device_id`, `inode`                  | Local identity evidence where supported               |
| `filesystem_type`, `mount_id`         | Mount/removable/network evidence                      |
| `git_common_dir`, `git_worktree_root` | Git/worktree identity                                 |
| `last_verified_at`                    | Last complete verification                            |
| `accessibility_state`                 | Explicit state enum                                   |

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

Milestone 12 implements that boundary with `portable-pty` and stable xterm
packages. Native code owns each PTY master, writer, child handle, Linux session
identity, output window, and project reservation. React receives only an
app-owned UUIDv7, project ID, title, state, dimensions, exit code, bounded
base64 chunks, and stable diagnostics; it cannot select or observe the cwd,
shell, environment, TTY, PID, process group, or session ID. Eight live or
recoverable tabs are allowed. Each live tab retains at most one MiB/512 chunks,
and a poll returns at most 128 KiB/64 chunks with explicit truncation state.

The child environment is cleared and rebuilt from a fixed system `PATH`,
terminal identity, and a narrow desktop/session allowlist. Credential-shaped
variables, SSH/GPG agent sockets, and Codex/QuireForge process configuration
are not inherited. User shell startup files may intentionally add environment
after launch. Closing a tab sends bounded HUP, TERM, and KILL phases only to
processes whose numeric `/proc` records still belong to the owned session,
then waits on the retained child handle. A process that deliberately creates a
new session is outside the tab ownership boundary.

SQLite migration 5 stores only the tab ID, project ID, title, state,
dimensions, exit code, and timestamps. Running/closing records become
interrupted on restart; QuireForge does not claim to recover process ownership
and never persists input, output, scrollback, history, cwd, environment, or
process identity. Integrated terminal input uses the Linux account's ordinary
privileges and remains separate from Codex approval semantics. See
[ADR 0017](DECISIONS/0017-native-integrated-terminal.md).

## Git and worktrees

Git commands use argv arrays, explicit cwd, bounded output, and no shell
interpolation. Read operations are automatic. Mutating operations identify the
repository/worktree and preview effects. Destructive cleanup is separate from
closing/removing an application project.

The Milestone 10A implementation limits status to the attached directory,
ignores global/system Git configuration, disables optional write/performance
features and extensible diff execution, reparses current status before each
path-specific action, and returns only normalized bounded records. It performs
no index, worktree, reference, configuration, or object mutation.

Milestone 10B keeps mutation outside that read-only bridge. Native-held,
single-use preview plans gate file-level stage/unstage, bounded regular-file
revert/recovery, and an attachment-scoped commit plumbing workflow. Exact status,
index, worktree, tree, and reference postconditions protect unrelated work.
Hooks, signing, prompts, global/system configuration, arbitrary arguments, and
advanced branch/worktree/remote operations remain unavailable.

Milestone 11A admits only one advanced branch operation: creating a new branch
as part of an app-managed worktree. The native plan binds source identity and
HEAD, owns the destination, disables checkout hooks/configured filters, and
revalidates every effect at confirmation. Existing-worktree paths come only
from the native picker. Milestone 11B adds no Git mutation: it runs independently
reserved Codex processes in up to four verified worktree projects and reads
only normalized Git status counts for the aggregate monitor. Milestone 11C adds
only fixed recovery registration and non-force managed-worktree removal. It
retains the branch and excludes generic prune, attached/external deletion,
direct filesystem deletion, and conflict resolution.

Codex-managed sessions and user worktrees are never removed as a side effect of
detaching a directory or deleting app metadata.

## Website

The Astro static site is isolated under `apps/website` in the root pnpm
workspace. It has no runtime backend and receives only version-controlled public
content. Central typed content drives static routes; reusable Astro components
consume layered CSS tokens and approved vector/raster brand exports. Screenshots
and compatibility data are curated release assets; local project, connector,
and account data never enter the site build.

Production is `https://quireforge.jamesjennison.net` on a Webuzo-managed
Apache origin, with Cloudflare retained as authoritative DNS and the proxied
TLS/cache edge. Source, validation, issues, and development activity are public
on GitHub; GitHub Pages and Cloudflare Pages remain disabled. The deployment
workflow builds outside public storage, records a per-file artifact manifest,
uses origin-only staging, applies a version-controlled Apache `.htaccess`, and
promotes only an exact approved artifact to the Webuzo-reported document root.
Domain creation, staging, and the one-record DNS cutover completed through
separate approvals and remain independently recoverable. The generated artifact
is checked for routes, links, assets, canonical metadata, disclaimers, inline
code, and version-controlled headers before browser tests exercise its
desktop/mobile structure, themes, overflow, and accessibility baseline.

The dormant download record is an unavailable/published typed union. A
published value must contain exactly one version-coherent x86_64 AppImage and
one Debian package, positive sizes, lowercase SHA-256 values, UTC publication
time, and credential-free HTTPS package/manifest/checksum URLs on the approved
QuireForge origin. The website build also requires an approved private security
reporting URL before it can render a published release. Package promotion to a
versioned owner-hosted directory, download-record activation, and website
deployment are independent exact-artifact operations with separate rollback
points. Public GitHub release records are secondary provenance and artifact
records; the website download contract continues to require credential-free
same-origin files on the owner-hosted QuireForge origin.

## Milestone 19 hardening boundary

The pre-packaging hardening pass does not add a privileged product capability.
It makes the existing boundary repeatable and fail-closed:

- the main Linux webview capability remains permission-empty, the global Tauri
  object and asset protocol are explicitly disabled, and production builds
  prune unused plugin commands;
- the production CSP begins at `default-src 'none'`, admits only local
  frontend assets, data images, and Tauri IPC, and pairs with restrictive
  response headers;
- direct production-frontend active HTML insertion, string evaluation,
  fetch/XHR, and WebSocket primitives fail repository validation;
- pnpm and Cargo dependency audits run in CI, GitHub Actions must use immutable
  SHAs, and the exact unavoidable Tauri/GTK3 RustSec exceptions remain visible;
- the startup entry, application shell, and heavy xterm workspace are separate
  production chunks with deterministic per-chunk and total asset budgets; an
  opaque loading overlay remains mounted until the application commits and
  paints;
- keyboard skip targets, reduced motion, forced colors, terminal confirmation
  focus ownership, and a raw-error-free React recovery boundary are covered in
  both unit and browser gates.

Tauri's `freezePrototype` remains disabled because enabling it breaks the
verified Vite/React production mount. Inline style permission remains limited
to the stable xterm renderer. These exceptions, the inherited GTK3 advisory
set, and the full acceptance record are documented in the
[Milestone 19 hardening review](MILESTONE_19_HARDENING.md).

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
- Exact frontend state/query libraries.
- Whether repository-scoped integration settings should be edited directly or
  only through Codex-supported configuration RPCs.
- Long-term manual artifact promotion versus a separately approved
  least-privilege protected deployment workflow.
- Functional validation that the selected Tauri, GTK, desktop-entry, D-Bus,
  and packaging toolchain versions preserve the canonical application ID and
  reverse-DNS desktop filename.
