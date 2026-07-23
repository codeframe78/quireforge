# Roadmap

The roadmap is gated milestone by milestone. Before each milestone, the
maintainer must inspect currently available Codex models, recommend the newest
suitable GPT model and reasoning level, provide the full milestone briefing,
and wait for manual confirmation.

No milestone may merge, access authenticated hosting, change DNS/SSL/provider
settings, deploy, publish a release, install an integration, or authorize a
connector without its required approval.

## Permanent identity migration

The discovery-stage name “Codex Linux Workbench” has been replaced by the
permanent product identity **QuireForge**: “Build boldly. Work locally.” The
migration preserves the repository and its history. Tracked documentation,
GitHub repository identity, original local working-copy path, and branding
assets are handled as separately verified and approval-gated migration steps.

The working copy moved through a controlled Codex-session handoff to
`/mnt/faststorage/quireforge`. The existing GitHub repository was first renamed
in place and was later transferred to the private
`James-Jennison/quireforge` organization location. None of those operations
authorized a push, public source link, website deployment, or release.

Migration status: the tracked identity contract, authoritative naming audit,
in-place GitHub repository rename, local working-copy handoff, and core vector
brand sources are complete. Milestone 1 also established the Apache-2.0 license,
repository guidance, contribution/security/conduct/support policies, issue and
pull-request templates, dependency automation, and initial repository CI. The
work through Milestone 6 is merged on `main`. Milestone 2 added the local static
website, production web exports, and automated website quality gates without
creating a hosting project or deployment. Milestone 3 added the locally verified Tauri
desktop foundation, narrow typed IPC contract, Linux app icons, and desktop
quality gates without producing an installable package. Milestone 4 added the
versioned Codex boundary, supervised app-server probe, normalized model catalog,
mock/failure tests, and selected generated schemas without starting a model
turn or modifying Codex state. Milestone 5 added normalized account status,
Codex-owned browser/device onboarding, exact cancellation/completion handling,
explicit logout, and redacted recovery without retaining secrets. Milestone 6
adds app-owned project metadata, native directory attachment, identity-aware
preflight, and an accessible project workspace without copying or deleting
source content. Milestone 7A adds the native conversation runtime, strict
normalized contracts, exact-turn interruption, and reference-only persistence;
Milestone 7B adds the responsive task composer, runtime-derived controls,
normalized progress stream, and exact stop interaction. Application packages
and external provider settings remain milestone- and approval-gated. Milestone
8A adds native resume, fork, archive/restore, Codex-authoritative reference
reconciliation, and conservative crash recovery. Milestone 8B adds the bounded
history/search/tabs presentation and accessible lifecycle actions.
Milestone 9A adds the native approval and detailed-activity contract with
app-owned correlation, one-turn decisions, redaction, and safe cancellation;
Milestone 9B adds the selectable expanded activity and bounded approval
interface over that contract.
Milestone 10A adds a fixed native read-only Git boundary, normalized status and
diff contracts, a responsive changed-file reviewer, and revalidated editor
handoff. Milestone 10B adds fixed stage, unstage, bounded revert/recovery, and
commit workflows behind native-held preview tokens, exact postconditions,
project concurrency, attachment scope, and secret review.
Milestone 11A adds the native managed-worktree foundation, strict inventory and
preview contracts, app-generated destinations, native-picker attachment, and
ordinary project registration without adding cleanup or concurrent execution.
Milestone 11B adds a four-task native conversation registry, independent
worktree execution and interruption, refresh recovery from normalized active
state, and aggregate activity/changed-file/conflict presentation without
adding destructive cleanup or automatic conflict resolution.
Milestone 11C adds opaque recovery for retained app-managed worktrees and
confirmed removal of clean, inactive, app-managed worktrees while preserving
their branches. Attached worktrees, force removal, generic prune, direct
directory deletion, and conflict resolution remain excluded.
Milestone 12 adds a bounded native PTY registry, controlled shell environment,
fresh project-cwd verification, tabbed xterm presentation, byte-preserving
input/output, resize, background-job ownership, and metadata-only restart
recovery without exposing raw paths or process identity to React.
Milestone 18 implements app-owned, policy-bounded model and reasoning selection
for the next turn. The current turn never replaces itself; Manual, Recommend,
and explicitly bounded Automatic ownership remain under visible user control.
Milestone 13A supplied the validated dynamic-tool lifecycle used by that
selector. Milestone 13B establishes live read-only integration discovery.
Milestone 14A establishes the confirmed native plugin and marketplace mutation
boundary. Milestone 14B adds the user-facing Integration Center over that
boundary without broadening it. Milestone 14C adds only reviewed connector/MCP
authorization, skill enablement, refresh, and connector prompt mentions. Later
Milestones 15A–15C complete the bounded local preview, conversation-image, and
desktop-integration surfaces. Milestone 16 completes the production static
website. Milestone 17A establishes read-only installed-plugin task-template
discovery; scheduling management and execution remain unsupported.

## Status

| Milestone | Scope                                                             | Size         | Status                                                                   |
| --------: | ----------------------------------------------------------------- | ------------ | ------------------------------------------------------------------------ |
|         0 | Existing project and feasibility discovery                        | Very large   | Complete; merged to `main`                                               |
|         1 | QuireForge rename, move, GitHub migration, and governance closure | Medium       | Complete; merged to `main`                                               |
|         2 | QuireForge brand and static website foundation                    | Large        | Complete; merged to `main`; deployed later through Milestone 16          |
|         3 | Desktop scaffold consolidation                                    | Large        | Complete; merged to `main`; not packaged                                 |
|         4 | Codex process adapter and contracts                               | Very large   | Complete; merged to `main`                                               |
|         5 | Authentication and onboarding                                     | Medium       | Complete; merged to `main`                                               |
|         6 | Projects and direct local-directory attachment                    | Very large   | Complete; merged to `main`                                               |
|         7 | Conversation MVP                                                  | Very large   | Complete; merged to `main`                                               |
|         8 | Session lifecycle and crash recovery                              | Large        | Complete; merged to `main`                                               |
|         9 | Approvals and command presentation                                | Large        | Complete and verified; publication recorded in repository history        |
|        10 | Git status, diff review, and controlled mutations                 | Large        | Complete and verified; publication tracked by this milestone change      |
|        11 | Worktrees and parallel work                                       | Very large   | Complete through 11C and verified locally                                |
|        12 | Integrated terminal                                               | Large        | Complete; merged to `main`; not packaged                                 |
|        13 | Integration discovery and compatibility                           | Very large   | Complete through 13B; verified locally                                   |
|        14 | Integration Center and installation workflows                     | Very large   | Complete through 14C; merged and verified on `main`                      |
|        15 | File previews and desktop integration                             | Large        | Complete through 15C; verified locally                                   |
|        16 | Complete Webuzo-hosted static website                             | Very large   | Complete through 16D; production and automatic origin TLS renewal active |
|        17 | Scheduled tasks and advanced supported features                   | Medium–Large | Complete through 17A locally; management/execution deferred              |
|        18 | Agent-directed model and reasoning selection                      | Large        | Complete and verified locally; not published                             |
|        19 | Security, accessibility, and performance hardening                | Very large   | Planned                                                                  |
|        20 | Packaging and release automation                                  | Large        | Planned                                                                  |
|        21 | Beta package publication and download activation                  | Very large   | Planned/approval-gated                                                   |

## Milestone definitions

### 0 — Existing Project Audit and Feasibility

Inspect the installed Codex CLI, app-server, plugins, marketplaces, skills,
MCP, apps/connectors, policy, authentication, local cwd behavior, Linux/Tauri
prerequisites, GitHub, public DNS/TLS/site behavior, and the selected Cloudflare
Pages account through a separately approved method. Document production
constraints, previews, security, cutover, and rollback. Make no hosting, DNS,
repository-setting, or production change.

### 1 — QuireForge Rename, Move, and GitHub Migration

Verify and reconcile the already-completed intact local move, permanent
QuireForge identity, in-place GitHub repository rename, package/application
contracts, historical references, and user-data conclusion. Complete the
required governance baseline—license, contribution/security/conduct/support
policies, templates, dependency automation, and initial CI—without repeating or
discarding completed migration work.

### 2 — Brand Identity and Cloudflare Website Foundation

Develop the approved QuireForge vectors into consuming assets and scaffold the
Astro site, design tokens, themes, navigation, metadata, responsive layout, and
accessibility foundation. Confirm the audited document-root/deployment design.
Do not touch staging or production without separate approval.

Completed locally: the Astro static package, 15-page information architecture,
design tokens, original brand exports, themes, navigation, metadata, custom
404, Cloudflare headers, deterministic artifact validation, and desktop/mobile
accessibility checks. No Cloudflare project, custom domain, DNS record, preview,
or production deployment was created.

### 3 — Desktop Scaffold Consolidation

Install/verify prerequisites, scaffold Tauri 2 + React + TypeScript + Rust,
establish typed frontend/native IPC, shell layout, lint/format/test commands,
and CI.

Completed locally: the Tauri/React desktop package, exact executable and runtime
application identities, Linux icon exports, responsive light/dark shell, one
versioned bootstrap command, shared Rust/TypeScript contract fixture, empty
plugin-permission capability, strict frontend/native checks, desktop/mobile
axe-core coverage, unbundled release build, and GNOME Wayland launch. No Codex
process, directory attachment, persistence, package, push, or release was
created.

### 4 — Codex Process Adapter

Implement version/capability probing, process lifecycle, stable normalized
events, app-server stdio adapter, CLI fallbacks, mock backend, generated schema
fixtures, and contract tests.

Completed locally: fixed-command CLI version detection, a versioned
`CodexBackend` contract, serialized runtime probing, newline-delimited
app-server request correlation, bounded messages/timeouts, deterministic mock
and failure processes, normalized capability/model/error records, strict
Rust/TypeScript fixtures, selected generated initialize/model schemas, and an
honest desktop status. A non-billable live probe verified Codex CLI 0.144.6 and
left no child process. Authentication, threads, turns, project paths,
persistence, configuration writes, package, push, and release remain absent.

### 5 — Authentication and Onboarding

Implement Codex detection, account status, Codex-managed browser/device login,
logout, diagnostics, redaction, and failure recovery without owning secrets.

Completed locally: stable generated account schemas, normalized read-only
status, a single-owner pending-login process, allowlisted browser/device
handoffs, exact completion correlation, cancellation races, explicit two-step
logout, stable redacted errors, strict Rust/TypeScript fixtures, accessible
onboarding UI, and deterministic failure tests. A live non-mutating
`account/read` probe returned only normalized state and left no child process.
No real login, browser authorization, logout, token handling, project,
conversation, package, push, deployment, or release occurred.

### 6 — Projects and Direct Local-Directory Attachment

Implement the persistent multi-root-ready project schema, native picker,
selected/resolved identity, Git/worktree and project-instruction detection,
confirmation, missing/read-only/mount states, detach, relink, and per-task cwd
preflight.

Completed locally: the native core owns a migrated SQLite metadata store,
UUIDv7 project/association IDs, selected and resolved path identity, mount and
Git/worktree evidence, project-instruction detection, confirmation-time change
detection, detach/archive/relink metadata operations, and fail-closed cwd
preflight. Deterministic Rust tests cover symlink retargeting, linked worktrees,
read-only and missing directories, duplicate roots, storage permissions, and
the no-source-deletion boundary. A strict TypeScript contract rejects unknown
or path-bearing input, while the accessible project workspace provides native
selection, confirmation, missing/read-only states, preflight, relink, and
two-step detach/archive controls. Desktop/mobile browser checks, an unbundled
release build, and a native Wayland/D-Bus launch are verified. No source
directory, Codex-owned state, package, deployment, or release was changed.

### 7 — Conversation MVP

Start threads/turns in the verified attached directory, stream normalized
output, stop tasks, persist references, and expose model, reasoning, sandbox,
and approval controls from capabilities.

Milestone 7A native-runtime checkpoint implemented locally: one serialized
native owner validates the project association and re-resolves the exact
attached cwd before starting work; discovers the live model/reasoning catalog;
starts `thread/start` and `turn/start` with explicit sandbox and approval
controls; emits only bounded normalized lifecycle, message, reasoning-summary,
plan, and coarse-activity events; interrupts the exact owned turn; and stores
only Codex reference IDs and lifecycle metadata in QuireForge SQLite. Active
execution reserves the project against detach, archive, or relink races.
Approval requests block and close the task rather than being guessed or
auto-approved. Deterministic tests use a mock app-server and make no model call.

Milestone 7B adds an accessible responsive composer for a verified attached
project, with model and reasoning options taken from the normalized runtime
catalog, explicit filesystem and approval controls, pre-IPC rejection of the
unsafe unrestricted/no-approval combination, an ordered bounded event view,
stable terminal diagnostics, and exact app-owned conversation interruption.
Browser preview remains visibly non-interactive and never simulates a native
task. Session lifecycle is handled by Milestone 8; approval decisions, command
details, diffs, packaging, and deployment remain later milestones.

### 8 — Session Lifecycle

Resume, fork, archive, restore, title search, tabs, app grouping, and crash
recovery while keeping Codex authoritative.

Milestone 8A implements the native lifecycle and recovery boundary. Fixed Tauri
commands accept only app-owned UUIDv7 references and bounded prompts; Rust
reloads reference-only metadata, revalidates the exact attached cwd, reads the
owned thread, and invokes reviewed `thread/list`, `thread/read`,
`thread/resume`, `thread/fork`, `thread/archive`, and `thread/unarchive`
contracts. Fork lineage and archive timestamps are app metadata only. Startup
conservatively converts stale active rows to interrupted and clears active-turn
ownership. Deterministic tests prove exact-ID/cwd correlation, bounded listing,
no transcript/path exposure, no source or thread deletion, child cleanup, and
no live model use.

Milestone 8B adds a second bounded `thread/list` title-search projection after
complete reconciliation, then intersects both results with app-owned
references. React receives only transient normalized titles, app/project IDs,
parent-app lineage, controls, timestamps, and stable lifecycle states. Titles
are not persisted. The responsive interface groups sessions by project and
fork lineage, provides keyboard-accessible tabs, and wires resume, fork,
archive, and restore through exact app-owned IDs. Browser preview remains
honestly non-interactive, and archive never becomes deletion.

### 9 — Approvals and Command Presentation

Render exact scoped command, file, MCP/app, and permission requests; implement
decision handling, safe cancellation, terminal-control sanitization, redaction,
and recovery. Live activity rows must be selectable and expand in place to show
normalized real-time command/tool/file/process progress, comparable to Codex's
own disclosed activity presentation, without exposing raw protocol payloads,
credentials, unsafe terminal sequences, or unredacted private paths.

Milestone 9A implements the native security and contract checkpoint. The
serialized conversation owner recognizes only reviewed stable command, file,
and permission approval methods; correlates the exact native thread, turn,
request, and item; and exposes only app-owned UUIDv7 approval/activity IDs.
Approve, decline, and cancel are bounded decisions. Session-wide acceptance,
policy amendments, unstable write-root grants, and unsupported request types
remain unavailable. Turn-scoped permission profiles are strictly parsed, and
cancel resolves the request before interrupting the exact turn.

Activity schema version 2 provides stable IDs, safe titles/details, exit codes,
bounded command-output and MCP-progress deltas, and approval requested/resolved
events. Native presentation strips terminal and bidirectional controls, redacts
credential-shaped values, reduces paths to project-relative or
`[outside project]`, buffers output to line boundaries, and discards raw tool
arguments and file diffs. Pending approval remains ephemeral and uses existing
conservative crash recovery; no database migration or sensitive persistence is
added.

Milestone 9B aggregates activity lifecycle and bounded output deltas by stable
app activity ID. Each semantic button expands in place to show only normalized
kind, detail, live output, and exit status, retains its open state while polling
updates arrive, and caps the rendered activity/output history. A prominent
approval card displays the normalized reason and details and renders only the
approve, decline, or cancel choices advertised for the exact pending request.
Decision submission is single-flight, uses the fixed typed bridge, and pauses
polling so stale waiting snapshots cannot overwrite a completed decision.
Desktop and mobile fixtures verify keyboard semantics, accessibility, bounded
layout, exact app-ID submission, and duplicate-submission prevention.

### 10 — Git and Diff Review

Add status, branch, changed-file list, diff viewer, inline review context,
editor integration, and explicit stage/revert/commit workflows.

Milestone 10A implements the read-only checkpoint. Three fixed Tauri commands
accept only an app-owned project ID plus a normalized current-status path and
closed staged/worktree area. Native code revalidates the attachment on every
operation, runs shell-free Git with bounded environment/output/time, limits
status to the attached directory, discards object IDs and raw headers, and
rejects escaping, deceptive, symlink, conflicted, submodule, or stale targets.
The responsive interface presents branch divergence, changed files, staged and
working-tree selections, normalized line-numbered diffs, binary/truncated
states, refresh, and an explicit revalidated default-editor handoff. Browser
preview never simulates repository data, and no Git or diff state is persisted.

Milestone 10B implements explicit stage, unstage, revert, and commit workflows.
Preview accepts only a closed operation with an app-owned project ID and either
one normalized attachment-relative path or one bounded commit message. Rust
revalidates writable Git/worktree identity, reserves the project against Codex,
captures exact evidence, and retains the plan behind a five-minute in-memory
UUIDv7. Confirmation consumes only that token, revalidates the evidence, and
checks exact postconditions; React cannot resubmit paths or messages.

Stage/unstage preserve exact prior index entries for failure rollback. Revert
is limited to reviewed tracked regular-file modifications of at most one MiB
and offers a 30-minute single-use, process-local atomic recovery. Commit refuses
staged paths outside the attachment, conflicts, submodules, repository
operations in progress, missing repository-local identity, oversized content,
and high-confidence secrets in files, filenames, or the message. Git plumbing
creates the reviewed tree without hooks, signing, editors, prompts, or
global/system configuration, then updates `HEAD` with expected-old evidence and
checks the final reference/index state. Branch/worktree/remote mutation, push,
pull, reset, checkout, stash, arbitrary Git commands, packages, deployment, and
release remain separately gated. See
[ADR 0013](DECISIONS/0013-reviewed-git-mutation-boundary.md).

### 11 — Worktrees and Parallel Work

Create/attach isolated worktrees, run concurrent threads, display status,
detect conflicts, and make cleanup explicit and safe.

Milestone 11A implements the managed-worktree foundation. A fixed native
inventory command accepts only an app-owned project ID and normalizes
`git worktree list --porcelain -z` without exposing object IDs, raw stderr, or
Git configuration. Each managed or attached worktree is also an ordinary
QuireForge project linked to its canonical source by schema migration 4.
Externally discovered worktrees remain unselectable until the user chooses the
exact directory with the native picker.

Creation accepts only a bounded new branch name. Rust generates the destination
beneath private app storage, captures source repository identity and current
HEAD internally, disables hooks and configured checkout filters, and retains a
five-minute one-use confirmation. Confirmation reserves every app-owned project
in the source repository group, revalidates identity, HEAD, branch absence, and
destination, then uses one fixed shell-free `git worktree add` workflow.
Metadata registration is transactional. If Git succeeds and registration
fails, the worktree is reported as recoverable and deliberately left in place.

Milestone 11B replaces the single active-process slot with a bounded registry
of at most four independently locked conversations. Starts reserve their exact
project before process creation, duplicate work in one project fails closed,
and poll, approval, and interruption route only through an app-owned
conversation ID. A strict normalized registry lets the webview recover active
tasks after refresh without receiving Codex IDs, cwd, commands, process
metadata, or raw protocol messages.

React polls each active task independently and presents one aggregate worktree
monitor. Selecting a row opens the existing expandable live activity stream;
read-only Git snapshots supply only normalized changed-file and conflict
counts. Process ownership does not survive an application restart, so stale
active records follow the existing interrupted-state recovery rule. Milestone
11B performs no conflict resolution or Git mutation.

Milestone 11C adds separately gated recovery and cleanup. Native inventory
issues opaque recovery IDs only for unregistered linked worktrees inside the
exact private managed-storage slot. Recovery registers the retained checkout
without changing Git or files. Cleanup accepts only app-owned project IDs and
removes only a clean, unlocked, non-current `managed` checkout after repository-
group reservation and confirmation-time relation, identity, branch, `HEAD`,
and status revalidation. Git removal never uses force, preserves the branch,
and must satisfy explicit path/inventory/branch postconditions before a
transaction detaches and archives project metadata.

If Git succeeds but metadata retirement fails, the missing managed entry can be
reviewed again for metadata-only finalization; no filesystem mutation is
retried. Attached/external worktrees, direct directory deletion, branch
deletion, conflict resolution, arbitrary Git arguments, and repository-wide
`git worktree prune` remain unavailable. See
[ADR 0014](DECISIONS/0014-managed-worktree-foundation.md),
[ADR 0015](DECISIONS/0015-bounded-parallel-worktree-execution.md), and
[ADR 0016](DECISIONS/0016-safe-managed-worktree-cleanup.md).

### 12 — Integrated Terminal

Implement Rust PTY lifecycle, tabs, verified project cwd startup, resize/input,
background processes, environment handling, and terminal safety tests.

Implemented and verified: a dedicated Rust `portable-pty` service owns up to eight
app-generated UUIDv7 terminal sessions, starts only after project reservation
and cwd identity revalidation, clears and reconstructs a narrow noncredential
environment, transports bounded base64 bytes, applies typed resize/input, and
ends the complete owned Linux session through bounded HUP/TERM/KILL cleanup.
React uses stable xterm APIs with the DOM renderer, inaccessible browser-preview
controls, responsive tabs, explicit close confirmation, and a visible warning
that terminal commands run with the Linux account rather than Codex approval
policy. SQLite migration 5 persists only presentation state and marks stale
sessions interrupted; input, output, history, cwd, environment, TTY, and
process/session IDs are never stored or exposed. Closing a tab does not delete
project files. Daemons that deliberately create a new session, remote shells,
shell selection, process inspection, command approvals, and terminal content
recovery remain outside this milestone. See
[ADR 0017](DECISIONS/0017-native-integrated-terminal.md).

Publication completed through
[PR #27](https://github.com/codeframe78/quireforge/pull/27) and successful
pull-request and `main` repository checks. No package or release was produced.

### 13 — Integration Discovery and Compatibility Layer

Normalize apps/connectors, plugins, marketplaces, skills, MCP, policy, runtime
requirements, scopes, and health. Use stable routes and deterministic mock
catalogs; preserve unknown/blocked/degraded states.

Milestone 13A refreshes the installed Codex 0.145.0 schema evidence and accepts
the category-preserving `codex-integration-v1` contract. It distinguishes
upstream availability from QuireForge implementation, defines bounded scope,
permission, requirement, policy, and health states, and validates a documented
client-owned dynamic-tool lifecycle through `thread/start` and
`item/tool/call`. This checkpoint is contract-only: it does not expose a live
catalog, install or authorize integrations, register the selector tool, or add
an Integration Center UI. See
[ADR 0018](DECISIONS/0018-normalized-integration-contracts.md).

Milestone 13B implements the read-only native discovery/normalization service,
strict IPC, exact CLI-minor routing, bounded cache invalidation, and
deterministic partial-failure tests against these contracts. It uses supported
app-server methods for connector, skill, MCP, and policy reads and stable CLI
JSON commands for plugin and marketplace discovery; experimental plugin RPCs,
raw paths/URLs/configuration, account identity, and tool arguments do not cross
the native boundary. Mutation and the user-facing Integration Center remain
Milestone 14.

Milestone 13A publication completed through
[PR #32](https://github.com/James-Jennison/quireforge/pull/32), merge commit
`7bc5f5f`, and successful pull-request and `main` repository checks. This
checkpoint produced no live integration discovery, installation, authorization,
package, release, or deployment.

Milestone 13B publication completed through
[PR #34](https://github.com/James-Jennison/quireforge/pull/34), merge commit
`007f5b7`, and successful pull-request workflow
[`29890814046`](https://github.com/James-Jennison/quireforge/actions/runs/29890814046)
and post-merge `main` workflow
[`29890942589`](https://github.com/James-Jennison/quireforge/actions/runs/29890942589).
This checkpoint made no integration, account, package, release, deployment, or
hosting mutation.

### 14 — Integration Center and Installation Workflows

Implement browse/search/filter/details, permission review, CLI-backed plugin and
marketplace operations, supported connector/MCP authorization handoff,
enable/disable/update/remove where validated, health/troubleshooting, prompt
mentions, and supply-chain warnings.

Completion requires a supported test-plugin lifecycle and an honest limitation
when connector management is unavailable.

Milestone 14A implements the native plugin and marketplace lifecycle only. It
uses the reviewed stable CLI 0.145.x JSON commands for plugin install/remove
and marketplace add/remove/upgrade, never the under-development app-server
plugin-management RPCs. Every operation starts with a fresh normalized catalog
and policy read, resolves an opaque entry ID to native-held CLI evidence,
reviews the source class and normalized permissions/warnings, and creates a
five-minute one-use UUIDv7 confirmation. Confirmation serializes mutation,
rechecks the CLI minor, policy, normalized entry, and exact raw evidence, then
accepts only the closed documented JSON result and verifies the resulting
catalog state. Repository marketplace adds accept only `owner/repository` plus
a 40- or 64-hex pinned reference. Raw paths, URLs, CLI arguments/results,
configuration, and credentials never cross IPC.

The deterministic test suite uses temporary state, while the ignored explicit
real-CLI proof runs a local fixture marketplace and plugin under temporary
`CODEX_HOME` and `HOME`; it does not read or change personal Codex state. No
connector authorization, MCP configuration, skill configuration, plugin
enable/disable, generic command execution, Integration Center UI, package,
release, deployment, or personal integration mutation is included. See
[ADR 0019](DECISIONS/0019-confirmed-integration-mutations.md).

Milestone 14A publication completed through
[PR #36](https://github.com/James-Jennison/quireforge/pull/36), implementation
commit `e46cb5c`, merge commit `a20919f`, successful pull-request workflow
[`29893588842`](https://github.com/James-Jennison/quireforge/actions/runs/29893588842),
and post-merge `main` workflow
[`29893692681`](https://github.com/James-Jennison/quireforge/actions/runs/29893692681).
This checkpoint made no personal integration or account mutation and produced
no package, release, deployment, or hosting change.

Milestone 14B implements the user-facing browse/search/filter/details and
permission-review Integration Center over the normalized discovery and 14A
mutation contracts. It exposes only capability-ready fixed operations, uses a
pinned-reference form for repository marketplace adds, presents separate hook
trust and supply-chain warnings, and keeps unavailable management explicit.
Desktop/mobile, keyboard, overflow, and automated accessibility checks pass
locally and in hosted CI. Publication completed through
[PR #38](https://github.com/James-Jennison/quireforge/pull/38), implementation
commit `42cff70`, merge commit `93e585f`, successful pull-request workflow
[`29918268480`](https://github.com/James-Jennison/quireforge/actions/runs/29918268480),
and post-merge `main` workflow
[`29918513538`](https://github.com/James-Jennison/quireforge/actions/runs/29918513538).
No personal integration or account state was read or mutated, and no package,
release, deployment, or hosting change was made. A later separately gated
Milestone 14 checkpoint must handle only supported connector/MCP authorization,
enable/disable or update flows, health/troubleshooting, and prompt mentions;
unsupported management must remain visibly unavailable.

Milestone 14C implements the supported portion of that next gate. A closed
native preview/confirm service authorizes a connector only through the official
URL returned by Codex, starts MCP OAuth only through
`mcpServer/oauth/login`, and changes skill enablement only through
`skills/config/write` with an exact postcondition. Browser handoff URLs, raw
connector IDs/paths, MCP names, and skill manifest paths remain native-only.
The Integration Center exposes those controls only for capability-ready,
eligible rows; explicit refresh rebuilds normalized health/catalog state.

New conversation turns may select up to eight authorized, enabled, healthy
connectors by opaque catalog ID. Native code re-resolves callable state and
constructs the documented `mention` plus `app://` path; the webview cannot
supply a path or raw Codex identifier. Generic connector installation or
configuration, plugin enable/disable, MCP add/remove/logout/configuration,
arbitrary health repair, and generic config writes remain unavailable. Routine
tests use deterministic fixtures only and do not read or mutate personal Codex
or integration state. See
[ADR 0020](DECISIONS/0020-confirmed-integration-authorization-and-controls.md).
Publication completed through
[PR #41](https://github.com/James-Jennison/quireforge/pull/41), implementation
commit `86a114d`, merge commit `e4d8333`, successful pull-request workflow
[`29950963936`](https://github.com/James-Jennison/quireforge/actions/runs/29950963936),
and post-merge `main` workflow
[`29951143628`](https://github.com/James-Jennison/quireforge/actions/runs/29951143628).
No personal integration or account state was read or mutated, and no package,
release, deployment, or hosting change was made.

### 15 — File Previews and Desktop Integration

Split this milestone so each security/desktop boundary is independently
reviewable:

- **15A — safe project-file previews:** use a native picker and opaque project
  ID; revalidate attachment identity, containment, symlink/regular-file state,
  and opened file identity. Return only attachment-relative names and bounded
  normalized UTF-8 text, PNG/JPEG data, or metadata-only PDF state through a
  strict contract. Browser preview cannot select or read local files. See
  [ADR 0021](DECISIONS/0021-safe-project-file-previews.md).
- **15B — drag/drop and conversation attachments:** define source ownership,
  staging/retention, model-interface support, explicit send semantics, size and
  count limits, cancellation, and cleanup without turning drag/drop into a
  general path bridge. The implemented checkpoint accepts only PNG/JPEG,
  disables Tauri's default path-bearing drag/drop events, stages validated
  browser bytes or one-use native-captured Linux file drops in private app
  data, sends only native `localImage` paths, and retains each consumed copy
  until its turn is terminal. See
  [ADR 0022](DECISIONS/0022-bounded-conversation-image-attachments.md).
- **15C — desktop handoffs and Linux verification:** add notifications and
  reviewed editor/open-with behavior, then verify native picker/handoff behavior
  on supported Wayland and X11 sessions. External destinations stay visible and
  allowlisted; no generic opener or arbitrary command IPC is allowed. The code
  checkpoint uses native-held one-use preview actions, an explicit system-
  default-application review, and fixed privacy-safe background notifications;
  the completed final Linux display-session gate is recorded below. See
  [ADR 0023](DECISIONS/0023-reviewed-desktop-handoffs-and-notifications.md).

Milestones 15A–15C are implemented and verified locally. The 15C handoff and
notification checkpoint uses the official Tauri notification plugin, a Linux
binding already present in the Tauri stack, and no source-path persistence,
unrelated user-file access, billable model call, package, release, or
deployment. Its production native Wayland project/file/image picker, bounded-
preview, real Nautilus-drop, and fixed-copy notification evidence is complete
against disposable app data. Complete XWayland and true-X11 picker/preview/
default-application/attachment paths remain separately recorded. Milestone 17
is the next planned implementation milestone.

### 16 — Complete the Webuzo-Hosted Static Website

Milestone 16A reconciles Home, Features, Integrations, Downloads, Installation,
Documentation, Compatibility, Roadmap, Releases, Security/Privacy, Development,
FAQ, Troubleshooting, and About for a public site backed by private source. It
retains the approved design, removes private repository/activity links,
supersedes the unimplemented Cloudflare Pages plan, and produces a verified
Apache-compatible static artifact.

Milestone 16B created the isolated Webuzo origin and staged the reviewed
artifact without public DNS. Trusted origin TLS, route/header validation, and
rollback rehearsal passed. Milestone 16C separately activated the canonical
hostname after owner approval. Public DNS, Full (Strict), scoped HSTS, live
route/accessibility checks, 100/100/100/100 mobile and desktop Lighthouse
results, and post-launch recovery verification passed. Milestone 16D then
completed provider-managed automatic origin TLS and renewal validation. Private
provider identifiers and operational diagnostics remain outside source
control.

### 17 — Scheduled Tasks and Advanced Features

Implement only capabilities exposed through supported interfaces. Distinguish
local scheduling from hosted scheduling and defer unsupported features.

Milestone 17A implements the supported read-only portion. The native
integration service queries stable `plugin/read` only for installed, enabled
plugins already established by the CLI catalog. Raw marketplace roots and
lookup values remain native-only. Scheduled task names and prompts are treated
as untrusted plugin content, normalized into bounded inert previews, and paired
with a strict hourly/daily/weekdays/weekly schedule. The existing integration
catalog read/refresh IPC advances to schema version 2, and the Scheduled
workspace exposes no action controls.

The reviewed stable request set and plugin CLI provide no task create, edit,
enable, run, pause, or delete route. QuireForge therefore implements no local
scheduler, hosted scheduler, official-client automation, or private web
integration. Those capabilities remain deferred pending a separately reviewed
supported interface and explicit approval. See
[ADR 0025](DECISIONS/0025-read-only-scheduled-task-catalog.md).

### 18 — Agent-Directed Model and Reasoning Selection

Milestone 18 is implemented and verified locally. It adds a typed, app-owned
selector-control boundary that lets Codex inspect the normalized `model/list`
catalog, current effective choice, pending next-turn choice, and user policy.
Codex may request at most one model/reasoning change per completed turn with a
short rationale. Native code revalidates the request against a fresh advertised
catalog and the configured policy before applying it to the next `turn/start`;
the executing turn never claims to replace itself.

Expose explicit Manual, Recommend, and Automatic ownership modes. Automatic
mode requires deliberate user opt-in and an allowlist or model/reasoning
ceiling. A user lock or later manual choice always wins. The UI must distinguish
effective from pending selection and show that Codex requested the change and
why. Prevent repeated oscillation and silent cost escalation, and persist only
QuireForge policy and bounded provenance—not prompts, account identifiers,
credentials, or raw protocol payloads.

Use only documented Codex interfaces and normalized typed IPC. Do not automate
the ChatGPT/Codex web selector, call private endpoints, or edit Codex-owned
configuration behind the user's back. Validate the exact supported
request/response lifecycle against the installed app-server schemas. If a
stable or explicitly accepted experimental control path is unavailable,
degrade to visible recommendation-only behavior rather than fabricating
automatic control. Deterministic mocks must cover prompt-injection attempts,
stale/unadvertised models, unsupported efforts, manual locks, policy ceilings,
one-change-per-turn enforcement, restart behavior, and next-turn application.

The implementation registers the closed `quireforge_model_selector` dynamic
tool, keeps exact request/thread/turn/call correlation native, stages a valid
request only after successful turn completion, persists bounded policy and
provenance separately from the effective choice, and revalidates immediately
before resume. Strict schema-v3 conversation/session contracts and the
`model_selection_update` command expose only app-owned state. The responsive UI
shows effective versus pending selection, provenance and rationale, manual
override, recommendation acceptance/dismissal, automatic allowlists/ceilings,
and the user lock. Registration rejection produces an explicit
recommendation-only state. See
[ADR 0026](DECISIONS/0026-policy-bounded-next-turn-selection.md).

### 19 — Security, Accessibility, and Performance Hardening

Revisit the threat model; audit secret handling, injection, filesystem races,
integration supply chain, credentials, Tauri permissions/CSP, accessibility,
performance, reliability, and crash recovery.

### 20 — Packaging and Release Automation

Produce AppImage and Debian packages on an appropriate baseline, checksums,
release workflows, install/upgrade/uninstall tests, and website download data.
Do not publish a release without approval.

### 21 — Beta Package Publication and Download Activation

Run final package and supported-platform QA; confirm the approved distribution
location, release artifact, checksums, provenance, download data, and rollback;
then request beta-publication approval. Update the already hosted website only
with the approved package metadata and verify downloads, installation guidance,
known limitations, and checksums. Website updates and application release
publication remain independently approval-gated.

## Forecast policy

The initial whole-project estimate is several hundred active engineering hours
and many real-world weeks. Each milestone receives a refreshed range covering
inspection, implementation, builds, tests, debugging, visual verification,
documentation, review, and commit preparation before work begins. Forecasts
will be compared with measurable actuals in milestone completion reports.
