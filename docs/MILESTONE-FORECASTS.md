# Milestone Forecasts

Forecasts use ranges because model latency, downloads, failures, approvals, and
external services vary. Each future milestone receives a preliminary forecast
before manual model selection and a calibrated forecast after the selected
model, current system state, caches, and any approved preflight are verified.

Approval waiting time is reported separately from active Codex and local
command time. Performance measurements come from required project work; caches
are not deleted merely to create benchmarks.

## Milestone 3 — Desktop Scaffold Consolidation

| Field                      | Record                                                                                                                         |
| -------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Forecast date              | 2026-07-19                                                                                                                     |
| Recommended model          | GPT-5.6 Sol, High reasoning                                                                                                    |
| Preliminary forecast       | 18–30 active engineering hours; low confidence                                                                                 |
| Calibrated forecast        | Not produced; system-calibrated forecasting was introduced during final validation                                             |
| Observed active execution  | Approximately 35–55 minutes across two active work periods; approval delay excluded and exact end-to-end time not instrumented |
| User approval/waiting time | Present for milestone confirmation and sudo prerequisite installation; not measured reliably                                   |
| Sessions                   | Two active work periods separated by the host-prerequisite checkpoint                                                          |
| Usage intensity            | High model/tool activity; moderate local CPU use during cold Rust builds                                                       |
| Completion status          | Complete locally; not pushed, merged, packaged, or released                                                                    |

The preliminary forecast substantially overestimated implementation time. It
was created before repository-native Rust measurements existed and treated the
large roadmap label as conventional manual engineering time. The actual work
benefited from a narrow scaffold scope, strong existing architecture contracts,
small deterministic fixtures, fast frontend tooling, sufficient system memory,
and effective dependency/compiler caches. A percentage accuracy claim would be
misleading because active time and approval waiting were not instrumented from
the start.

### Required build and test observations

- Workspace dependency installation completed in about five seconds.
- The first successful native check took about 37 seconds.
- The first Rust test-profile build took about 44 seconds.
- The first unbundled release build took about 1 minute 18 seconds.
- Warm native validation fell to about five seconds in aggregate.
- The combined responsive/accessibility suites completed in about eight seconds.

### Unexpected work

- Tauri's Linux development headers required an owner-entered sudo operation.
- The light-theme accessibility check exposed contrast failures that were fixed
  before completion.
- Tauri generated local capability schemas that needed an explicit ignore rule.
- Cross-language contract drift was controlled with one shared JSON fixture and
  runtime TypeScript validation rather than adding a code-generation system.

### Forecasting lessons

- Separate model-driven implementation time from cold compiler/linker time.
- Use the measured warm Cargo gate when forecasting iterative adapter work.
- Reserve cold release builds for acceptance gates rather than ordinary edits.
- Keep approval-bound system dependencies outside active execution ranges.
- Reassess after the first real Codex adapter contract fixture; Milestone 4 has
  materially greater protocol and process-lifecycle uncertainty than the shell.

## Milestone 4 — Codex Process Adapter and Contracts

| Field                      | Record                                                                                                                                                           |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date              | 2026-07-19                                                                                                                                                       |
| Selected model             | GPT-5.6 Sol, High reasoning; manually confirmed                                                                                                                  |
| Preliminary forecast       | 2–4 active hours; about 2.25–4.5 total elapsed hours; low-to-medium confidence                                                                                   |
| Calibrated forecast        | 4–7 active hours; about 4.5–8 total elapsed hours across 2–3 sessions; low-to-medium confidence                                                                  |
| Calibration cause          | The generated experimental protocol bundle contained 337 files / about 3.16 MB, and process/redaction failure paths were broader than the preliminary assumption |
| Observed active execution  | Approximately 25–35 minutes from branch creation through implementation, required validation, documentation, and commit preparation; approval delay excluded     |
| Local command time         | Approximately 2–4 minutes across required dependency, compile, test, build, browser, live-probe, and repeated failure-correction commands                        |
| User approval/waiting time | Manual model and start confirmations occurred before branch creation; waiting time was not included                                                              |
| Sessions                   | One active implementation session with recoverable contract, transport, UI, and validation checkpoints                                                           |
| Usage intensity            | High model/tool activity; low-to-moderate local CPU and memory use                                                                                               |
| Completion status          | Complete locally; not pushed, merged, authenticated, packaged, deployed, or released                                                                             |

Both forecasts overestimated execution time. The calibrated forecast correctly
identified the protocol and lifecycle risks but assumed substantially more
manual implementation/debugging time. Existing Milestone 0 architecture,
official app-server discovery, small selected schema scope, deterministic shell
fixtures, GPT-5.6 Sol's implementation speed, warm caches, and fast local builds
compressed the critical path without reducing the accepted milestone scope.

### Required build and test observations

- Schema generation completed in about 0.33 seconds; only four reviewed schemas
  and their hashes were retained.
- The first feature-expanded check took about 5.9 seconds and first test build
  about 15.1 seconds.
- The live non-billable probe took about 0.32 seconds and left no app-server
  process.
- The successful release build took about 19.5 seconds at roughly 1.25 GiB peak
  RSS.
- Full non-browser validation took about 23.1–29.7 seconds; all 14 browser tests
  took about 9.8 seconds.

### Unexpected work and lessons

- Release compilation exposed a Tauri async-command `Result` requirement not
  caught by ordinary checking; release build remains an essential gate.
- Concurrent build and preview testing can serve stale `dist`; keep them
  sequential even when resource headroom permits concurrency.
- Full upstream schema size is not a reason to commit it. Generate in temporary
  storage and retain only reviewed method schemas with hashes.
- Process failure fixtures should block without spawning helper children, so
  timeout tests validate the owned process rather than create orphan risk.

## Milestone 5 — Authentication and Onboarding

| Field                      | Record                                                                                                                                                                      |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date              | 2026-07-19                                                                                                                                                                  |
| Selected model             | GPT-5.6 Sol, High reasoning; manually confirmed                                                                                                                             |
| Preliminary forecast       | 2–4 active hours; about 2.25–4.5 total elapsed hours across 1–2 sessions; low-to-medium confidence                                                                          |
| Calibrated forecast        | 2–4 active hours; about 2.25–4.5 total elapsed hours across 1–2 sessions; medium confidence                                                                                 |
| Calibration basis          | Stable installed authentication schemas, ample current memory, warm Milestone 4 caches, six Cargo workers, and two Playwright workers                                       |
| Observed active execution  | Approximately 25–40 minutes from branch creation through implementation, required validation, visual checks, documentation, and commit preparation; approval delay excluded |
| Local command time         | Approximately 5–8 minutes across schema inspection, dependency resolution, compile, test, browser-cache restoration, release build, smoke test, and validation commands     |
| User approval/waiting time | Manual model, reasoning-strength, audit, and start confirmations occurred before branch creation; waiting time was not included                                             |
| Sessions                   | One active implementation session with recoverable native-service, UI, validation, and documentation checkpoints                                                            |
| Usage intensity            | High model/tool activity; moderate local CPU use and low memory pressure during cold native builds                                                                          |
| Completion status          | Complete locally; not pushed, merged, logged in, logged out, packaged, deployed, or released                                                                                |

The forecast overestimated model-driven implementation and debugging time. The
stable installed protocol schemas, existing owned-process adapter, strict shared
fixture pattern, fast local builds, and deterministic mocked login flows reduced
the critical path without reducing the accepted scope. The live compatibility
check remained read-only; real browser/device authentication and logout were
intentionally excluded because they mutate external authentication state.

### Required build and test observations

- Stable schema calibration completed in about 0.43 seconds, and generation of
  the 13 reviewed schemas completed in about 0.98 seconds.
- The first dependency-expanded native check took about 39.4 seconds; the first
  authentication test build took about 55.6 seconds.
- The read-only live `account/read` probe took about 0.57 seconds and left no
  app-server process.
- The release build took about 1 minute 18 seconds at roughly 1.37 GiB peak RSS.
- Full non-browser validation took about 24.3–25.7 seconds; the final combined
  browser suites took about 8.4 seconds after the pinned cache was restored.

### Unexpected work and lessons

- The native browser opener added a cold dependency path, returning release
  timing to the Milestone 3 cold baseline; subsequent native gates were warm.
- The pinned Playwright Chromium revision was absent. Restoring only that
  project cache was sufficient; no system package or persistent configuration
  change was required.
- Authentication callback ownership belongs in the native service: exact login
  correlation, cancellation races, raw error reduction, and process shutdown
  are easier to enforce before any value reaches IPC.
- A fixed native browser command with no URL parameter keeps the untrusted
  webview from becoming an arbitrary URL opener.

## Milestone 6 — Workspace Attachment and QuireForge Metadata

| Field                                  | Record                                                                                                                                                                                    |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date                          | 2026-07-19                                                                                                                                                                                |
| Selected model                         | GPT-5.6 Sol, XHigh reasoning; manually confirmed for Milestone 6A                                                                                                                         |
| Preliminary forecast                   | 3–6 active hours; about 3.5–7 total elapsed hours across 1–2 sessions; low-to-medium confidence                                                                                           |
| Calibrated complete-milestone forecast | 2.5–5 active hours; 15–35 minutes of local commands; about 3–6 total elapsed hours; medium confidence                                                                                     |
| Calibrated Milestone 6A forecast       | 1.5–3 active hours; 8–20 minutes of local commands; about 1.75–3.5 total elapsed hours; medium confidence                                                                                 |
| Observed Milestone 6A execution        | Approximately 45–70 active minutes through implementation, security review, expanded native tests, documentation, final validation, and checkpoint publication; approval delay excluded   |
| Observed local command time            | Approximately 3–5 minutes across dependency-expanded checking, targeted/full Rust gates, and repeated full repository validation after correcting stale expectations; file reads excluded |
| User approval/waiting time             | Manual model, reasoning, audit, and start confirmations occurred before implementation; waiting time was excluded                                                                         |
| Sessions                               | Two gated checkpoints: native core followed by frontend/integration                                                                                                                       |
| Usage intensity                        | High model/tool activity; moderate CPU and low memory pressure                                                                                                                            |
| Milestone 6B selected model            | GPT-5.6 Sol, High reasoning; manually confirmed                                                                                                                                           |
| Preliminary Milestone 6B forecast      | 1–2.25 active hours; 5–15 minutes of local commands; about 1.25–2.75 total elapsed hours; medium confidence                                                                               |
| Calibrated Milestone 6B forecast       | 1–2 active hours; 5–12 minutes of local commands; about 1.25–2.5 total elapsed hours; medium confidence                                                                                   |
| Observed Milestone 6B execution        | Approximately 35–55 active minutes through strict contract/UI implementation, tests, visual review, release/native verification, documentation, and final gates; approval delay excluded  |
| Milestone 6B command observations      | Full non-browser gate 30.0 seconds at about 1.23 GiB peak RSS; combined browser gate 8.3 seconds at about 251 MiB; unbundled release build 67.0 seconds at about 1.36 GiB                 |
| Completion status                      | Milestone 6 complete; merged to `main` in PR #3                                                                                                                                           |

The Milestone 6A forecast overestimated implementation and command time. The
accepted architecture, fast bundled-SQLite compilation, warm Tauri caches,
small transaction surface, and deterministic temporary-directory fixtures
compressed the critical path without reducing scope. XHigh reasoning was
useful for migration invariants, path identity, TOCTOU handling, permission
boundaries, and fail-closed diagnostics. The planned frontend/integration
checkpoint should be reevaluated at High reasoning rather than automatically
retaining XHigh.

Milestone 6B also completed below its forecast. The native contract was already
stable, frontend caches were warm, and deterministic fixtures kept debugging
short. High reasoning was sufficient for the strict schema invariants,
confirmation state machine, path-boundary review, and accessible destructive-
action confirmations; XHigh would not have provided a material advantage for
the predominantly integration and verification work. Across both checkpoints,
Milestone 6 took approximately 80–125 active minutes plus roughly 6–11 minutes
of local command time, excluding approval waits.

### Milestone 6A required observations and lessons

- The first dependency-expanded `cargo check` took about 14.2 seconds at about
  718 MiB peak RSS and locked 22 packages, including bundled SQLite and the
  native dialog plugin.
- The warm corrected `cargo check` took about 1.5 seconds at about 452 MiB peak
  RSS.
- The final 20-test project suite took about 5.0 seconds at about 1.24 GiB peak
  RSS; all tests passed.
- Warm Clippy with warnings denied took about 2.0 seconds at about 495 MiB peak
  RSS. The final full non-browser repository gate took about 25.8 seconds at
  about 605 MiB peak RSS and included 40 passing Rust tests with two deliberate
  live probes ignored.
- No swap activity, OOM, thermal concern, GPU workload, cache deletion, or
  source-directory mutation occurred.
- Confirmation must bind not only path/device/mount/Git identity but also the
  detected `AGENTS.md` and `.codex` state; a deterministic test now prevents a
  configuration-change TOCTOU regression.
- Read-only state needs both directory mode and mount-option evidence. Mount ID
  and filesystem type remain advisory signals and preflight fails closed if
  they cannot be verified.

### Milestone 6B required observations and lessons

- The strict TypeScript fixture is serialized by Rust and parsed by the
  frontend, preventing silent drift at the native/webview boundary.
- The warm desktop production build took about 2.1 seconds at about 300 MiB
  peak RSS. The desktop-only Playwright suite took about 5.7 seconds at about
  236 MiB peak RSS.
- The unbundled release build took about 67.0 seconds at about 1.36 GiB peak
  RSS. An isolated native launch owned the exact D-Bus identity and created its
  temporary app-data directory and SQLite file with owner-only permissions.
- The final non-browser repository gate took about 30.0 seconds at about
  1.23 GiB peak RSS, including 27 desktop tests, 3 website tests, and 41 passing
  Rust tests with two deliberate live probes ignored.
- The final combined desktop/website browser gate passed 14 tests in about
  8.3 seconds at about 251 MiB peak RSS.
- No swap growth, OOM, thermal concern, GPU-compute workload, cache deletion,
  Codex-owned state change, source-directory mutation, or package/deployment
  operation occurred.

## Milestone 7 — Conversation MVP

### Milestone 7A — Native Conversation Runtime

| Field                       | Record                                                                                                                                                                                                                      |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-19                                                                                                                                                                                                                  |
| Selected model              | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                            |
| Preliminary forecast        | 3–5 active hours; 15–35 minutes of local commands; about 3.5–6 total elapsed hours; medium confidence                                                                                                                       |
| Calibrated forecast         | 2.5–4.5 active hours; 10–25 minutes of local commands; about 3–5.25 total elapsed hours across 1–2 sessions; medium confidence                                                                                              |
| Calibration basis           | Clean Milestone 6 `main`, stable Codex 0.144.6 schemas, 46 GiB available RAM, warm 13 GiB Rust target cache, warm pnpm/browser caches, four Cargo workers, and two Playwright workers                                       |
| Observed active execution   | Approximately 45–65 minutes through protocol review, implementation, deterministic tests, security correction, documentation, full validation, release/native verification, and commit preparation; approval delay excluded |
| Observed local command time | Approximately 3–5 minutes across schema generation, iterative checking/tests, the 27.81-second full gate, 8.48-second browser gate, 28.86-second release build, and isolated native smoke check                             |
| User approval/waiting time  | Manual model, reasoning-strength, audit, and milestone-start confirmations occurred before implementation; waiting time was excluded                                                                                        |
| Sessions                    | One active implementation session with recoverable contract, native-service, validation, and publication checkpoints                                                                                                        |
| Usage intensity             | High model/tool activity; moderate CPU use and low memory pressure                                                                                                                                                          |
| Completion status           | Milestone 7A complete; Milestone 7B user interface pending                                                                                                                                                                  |

The forecast overestimated model-driven implementation and build time. The
existing app-server process owner, project preflight service, SQLite repository,
strict shared-fixture pattern, and warm caches compressed the critical path
without reducing scope. XHigh reasoning was useful for protocol correlation,
path/task lifetime binding, exact interruption, reference-only persistence,
approval fail-closed behavior, and the final security review. The review found
and corrected a startup-error cleanup path so a spawned child is explicitly
closed and waited rather than relying only on kill-on-drop.

### Milestone 7A required observations and lessons

- Full experimental schema calibration took about 0.32 seconds at about 45 MiB
  peak RSS; generation of the 28 reviewed schemas took about 1.01 seconds at
  about 141 MiB and was idempotent.
- The final conversation-focused Rust suite passed 9 tests after about 5–6
  seconds of incremental compilation. The full locked Rust suite passed 50
  tests with 2 deliberate live probes ignored.
- The final full non-browser repository gate took 27.81 seconds at about 660
  MiB peak RSS. The combined browser gate passed 14 tests in 8.48 seconds at
  about 251 MiB peak RSS.
- The warm unbundled release build took 28.86 seconds at about 1.45 GiB peak
  RSS, materially faster than Milestone 6B's 67-second release measurement.
- An isolated native launch owned the exact D-Bus identity, migrated an empty
  database through schema version 2, created its application-data directory and
  metadata database with `0700`/`0600` permissions, and left no app-server child.
- No live model turn, approval decision, package, deployment, source-directory
  mutation, cache deletion, OOM, heavy swapping, or GPU-compute workload
  occurred.
- Milestone 7B is predominantly frontend integration and interaction design,
  so its reasoning strength and forecast must be gated independently rather
  than automatically retaining XHigh.

### Milestone 7B — Conversation User Interface

| Field                       | Record                                                                                                                                                                                                                       |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-19                                                                                                                                                                                                                   |
| Selected model              | GPT-5.6 Sol, High reasoning; manually confirmed                                                                                                                                                                              |
| Preliminary forecast        | 1.5–3 active hours; 5–15 minutes of local commands; about 2–4 total elapsed hours; medium confidence                                                                                                                         |
| Calibrated forecast         | 1.5–3 active hours; 5–15 minutes of local commands; about 2–4 total elapsed hours in one session; medium-to-high confidence                                                                                                  |
| Calibration basis           | Clean Milestone 7A `main`, stable strict frontend/native contracts, 45–46 GiB available RAM, warm pnpm/Vite/Playwright/Rust caches, four Cargo workers, and two Playwright workers                                           |
| Observed active execution   | Approximately 30–50 minutes through shell integration, UI implementation, focused tests, responsive/accessibility review, documentation, full validation, and commit preparation; approval and hosted-runner delays excluded |
| Observed local command time | Approximately 2–4 minutes across iterative type/lint/test runs, a 2.37-second frontend build, a 5.54-second desktop browser gate, full repository gates, and release/native checks                                           |
| User approval/waiting time  | Manual reasoning and milestone-start confirmations occurred before implementation and were excluded; GitHub-hosted CI runner queue time was also tracked separately                                                          |
| Sessions                    | One active implementation session with recoverable UI, test, visual-review, validation, and publication checkpoints                                                                                                          |
| Usage intensity             | High model/tool activity; low local CPU and memory pressure                                                                                                                                                                  |
| Completion status           | Complete and verified locally; publication remains separately recorded in repository history                                                                                                                                 |

The forecast overestimated implementation and debugging time. Milestone 7A had
already established the strict request/snapshot contracts and fixed native IPC,
so 7B could remain a focused React state-and-presentation layer with no new
dependencies or native protocol work. High reasoning was appropriate for the
poll lifecycle, event deduplication, security-control gating, terminal states,
and the review boundary; a stronger setting would not have materially improved
the largely frontend implementation.

### Milestone 7B required observations and lessons

- The frontend production build completed in 2.37 seconds at about 316 MiB peak
  RSS, transforming 112 modules with warm caches.
- The focused desktop browser gate passed six desktop/mobile tests in 5.54
  seconds at about 244 MiB peak RSS, including axe-core and overflow checks.
- The component and integration suite passed 42 desktop tests. The full
  repository gate passed those plus 3 website tests and 50 Rust tests, with 2
  deliberate live probes ignored. Coverage includes runtime-derived controls,
  verified-project gating, unsafe-policy rejection, exact-ID stop,
  start/poll/completion wiring, event deduplication, and the 256-event UI bound.
- The combined desktop/website browser gate passed all 14 tests in 8.33 seconds
  at about 251 MiB peak RSS. The warm unbundled release build completed in
  28.35 seconds at about 1.46 GiB peak RSS, and an isolated native launch verified the exact D-Bus name,
  `0700`/`0600` metadata permissions, and no remaining app-server child.
- Desktop and mobile screenshots confirmed responsive stacking, readable form
  controls, visible preview limitations, and no horizontal overflow.
- No dependency install, native protocol change, live model call, approval
  decision, cache deletion, heavy swapping, OOM, or GPU-compute workload was
  required.
- One validation invocation initially lacked the installed Cargo directory in
  the package-runner `PATH`; preserving the existing path fixed the environment
  issue without installation or configuration changes. An initial smoke-check
  assertion also guessed the old database filename and was corrected to locate
  `metadata.sqlite3` only within the fresh temporary XDG root.
- Milestone 8 adds native lifecycle/reconciliation work and should receive a new
  model and reasoning gate rather than inheriting High automatically.

### Milestone 8A — Native Session Lifecycle and Recovery

| Field                       | Record                                                                                                                                                                                                                                                                                                    |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-19                                                                                                                                                                                                                                                                                                |
| Selected model              | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                                                                                                          |
| Preliminary forecast        | 3–5 active hours; 15–35 minutes of local commands; about 3.5–6 total elapsed hours; medium confidence                                                                                                                                                                                                     |
| Calibrated forecast         | 4–7 active hours; 15–35 minutes of local commands; about 4.5–8 total elapsed hours across up to two sessions; medium confidence                                                                                                                                                                           |
| Calibration basis           | Clean merged Milestone 7B `main`, reviewed Codex 0.144.6 lifecycle schemas, 44 GiB available RAM, 726 GiB free NVMe space, warm 13 GiB Cargo target and pnpm/browser caches, four Cargo workers, and two Playwright workers                                                                               |
| Observed active execution   | Approximately 60–90 minutes through protocol review, schema/storage changes, native lifecycle implementation, strict frontend contracts, deterministic tests, security review, documentation, full gates, release/native verification, and commit preparation; approval and hosted-runner delays excluded |
| Observed local command time | Approximately 3–5 minutes across schema generation, iterative checks/tests, the final 29.45-second full gate, 8.60-second browser gate, final 25.13-second release build, and isolated native smoke check                                                                                                 |
| User approval/waiting time  | Manual model, reasoning-strength, audit, and milestone-start confirmations occurred before implementation and were excluded; GitHub-hosted queue time is tracked separately                                                                                                                               |
| Sessions                    | One active implementation session with recoverable schema, storage, native boundary, frontend contract, validation, and publication checkpoints                                                                                                                                                           |
| Usage intensity             | High model/tool activity; moderate local CPU and low memory pressure                                                                                                                                                                                                                                      |
| Completion status           | Complete and verified locally; publication is recorded in repository history                                                                                                                                                                                                                              |

The calibrated forecast substantially overestimated implementation time. The
Milestone 7A serialized process owner, strict conversation types, reference-only
SQLite repository, shared fixture pattern, and warm build graph could be
extended without a second runtime owner or dependency change. XHigh reasoning
was still useful for protocol-field minimization, crash semantics, exact
identity/cwd validation, archive-versus-delete separation, fork rollback, and
the final native/webview security review.

### Milestone 8A required observations and lessons

- Schema generation took 0.48 seconds at about 45 MiB peak RSS and expanded the
  committed reviewed subset from 28 to 42 schemas without retaining the full
  generated bundle.
- SQLite schema version 3 needs only parent application lineage and archive
  timestamp fields. Startup recovery can safely clear active-turn ownership and
  mark stale starting/running/stopping rows interrupted without inspecting or
  mutating Codex transcript content.
- Resume, fork, archive, and restore remain serialized with active turns. Native
  code reads and validates the stored thread before mutation, sends no rollout
  path/history/config/runtime roots, and correlates exact UUIDv7/cwd responses.
- Current and archived listing is batched across exact revalidated cwd filters,
  bounded to eight 256-item pages per state, and matches only already-owned app
  references. Unrelated Codex threads are never imported.
- The full repository gate passed 51 JavaScript and 57 Rust tests with 2
  deliberate live probes ignored. Browser regression passed 14 tests; the warm
  release build and isolated native launch also passed.
- Available RAM stayed above 43 GiB, swap did not grow, and no OOM, throttling,
  download, cache deletion, live model call, approval decision, project-file
  mutation, destructive thread operation, or GPU workload occurred.
- Milestone 8B is now a bounded frontend/session-presentation checkpoint. Its
  preliminary range is approximately 2–4 active hours, 5–15 minutes of local
  commands, and 2.5–5 total elapsed hours in one session, medium confidence;
  model and reasoning must still receive a separate gate before work begins.

### Milestone 8B — Session History, Search, and Tabs

| Field                       | Record                                                                                                                                                                                                                                                                                                     |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-19                                                                                                                                                                                                                                                                                                 |
| Selected model              | GPT-5.6 Sol, High reasoning; manually confirmed                                                                                                                                                                                                                                                            |
| Preliminary forecast        | 2–4 active hours; 5–15 minutes of local commands; about 2.5–5 total elapsed hours; medium confidence                                                                                                                                                                                                       |
| Calibrated forecast         | 1.5–3 active hours; initially 5–12, then revised to 8–18 minutes of local commands; about 2–4 total elapsed hours in one session; medium-to-high confidence                                                                                                                                                |
| Calibration basis           | Clean merged Milestone 8A `main`, confirmed stable `thread/list.searchTerm` and optional title fields, 44 GiB available RAM, 726 GiB free NVMe, warm 13 GiB Cargo target and pnpm/browser caches, four Cargo workers, and two Playwright workers                                                           |
| Observed active execution   | Approximately 35–55 minutes through contract design, native title projection, React history/tabs/actions, deterministic tests, accessibility corrections, documentation, security/diff review, full gates, release/native verification, and commit preparation; approval and hosted-runner delays excluded |
| Observed local command time | Approximately 4–7 minutes across focused native/frontend checks, repeated responsive accessibility checks, the final 41.37-second full gate, 8.81-second combined browser gate, 30.97-second release build, and isolated native smoke check                                                                |
| User approval/waiting time  | Manual model/reasoning and milestone-start confirmations were excluded; the user's new expandable-process-detail requirement was routed to Milestone 9 without pausing or expanding 8B                                                                                                                     |
| Sessions                    | One active implementation session with recoverable contract, UI/test, documentation, validation, and publication checkpoints                                                                                                                                                                               |
| Usage intensity             | High model/tool activity; moderate local CPU and low memory pressure                                                                                                                                                                                                                                       |
| Completion status           | Complete and verified locally; publication is recorded in repository history                                                                                                                                                                                                                               |

The calibrated forecast overestimated implementation time. Milestone 8A had
already established fixed lifecycle commands, strict session schemas, app-owned
lineage, complete reconciliation, and deterministic mock process patterns.
Codex 0.144.6 already exposed both the stable title filter and optional title,
so 8B needed a bounded second projection rather than a new content index or
database migration. High reasoning was appropriate for keeping filtered search
separate from missing-state reconciliation, UI/task state coordination, and
the accessibility/security review; XHigh would not have materially improved
the predominantly bounded integration work.

### Milestone 8B required observations and lessons

- A filtered title result cannot stand in for complete reconciliation. Native
  code first obtains the complete current/archived set, requires filtered IDs
  to be a subset, and only then intersects with app-owned references.
- Titles are trimmed, control-safe, bounded to 256 characters, exposed only in
  the version 2 session snapshot, and never persisted to SQLite. Paths, native
  IDs, previews, transcripts, raw status objects, and protocol payloads remain
  outside React.
- Component and shell tests cover project/fork grouping, exact-ID actions,
  keyboard tabs, search/clear/refresh, unavailable and preview states, mobile
  overflow, and axe-core. The first browser run found and corrected tablist
  ownership and light-theme primary-button contrast before the gate.
- The full gate passed 57 JavaScript tests and 58 Rust tests with 2 deliberate
  live probes ignored. The combined browser gate passed 16 desktop/website
  tests; the release build and isolated native launch also passed.
- The full gate took 41.37 seconds, about 40% above the prior 29.45-second
  baseline because the changed Rust boundary recompiled and the suite grew. The
  local-command forecast was revised, but overall elapsed time remained within
  the calibrated range.
- Available RAM remained about 44 GiB, swap stayed at about 175 MiB with zero
  measured swaps, and no OOM, throttling, dependency download, cache deletion,
  live model call, approval decision, deletion, project-file mutation, or GPU
  workload occurred.
- Selectable expandable real-time command/tool/file/process details are now an
  explicit Milestone 9 requirement and require a separate normalized redacted
  event contract rather than raw protocol display.

### Milestone 9A — Native Approvals and Detailed Activity Contract

| Field                       | Record                                                                                                                                                                                                                                                                                                        |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-19                                                                                                                                                                                                                                                                                                    |
| Selected model              | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                                                                                                              |
| Preliminary forecast        | 4–7 active hours; 15–35 minutes of local commands; about 5–8.5 total elapsed hours across one or two sessions; medium confidence                                                                                                                                                                              |
| Calibrated forecast         | 4–7 active hours; 15–35 minutes of local commands; about 5–8.5 total elapsed hours across one or two sessions; medium confidence                                                                                                                                                                              |
| Calibration basis           | Clean merged Milestone 8 `main`, reviewed Codex 0.144.6 request/activity schemas, 44 GiB available RAM, 726 GiB free NVMe, warm 13 GiB Cargo target and dependency/browser caches, four Cargo workers, and two Playwright workers                                                                             |
| Observed active execution   | Approximately 1.5–2.5 hours through protocol/security review, strict parsing, native state/decision handling, redaction, schema-v2 IPC, deterministic tests, minimal presentation, documentation, full gates, release/native verification, and commit preparation; approval and hosted-runner delays excluded |
| Observed local command time | Approximately 5–9 minutes across schema generation, iterative checks/tests, the 37.57-second full gate, browser gates, 32.03-second release build, and isolated native smoke checks                                                                                                                           |
| User approval/waiting time  | Manual model/reasoning, system-audit, milestone-start, and publication approvals occurred outside execution time; GitHub-hosted queue time is tracked separately                                                                                                                                              |
| Sessions                    | One logical implementation session with a compacted handoff and recoverable parser, state-machine, frontend-contract, documentation, validation, and publication checkpoints                                                                                                                                  |
| Usage intensity             | Very high model/tool activity; moderate local CPU and low memory pressure                                                                                                                                                                                                                                     |
| Completion status           | Milestone 9A complete; merged to `main` in PR #9                                                                                                                                                                                                                                                              |

The forecast substantially overestimated implementation time. The existing
serialized process owner, app-reference model, strict fixtures, conservative
crash recovery, and warm build graph could be extended without a database
migration, dependency change, or second runtime task. XHigh reasoning was still
appropriate for server/client request-ID collision, authority scope, exact
correlation, cancel ordering, streamed-secret redaction, unsupported request
handling, and the final native/webview security review.

### Milestone 9A required observations and lessons

- The reviewed schema generator took 1.04 seconds at about 141 MiB peak RSS and
  expanded the committed Codex 0.144.6 subset from 42 to 52 files.
- Only stable command, file-change, and permission requests are accepted.
  Session acceptance and policy amendments are filtered out; unstable
  session-wide file write-root grants cannot be approved; permission grants are
  strictly parsed and limited to turn scope.
- Native request/item/thread/turn identity remains behind Tauri. React receives
  app-owned UUIDv7 approval/activity IDs, and deterministic tests cover stale
  IDs, unavailable decisions, matching numeric client/server IDs, and
  cancel-before-interrupt ordering.
- Line-boundary buffering prevents a credential assignment split across output
  chunks from bypassing redaction. Terminal/bidirectional controls, raw tool
  arguments, file diffs, external/escaping paths, and credential-shaped values
  do not enter the schema-v2 snapshot.
- The full non-browser gate took 37.57 seconds at about 1.25 GiB peak RSS and
  passed 59 JavaScript tests plus 68 Rust tests, with two deliberate live probes
  ignored. The warm unbundled release build took 32.03 seconds at about 1.55 GiB
  peak RSS.
- PR #9 and the merge commit's `main` workflow both passed all repository,
  website, desktop, browser, native-test, and unbundled-release checks. Their
  cold hosted desktop jobs took 9 minutes 3 seconds and 9 minutes 38 seconds,
  respectively; preserving each active run avoided discarding compilation
  progress.
- Desktop/mobile activity-fixture checks passed accessibility and overflow
  analysis. Visual inspection confirmed readable waiting, command detail,
  progress, and approval-request presentation; the selectable expanded and
  decision-control UX remains Milestone 9B.
- An isolated release launch owned the exact D-Bus name, opened schema version
  3 with `0700`/`0600` metadata permissions, started no Codex child, and cleaned
  up its temporary XDG tree.
- Available RAM remained about 44 GiB, swap stayed at about 175 MiB with zero
  measured swaps, and no OOM, thermal issue, dependency download, clean build,
  live model turn, real approval, persistent grant, source mutation, or GPU
  workload occurred.
- Milestone 9B is predominantly interaction/presentation work but still handles
  explicit user consent. It requires a fresh model/reasoning gate; a preliminary
  range is approximately 2–4 active hours, 5–15 minutes of local commands, and
  2.5–5 total elapsed hours in one session, medium confidence.

### Milestone 9B — Selectable Activity and Approval Interface

| Field                       | Record                                                                                                                                                                                                                                       |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-20                                                                                                                                                                                                                                   |
| Selected model              | GPT-5.6 Sol, High reasoning; manually confirmed                                                                                                                                                                                              |
| Preliminary forecast        | 2–4 active hours; 5–15 minutes of local commands; about 2.5–5 total elapsed hours in one session; medium confidence                                                                                                                          |
| Calibrated forecast         | 2–4 active hours; 5–15 minutes of local commands; about 2.5–5 total elapsed hours in one session; medium confidence                                                                                                                          |
| Calibration basis           | Clean merged Milestone 9A `main`, 44 GiB available RAM, low host load, 726 GiB free NVMe, warm Cargo/pnpm/Playwright caches, existing schema-v2 activity and decision bridge, four Cargo workers, and two Playwright workers                 |
| Observed active execution   | Approximately 20–35 minutes through contract review, implementation, deterministic tests, documentation, complete gates, rendered-state inspection, security/diff review, and commit preparation; approval and hosted-runner delays excluded |
| Observed local command time | Approximately 3–5 minutes across iterative checks, the 36.70-second full gate, 9.99-second browser gate, 30.73-second release build, focused visual trace, and isolated launch check                                                         |
| User approval/waiting time  | Manual model/reasoning and milestone-start confirmations occurred before branch creation and were excluded; hosted-runner queue time is tracked separately                                                                                   |
| Sessions                    | One logical implementation session with recoverable contract, UI, test, documentation, and publication checkpoints                                                                                                                           |
| Usage intensity             | High model/tool activity; moderate local CPU and low memory pressure                                                                                                                                                                         |
| Completion status           | Complete and verified locally; publication recorded in repository history                                                                                                                                                                    |

The forecast overestimated active implementation time. Milestone 9A had already
established strict schemas, native redaction, exact approval correlation, and a
fixed decision bridge, so 9B stayed within a React-derived view, App polling
coordination, styling, deterministic fixtures, and documentation. High
reasoning remained appropriate for the user-consent surface, duplicate-submit
guard, stale-poll race, event aggregation bounds, and final trust-boundary
review. A lower setting would have increased the risk of presenting an
unadvertised decision or allowing an older waiting snapshot to overwrite a
completed approval.

### Milestone 9B required observations and lessons

- Activity lifecycle and output events can be grouped entirely in React by the
  app-owned activity ID without widening IPC or persisting output. The derived
  view caps history at 64 activities and each output tail at 32 KiB.
- Expansion state is keyed by the stable app activity ID, so a row remains open
  as polling replaces its normalized status and adds output.
- The approval card renders only the snapshot's advertised decision enum.
  A synchronous single-flight ref prevents double submission before React can
  disable the controls, while App suspends polling for the entire decision.
- The complete repository gate passed in 36.70 seconds at about 1.25 GiB peak
  RSS: 61 desktop JavaScript tests, 3 website tests, and 68 Rust tests passed;
  2 deliberate live probes remained ignored.
- The combined desktop/website browser gate passed all 18 desktop/mobile checks
  in 9.99 seconds at about 283 MiB peak RSS. Axe found no violations, no
  horizontal overflow occurred, and the fixture completed the exact approval
  transition.
- Visual trace inspection confirmed that the waiting card, three advertised
  decisions, expanded command detail/output, and completed activity state are
  readable in place. The isolated native launch created schema migrations 1–3
  with `0700`/`0600` metadata permissions and no child remained after exit.
- The first aggregate gate stopped at formatting because the transient command
  environment omitted the already-installed Cargo directory from `PATH`.
  Explicitly preserving that path fixed the harness; no repository or system
  configuration changed.
- Available RAM remained about 44 GiB, swap stayed near 175 MiB, measured
  commands reported zero swaps, and there was no OOM, throttling, dependency
  download, cache deletion, clean build, live model turn, real approval,
  deployment, or GPU workload.
- Milestone 10 requires a fresh gate. It adds native Git status/diff and
  mutation-sensitive workflows, so its reasoning and forecast must be based on
  the exact Git architecture and approval boundaries rather than reusing 9B's
  presentation-oriented setting.

## Milestone 10 — Git and Diff Review

### Milestone 10A — Read-only Status, Diff, and Editor Review

| Field                       | Record                                                                                                                                                                                                                                                                                                                                        |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-20                                                                                                                                                                                                                                                                                                                                    |
| Selected model              | GPT-5.6 Sol, High reasoning; manually confirmed                                                                                                                                                                                                                                                                                               |
| Preliminary forecast        | Approximately 4–6 active hours; 15–30 minutes of local commands; about 4.5–7 total elapsed hours in one or two sessions; medium confidence                                                                                                                                                                                                    |
| Calibrated forecast         | 3.5–5.5 active hours; 10–25 minutes of local commands; about 4–6.5 total elapsed hours in one session; medium confidence                                                                                                                                                                                                                      |
| Calibration basis           | Clean synchronized Milestone 9 `main`, installed Git 2.53.0, 43–44 GiB available RAM, low host load, 726 GiB free NVMe, warm 13 GiB Cargo target and pnpm/browser caches, no Git library dependency, four Cargo workers, and two Playwright workers                                                                                           |
| Observed active execution   | Approximately 55–80 minutes through repository/Git architecture review, native process and path contract, React interface, deterministic tests, accessibility correction, documentation, full gates, release/native verification, visual inspection, security/diff review, and commit preparation; approval and hosted-runner delays excluded |
| Observed local command time | Approximately 5–9 minutes across focused checks, the 31.72-second publication repository gate, 11.64-second combined browser gate, 31.18-second final release build, temporary-repository tests, rendered-state capture, and isolated launch                                                                                                  |
| User approval/waiting time  | Manual model/reasoning and milestone-start confirmations occurred before implementation and were excluded; publication and hosted-runner waiting are tracked separately                                                                                                                                                                       |
| Sessions                    | One logical implementation session with recoverable native-contract, frontend, test, documentation, validation, and publication checkpoints                                                                                                                                                                                                   |
| Usage intensity             | High model/tool activity; moderate local CPU and low memory pressure                                                                                                                                                                                                                                                                          |
| Completion status           | Complete and verified locally; publication pending at this record checkpoint                                                                                                                                                                                                                                                                  |

The forecast substantially overestimated implementation time. Project identity,
strict shared fixtures, fixed Tauri command patterns, the existing opener
dependency, and warm build caches were directly reusable. Splitting mutation
work into 10B also kept 10A's authority genuinely read-only instead of forcing
approval and recovery design into the status/diff checkpoint. High reasoning
was appropriate for Git configuration isolation, attached-subdirectory scope,
read-only repositories, output/process bounds, stale path revalidation,
symlink/deceptive-path refusal, and the final security review. XHigh would not
have materially improved this closed read-only contract.

### Milestone 10A required observations and lessons

- Fixed Git CLI commands preserved installed-Git semantics without a new
  dependency. Environment clearing, system/global-config exclusion, optional
  lock/fsmonitor/untracked-cache disabling, no external diff/text conversion,
  and timeout/kill/wait behavior keep repository inspection bounded.
- Status is scoped to the exact attachment. Every diff/editor action requires a
  fresh matching status path; worktree reads and editor handoff additionally
  require a regular non-symlink file contained by the attachment.
- Porcelain-v2 `-z` paths remain repository-root-relative when Git runs from an
  attached subdirectory. Native code therefore computes and strips the exact
  revalidated worktree-to-attachment prefix; a regression test overrides local
  relative-path and external-diff configuration while proving outside changes
  remain excluded.
- React receives no Git arguments, cwd, absolute paths, object IDs, raw headers,
  stderr, or configuration. Browser preview does not simulate Git data, and no
  review content is persisted.
- The first browser run found insufficient light-theme contrast over tinted
  hunk/addition/deletion rows. Using the primary text token corrected it; the
  desktop/mobile focused rerun and final combined gate passed axe and overflow.
- The publication repository gate passed in 31.72 seconds at about 804 MiB peak
  RSS: 70 JavaScript tests and 78 Rust tests passed, with 2 deliberate live
  probes ignored. This was about 14% faster than the Milestone 9B baseline and
  did not cross the forecast-update threshold.
- The combined browser gate passed 20 checks in 11.64 seconds at about 315 MiB
  peak RSS. The final release build passed in 31.18 seconds at about 1.61 GiB
  peak RSS, including a 27.16-second release compile/link.
  Visual inspection confirmed the two-column desktop and stacked mobile source
  review remain legible after the contrast correction.
- An isolated release launch owned the exact D-Bus name, opened schema version
  3 with `0700`/`0600` metadata permissions, started no Codex app-server, and
  removed its generated temporary tree after inspection.
- Two aggregate attempts stopped safely at ordinary lint/format/Clippy findings
  before the successful gate. The corrections changed only effect scheduling,
  rustfmt output, and a test literal; no cache or repository state was reset.
- Available RAM remained about 43 GiB, swap stayed near 175 MiB with zero
  measured swaps, and there was no OOM, throttling, dependency download, clean
  build, user-repository Git mutation, live model call, or GPU workload.
- Milestone 10B requires a fresh reasoning gate. Its preliminary range is
  approximately 4–7 active hours, 15–35 minutes of local commands, and 5–8.5
  total elapsed hours across one or two sessions, medium confidence. GPT-5.6
  Sol with XHigh reasoning is provisionally appropriate because stage, unstage,
  revert, commit, secret review, concurrency, postconditions, and failure
  recovery are mutation-sensitive.

### Milestone 10B — Reviewed Stage, Unstage, Revert, and Commit

| Field                       | Record                                                                                                                                                                                                                                                                                                                                                                    |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-20                                                                                                                                                                                                                                                                                                                                                                |
| Selected model              | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                                                                                                                                                                          |
| Preliminary forecast        | Approximately 4–7 active hours; 15–35 minutes of local commands; about 5–8.5 total elapsed hours across one or two sessions; medium confidence                                                                                                                                                                                                                            |
| Calibrated forecast         | Approximately 5–8 active hours; 20–45 minutes of local commands; about 6–10 total elapsed hours across one or two sessions; medium-low confidence                                                                                                                                                                                                                         |
| Calibration basis           | Clean synchronized Milestone 10A `main`, Git 2.53.0, about 43 GiB available RAM, low initial host load, 726 GiB free NVMe, warm 13 GiB Cargo target and pnpm/browser caches, no dependency change, four Cargo workers, two Playwright workers, and CPU/system-memory execution with no GPU workload                                                                       |
| Observed active execution   | Approximately 65–90 minutes through Git-plumbing and recovery design, native implementation, strict contracts, React confirmation UI, disposable-repository tests, security hardening, documentation, complete gates, release/native verification, publication, post-merge CI diagnosis, and the lock-ownership correction; pre-gate approval and hosted waiting excluded |
| Observed local command time | Approximately 7–11 minutes across incremental compile/test cycles, disposable Git probes, the 43.16-second final repository gate, 12.31-second combined browser gate, 34.56-second release build, a three-second isolated launch, visual captures, formatting, focused checks, and the 11.35-second post-merge native correction gate                                     |
| User approval/waiting time  | Manual model/reasoning and milestone-start confirmations occurred before implementation and are excluded; publication and hosted-runner waiting are tracked separately                                                                                                                                                                                                    |
| Sessions                    | One logical implementation session with native-design, contract/UI, deterministic-test, documentation, validation, and publication checkpoints                                                                                                                                                                                                                            |
| Usage intensity             | XHigh model/tool activity; moderate local CPU and low memory pressure                                                                                                                                                                                                                                                                                                     |
| Completion status           | Complete and published through PR #20; the post-merge lock-ownership correction is tracked by the follow-up milestone change                                                                                                                                                                                                                                              |

The forecast substantially overestimated implementation time. The Milestone
10A Git runner, status parser, attachment identity, project reservation,
strict-fixture pattern, React source-review surface, and warm caches were all
directly reusable. The estimate intentionally carried a large debugging and
recovery allowance because commit/index/reference races and no-data-loss
behavior were previously unmeasured. XHigh reasoning was appropriate for the
commit-plumbing choice, native-held confirmation model, secret review,
subdirectory scope, narrow rollback, recovery limitations, and final lock
ownership audit; a lower setting would have increased the risk of using
porcelain commit hooks or overstating recovery guarantees.

### Milestone 10B required observations and lessons

- `git commit --no-verify` is insufficient for this boundary because not every
  commit hook is suppressed. The implementation uses `write-tree`,
  `commit-tree`, and expected-old `update-ref`, with signing, hooks, prompts,
  global/system configuration, and inherited environment disabled.
- `write-tree` needs its own index lock. Commit therefore writes the candidate
  tree before acquiring QuireForge's exact index lock, then revalidates all
  evidence and verifies cached index equality with that tree under the lock.
  Lock cleanup records device/inode identity and retains the owned open file
  handle until cleanup, preventing inode reuse from making a replacement lock
  appear owned.
- Confirmation does not accept paths or messages. A five-minute native UUIDv7
  identifies one exact plan and is consumed before execution. New preview
  attempts invalidate older pending intent for the same project.
- Stage/unstage snapshot exact index entries. Revert is deliberately narrower:
  one tracked regular-file modification, at most one MiB, with a 30-minute
  one-use in-memory backup restored atomically only if indexed content remains
  untouched. Application exit loses recovery, so it is not a durable backup.
- Commit secret review is bounded to one MiB per staged blob and four MiB total.
  Sensitive filenames plus high-confidence private-key, GitHub-token, and
  OpenAI-key patterns in staged blobs or the message block the preview. This is
  defense in depth rather than proof that staged content contains no secret.
- Temporary-repository tests cover stale previews, exact stage/unstage,
  replacement index locks, project ownership, existing and unborn branch
  commits, hooks/signing, unstaged-work preservation, secret/message refusal,
  attachment scope, revert/recovery, and single-use behavior. No user
  repository was mutated by routine validation.
- The final complete non-browser gate passed in 43.16 seconds at about 1.33 GiB peak
  RSS: 73 JavaScript tests and 86 Rust tests passed; 2 deliberate live probes
  remained ignored. This is slower than 10A because the native crate and test
  binary were recompiled after the final security change, but remained far
  below the forecasted local-command allowance.
- The combined browser gate passed 20 desktop/mobile checks in 12.31 seconds at
  about 318 MiB peak RSS. Axe found no violations and neither viewport
  overflowed. Visual captures confirmed the modal target review and applied
  state in the two-column desktop and stacked mobile layouts.
- The warm unbundled release build passed in 34.56 seconds at about 1.77 GiB
  peak RSS, including a 30.45-second release compile. A three-second isolated
  launch created schema migrations 1–3 with final `0700`/`0600` metadata
  permissions and left no QuireForge or Codex app-server process.
- The first aggregate gate stopped after 0.73 seconds because an intentionally
  secret-shaped test fixture correctly triggered the repository scanner.
  Constructing the value from non-secret fragments preserved the test without
  weakening validation; no cache, configuration, or repository state was
  reset.
- PR #20 passed all three hosted jobs, including the 9-minute-41-second desktop
  gate. The identical merge commit's first `main` run then exposed immediate
  inode reuse in the replacement-lock test on a different ephemeral runner.
  Retaining the original open lock handle fixed the real cleanup race; the
  focused regression, all 88 native tests, rustfmt, and Clippy passed locally
  in 11.35 seconds before the corrective publication.
- About 42 GiB RAM and 725 GiB NVMe remained available after the gates. Swap
  stayed near 176 MiB, timed commands reported zero swaps, and there was no OOM,
  throttling, dependency download, clean build, user-repository mutation, live
  model call, or GPU-compute workload.
- Milestone 11 requires a fresh gate. A preliminary range is approximately
  8–14 active hours, 30–75 minutes of local commands, and 10–18 total elapsed
  hours across two or three recoverable sessions, low-to-medium confidence.
  GPT-5.6 Sol with XHigh reasoning is provisionally appropriate because managed
  worktree lifecycle, concurrent Codex ownership, cleanup separation, branch
  safety, and crash recovery are architecture- and data-loss-sensitive.

## Milestone 11 — Worktrees and Parallel Work

The gated implementation split Milestone 11 into recoverable boundaries:
11A managed inventory/create/attach, 11B parallel execution and aggregated
status/conflicts, and 11C explicit cleanup/recovery. This prevented the first
worktree mutation from also authorizing concurrent Codex ownership or
data-loss-sensitive removal.

### Milestone 11A — Managed Worktree Foundation

| Field                       | Record                                                                                                                                                                                                                                                                                                        |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-20                                                                                                                                                                                                                                                                                                    |
| Selected model              | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                                                                                                              |
| Preliminary forecast        | Approximately 3–5 active hours; 15–35 minutes of local commands; about 4–7 total elapsed hours in one session; medium confidence                                                                                                                                                                              |
| Calibrated forecast         | Approximately 3.5–5.5 active hours; 20–40 minutes of local commands; about 4.5–7.5 total elapsed hours in one or two sessions; low-to-medium confidence                                                                                                                                                       |
| Calibration basis           | Clean synchronized Milestone 10B `main`, Git 2.53.0, about 42 GiB available RAM, 725–726 GiB free NVMe, warm 13 GiB Cargo target and pnpm/browser caches, no dependency change, four Cargo workers, two Playwright workers, and CPU/system-memory execution with no GPU workload                              |
| Observed active execution   | Approximately 75–110 minutes through architecture and threat review, schema/service/IPC/UI implementation, disposable-repository adversarial tests, documentation, complete gates, release/native verification, visual inspection, and publication preparation; pre-gate approval and hosted waiting excluded |
| Observed local command time | Approximately 8–12 minutes across incremental compiles/tests, disposable Git probes, two stopped aggregate checks, the 45.99-second passing full gate, 14.59-second combined browser gate, 33.98-second release build, isolated launch, and rendered-state capture                                            |
| User approval/waiting time  | Manual model/reasoning and milestone-start confirmations occurred before implementation and are excluded; publication and hosted-runner waiting are tracked separately                                                                                                                                        |
| Sessions                    | One logical implementation session with architecture, native boundary, strict contract/UI, adversarial fixture, documentation, validation, release, and publication checkpoints                                                                                                                               |
| Usage intensity             | XHigh model/tool activity; moderate local CPU and low memory pressure                                                                                                                                                                                                                                         |
| Completion status           | Implemented, fully verified, and merged to `main` by pull request 22                                                                                                                                                                                                                                          |

The forecast substantially overestimated active and elapsed time. Existing
project identity/revalidation, project reservation, fixed Git runner patterns,
SQLite migrations, preview tokens, strict Zod contracts, and responsive card
styles were reusable. The original whole-milestone uncertainty also included
parallel process ownership and destructive cleanup, which the safer 11A/11B/11C
split removed from this checkpoint. XHigh reasoning remained appropriate for
the partial-failure no-cleanup decision, source-HEAD binding, group reservation,
repository-local hook/filter suppression, and final data-loss review.

### Milestone 11A required observations and lessons

- Each linked worktree is an ordinary QuireForge project. Schema migration 4
  stores only its canonical source project, worktree project, managed/attached
  ownership, and optional normalized branch.
- The frontend can supply only a project ID and bounded new branch name. Native
  code owns the destination, picker path, Git cwd/argv, source common directory,
  and base commit; confirmation accepts only a five-minute one-use UUIDv7.
- Source HEAD must be captured internally at preview and compared again at
  confirmation. Passing that reviewed object directly to fixed `worktree add`
  prevents a concurrent HEAD move from changing the branch base silently.
- `core.hooksPath=/dev/null` is not enough for checkout. Effective configured
  filter drivers are enumerated with bounded Git output and overridden for the
  create command; a disposable fixture proves neither a post-checkout hook nor
  smudge filter executes.
- A source project may attach a repository subdirectory. Inventory must match
  the revalidated Git worktree root, not the selected subdirectory, to identify
  the canonical source correctly.
- If Git creates the worktree and the SQLite transaction fails, automatic
  removal would turn metadata failure into possible data loss. The service
  reports the retained display path and performs no cleanup.
- The complete repository gate passed in 45.99 seconds at about 1.27 GiB peak
  RSS: 82 JavaScript tests and 95 Rust tests passed, with 2 live probes ignored.
  This was about 7% slower than 10B and below the 25% update threshold.
- The combined browser gate passed 22 checks in 14.59 seconds at about 337 MiB
  peak RSS. The release build passed in 33.98 seconds at about 1.73 GiB peak
  RSS, including a 29.93-second release compile/link.
- The isolated launch owned the exact D-Bus name, applied migrations 1–4 with
  `0700`/`0600` permissions, started with Codex unavailable on its restricted
  path, and left no application/Codex process or temporary launch tree.
- Approximately 42 GiB RAM and 725 GiB NVMe remained available. Swap stayed
  near 187 MiB, every timed gate reported zero swaps, and there was no OOM,
  throttling, dependency download, clean build, user-repository mutation, live
  model call, or GPU-compute workload.
- Milestone 11B requires a fresh reasoning/model/start gate. A preliminary
  range is approximately 4–7 active hours, 20–45 minutes of local commands, and
  5–9 total elapsed hours across one or two sessions, low-to-medium confidence.
  GPT-5.6 Sol with XHigh reasoning is provisionally appropriate because
  concurrent process ownership, cross-worktree status/conflicts, cancellation,
  and crash recovery remain architecture-sensitive.

### Milestone 11B — Bounded Parallel Worktree Execution

| Field                       | Record                                                                                                                                                                                                                                                                                                                                         |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-20                                                                                                                                                                                                                                                                                                                                     |
| Selected model              | GPT-5.6 Sol, Extra High reasoning; manually confirmed                                                                                                                                                                                                                                                                                          |
| Preliminary forecast        | Approximately 4–7 active hours; 20–45 minutes of local commands; about 5–9 total elapsed hours across one or two sessions; low-to-medium confidence                                                                                                                                                                                            |
| Calibrated forecast         | Approximately 5–8 active hours; 30–60 minutes of local commands; about 6–10 total elapsed hours across one or two sessions; low-to-medium confidence                                                                                                                                                                                           |
| Calibration basis           | Clean synchronized Milestone 11A `main`; Ryzen 5 5600G with 6 physical/12 logical processors; about 42 GiB available of 61 GiB RAM; 8 GiB swap with about 187 MiB used; 725 GiB free NVMe/ext4; warm 13 GiB Cargo target and pnpm/Playwright caches; no dependency change; four Cargo workers and two Playwright workers; no GPU workload      |
| Observed active execution   | Approximately 90–135 minutes through concurrency architecture, per-task native ownership, strict registry/bridge/UI implementation, deterministic multi-process tests, documentation/security review, complete gates, release/native verification, visual inspection, and publication preparation; approval and hosted-runner waiting excluded |
| Observed local command time | Approximately 10–16 minutes across incremental focused checks, the 42.19-second passing final full gate, 19.29-second final combined browser gate, 35.73-second final release build, isolated launch, and targeted rendered-state capture                                                                                                      |
| User approval/waiting time  | Manual model/reasoning and milestone-start confirmations occurred before implementation and are excluded; publication and hosted-runner waiting are tracked separately                                                                                                                                                                         |
| Sessions                    | One logical implementation session with a recoverable native-contract, frontend, documentation, validation, visual, and publication sequence                                                                                                                                                                                                   |
| Usage intensity             | Extra High model/tool activity; moderate local CPU and low memory pressure under the Balanced profile                                                                                                                                                                                                                                          |
| Completion status           | Implemented and fully verified locally on `feat/milestone-11b-parallel-execution`; publication prepared                                                                                                                                                                                                                                        |

The calibrated forecast overestimated active and elapsed time. The necessary
native ownership change was architecture-sensitive, but the existing project
reservation, app-owned identity, process supervisor, normalized conversation
snapshot, activity UI, worktree inventory, and read-only Git status boundaries
were reusable. No schema migration, dependency operation, raw-protocol change,
or release packaging was needed. Extra High reasoning remained appropriate for
the registry/per-task lock split, provisional-capacity accounting, exact
terminal cleanup, per-task stale-response protection, restart semantics, and
security review even though implementation completed within one session.

### Milestone 11B required observations and lessons

- A single global `Option<ActiveConversation>` held across awaited process I/O
  cannot safely become parallel by adding workers. Capacity/membership and
  exact process serialization need separate registry and per-conversation
  locks.
- The four-task limit includes provisional starts. Otherwise several slow
  app-server launches could pass a capacity check before any active entry is
  inserted. Same-project starts are separately excluded before process spawn.
- Terminal cleanup marks a slot finished, reaps its exact child, releases its
  exact project, and removes the entry only if the registered allocation still
  matches. This prevents duplicate in-flight calls from releasing newer work.
- The refresh contract needs only active normalized snapshots with empty event
  batches. React can resume polling without IPC exposure or persistence of
  Codex IDs, cwd, processes, transcript content, or prior activity output.
- Frontend stale-response protection must be per task. Approval or interruption
  for one worktree must not pause or invalidate another worktree's poll.
- The aggregate monitor composes current worktree inventory, normalized
  conversation state, and read-only Git changed/conflict counts. It performs no
  conflict resolution, Git mutation, worktree removal, pruning, or cleanup.
- The complete repository gate passed in 42.19 seconds at about 1.27 GiB peak
  RSS: 86 JavaScript tests and 101 Rust tests ran, with 99 Rust tests passing
  and 2 deliberate live probes ignored. This was faster than the 45.99-second
  Milestone 11A baseline.
- The final combined browser gate passed 24 checks in 19.29 seconds at about
  372 MiB peak RSS. This was 32% above the 14.59-second Milestone 11A browser
  baseline after the new fixture expanded live activity in both viewports, so
  19–20 seconds becomes the next warm browser baseline; scope and total
  milestone range did not change. The final warm unbundled native release build
  passed in 35.73 seconds at about 1.73 GiB peak RSS, including a 31.53-second
  release compile/link.
- An isolated release launch applied schema migrations 1–4 with `0700`/`0600`
  app metadata permissions, performed no conversation work, and left no
  QuireForge process. The temporary launch tree and visual captures were
  removed after inspection.
- Visual inspection confirmed the monitor's approval/conflict hierarchy and
  the click-through conversation view's expanded command detail and bounded
  live output at desktop size; both desktop and mobile browser fixtures passed
  axe-core and overflow checks.
- Timed gates reported zero swaps. There was no OOM, throttling, dependency
  download, clean build, cache reset, competing project build, user-repository
  mutation, live model call, or GPU-compute workload. The RTX 3050 was
  correctly unused.
- Milestone 11C requires a fresh reasoning/model/start gate. A preliminary
  range is approximately 4–7 active hours, 20–45 minutes of local commands, and
  5–9 total elapsed hours across one or two sessions, low confidence. A strong
  reasoning setting is provisionally appropriate because recovery ownership,
  dirty/conflicted worktree refusal, branch retention, symlink/path races,
  partial deletion failures, and explicit user confirmation are data-loss-
  sensitive.

### Milestone 11C — Safe Managed-Worktree Recovery and Cleanup

| Field                       | Record                                                                                                                                                                                                                                                                                                                       |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-20                                                                                                                                                                                                                                                                                                                   |
| Selected model              | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                                                                                                                             |
| Preliminary forecast        | Approximately 4–7 active hours; 20–45 minutes of local commands; about 5–9 total elapsed hours across one or two sessions; low confidence                                                                                                                                                                                    |
| Calibrated forecast         | Approximately 4–7 active hours; 25–50 minutes of local commands; about 5–9 total elapsed hours across one or two sessions; low-to-medium confidence                                                                                                                                                                          |
| Calibration basis           | Clean synchronized Milestone 11B `main`; Ryzen 5 5600G with 6 physical/12 logical processors; about 40 GiB available of 61 GiB RAM; 8 GiB swap; 725 GiB free NVMe/ext4; warm Cargo, pnpm, Vite, Astro, and Playwright caches; no dependency or migration change; four Cargo workers, two Playwright workers, no GPU workload |
| Observed active execution   | Approximately 80–120 minutes through cleanup architecture, native/SQLite/IPC/UI implementation, adversarial disposable-repository tests, documentation and security review, complete gates, release/native verification, visual inspection, and publication preparation; approval and hosted-runner waiting excluded         |
| Observed local command time | Approximately 9–15 minutes across focused iterations, the 33.38-second final passing complete gate, 17.12-second combined browser gate, 31.31-second release build, isolated launch, rendered-state capture, and corrective warm reruns                                                                                      |
| User approval/waiting time  | Manual model/reasoning and milestone-start confirmations occurred before implementation and are excluded; publication and hosted-runner waiting are tracked separately                                                                                                                                                       |
| Sessions                    | One logical implementation session with recoverable native-boundary, strict-contract/UI, documentation, validation, visual, and publication checkpoints                                                                                                                                                                      |
| Usage intensity             | XHigh model/tool activity; moderate local CPU and low memory pressure under the Balanced profile                                                                                                                                                                                                                             |
| Completion status           | Implemented and fully verified locally on `feat/milestone-11c-safe-worktree-cleanup`; publication prepared                                                                                                                                                                                                                   |

The calibrated forecast substantially overestimated active and elapsed time.
Existing managed-worktree identity, source-group reservations, fixed Git
runner, project archival, expiring confirmation, strict Zod contract, and
responsive review-card patterns were reusable. XHigh reasoning remained
appropriate because the implementation introduced a deliberately destructive
operation and required safe behavior across dirty-state changes, symlink/path
races, repository-controlled filters, active work, ownership confusion, and a
Git-success/metadata-failure split.

### Milestone 11C required observations and lessons

- Inventory issues an expiring opaque recovery ID only when an unregistered
  linked worktree occupies its exact app-managed private slot. Recovery consumes
  that ID, revalidates the candidate, and registers metadata without mutating
  Git or files.
- Removal accepts only an app-owned target project related as `managed` to the
  exact source repository. It refuses current, attached/external, dirty,
  conflicted, locked, prunable, busy, identity-changed, symlink-replaced, and
  branch/`HEAD`-changed targets.
- `git worktree remove` runs without `--force`; success requires the checkout
  and inventory entry to be absent while the branch still exists. Product code
  offers no generic prune, branch deletion, or direct recursive deletion.
- Git removal precedes transactional metadata retirement. If the transaction
  fails, a fresh metadata-only confirmation can archive and detach the now-
  missing managed project without repeating filesystem mutation.
- Removal itself invokes Git's clean-status conversion. Repository-configured
  clean, smudge, and process filters therefore need the same fixed identity
  overrides as explicit status/create operations. A disposable adversarial
  test proves neither those helpers nor hooks execute.
- The final complete repository gate passed in 33.38 seconds at about 891 MiB peak
  RSS; the browser gate passed 24 checks in 17.12 seconds at about 370 MiB; and
  the release build passed in 31.31 seconds at about 1.98 GiB. Every timed gate
  reported zero swaps.
- The isolated native launch created only private test metadata, started no
  task, and left no QuireForge process. Desktop and mobile inspection confirmed
  the destructive-review hierarchy, explicit branch preservation, separate
  recovery flow, and absence of force/prune/branch-delete actions.
- No dependency, schema migration, clean build, cache reset, user-repository
  mutation, live model call, or GPU computation was required. The RTX 3050 was
  correctly unused.
- Milestone 12 requires a fresh reasoning/model/start gate. A preliminary range
  is approximately 5–8 active hours, 30–60 minutes of local commands, and 6–10
  total elapsed hours across one or two sessions, low confidence. GPT-5.6 Sol
  with XHigh reasoning is provisionally appropriate because PTY ownership,
  verified cwd/environment setup, process groups, resize/input concurrency,
  output normalization, terminal escape handling, and cleanup are security- and
  lifecycle-sensitive.

## Milestone 12 — Integrated Terminal

| Field                             | Record                                                                                                                                                                                                                                                                                                                                                                      |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date                     | 2026-07-20                                                                                                                                                                                                                                                                                                                                                                  |
| Selected model                    | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                                                                                                                                                                            |
| Preliminary forecast              | Approximately 5–8 active hours; 30–60 minutes of local commands; 6–10 total elapsed hours across one or two sessions; low confidence                                                                                                                                                                                                                                        |
| Calibrated forecast               | Approximately 5–8 active hours; 35–80 minutes of local commands; 6–10 total elapsed hours across one or two sessions; low-to-medium confidence                                                                                                                                                                                                                              |
| Calibration basis                 | Clean synchronized Milestone 11C `main`; Ryzen 5 5600G with 6 physical/12 logical processors; about 41 GiB available of 61 GiB RAM; 8 GiB swap with no new pressure; 725 GiB free NVMe/ext4; warm Cargo, pnpm, Vite, Astro, and Playwright caches; new `portable-pty`, xterm, base64, and libc dependencies; four Cargo workers and two Playwright workers; no GPU workload |
| Expected active Codex time        | 5–8 hours                                                                                                                                                                                                                                                                                                                                                                   |
| Expected local build/test time    | 35–80 minutes, able to overlap independent documentation/review only when resource headroom remains                                                                                                                                                                                                                                                                         |
| Expected total real-world elapsed | 6–10 hours, excluding unbounded user approval delay                                                                                                                                                                                                                                                                                                                         |
| Expected user approval time       | Model/reasoning/start and publication approvals listed separately; not included in counted project time                                                                                                                                                                                                                                                                     |
| Sessions and usage                | One or two recoverable sessions; high model intensity, moderate CPU, low-to-moderate memory, negligible GPU                                                                                                                                                                                                                                                                 |
| Observed active execution         | Approximately 1.16 hours, reconstructed from the complete elapsed interval after subtracting evidenced local and hosted automated waits                                                                                                                                                                                                                                     |
| Observed automated wait           | Approximately 0.38 hour: at least 0.05 hour of timed local commands plus 0.16-hour pull-request and 0.16-hour `main` workflow critical paths                                                                                                                                                                                                                                |
| Observed counted / elapsed        | 1.53 counted project hours / 1.53 total elapsed hours; no post-start user-blocked interval                                                                                                                                                                                                                                                                                  |
| Forecast variance                 | Approximately 4.97 hours (76.4%) below the 6.5-hour active-forecast midpoint when compared with counted project time                                                                                                                                                                                                                                                        |
| Completion status                 | Complete; merged by [PR #27](https://github.com/codeframe78/quireforge/pull/27) as `07ba046`, with successful PR and `main` repository checks                                                                                                                                                                                                                               |

| Component                                               |        Duration range | Confidence | Primary uncertainty                                             | Resource class            | Safe overlap                              |
| ------------------------------------------------------- | --------------------: | ---------- | --------------------------------------------------------------- | ------------------------- | ----------------------------------------- |
| Repository/system inspection                            |             15–25 min | High       | None material after preflight                                   | Model/storage-bound       | Batched read-only checks                  |
| Architecture/security planning                          |             35–60 min | Medium     | Linux PTY/session ownership edge cases                          | Model-bound               | Documentation only                        |
| Native PTY, metadata, and cleanup implementation        |                 2–3 h | Low–Medium | Process-session cleanup and failure recovery                    | Model/CPU-bound           | Targeted frontend work when not compiling |
| Typed IPC and xterm UI                                  |               1–1.5 h | Medium     | Renderer lifecycle, input/poll ordering, responsive tabs        | Model-bound               | Native targeted tests                     |
| Dependency operations and compilation                   |             15–30 min | Medium     | First dependency compile/link                                   | Network/CPU/storage-bound | Read-only review after downloads          |
| Automated native/frontend/browser tests                 |             20–40 min | Medium     | PTY timing and UI lifecycle failures                            | CPU/storage-bound         | Independent documentation at low load     |
| Native/visual verification                              |             10–20 min | Medium     | GUI/PTY behavior under isolated XDG state                       | Approval/model-bound      | None for launch                           |
| Debugging allowance                                     |             45–90 min | Low        | Newly discovered concurrency or cleanup defect                  | Model/CPU-bound           | Case dependent                            |
| Documentation, security/diff review, commit preparation |             45–75 min | Medium     | Cross-document consistency and historical ledger reconstruction | Model-bound               | Warm builds/tests                         |
| GitHub publication and main CI                          | 10–25 min runner time | Medium     | Hosted queue/runner behavior                                    | Network/approval-bound    | Local completion review                   |

Critical path: native ownership and storage contract → strict IPC → xterm
lifecycle → full repository/browser/release/native verification → security and
diff review → publication and successful `main` CI. The Balanced profile keeps
four Cargo workers and two Playwright workers, preserves all caches, and runs
independent frontend/native focused checks concurrently only with measured
headroom. Release, browser preview, and isolated native launch remain
sequential. The RTX 3050 is intentionally unused because the DOM terminal and
ordinary Rust/TypeScript builds are CPU/system-memory workloads.

### Milestone 12 required observations and lessons

- Locked dependency installation completed in 6.38 seconds at about 1.04 GiB
  peak RSS. Warm frontend type checking took 1.66 seconds and `cargo check`
  about 0.30 second before implementation.
- Focused terminal tests exercise a real local PTY and process session while
  using temporary attached directories. The warm suite passes nine tests;
  incremental compilation peaked around 1.31–1.47 GiB RSS.
- The complete frontend type/lint/unit pass completed in about 17.7 seconds and
  passed 97 tests across 18 files. No model call, user-repository mutation,
  CUDA, clean build, or cache reset occurred.
- A failed focused compile exposed missing test-only imports and cost about 5.8
  seconds; it did not reveal a product defect. Review then added pre-decode
  native input bounds, canonical UUIDv7 validation, stale-record-aware registry
  capacity accounting, and fail-closed over-capacity metadata loading.
- The final complete repository gate passed in 39.01 seconds at about 929 MiB,
  including 100 JavaScript tests and 119 Rust tests (117 passed, two deliberate
  live probes ignored). The final browser gate passed 26 desktop/mobile checks
  in 19.24 seconds at about 393 MiB.
- The release build took 88.76 seconds at about 1.82 GiB, 184% longer than the
  Milestone 11C warm baseline because the new native PTY dependency graph needed
  its first optimized compile. This crossed the per-operation update threshold
  but remained comfortably inside the aggregate command and elapsed forecasts,
  so the approved range did not change.
- An isolated release launch created only private metadata at mode `0600`,
  emitted no GVFS warning, timed out as expected, and left no process.
  Controlled desktop/mobile fixture inspection confirmed the terminal selector,
  privilege boundary, running tab, close control, cursor, responsive surface,
  and absence of horizontal overflow.
- All timed final gates reported zero swaps. No OOM, throttling, clean build,
  cache reset, user-repository mutation, live model call, CUDA, or GPU rendering
  occurred. Temporary launch and visual-capture data were removed.
- Pull-request workflow `29796398390` passed with source validation in 6
  seconds, the website gate in 1 minute 56 seconds, and the desktop gate in 9
  minutes 46 seconds. After merge, `main` workflow `29796847052` passed with
  source validation in 5 seconds, the website gate in 1 minute 50 seconds, and
  the desktop gate in 9 minutes 33 seconds.
- GitHub-hosted jobs receive fresh runner environments. The workflow already
  restores pnpm dependencies, but it does not yet restore Cargo build outputs
  or Playwright browser downloads. A credential-free, lockfile/toolchain-keyed
  cache is a separately scoped optimization candidate rather than a Milestone
  12 product change.
