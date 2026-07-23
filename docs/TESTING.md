# Testing QuireForge

Status: Milestones 2–12 establish repository, website, desktop frontend,
native contract, Codex adapter, authentication, project attachment,
conversation/runtime lifecycle, approvals, reviewed Git read/write checks, and
managed worktrees with bounded parallel execution and safe cleanup/recovery,
plus a native PTY boundary and integrated terminal interface.

Milestone 13A adds versioned integration and dynamic-tool contract tests.
Milestone 13B adds live read-only native discovery, strict CLI/app-server
normalization, invalidation, partial-failure, version-gate, and IPC tests.
Milestones 14A–14C add confirmed lifecycle, Integration Center, authorization,
skill-control, refresh, and connector-mention coverage.
Milestone 15A adds a strict bounded file-preview contract, temporary-file
native tests, and honest browser/native presentation coverage.
Milestone 15B adds strict conversation-image staging, lifecycle, turn-input,
component, and browser coverage.
Milestone 15C adds opaque one-use handoff, native revalidation, fixed-copy
notification, focus/deduplication state-machine, component, and bridge coverage;
real display-session results remain separately labeled manual evidence.
Milestone 17A adds schema-v2 scheduled-template contract, installed-plugin
lookup, strict `plugin/read` normalization, prompt/schedule safety, component,
responsive browser, overflow, and axe-core coverage.
Milestone 18 adds next-turn selector policy, lifecycle, migration,
revalidation, prompt-injection, ownership, and responsive control coverage.
Milestone 19 adds Node/Rust dependency audits, immutable-action and
active-content repository checks, Tauri policy assertions, desktop asset
budgets, crash-boundary privacy tests, and keyboard/reduced-motion/forced-color
coverage for desktop and website.
Milestone 20 adds package-contract unit tests, canonical Debian/AppImage
inspection, checksum and release-manifest validation, a GLIBC 2.35 ceiling,
disposable install/upgrade/uninstall preservation checks, isolated visible X11
launch probes, inactive website-download assertions, and a guarded manual
release-workflow policy.

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

Dependency audits are deliberately separate because they refresh external
advisory databases:

```bash
pnpm security:audit:node
cargo install cargo-audit --locked --version 0.22.2
pnpm security:audit:rust
```

CI installs the pinned RustSec auditor before running both audits. Local
`cargo-audit` installation is a developer-tool prerequisite, not a project
runtime dependency.

## Linux package checks

The authoritative candidate gate is:

```bash
./scripts/run_linux_package_container.sh
```

It builds in the pinned Ubuntu 22.04 container and runs structural, checksum,
AppStream, GLIBC, lifecycle, and visible X11 checks for both package formats.
It uses isolated ignored caches, no personal Codex home or credentials, no live
model call, no host package installation, and no local Vite server.

The dependency-free source-contract subset is also part of `pnpm validate`:

```bash
pnpm package:test
```

For an already produced artifact directory:

```bash
python3 scripts/validate_release_artifacts.py \
  --artifact-dir target/ubuntu-22.04/release/packages
```

`--lifecycle` adds the disposable Debian sequence and `--smoke` adds isolated
window launches. `--require-publishable` is reserved for the exact clean,
tagged, pinned-builder publication boundary and also requires
`--expected-tag`. Routine local dirty builds are intentionally
`local-candidate` manifests.
Project-core tests cover transactional schema migration, forward-schema
refusal, app-data permissions, selected/resolved path identity, mount state,
Git repositories and linked worktrees, duplicate roots, confirmation-time
changes, relink/detach/archive behavior, and fail-closed cwd preflight.
Frontend project tests share a normalized fixture with Rust, reject unknown or
path-bearing bridge input, and cover confirmation, missing/read-only states,
relink, preflight, and two-step detach/archive controls.
Conversation tests cover verified-cwd start, live catalog validation, strict
control combinations, UUIDv7 correlation, bounded normalized events, exact
interrupt, project reservation, exact approval correlation and decisions,
pending-approval cancellation, detailed activity identity, split-secret
redaction, protocol mismatch, child reaping, and reference-only persistence.
Milestone 11B fixtures additionally start independent mock app-server children
for distinct worktree projects, prove exact per-app-ID interruption, reject a
second task in the same project, enforce the four-task capacity while starts
are provisional, and reap every child without a live or billable model call.
TypeScript tests reject cwd,
Codex thread/turn IDs, unknown fields, raw protocol payloads, and path-bearing
bridge input before native invocation.
Session-lifecycle tests cover schema migration, stale-turn crash reconciliation,
bounded exact-cwd list matching, owned-thread reads, resume, fork lineage,
archive/restore without deletion, mismatched-cwd rejection, project-reservation
release, child reaping, and shared strict Rust/TypeScript fixtures. They use
deterministic mock app-server processes and never read a personal transcript or
start a live model turn.
Git-review tests cover shared strict fixtures, porcelain-v2 parsing without
object IDs, repository and read-only attachment revalidation, fixed-command
status/diff execution in temporary repositories, binary projection, deceptive
and escaping path refusal, control stripping, frontend path rejection, browser
preview honesty, responsive layout, overflow, and axe-core. Mutation tests use
only per-test temporary repositories and cover strict request/token contracts,
stale preview refusal, exact stage/unstage, project ownership, index locks,
bounded revert and single-use recovery, unborn and existing-branch commits,
unstaged-work preservation, hooks/signing refusal, attachment scope,
repository-local identity, sensitive filenames/content/messages, and final
postconditions. They never alter a user repository or invoke a model.
Worktree tests use disposable repositories and app-data roots to cover
porcelain inventory without object IDs, strict branch/token contracts, source
HEAD changes, single-use confirmation, external-before-attach state,
native-selected linked-worktree identity, managed creation, configured checkout
filter suppression, schema migration 4, transactional registration, and the
recoverable-worktree path after a forced metadata failure. Milestone 11C adds
disposable-fixture coverage for opaque recovery adoption, managed-only cleanup,
dirty/busy/attached/symlink refusal, branch retention, configured-filter
neutralization during Git's internal removal check, post-Git metadata failure,
and non-destructive finalization. Frontend and browser tests cover strict
schema-v2 payloads, fixed bridge commands, explicit destructive copy,
responsive recovery/cleanup controls, overflow, and axe-core.
Parallel-registry contract tests share one empty fixture with Rust, require
active-only unique project/conversation IDs, and reject unknown or over-capacity
state. App/component/browser tests recover and poll multiple task IDs
independently, show normalized changed-file/conflict counts, open the selected
task's expandable activity, and verify both configured viewports with axe-core
and overflow analysis.

Terminal tests use temporary attached directories and real local PTYs without
starting Codex or making a model call. Rust covers metadata migration/restart
interruption, absence of content/process columns, exact project reservations,
verified cwd startup, explicit noncredential environment inheritance,
pre-decode input bounds, byte-preserving output and truncation, provisional
capacity/title handling, unknown app-ID refusal, resize/write state, and
foreground/background session cleanup. TypeScript shares strict empty-registry
and capability fixtures, rejects unknown/path/process-bearing payloads before
or after IPC as appropriate, and verifies exact fixed bridge calls. Component
and browser tests cover native/preview honesty, project selection, live byte
polling, responsive tabs, explicit process-ending confirmation, recovery copy,
xterm layout, axe-core, and overflow at desktop and mobile viewports.

Milestone 13A integration tests share one deterministic catalog between Rust
and TypeScript. They require category-preserving connector, plugin,
marketplace, skill, and MCP entries; closed scope/source/state enums; unique
capability references; confirmation for every mutation; consistent health and
diagnostic states; and a contract-only dynamic-tool lifecycle. Strict Zod
validation rejects raw protocol payloads, account identity, dangling
capabilities, unconfirmed mutations, unsafe display characters, oversized
collections, and any claim that the executing turn can change its own model.
Schema refresh uses only `codex --version` and local
`codex app-server generate-json-schema --experimental`; it performs no account
request or model call.

Milestone 13B native tests use deterministic app-server shell fixtures and
sanitized CLI JSON values. They verify connector list/installed state, stable
CLI plugin and marketplace projection, skill/MCP/policy reads, category-only
invalidation refresh, independent source failure, exact 0.145.x version gating,
unknown-field/enum refusal, display sanitization, bounded identifiers, and the
absence of raw paths, URLs, configuration, tool arguments, and upstream errors
from serialized snapshots. The TypeScript bridge test validates the one fixed
`integration_catalog_read` command against the same strict schema. No routine
test reads a personal integration catalog, installs an integration, starts an
authorization flow, changes Codex configuration, or makes a model call.

Milestone 17A extends that deterministic fixture with one installed, enabled
plugin and one stable `plugin/read` response. Rust tests verify native-only
marketplace paths, strict plugin/source correlation, bounded counts, unsafe
prompt control removal, whitespace normalization, truncation, schedule
validation, stable partial-failure diagnostics, and path-free serialization.
The shared Rust/TypeScript fixture rejects orphaned plugins, duplicate task IDs
and weekdays, invalid times/intervals, unsafe display content, unknown fields,
and schema drift. Component and desktop/mobile Playwright tests assert the
read-only boundary, absence of task action buttons, inert prompt labeling,
responsive overflow, and axe-core accessibility. Routine tests neither inspect
personal plugins nor create, execute, mutate, or authorize a task.

Milestone 14A adds a second shared Rust/TypeScript fixture for the closed
mutation preview/result contract and strict bridge requests. Deterministic
native tests run fixed shell fixtures in private temporary directories and
cover plugin install/remove, marketplace add/remove/upgrade, pinned-source
enforcement, exact-entry staleness, one-use confirmation replay, fixed JSON
result validation, and list postconditions. TypeScript tests reject
operation-inconsistent requests, raw source paths/URLs, unexpected fields,
destructive-label mismatches, and applied results without catalog refresh.

The real CLI lifecycle proof is `#[ignore]` and must be invoked explicitly. It
creates temporary `CODEX_HOME` and `HOME`, registers a local fixture marketplace,
installs and removes its local fixture plugin, removes the marketplace, and
deletes that temporary tree. It does not inherit personal Codex configuration,
authentication, or integration state and makes no model call. A separate
redacted shape-only read during security review returned no names, paths, URLs,
or account data and is not part of routine validation.

Milestone 14C adds a third strict shared Rust/TypeScript fixture for control
preview/result state. Deterministic app-server shell fixtures verify exact
`skills/config/write` and `mcpServer/oauth/login` requests, skill
postconditions, one-use confirmations, native-only handoff URLs, exact MCP
completion-name correlation, unsafe-URL refusal, and authorized/enabled/
callable connector mention resolution. Conversation tests assert that native
code constructs the documented mention while the normalized catalog ID and
`app://` path do not enter serialized snapshots. Component, bridge, and
application tests exercise permission/warning review, opaque browser actions,
status polling, catalog refresh, and composer selection. No routine test reads
or changes personal Codex/integration state, opens a real authorization page,
or makes a model call.

Milestone 15A adds a strict shared Rust/TypeScript preview fixture. Native tests
use only temporary attached directories and cover identity/containment,
symlinks, binary refusal, malformed project IDs, text normalization, byte/line
and image-dimension bounds, metadata-only PDF handling, and a full-file APNG
marker beyond the sniff window. TypeScript rejects absolute paths, unsafe
controls, unknown fields, oversized content, and inconsistent kind/rendering
payloads. Bridge, component, application, and browser tests verify the one
fixed opaque-ID command, native-picker ownership, browser honesty, axe-core,
and overflow. They never read a user's project files.

Milestone 15B adds a strict shared Rust/TypeScript conversation-attachment
fixture. Native tests use temporary images and app-data roots to cover native
picker, dragged-byte, and one-use native-capture sources, symlink/name/type/size
refusal, private file permissions, expiry, cancellation, tamper detection, one-
use claim, path-free serialization, startup reconciliation, documented
`localImage` construction, and terminal-turn cleanup. TypeScript rejects
unknown fields, unsafe names, non-PNG/JPEG types, malformed IDs/base64, and
inconsistent states. Bridge, component, application, and browser tests cover
the five fixed commands, explicit start/resume/fork IDs, browser drag/drop/
picker honesty, responsive layout, axe-core, and overflow. They use
deterministic mock app-server processes and make no live model call.

## Milestone 13A contract checklist

- Confirm the generated manifest identifies CLI 0.145.0, hashes every selected
  schema, and retains the prior 0.144.6 fixture set.
- Confirm connector/app, plugin, marketplace, skill, MCP, policy,
  permission-profile, invalidation, and dynamic-tool schemas are represented.
- Confirm every deterministic capability remains `contract-only`; no UI,
  bridge command, installation, authorization, or configuration mutation is
  implied by this checkpoint.
- Confirm `thread/start` registration, `item/tool/call` invocation, exact JSON-
  RPC correlation, and bounded result content are documented while the
  executing turn's model remains immutable.
- Run strict TypeScript/Rust contract tests, repository validation, complete
  non-browser gates, and a warm release build. A browser or native visual run
  is unnecessary because 13A introduces no user-facing surface.

## Milestone 13B discovery checklist

- Confirm the native service rejects CLI versions outside reviewed minor
  0.145.x before starting discovery.
- Confirm connectors, skills, MCP, and policy use only reviewed app-server
  reads; plugin and marketplace reads use bounded stable CLI JSON rather than
  under-development plugin RPCs.
- Confirm app, skill, MCP-startup, config-warning, and account events retain
  only closed refresh reasons and never their raw payloads.
- Confirm a malformed or unavailable source degrades only its capabilities and
  preserves independently discovered categories.
- Confirm the sole frontend bridge is fixed-purpose and read-only; all mutation,
  authorization, prompt-mention, and dynamic-tool capabilities remain
  contract-only.
- Run the complete non-browser gate, both Playwright viewport profiles, and a
  warm unbundled native release build. No new visual surface is expected, so
  browser verification is regression coverage rather than a new screenshot
  approval.

## Milestone 14A mutation checklist

- Confirm only fixed plugin install/remove and marketplace add/remove/upgrade
  operations exist; React cannot supply a command, argument vector, path, URL,
  configuration object, or raw CLI payload.
- Confirm preview refreshes the catalog/policy, requires a reviewed 0.145.x
  minor and ready capability, reviews local/pinned repository/exact package
  sources, and returns only normalized permissions and closed warnings.
- Confirm marketplace repository adds require a 40- or 64-hex reference and
  marketplace upgrades expose the mutable-remote-source warning.
- Confirm the UUIDv7 token expires after five minutes, is consumed once, and
  confirmation serializes mutations before revalidating CLI version, policy,
  normalized entry, and exact native source evidence.
- Confirm every fixed command has neutral cwd, null stdin/stderr, bounded
  output/time, credential removal, child reaping, a closed JSON result, and a
  follow-up catalog postcondition.
- Run strict shared contract/bridge tests, deterministic native lifecycle and
  adversarial tests, the explicit temporary-home real-CLI proof, complete
  repository gates, and both Playwright viewports. Browser verification is
  regression-only because 14A adds no Integration Center UI.

## Milestone 14B Integration Center checklist

- Confirm search matches only bounded name, summary, and publisher metadata and
  that category/health filters preserve the normalized entry category and
  honest blocked, degraded, unavailable, and unknown states.
- Confirm details expose only normalized source, scope, installation,
  enablement, authentication, policy, publisher, version, permissions,
  requirements, warnings, and health; no raw protocol, CLI, path, URL,
  credential, account, or tool-argument field reaches React.
- Confirm controls appear only for fixed 14A operations whose capability is
  available and implemented, while connector/MCP authorization, enable/disable,
  skill configuration, prompt mentions, and repair stay visibly unavailable.
- Confirm repository marketplace adds accept only the strict pinned-reference
  request and every mutation displays the native preview's permissions,
  warnings, destructive status, and separate hook trust before confirmation.
- Verify dialog focus entry, containment, restoration, and Escape behavior;
  bounded loading/error/result states; and a fresh catalog read after an
  applied result.
- Run deterministic component and application wiring tests, complete repository
  gates, desktop/mobile Playwright with axe-core and overflow assertions, visual
  review in both viewports, and a warm unbundled native release build. Do not
  read or mutate personal integration state.

## Milestone 14C authorization and control checklist

- Confirm React can request only connector authorization, MCP authorization,
  skill enable, or skill disable with an opaque normalized entry ID; URLs,
  paths, MCP names, config values, and protocol methods are rejected or absent.
- Confirm preview requires ready capability/current eligible state and creates
  a bounded one-use UUIDv7. Confirmation must re-resolve identical native
  evidence, serialize control execution, and refuse expiry, replay, staleness,
  policy changes, or malformed upstream responses.
- Confirm skill enable/disable uses the exact native manifest path from
  `skills/list`, accepts only the expected effective response, and verifies a
  fresh list postcondition.
- Confirm connector/MCP authorization keeps the returned URL native-only,
  allows credential-free HTTPS or loopback HTTP only, opens it solely from an
  opaque action ID, and correlates completion to fresh connector accessibility
  or the exact MCP completion name.
- Confirm conversation start accepts no more than eight unique normalized
  connector IDs and native code requires accessible, enabled, callable state
  before constructing the documented mention and `app://` path.
- Confirm explicit refresh is fixed-purpose/non-destructive and unsupported
  generic configuration, plugin enablement, connector install/configuration,
  MCP management, and repair remain visibly unavailable.
- Run strict contract, native process, bridge, component, and application tests,
  complete repository gates, desktop/mobile Playwright with axe-core and
  overflow assertions, visual review, and a warm unbundled native release
  build. Do not inspect or mutate personal integration state or complete a real
  third-party authorization during routine validation.

## Milestone 15A safe file-preview checklist

- Confirm React supplies only a canonical project UUIDv7; native code opens the
  picker only after validating it and never serializes an absolute path.
- Confirm every selection reloads attachment metadata, revalidates readable
  identity, canonical containment, symlink/regular-file status, and the opened
  device/inode under `O_NOFOLLOW`.
- Confirm the source, text byte/line, image byte/dimension/pixel, IPC data-URL,
  path, and schema collection limits fail closed with stable diagnostics.
- Confirm controls and bidi overrides are normalized from UTF-8 text; only
  PNG/JPEG can produce image data; APNG, unknown binary content, and malformed
  images are refused; HTML/SVG source remains inert text; PDF bytes never enter
  the webview.
- Confirm the production CSP permits `data:` only for `img-src`, the preview is
  never persisted, and browser preview has no local picker/input simulation.
- Run strict contracts, native temporary-file tests, bridge/component/app unit
  tests, desktop/mobile Playwright with axe-core/overflow, complete repository
  gates, and a warm unbundled release build. Do not inspect a user's files or
  create a package, release, deployment, or hosting change.

## Milestone 15B conversation-image checklist

- Confirm Tauri default file-drop events are disabled, picker paths stay native,
  and browser drops transmit only explicitly read bounded bytes plus a safe
  name and declared PNG/JPEG type. On Linux, confirm an empty WebKitGTK HTML
  `FileList` claims only a 30-second native GTK capture through the fixed path-
  free command, consumes it once, and returns no source path.
- Confirm Rust independently validates real type/structure, dimensions, names,
  4 MiB per-file, four-file, and 16 MiB aggregate limits before creating
  mode-`0600` UUIDv7 copies under a mode-`0700` app-data root.
- Confirm snapshots contain only opaque project/attachment IDs and normalized
  metadata; source/staged paths, bytes, filesystem handles, Codex IDs, and raw
  protocol input never cross IPC or enter SQLite.
- Confirm drafts expire after 15 minutes, are consumed once, remain project-
  bound, and are cleaned by cancel, failed send, terminal turn, or startup.
  Tampered/replaced copies must fail closed at claim.
- Confirm start, resume, and fork construct only documented `localImage` inputs
  natively and retain claimed files until the normalized turn is terminal.
  Generic file and arbitrary path inputs must remain unavailable.
- Run strict contracts, temporary native tests, bridge/component/app tests,
  desktop/mobile Playwright with axe-core/overflow, complete repository gates,
  and a warm unbundled release build. Do not inspect user files, start a live
  model turn, or create a package, release, deployment, or hosting change.

## Milestone 15C desktop-integration checklist

- Confirm a ready preview exposes only one UUIDv7 open action and relative
  display name. The command must accept no path, URL, MIME, application,
  executable, argument, or working directory.
- Confirm the UI names `System default application` and requires a separate
  handoff confirmation. Rust must revalidate attachment identity, canonical
  containment, regular non-symlink state, descriptor path, and device/inode
  before opening. Replacement, clear, expiry, replay, and tamper cases must
  fail closed without creating a generic opener.
- Confirm the notification command accepts only an app-owned conversation ID,
  freshly permits pending approval/completed/blocked/failed state, suppresses
  foreground delivery, and deduplicates approval/terminal identity. Fixed
  notification copy must not interpolate project names, prompts, paths,
  model/account data, output, diagnostics, or raw protocol fields.
- Confirm the official notification plugin is invoked only from Rust and the
  main webview capability list remains empty. Delivery failure must not change
  task state and may retry only the same native-reviewed notification.
- Build the disabled-by-default native probe with
  `pnpm desktop:build:notification-probe`, launch it only with the exact
  `--manual-notification-probe` process flag and disposable app data, and verify
  the real desktop service receives the production fixed copy. The probe must
  expose no Tauri command, webview permission, or caller-supplied content. Run
  `pnpm desktop:build` afterward and confirm the normal artifact excludes it.
- Run strict contracts, temporary-file/native state-machine tests, bridge,
  component, and application tests, desktop/mobile Playwright with axe-core and
  overflow, complete repository gates, and a warm unbundled release build.
- On a disposable app-data root, manually exercise attach, file picker,
  preview, open confirmation, attachment picker/drop, and a privacy-safe
  notification on supported Wayland and X11 sessions. Record Wayland,
  XWayland, and true X11 accurately; one cannot substitute for another. Do not
  inspect unrelated user files, start a live model turn, or create a package,
  release, deployment, or hosting change.

Current display evidence: the configured unbundled Tauri production artifact
starts under native Wayland. With disposable app data it completed project
attach/review, the `README.md` picker and bounded preview, the native image
picker, and a real Nautilus image drop that staged normalized `drag drop`
metadata. Test-only compositor input came from an approved XDG Remote Desktop
portal session that was closed after the pass; the product did not gain a
remote-control permission or dependency. It separately completed the attached-
project picker, file picker, bounded README preview, second confirmation,
system-default opener, and consumed-action state under XWayland. In a separate
Ubuntu 24.04 GNOME 46 `ubuntu-xorg` QA guest, `loginctl` reported `Type=x11`, an
Xorg server owned display `:0`, and the shell environment had no
`WAYLAND_DISPLAY`. Against the attached repository mounted in place, the
production artifact completed project attach/review, README picker/preview/
default-app handoff, attachment picker, and a real Nautilus image drop. That
drop initially exposed an empty WebKitGTK HTML `FileList`; after the native-only
GTK capture correction, the UI staged normalized `drag drop` metadata without
a source path. The feature-gated probe delivered the fixed completed-task copy
through the X11 session's Freedesktop service, and a filtered D-Bus capture
contained only QuireForge identity plus the fixed title/body. The normal
artifact was rebuilt afterward and contains no probe flag or delivery string.
This closes the Milestone 15 display-session gate; native Wayland, XWayland, and
true X11 remain accurately distinguished.

## Milestone 17A read-only scheduled-template checklist

- Confirm only installed, enabled plugins become `plugin/read` targets and raw
  marketplace roots never serialize through IPC.
- Reject unsupported CLI minors, unknown response fields, mismatched plugin or
  marketplace identity, oversized collections, duplicate task/weekday
  identity, invalid hourly intervals/times, and unsafe display controls.
- Confirm task prompts are whitespace-normalized, bounded, visibly labeled
  untrusted/inert, never persisted or submitted, and accurately marked when
  truncated.
- Degrade only `scheduled-task.catalog` for a malformed read while retaining
  valid unrelated integration categories and safe task entries.
- Parse the shared schema-v2 fixture in Rust and TypeScript and verify every
  task references an existing plugin entry.
- Run component and desktop/mobile browser checks for all schedule labels,
  empty/degraded states, absence of task action buttons, responsive overflow,
  and axe-core accessibility.
- Make no personal plugin/account read, task mutation, task execution, hosted
  scheduler request, official-client automation, or billable model call.

## Milestone 18 acceptance checklist

- Use deterministic mock catalogs and control requests; do not make a live or
  billable model call during routine verification.
- Verify Codex sees only normalized available models, supported efforts, the
  current effective selection, one pending selection, and the app-owned policy.
- Confirm a request is staged only after the current turn completes and is
  revalidated immediately before the next `turn/start`; no surface may claim a
  mid-turn model replacement.
- Attempt stale, unadvertised, malformed, and unsupported model/effort values.
  Confirm each fails closed without changing the effective selection.
- Verify Manual, Recommend, and Automatic modes; Automatic must require explicit
  opt-in and an allowlist or ceiling, while a user lock or later manual choice
  always wins.
- Submit repeated and contradictory requests in one turn. Confirm only one can
  remain pending, cost escalation cannot exceed policy, and visible provenance
  identifies Codex plus its bounded rationale.
- Relaunch with a staged change and verify the documented persistence/recovery
  rule without retaining a prompt, credential, account ID, or raw payload.
- Run against a fixture where the supported control lifecycle is unavailable.
  Confirm honest recommendation-only behavior and no web automation, private
  endpoint, or fabricated success.

Automated Rust coverage exercises closed dynamic-tool parsing, exact
correlation, Manual/Recommend/Automatic policy, lock/allowlist/ceiling
precedence, one request per turn, completion-time staging, migration/restart
persistence, stale-choice discard, fresh next-turn application, and
registration rejection. TypeScript/component/browser coverage validates strict
schema-v3 IPC, provenance, effective versus pending presentation,
recommendation acceptance/dismissal, automatic opt-in limits, lock controls,
desktop/mobile overflow, and axe-core accessibility. Routine verification uses
no live turn or personal account mutation.

## Milestone 19 acceptance checklist

- Run the dependency-free repository validator and confirm the capability stays
  Linux/main-window scoped and permission-empty, all remote Actions use full
  SHAs, reviewed CSP/header/asset-protocol settings remain exact, and no direct
  active-content/evaluation/network primitive enters production frontend code.
- Run the Node high-severity audit and warning-denying RustSec audit. Confirm
  the lock graph uses `fast-uri` 3.1.4 and every ignored RustSec ID matches the
  reviewed Tauri/GTK3 or `tauri-utils` transitive graph.
- Build the desktop frontend and run `desktop validate:dist`. Confirm the
  startup entry, application shell, and terminal renderer remain separate, the
  per-chunk/total JS/CSS budgets pass, and the generated HTML loads no external
  origin.
- Trigger the React render boundary with sensitive-shaped fixture text.
  Confirm the raw value is absent and only the bounded reload/reconciliation
  copy is presented.
- Navigate both surfaces by keyboard. Verify skip-link focus transfer, visible
  focus, semantic desktop navigation, reduced-motion CSS and scripted
  scrolling, and forced-color control boundaries in both browser profiles.
- Open the terminal close review. Verify accessible name/description, initial
  focus, Tab/Shift+Tab containment, Escape dismissal, prior-focus restoration,
  axe-core, and the no-project-deletion copy.
- Run all source, website, desktop, browser, native, configured production
  build, secret, and diff gates. Do not package, deploy, authorize an
  integration, inspect personal Codex state, or make a live/billable model call.
- Launch the configured production binary without a Vite server and with
  isolated QuireForge/Codex data. Capture the first cold frames and the complete
  workspace; the bounded startup overlay must remain visible until the app
  paints, with no intermediate black frame or surviving child process.

## Manual Milestone 12 checklist

- Use an isolated QuireForge data directory and a disposable attached project.
  Open two tabs and verify each starts at the freshly revalidated project root,
  accepts ordinary input, resizes with the window, and remains independently
  selectable.
- Produce UTF-8, ANSI color, split output, and a background `sleep` job. Confirm
  rendering remains bounded and closing the tab requires explicit confirmation,
  ends the foreground/background session, leaves project files untouched, and
  leaves no QuireForge-owned process.
- Relaunch after an intentionally interrupted app session. Confirm metadata is
  shown as interrupted without invented scrollback or process ownership and can
  be dismissed safely.
- Confirm browser preview cannot start or simulate a shell, no clickable links
  are synthesized from output, and the Linux-account privilege warning remains
  distinct from Codex approvals.

## Manual Milestone 11C checklist

- Use only disposable repositories and private temporary app-data roots. Do not
  point cleanup validation at a user or project worktree.
- Force post-create registration failure, refresh inventory, and confirm only
  the retained exact managed-storage checkout receives an opaque recovery ID.
  Recover it and verify no file or branch changes.
- Preview cleanup from a different selected project. Verify source, current,
  attached, external, locked, prunable, dirty, conflicted, and submodule-dirty
  worktrees cannot reach destructive confirmation.
- After a valid preview, change `HEAD`, add an untracked file, reserve a related
  project, or replace the reviewed path with a symlink. Confirm each case fails
  closed and preserves the checkout.
- Configure clean/smudge/process filters and checkout hooks with marker files.
  Confirm create, explicit status, Git's internal removal check, and removal do
  not execute repository-controlled helpers.
- Remove one clean managed fixture. Confirm only its directory and inventory
  entry disappear, its branch remains, other worktrees remain, and no force or
  generic prune command is available.
- Force metadata retirement to fail after Git removal. Confirm the missing
  managed entry remains visible, then use a second non-destructive preview to
  finalize metadata while no filesystem deletion is retried.
- Exercise desktop/mobile destructive and recovery previews with keyboard,
  axe-core, overflow, complete repository validation, release build, isolated
  launch, and visual inspection before publication.

## Manual Milestone 11B checklist

- Use deterministic mock app-server processes and disposable repositories only;
  do not start a live or billable model turn.
- Start tasks in distinct attached worktree projects and confirm up to four run
  independently. A fifth start and a second start in the same project must fail
  closed without spawning another child.
- Interrupt one exact app conversation ID while another continues. Exercise
  independent poll and approval routing, then confirm every terminal path
  closes and waits for only its owned child and releases only its project.
- Refresh the webview with multiple native tasks active. Confirm the strict
  registry restores each active task with no Codex ID, cwd, process metadata,
  arguments, environment, raw output, or replayed event batch.
- Select each worktree task from the aggregate monitor and expand its live
  normalized activity. Confirm changed-file and conflict counts come from the
  read-only Git snapshot and never trigger conflict resolution or mutation.
- Exercise terminal, capacity, unavailable-Git, stale-poll, desktop/mobile,
  keyboard, axe-core, and overflow states. Confirm no task's late response can
  overwrite another task or a newer action on the same task.
- Restart against a fixture database with stale running rows and confirm the
  existing crash recovery marks them interrupted; do not claim native process
  ownership survives application exit.
- Confirm there is no remove, prune, cleanup, conflict-resolution, generic Git,
  cwd, executable, or argument-vector control. Run the complete repository and
  browser gates, warm release build, isolated launch, and visual inspection.

## Manual Milestone 11A checklist

- Use disposable repositories only. Inspect the user repository before and
  after verification and confirm routine tests created no user worktree or
  branch.
- Confirm inventory returns normalized branch/path/ownership/state records and
  never exposes object IDs, Git directories, common-directory paths, stderr,
  configuration, cwd, executable, or arbitrary arguments.
- Preview a bounded new branch and confirm the destination is generated beneath
  private app storage. Change source HEAD, create the branch elsewhere, or
  change repository identity before confirmation and confirm the plan fails
  closed.
- Configure a checkout filter and hook in a disposable repository; confirm
  managed creation runs neither. Confirm global/system configuration, prompts,
  pagers, credentials, and inherited shell environment are unavailable.
- Discover an external linked worktree and confirm it has no selectable project
  ID until the exact directory is chosen through the native picker. Reject a
  primary checkout, a worktree from another common directory, and a retargeted
  path.
- Force metadata registration to fail after fixture creation. Confirm the
  worktree remains on disk with a visible recovery path and QuireForge performs
  no implicit remove, prune, or cleanup.
- Confirm there is no remove, delete, prune, clean, generic checkout, reset,
  stash, remote, push, pull, or arbitrary Git control in native IPC or React.
- Run native/frontend suites, both Playwright viewports, axe-core, overflow,
  complete repository validation, warm release build, isolated schema-4 launch,
  and visual inspection before publication.

## Manual Milestone 10 checklist

- Attach repository roots and repository subdirectories; confirm status is
  limited to the exact attachment and never falls back to a parent, recent, or
  home directory.
- Confirm connected read-only repositories can be reviewed while missing,
  identity-changed, malformed, non-repository, conflicted, and submodule states
  fail closed or remain visibly non-reviewable.
- Exercise staged, working-tree, renamed, deleted, untracked, binary, long, and
  malformed fixtures. Confirm raw Git headers, stderr, object IDs, absolute
  paths, repository configuration, and non-UTF-8/control/directional paths do
  not cross IPC.
- Confirm every diff and editor request is present in a fresh status snapshot,
  stays within the attachment, and refuses symlinks or non-regular files.
- Confirm the webview cannot submit cwd, revisions, Git options, commands, or
  environment values and browser preview never fabricates Git data.
- Run component/bridge/native tests, both Playwright viewports, axe-core,
  overflow checks, the complete repository gate, unbundled native build, and an
  isolated launch. Inspect the QuireForge working copy before and after to prove
  routine mutation tests touched only their disposable temporary repositories.
- In a disposable repository, confirm stage/unstage preview one exact file,
  confirmation sends only the native token, a changed/expired preview fails,
  and Codex ownership or an index lock prevents the write.
- Confirm revert is limited to a bounded tracked regular-file modification,
  preserves staged content, offers one process-local recovery, refuses recovery
  after newer edits, and never deletes an untracked file.
- Confirm commit refuses staged paths outside a subdirectory attachment,
  conflicts, submodules, active repository operations, missing local identity,
  oversized blobs, sensitive filenames, and high-confidence secrets in files
  or the message. Confirm hooks, signing, editors, and prompts do not run and
  unrelated unstaged changes remain intact.
- Confirm Milestone 10 exposes no branch, worktree, reset, checkout, stash,
  remote, push, pull, or arbitrary Git action. The separately gated Milestone
  11A service may only create one bounded new branch as part of a confirmed
  app-managed worktree.

## Manual Milestone 9 checklist

- Use deterministic mock app-server fixtures only; do not start a billable
  model turn or approve a real command during routine validation.
- Confirm command, file, and permission requests correlate the exact native
  thread/turn/request/item while React receives only app-owned conversation,
  approval, and activity IDs.
- Confirm `acceptForSession`, policy-amendment objects, unstable file write-root
  grants, experimental tool requests, duplicate requests, stale IDs, and
  unadvertised decisions cannot be approved.
- Confirm permission approval echoes only a strictly parsed profile with turn
  scope; decline/cancel grant an empty profile; stop resolves a pending request
  before interrupting the exact turn.
- Confirm the project remains reserved while approval is pending and becomes
  available after every terminal path. Reopen a fixture database with the
  active status and verify existing crash recovery marks it interrupted without
  persisting or replaying approval content.
- Feed terminal controls, bidirectional controls, credential flags/environment
  names, credential-bearing URLs, internal/external/escaping paths, split
  secret assignments, oversized/incomplete output, raw tool arguments, and file
  diffs; confirm only bounded redacted normalized presentation crosses IPC.
- Confirm a stable app activity ID links item start, output/progress, and
  completion, and that the strict schema-v2 frontend rejects raw or unknown
  fields.
- Run focused native/frontend suites, the complete repository and browser
  gates, the warm unbundled native build, and an isolated launch smoke check.
- Confirm each activity is one keyboard-operable row, expansion stays open as
  lifecycle/output updates arrive, detail is normalized, and long output is
  tail-bounded without horizontal page overflow.
- Confirm the pending card displays only advertised decisions, sends the exact
  app conversation/approval IDs, rejects duplicate submission, and cannot be
  overwritten by a stale waiting-state poll. Run these checks with deterministic
  fixtures only; do not approve a real command during routine validation.

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
keyboard focus, native bridge status, resizing, and clean exit. Confirm that the
workspace DOM is visibly rendered rather than accepting a native title bar over
an empty black webview. The repository validator keeps Tauri
`freezePrototype: false` because the current verified production bundle cannot
mount against a frozen `Object.prototype`. On Linux the running application
must own `io.github.codeframe78.QuireForge` on the session bus. An unbundled
launch does not validate package installation or desktop-file naming; those
remain packaging-milestone obligations.

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
- Confirm an unsupported server request becomes a stable blocked state and no
  response is fabricated; reviewed stable approvals follow the Milestone 9A
  checklist above.
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
- Confirm the built `.htaccess`, `robots.txt`, sitemap, manifest, icons, and 404
  page are present.

Milestone 16C completed production-origin and Cloudflare-edge header,
desktop/mobile Lighthouse, route, overflow, and axe measurements. Both
Lighthouse profiles scored 100 for Performance, Accessibility, Best Practices,
and SEO; all 28 live route/profile combinations passed with no automatically
detectable axe violations. Future deployments must repeat these measurements,
and any miss against the published quality targets requires recorded
remediation.
