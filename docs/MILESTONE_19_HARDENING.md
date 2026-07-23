# Milestone 19 Hardening Review

Status: complete and verified locally; no package, publication, deployment, or
live model call was performed.

Milestone 19 is a local security, privacy, accessibility, performance,
reliability, and crash-recovery review of the implemented QuireForge desktop
and static website. It does not package, publish, deploy, authorize, or run a
live model.

## Reviewed boundaries

- React/webview to fixed typed Tauri commands and normalized results.
- Native process, project, Git, worktree, PTY, preview, attachment, and
  integration services.
- QuireForge-owned SQLite and staged-file roots versus Codex-owned
  authentication, configuration, sessions, and connector state.
- Tauri capabilities, custom protocols, CSP, response headers, frontend active
  content, and production command registration.
- pnpm, Cargo, GitHub Actions, Dependabot, and integration supply-chain
  controls.
- Desktop and website keyboard flow, focus ownership, reduced motion, forced
  colors, responsive reflow, automated accessibility, and recovery copy.
- Desktop production asset size, initial JavaScript execution, native build
  viability, and startup reconciliation.

## Findings and remediation

### Dependency and workflow supply chain

The baseline Node audit found one high-severity development-only transitive
advisory: `fast-uri` 3.1.3 arrived through the Astro language/checking toolchain.
The reviewed workspace override now resolves 3.1.4, and
`pnpm audit --audit-level high` reports no known vulnerability. The override is
locked and repository validation prevents an unnoticed regression.

The RustSec scan covers all 503 entries in `Cargo.lock`. It reports no
unaccepted vulnerability after applying 17 explicit reviewed exceptions:

- ten unmaintained gtk-rs GTK3 crates inherited by Tauri's supported Linux
  WebKit runtime;
- unmaintained `proc-macro-error` and the `glib` 0.18 iterator-unsoundness
  notice inherited by that same GTK3 graph; and
- five unmaintained `unic` crates inherited by
  `tauri-utils -> urlpattern`.

Those exceptions are exact advisory IDs in `.cargo/audit.toml`, are enforced by
the repository validator, and must be revisited on every Tauri update and
before release. QuireForge does not directly use the affected
`glib::VariantStrIter`. The audit still denies every new warning rather than
silencing an advisory category.

Milestone 21B performed the required pre-publication refresh on 2026-07-23. The
pinned auditor scanned all 503 locked crates against 1,169 current advisories
and reported zero unaccepted vulnerability or warning. The same 17 explicit
exceptions remain inherited and visible; the terminal exact-source release
gate must run the audit again.

CI now installs pinned `cargo-audit` 0.22.2 and runs both Node and Rust audits.
Dependabot adds the Cargo ecosystem. Repository validation also requires every
non-local GitHub Action to use a full immutable commit SHA.

### Webview and active-content boundary

The main Linux capability remains limited to the main window and grants no
plugin permission. Tauri now explicitly:

- leaves the global Tauri JavaScript object unavailable;
- disables the asset protocol with an empty scope;
- retains Tauri's compile-time CSP asset injection;
- removes unused plugin commands from production builds;
- limits production content to local scripts/fonts/styles, local/data images,
  and Tauri IPC;
- denies objects, forms, frames, workers, media, manifests, and base-URL
  changes; and
- emits same-origin opener/resource policy, a browser-feature deny policy, and
  MIME-sniffing protection.

`style-src 'unsafe-inline'` remains necessary for the stable xterm renderer.
`freezePrototype` also remains `false`: enabling it prevents the verified
Vite/React production bundle from mounting. These are explicit residual
exceptions, compensated by no privileged remote content, no asset protocol,
empty capability permissions, strict CSP, locked dependencies, typed IPC, and
repository checks that reject direct frontend HTML injection, code evaluation,
fetch, XHR, and WebSocket primitives.

### Accessibility and recovery

The desktop and website now provide focusable skip targets. Desktop navigation
uses semantic anchors rather than page-current buttons, and scripted scrolling
honors reduced-motion preference. Both surfaces apply high-contrast and
forced-colors support, and the website reduced-motion rule now also suppresses
animation.

The terminal close review now has an accessible name and description, moves
focus into the alert, traps Tab/Shift+Tab, closes on Escape when safe, and
restores prior focus. The terminal selector no longer places an unrelated close
button inside an ARIA tablist; it uses an accessible list of pressed selector
buttons with adjacent close actions.

A top-level React error boundary replaces render failures with bounded recovery
copy and a reload action. It never renders or retains the raw error. Reloading
uses the existing native startup reconciliation: stale app-owned conversation
and terminal metadata becomes interrupted rather than claiming recovered
process ownership, and no project file or Codex history is deleted.

### Performance

The baseline desktop build emitted one 805,736-byte JavaScript entry. The
production graph now separates the startup entry (193,549 bytes), application
shell (266,135 bytes), and stable xterm terminal workspace (350,014 bytes).
The entry is 76.0% smaller than the baseline. The 459,684-byte pre-terminal
path is 42.9% smaller, while total JavaScript remains about 791 KiB and total
CSS about 82 KiB.

The first process-only smoke was intentionally rejected after its three-second
window showed only the native background. Policy bisection ruled out CSP,
response headers, and command pruning. A minimal mount and explicit app-module
split established that cold WebKit module compilation and paint outlasted the
short probe. The retained production loader keeps an opaque, bounded startup
view mounted through two post-commit animation frames, so compilation cannot
produce an unexplained black interval.

`desktop validate:dist` enforces:

- a 256 KiB startup-entry ceiling;
- a 300 KiB application-shell ceiling;
- an 850 KiB total JavaScript ceiling;
- a 100 KiB total CSS ceiling;
- distinct application and terminal-renderer chunks;
- no external origin in the generated desktop HTML.

These are regression budgets, not a claim that later route-level state or
runtime profiling is complete.

## Residual release risks

- The reviewed Tauri Linux graph still carries GTK3 maintenance and `glib`
  advisories. Packaging must re-run the exact audit and reassess whether a
  supported upstream migration exists.
- `freezePrototype: false` and inline styles for xterm remain explicit
  webview exceptions.
- Automated browser checks cannot replace screen-reader and multiple Linux
  desktop-environment QA on the eventual packages.
- QuireForge remains pre-release. Package signing, update metadata, oldest
  supported distribution, install/upgrade/uninstall, and final release
  provenance belong to Milestones 20–21.

## Acceptance evidence

- The complete non-browser gate passed 152 desktop tests, five website tests,
  and 174 runnable Rust tests with three deliberate live probes ignored.
- All 32 desktop and eight website Playwright scenarios passed across desktop
  and mobile profiles, including axe, keyboard, contrast, reduced-motion, and
  overflow coverage.
- Both high-severity Node and warning-denying RustSec dependency audits passed.
  The Rust scan covered all 503 locked dependencies and only the 17 exact
  reviewed exceptions above.
- The final configured unbundled Tauri build completed in 26.37 seconds at
  about 1.78 GiB peak RSS with zero swaps. The resulting binary is 18,341,600
  bytes and used the reviewed embedded assets; no Vite server was listening.
- The exact final cold isolated X11 launch showed the bounded loading view at
  the 1.27-second sample and the complete native workspace at the 8.44-second
  sample. It emitted zero stdout/stderr, created a `0700`
  app-data directory and `0600` six-migration database, persisted no project,
  conversation, or terminal record, and reaped its dry isolated Codex
  app-server probe on exit.
- The final generated assets are 193,549-byte entry, 266,135-byte app shell,
  350,014-byte terminal chunk, 809,698 total JavaScript bytes, and 84,133 total
  CSS bytes. Generated desktop HTML loads no external origin.

Routine gates used deterministic fixtures and no personal Codex state,
connector authorization, package, publication, deployment, or billable model
call. The staged secret and unrelated-change review is part of the final local
commit audit.
