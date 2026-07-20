# Testing QuireForge

Status: Milestones 2–6 establish repository, website, desktop frontend, native
contract, Codex adapter, authentication, project attachment, and Tauri build
checks. PTY, broader Git fixtures, and conversation suites arrive with the
milestones that introduce those systems.

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
