# Testing QuireForge

Status: Milestones 2–8A establish repository, website, desktop frontend, native
contract, Codex adapter, authentication, project attachment, native
conversation/runtime lifecycle, and Tauri build checks. PTY and broader Git
fixtures arrive with the milestones that introduce those systems.

## Repository, website, and desktop checks

Run these commands from the repository root after installing locked
dependencies:

```bash
python3 scripts/validate_repository.py
pnpm check
pnpm lint
pnpm format:check
pnpm test
pnpm build
pnpm validate:dist
pnpm rust:check
pnpm rust:clippy
pnpm rust:test
```

`pnpm validate` runs that non-browser sequence as one command. The checks cover
required repository files, secret-like tracked files, local documentation
links, QuireForge identity values, Astro and TypeScript correctness, linting,
formatting, content-model unit tests, the production build, routes, generated
assets, internal links, canonical URLs, the unofficial disclaimer, inline-code
restrictions, and version-controlled security headers. Desktop checks cover
strict TypeScript, linting, shared Rust/TypeScript IPC fixtures, frontend
behavior, Rust formatting, Clippy with warnings denied, native tests, and
compilation against the locked Cargo graph. Codex adapter tests cover CLI
version validation, deterministic mock snapshots, selected generated schemas,
response correlation, notification-payload discard, catalog normalization,
duplicate/default rejection, early exit, timeout, and child reaping.
Project-core tests cover transactional schema migration, forward-schema
refusal, app-data permissions, selected/resolved path identity, mount state,
Git repositories and linked worktrees, duplicate roots, confirmation-time
changes, relink/detach/archive behavior, and fail-closed cwd preflight.
Frontend project tests share a normalized fixture with Rust, reject unknown or
path-bearing bridge input, and cover confirmation, missing/read-only states,
relink, preflight, and two-step detach/archive controls.
Conversation tests cover verified-cwd start, live catalog validation, strict
control combinations, UUIDv7 correlation, bounded normalized events, exact
interrupt, project reservation, approval-blocked shutdown, protocol mismatch,
child reaping, and reference-only persistence. TypeScript tests reject cwd,
Codex thread/turn IDs, unknown fields, raw protocol payloads, and path-bearing
bridge input before native invocation.
Session-lifecycle tests cover schema migration, stale-turn crash reconciliation,
bounded exact-cwd list matching, owned-thread reads, resume, fork lineage,
archive/restore without deletion, mismatched-cwd rejection, project-reservation
release, child reaping, and shared strict Rust/TypeScript fixtures. They use
deterministic mock app-server processes and never read a personal transcript or
start a live model turn.

## Responsive browser and accessibility checks

Install the Playwright Chromium browser once, then run the suite:

```bash
pnpm --filter @quireforge/website exec playwright install chromium
pnpm test:e2e
```

On a Linux workstation with an already installed compatible Chromium, avoid a
download by setting its executable only for the test command:

```bash
PLAYWRIGHT_CHROMIUM_EXECUTABLE=/path/to/chromium pnpm test:e2e
```

The website suite exercises desktop and mobile viewports, every public route,
horizontal overflow, semantic page structure, light/dark theme persistence,
and axe-core checks on the home and integration pages. The desktop browser suite
exercises its responsive semantic shell, honest browser-preview state, theme
persistence, overflow, and axe-core baseline in both viewports. Automated
accessibility checks complement rather than replace keyboard, screen-reader,
zoom, and visual review.

GitHub Actions installs its own isolated Chromium and runs the same suite. It
does not deploy the site and receives no Cloudflare credentials.

## Native desktop validation

With the Linux prerequisites from [Building](BUILDING.md) installed:

```bash
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
pnpm desktop:build
pnpm desktop:dev
```

The automated Rust test serializes the native bootstrap contract and compares
it with the exact JSON fixture parsed by TypeScript. The final command remains a
manual launch check: verify the QuireForge title and icon, light/dark themes,
keyboard focus, native bridge status, resizing, and clean exit. On Linux the
running application must own `io.github.codeframe78.QuireForge` on the session
bus. An unbundled launch does not validate package installation or desktop-file
naming; those remain packaging-milestone obligations.

The routine suite does not require Codex authentication or make billable model
calls. An ignored compatibility test performs only local initialization and
`model/list` against the installed CLI:

```bash
cargo test --locked --workspace \
  live_probe_uses_the_supported_local_app_server -- --ignored
```

Run it deliberately when validating a Codex version. A second ignored probe
performs only `account/read` with proactive refresh disabled and asserts that
the serialized result has no identity/secret fields:

```bash
cargo test --locked --workspace \
  live_status_returns_only_normalized_account_state -- --ignored
```

Confirm either test leaves no additional `codex app-server` process. It must
not start a thread or turn, write configuration, inspect session content, or
print the account-visible catalog.

## Manual Milestone 8A checklist

- Use deterministic mock app-server fixtures only; do not resume or fork a
  personal thread or start a billable turn during routine validation.
- Confirm list/read/resume/fork/archive/restore commands accept only app-owned
  conversation IDs; resume/fork additionally accept only a bounded prompt.
- Confirm native code revalidates the original project identity and exact cwd,
  and never accepts a frontend cwd, Codex thread/turn ID, rollout path, history,
  configuration object, or runtime workspace root.
- Confirm session snapshots contain no cwd, Codex ID, preview, transcript, raw
  thread status, command output, reasoning, diff, or protocol payload.
- Reopen a fixture database containing starting/running/stopping rows; confirm
  each becomes interrupted, its active-turn ID clears, and the project is not
  left reserved.
- Confirm reconciliation uses bounded exact-cwd current/archived lists, matches
  only QuireForge-owned references, and reports a missing thread without
  importing another Codex thread.
- Confirm fork creates a distinct app reference with bounded parent-app lineage,
  while archive and restore retain the reference and never delete project or
  Codex content.
- Exercise wrong IDs, wrong cwd responses, malformed cursors, process exit, RPC
  rejection, and metadata failure; verify stable diagnostics and no child
  process remains.

## Manual Milestone 8B checklist

- Confirm an empty title query performs complete reconciliation only; a
  non-empty query performs complete reconciliation before the separate bounded
  `searchTerm` projection.
- Confirm filtered IDs must exist in the complete authoritative result and only
  QuireForge-owned references reach the UI; an unmatched session must not be
  relabeled missing.
- Confirm titles are trimmed, bounded to 256 characters, rejected when they
  contain control or directional-formatting characters, and never written to
  SQLite.
- Search, clear, and refresh history across project groups. Open parent and fork
  rows as tabs; exercise Arrow Left/Right, Home, End, close, focus indication,
  and mobile overflow behavior.
- Resume and fork with a bounded prompt, then confirm the task stream uses the
  returned app conversation ID. Archive and restore by exact app ID and confirm
  neither operation presents or performs deletion.
- Confirm archived, missing, busy, unavailable, empty-result, and browser-
  preview states prevent inappropriate actions and expose no Codex ID, cwd,
  preview, transcript, raw status, or protocol payload.
- Run component and shell integration tests, both Playwright viewports, axe-core,
  the complete repository validator, the warm unbundled native build, and the
  isolated launch smoke check without a live model turn.

## Manual Milestone 7 checklist

- Use deterministic mock app-server fixtures only; do not start a live or
  billable model turn as routine validation.
- Confirm conversation start revalidates the active association and passes the
  exact resolved cwd only inside the native app-server request.
- Confirm the webview cannot submit or receive cwd, native Codex thread/turn
  IDs, commands, environment, raw reasoning, command output, diffs, or paths.
- Confirm model and reasoning choices must be advertised by the live catalog,
  and unsafe sandbox/approval combinations fail before a task starts.
- Confirm the project cannot be detached, archived, or relinked while its task
  is active and becomes available after every terminal path.
- Confirm interruption targets only the native-owned exact thread and turn and
  every terminal/failure path closes and waits for the child.
- Inspect the migrated SQLite schema and confirm conversation records contain
  references, selected controls, status, and timestamps only—never prompt or
  transcript content.
- Confirm an approval server request becomes a stable blocked state and no
  approval response is fabricated.
- Confirm the composer stays disabled without a verified writable project and
  ready native runtime; browser preview must not simulate a task.
- Confirm model/reasoning choices come only from the normalized runtime catalog,
  and unrestricted execution with approvals disabled is visibly rejected before
  IPC.
- Confirm streamed batches are ordered and deduplicated, terminal states remain
  understandable, and stop sends only the app-owned conversation ID.
- Check the composer and event stream at desktop/mobile widths, by keyboard, and
  with automated accessibility analysis.
- Verify the unbundled release starts under isolated XDG directories, performs
  no conversation work without user action, and leaves no app-server child.

## Manual Milestone 6 checklist

- Confirm the folder picker is native and no command accepts a frontend path.
- Preview and cancel an attachment without writing project metadata.
- Confirm an attachment only after reviewing selected/resolved paths,
  read-only state, Git/worktree state, and project-instruction indicators.
- Retarget a selected symlink or change `AGENTS.md`/`.codex` after preview and
  confirm that save fails closed.
- Detach, archive, and relink temporary fixture directories; confirm none of
  those metadata actions delete or modify source content.
- Move or remove a fixture directory and confirm cwd preflight never falls back
  to home, the application directory, or another project.
- Inspect the temporary metadata schema and permissions; confirm it contains no
  Codex authentication, session, connector, or project-file content.
- Do not use a personal source directory for destructive validation. The
  automated suite uses only temporary or in-memory fixtures.
- Verify the release executable starts with isolated temporary XDG directories,
  owns `io.github.codeframe78.QuireForge` on the session bus, creates its app
  data directory with owner-only permissions, and exits cleanly.

## Manual Milestone 5 checklist

- Run the non-mutating live account-status probe against the intended CLI and
  confirm it prints no account data and leaves no app-server child.
- Exercise authenticated, unauthenticated, not-required, unavailable, browser
  pending, device pending, completion, failure, cancellation, stale-ID, and
  invalid-URL fixtures.
- Confirm raw email, plan, login ID, tokens, API keys, and completion error text
  cannot enter the frontend snapshot.
- Confirm the browser command accepts no URL argument and only native-validated
  HTTPS OpenAI/ChatGPT handoffs are accepted.
- Confirm browser preview never simulates native account state and the onboarding
  panel passes desktop/mobile axe-core checks.
- Confirm logout requires a second explicit action. Do not exercise live login,
  browser authorization, or logout without separate approval.
- Confirm native launch owns the exact application identity, all account probes
  finish, and no app-server child remains after exit.

## Manual Milestone 4 checklist

- Run the ignored non-billable live probe against the intended Codex CLI.
- Confirm the native shell reports the adapter as ready without exposing raw
  app-server fields or catalog details beyond normalized model metadata.
- Confirm browser preview reports that the native probe is unavailable.
- Exercise missing-CLI, invalid-version, early-exit, timeout, duplicate-model,
  multiple-default, and unexpected-server-request fixtures.
- Confirm all owned child processes are reaped after success and failure.
- Confirm the Tauri capability still grants no broad plugin permission.
- Confirm no login, model turn, project path, configuration write, Codex
  session mutation, package, or deployment occurs.

## Manual Milestone 3 checklist

- Launch the release or development binary on GNOME Wayland.
- Confirm the application registers `io.github.codeframe78.QuireForge` and
  releases it on exit.
- Confirm the shell says that no project is attached and future capabilities
  remain labeled by milestone.
- Confirm the native IPC status changes to verified without enabling broad
  Tauri plugin permissions.
- Inspect light and dark themes, keyboard focus, reduced motion, resizing, and
  narrow browser fallback behavior.
- Confirm no package, Codex session, project, credential, configuration, or
  integration state is created.

## Manual Milestone 2 checklist

- Inspect Home and Integrations in light and dark themes.
- Inspect at desktop and narrow mobile widths.
- Navigate the header, theme control, page content, and footer by keyboard.
- Confirm focus remains visible and reduced motion is honored.
- Confirm no clipped text, horizontal scroll, stale identity, or broken asset.
- Confirm Downloads and Installation do not claim an unreleased package.
- Confirm the unofficial-project disclaimer remains visible.
- Confirm the built `_headers`, `robots.txt`, sitemap, manifest, icons, and 404
  page are present.

Production-origin Lighthouse and live-header measurements are deferred until a
separately approved Cloudflare preview or production deployment exists. Any
miss against the published quality targets must be recorded with remediation.
