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
`/mnt/faststorage/quireforge`. The existing GitHub repository was renamed in
place to `codeframe78/quireforge`; neither operation authorized a push, merge,
website deployment, or release.

Migration status: the tracked identity contract, authoritative naming audit,
in-place GitHub repository rename, local working-copy handoff, and core vector
brand sources are complete. Milestone 1 also established the Apache-2.0 license,
repository guidance, contribution/security/conduct/support policies, issue and
pull-request templates, dependency automation, and initial repository CI. The
work through Milestone 6 is merged on `main`. Milestone 2 added the local static
website, production web exports, and automated website quality gates without
creating a Cloudflare project or deployment. Milestone 3 added the locally verified Tauri
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

## Status

| Milestone | Scope                                                             | Size         | Status                                                                         |
| --------: | ----------------------------------------------------------------- | ------------ | ------------------------------------------------------------------------------ |
|         0 | Existing project and feasibility discovery                        | Very large   | Complete; merged to `main`                                                     |
|         1 | QuireForge rename, move, GitHub migration, and governance closure | Medium       | Complete; merged to `main`                                                     |
|         2 | QuireForge brand and Cloudflare website foundation                | Large        | Complete; merged to `main`; not deployed                                       |
|         3 | Desktop scaffold consolidation                                    | Large        | Complete; merged to `main`; not packaged                                       |
|         4 | Codex process adapter and contracts                               | Very large   | Complete; merged to `main`                                                     |
|         5 | Authentication and onboarding                                     | Medium       | Complete; merged to `main`                                                     |
|         6 | Projects and direct local-directory attachment                    | Very large   | Complete; merged to `main`                                                     |
|         7 | Conversation MVP                                                  | Very large   | Complete; merged to `main`                                                     |
|         8 | Session lifecycle and crash recovery                              | Large        | Complete; merged to `main`                                                     |
|         9 | Approvals and command presentation                                | Large        | Complete and verified; publication recorded in repository history              |
|        10 | Git status, diff review, and controlled mutations                  | Large        | Complete and verified; publication tracked by this milestone change             |
|        11 | Worktrees and parallel work                                       | Very large   | Planned                                                                        |
|        12 | Integrated terminal                                               | Large        | Planned                                                                        |
|        13 | Integration discovery and compatibility                           | Very large   | Planned                                                                        |
|        14 | Integration Center and installation workflows                     | Very large   | Planned                                                                        |
|        15 | File previews and desktop integration                             | Large        | Planned                                                                        |
|        16 | Complete Cloudflare Pages website                                 | Very large   | Planned                                                                        |
|        17 | Scheduled tasks and advanced supported features                   | Medium–Large | Planned/dependency-gated                                                       |
|        18 | Security, accessibility, and performance hardening                | Very large   | Planned                                                                        |
|        19 | Packaging and release automation                                  | Large        | Planned                                                                        |
|        20 | Cloudflare Pages production deployment and beta release           | Very large   | Planned/approval-gated                                                         |

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

### 12 — Integrated Terminal

Implement Rust PTY lifecycle, tabs, verified project cwd startup, resize/input,
background processes, environment handling, and terminal safety tests.

### 13 — Integration Discovery and Compatibility Layer

Normalize apps/connectors, plugins, marketplaces, skills, MCP, policy, runtime
requirements, scopes, and health. Use stable routes and deterministic mock
catalogs; preserve unknown/blocked/degraded states.

### 14 — Integration Center and Installation Workflows

Implement browse/search/filter/details, permission review, CLI-backed plugin and
marketplace operations, supported connector/MCP authorization handoff,
enable/disable/update/remove where validated, health/troubleshooting, prompt
mentions, and supply-chain warnings.

Completion requires a supported test-plugin lifecycle and an honest limitation
when connector management is unavailable.

### 15 — File Previews and Desktop Integration

Add safe previews, drag/drop and attachments, notifications, editor/open-with,
and Wayland/X11 verification.

### 16 — Complete Cloudflare Pages Website

Build Home, Features, Integrations, Downloads, Installation, Documentation,
Compatibility, Roadmap, Changelog, Security/Privacy, Contributing, FAQ,
Troubleshooting, About, authentic screenshots, and comprehensive
production-origin/responsive/accessibility validation. Build a verified static
artifact and deploy only to separately approved non-production staging.

### 17 — Scheduled Tasks and Advanced Features

Implement only capabilities exposed through supported interfaces. Distinguish
local scheduling from hosted scheduling and defer unsupported features.

### 18 — Security, Accessibility, and Performance Hardening

Revisit the threat model; audit secret handling, injection, filesystem races,
integration supply chain, credentials, Tauri permissions/CSP, accessibility,
performance, reliability, and crash recovery.

### 19 — Packaging and Release Automation

Produce AppImage and Debian packages on an appropriate baseline, checksums,
release workflows, install/upgrade/uninstall tests, and website download data.
Do not publish a release without approval.

### 20 — Cloudflare Pages Production Deployment and Beta Release

Run final website/package QA; confirm project, custom domain, artifact, DNS
cutover, and rollback; request approval for Cloudflare production deployment;
verify DNS, HTTPS, headers, live assets, the unaffected main site, and rollback;
then request separate beta-release approval and verify downloads and checksums.
Deployment and release remain independently approval-gated.

## Forecast policy

The initial whole-project estimate is several hundred active engineering hours
and many real-world weeks. Each milestone receives a refreshed range covering
inspection, implementation, builds, tests, debugging, visual verification,
documentation, review, and commit preparation before work begins. Forecasts
will be compared with measurable actuals in milestone completion reports.
