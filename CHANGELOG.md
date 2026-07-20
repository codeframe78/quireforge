# Changelog

All notable project changes will be documented here. The project has not
released a usable application.

## Unreleased

### Added

- Dedicated public repository and an explicit unofficial-project disclaimer.
- Milestone 0 Codex integration, compatibility, feature-parity, architecture,
  threat-model, GitHub Pages, roadmap, and architecture-decision documentation.
- Permanent QuireForge identity contract covering application, package,
  repository, GitHub Pages, integration-client, and XDG storage identifiers.
- Original path-based QuireForge mark, wordmark, light/dark lockups, favicon,
  application-icon source, social card, palette, and brand usage guidance.
- Cloudflare Pages capability findings and a deployment plan.
- Sanitized owner-mediated Cloudflare account audit covering the Free plan,
  Workers & Pages availability, DNS, managed TLS, and security settings.
- ADR 0006 selecting Cloudflare Pages as the production website host and
  authoritative DNS.
- Apache License 2.0 and repository-wide contributor guidance.
- Security, contribution, conduct, support, issue, and pull-request policies.
- GitHub Actions dependency updates and a minimum-permission repository-checks
  workflow pinned to a reviewed checkout revision.
- A dependency-free repository validator for required files, local links,
  QuireForge identity contracts, SVG XML, text encoding, and high-confidence
  secret patterns.
- A pinned pnpm monorepo and Astro 7 static website under `apps/website` with
  15 public pages, a custom 404, sitemap, robots policy, manifest, and canonical
  and social metadata.
- Reusable QuireForge website components, centralized light/dark design tokens,
  responsive navigation, visible keyboard focus, reduced-motion support, and
  generated favicon, application-touch, and social-card assets derived from the
  approved vector sources.
- Deterministic website type, lint, format, unit, artifact, route, responsive,
  theme, and axe-core accessibility checks in local scripts and minimum-
  permission GitHub Actions.
- Cloudflare Pages security headers with a strict static-site content policy;
  HSTS remains intentionally deferred until live HTTPS is verified.
- Website build and testing documentation with an explicit no-deployment
  boundary.
- A pinned Tauri 2, React 19, TypeScript, Vite, and Rust desktop package under
  `apps/desktop`, with the accepted Linux application identity and original
  QuireForge icon exports.
- Generalized local build-performance and milestone-forecast histories for
  system-calibrated planning.
- A responsive, accessible light/dark desktop shell and one versioned
  `desktop_bootstrap` command validated against a shared Rust/TypeScript fixture.
- Desktop type, lint, format, unit, native contract, Clippy, build, responsive,
  theme, overflow, and axe-core accessibility gates in local scripts and CI.
- A versioned `CodexBackend` boundary, fixed-command CLI detection, supervised
  JSONL app-server lifecycle, normalized capability/model contracts, and
  explicit unavailable/degraded diagnostics.
- Deterministic Codex mocks, bounded protocol failure tests, and a reproducible
  generator that commits only the reviewed initialize and `model/list` schema
  subset for Codex CLI 0.144.6.
- A narrow `codex_runtime_probe` Tauri command and strict TypeScript runtime
  schema that prevent raw app-server, account, installation, path, or user-agent
  fields from reaching React.
- A Codex-owned authentication service with normalized account status,
  allowlisted browser/device handoffs, exact login correlation, cancellation,
  explicit two-step logout, stable redacted diagnostics, and deterministic
  lifecycle/failure tests.
- An accessible Milestone 5 onboarding panel that never displays or persists
  email, account IDs, tokens, raw errors, or completed sign-in URLs.
- A native Milestone 6 project core with app-owned migrated SQLite metadata,
  UUIDv7 identities, selected/resolved directory evidence, Git/worktree and
  project-instruction detection, explicit attach/relink/detach/archive
  lifecycle operations, and fail-closed cwd preflight.
- Deterministic project security tests for symlink and configuration changes,
  duplicate roots, linked worktrees, invalid Git pointers, read-only and
  missing directories, malformed IDs, schema ownership, metadata permissions,
  and the no-source-deletion boundary.
- A strict normalized project-workspace contract, fixed-command TypeScript
  bridge, and accessible responsive project UI for native selection,
  confirmation, missing/read-only states, preflight, relink, and two-step
  detach/archive actions.
- A serialized native Milestone 7A conversation service that revalidates the
  attached cwd, starts a supported Codex thread and turn with explicit model,
  reasoning, sandbox, and approval controls, normalizes bounded stream events,
  and interrupts the exact owned turn.
- Reference-only conversation persistence, active-project execution
  reservation, reviewed Codex 0.144.6 thread/turn schemas, strict Rust and
  TypeScript contracts, and deterministic lifecycle, cancellation, policy,
  mismatch, path-boundary, and redaction tests.
- A responsive Milestone 7B conversation workspace with a bounded task
  composer, runtime-derived model/reasoning choices, explicit filesystem and
  approval controls, normalized progress and response rendering, stable
  diagnostics, and exact app-owned stop behavior.
- Deterministic conversation UI tests for prerequisite gating, unsafe-policy
  rejection, start/poll/terminal transitions, event deduplication and bounds,
  browser-preview honesty, responsive layout, and accessibility.
- A native Milestone 8A lifecycle boundary for app-reference-only session list,
  read, resume, fork, archive, and restore operations against revalidated
  attached directories and supervised Codex app-server processes.
- SQLite schema version 3 with bounded parent-app lineage and archive timestamps,
  plus startup reconciliation that marks stale active work interrupted and
  clears obsolete active-turn ownership without deleting Codex or project data.
- Strict Rust/TypeScript lifecycle fixtures, fixed Tauri commands, reviewed
  Codex 0.144.6 lifecycle schemas, bounded exact-cwd reconciliation, and
  deterministic mismatch, recovery, fork-lineage, archive/restore, child-
  cleanup, and raw-identity/path rejection tests.
- A Milestone 8B session-history workspace with bounded Codex-authoritative
  title search, project and fork grouping, keyboard-operable tabs, transient
  titles, and accessible exact-reference resume, fork, archive, and restore
  controls.
- Deterministic native, component, shell-integration, responsive, overflow, and
  axe-core coverage proving that title filtering does not corrupt complete
  reconciliation or expose paths, Codex IDs, previews, transcripts, or raw
  protocol records.

### Changed

- Adopted **QuireForge** as the permanent product name, with the tagline
  “Build boldly. Work locally.” The former “Codex Linux Workbench” name was a
  temporary discovery-stage label.
- Updated the planned repository-project website to
  `https://codeframe78.github.io/quireforge/` with base path `/quireforge/`.
- Replaced the initially proposed `quireforge.desktop` filename with the
  freedesktop-aligned `io.github.codeframe78.QuireForge.desktop`; the executable
  and Debian package remain `quireforge`.
- Defined app-server initialization as `clientInfo.name = "quireforge"`,
  `clientInfo.title = "QuireForge"`, and the real application version.
- Selected `https://quireforge.jamesjennison.net` as the production website;
  GitHub remains the source, CI, issue, and release host.
- Selected Cloudflare Pages as the production host. Codex changed no provider,
  DNS, project, or production setting as part of that decision.
- Recorded the owner's separately completed move of authoritative DNS to
  Cloudflare and the temporary absence of the QuireForge hostname in the new
  zone; no DNS record was created by Codex.
- Recorded owner confirmation that Cloudflare two-factor authentication is now
  enabled without retaining factor or recovery details.
- Removed obsolete provider-specific hosting audits and deployment plans from
  the current project tree.
- Completed Milestone 0 feasibility documentation locally; no hosting project,
  DNS record, deployment, push, or release was created by Codex.
- Refreshed account-scoped Codex discovery without publishing catalog entries
  or integration identifiers.
- Reconciled the completed QuireForge path/repository migration, classified all
  remaining former-name references as intentional history, confirmed that no
  pre-release application data requires migration, and completed Milestone 1
  locally without pushing or changing repository settings.
- Completed the Milestone 2 brand and static website foundation locally without
  creating a Cloudflare project, changing DNS, deploying, pushing, or merging.
- Completed the Milestone 3 desktop scaffold locally, including an unbundled
  Wayland launch and runtime application-identity check, without implementing
  Codex workflows, packaging, pushing, or merging.
- Completed the Milestone 4 Codex process adapter locally, including a
  non-billable live app-server probe, bounded failure recovery, exact process
  cleanup, and normalized desktop status without login, conversation turns,
  configuration writes, packaging, pushing, or merging.
- Completed Milestone 5 authentication and onboarding locally. A non-mutating
  live `account/read` probe verified normalized state and exact child cleanup;
  no login, browser authorization, logout, model turn, push, package, or
  deployment was performed by Codex.
- Completed Milestone 6 project attachment locally, including its native
  storage/identity core, strict frontend boundary, accessible workspace, browser
  verification, unbundled release build, and isolated native launch. No source
  directory, Codex state, package, deployment, or release was changed.
- Completed the Milestone 7A native conversation-runtime checkpoint locally.
  No live model turn, approval decision, Codex-owned session mutation, package,
  deployment, or release was performed.
- Completed the Milestone 7B conversation UI and native-shell integration
  locally. No live model call, approval decision, deployment, package, or
  release was performed.
- Completed the Milestone 8A native session-lifecycle and crash-recovery
  checkpoint locally. No live model call, approval decision, thread deletion,
  project-file mutation, deployment, package, or release was performed; the
  history/search/tabs interface remains Milestone 8B.
- Completed the Milestone 8B history/search/tabs interface locally. Titles
  remain transient, lifecycle actions use app-owned IDs, and no live model
  call, approval decision, deletion, deployment, package, or release was
  performed.

### Migration note

- The existing Git history and discovery work were migrated in place; no
  replacement project or rewritten history was involved.
- The GitHub repository was renamed in place to `codeframe78/quireforge`, and
  the intact working copy moved to `/mnt/faststorage/quireforge`, through
  separate approval-gated operations.
- No released or development application data was detected under the temporary
  identity, so there is currently no user configuration to move. Future
  releases must preserve old data and never modify Codex-owned authentication
  or sessions.

### Known limitations

- The desktop adapter, Codex-owned authentication workflow, project attachment,
  conversation MVP, native session lifecycle and history interface, and website
  are locally verified, but approval/expanded process details, installable
  packages, website deployment, and a release workflow do not exist.
- Integration compatibility is based on Codex CLI 0.144.6 and must be probed at
  runtime.
