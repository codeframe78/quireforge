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
source content. Application packages and external provider settings remain
milestone- and approval-gated.

## Status

| Milestone | Scope | Size | Status |
|---:|---|---|---|
| 0 | Existing project and feasibility discovery | Very large | Complete; merged to `main` |
| 1 | QuireForge rename, move, GitHub migration, and governance closure | Medium | Complete; merged to `main` |
| 2 | QuireForge brand and Cloudflare website foundation | Large | Complete; merged to `main`; not deployed |
| 3 | Desktop scaffold consolidation | Large | Complete; merged to `main`; not packaged |
| 4 | Codex process adapter and contracts | Very large | Complete; merged to `main` |
| 5 | Authentication and onboarding | Medium | Complete; merged to `main` |
| 6 | Projects and direct local-directory attachment | Very large | Complete; merged to `main` |
| 7 | Conversation MVP | Very large | Planned |
| 8 | Session lifecycle and crash recovery | Large | Planned |
| 9 | Approvals and command presentation | Large | Planned |
| 10 | Git status and diff review | Large | Planned |
| 11 | Worktrees and parallel work | Very large | Planned |
| 12 | Integrated terminal | Large | Planned |
| 13 | Integration discovery and compatibility | Very large | Planned |
| 14 | Integration Center and installation workflows | Very large | Planned |
| 15 | File previews and desktop integration | Large | Planned |
| 16 | Complete Cloudflare Pages website | Very large | Planned |
| 17 | Scheduled tasks and advanced supported features | Medium–Large | Planned/dependency-gated |
| 18 | Security, accessibility, and performance hardening | Very large | Planned |
| 19 | Packaging and release automation | Large | Planned |
| 20 | Cloudflare Pages production deployment and beta release | Very large | Planned/approval-gated |

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

### 8 — Session Lifecycle

Resume, fork, archive, restore, title search, tabs, app grouping, and crash
recovery while keeping Codex authoritative.

### 9 — Approvals and Command Presentation

Render exact scoped command, file, MCP/app, and permission requests; implement
decision handling, safe cancellation, terminal-control sanitization, redaction,
and recovery.

### 10 — Git and Diff Review

Add status, branch, changed-file list, diff viewer, inline review context,
editor integration, and explicit stage/revert/commit workflows.

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
