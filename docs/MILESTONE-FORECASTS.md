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

## Milestone 13A — Protocol refresh and integration contract architecture

| Field                       | Record                                                                                                                                                                                                            |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-21                                                                                                                                                                                                        |
| Selected model              | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                  |
| Preliminary forecast        | Approximately 3.5–5.5 active hours; 25–60 minutes of local commands; 4.5–7 total elapsed hours; medium confidence                                                                                                 |
| Calibrated forecast         | Approximately 3.5–6 active hours; 20–50 minutes of local commands; 4.5–7.5 total elapsed hours across one or two sessions; medium confidence                                                                      |
| Calibration basis           | Clean `main` at `3ce57b9`; Codex CLI 0.145.0 versus 0.144.6 fixtures; 43 GiB available RAM and 720 GiB free NVMe; warm Cargo/pnpm caches; four Cargo workers; no GPU work                                         |
| Observed active execution   | Approximately 0.43 hour across inspection, architecture, implementation, security hardening, documentation, review, and publication operations                                                                    |
| Observed local command time | About 5.2 measured minutes through security hardening and the final complete/release reruns                                                                                                                       |
| Observed automated wait     | Approximately 0.14 hour: about 0.087 hour of measured local commands plus 104-second pull-request and 83-second `main` workflow critical paths                                                                    |
| Observed counted / elapsed  | 0.57 counted project hour / 0.68 total elapsed hour; a 0.11-hour environment-permission block is included only in elapsed time                                                                                    |
| Forecast variance           | Approximately 4.18 hours (88.0%) below the 4.75-hour calibrated active-forecast midpoint when compared with counted project time                                                                                  |
| Resource observations       | Final complete gate 37.77 seconds/about 935 MiB RSS; final release build 38.41 seconds/about 1.81 GiB RSS; zero swaps; persistent UpCloud CI desktop jobs completed in 1 minute 20 seconds to 1 minute 41 seconds |
| Completion status           | Complete; merged by [PR #32](https://github.com/James-Jennison/quireforge/pull/32) as `7bc5f5f`, with successful PR and `main` repository checks                                                                  |

Critical path: installed schema inventory → route/stability classification →
category-preserving normalized contract → dynamic-tool lifecycle decision →
strict Rust/TypeScript fixture tests → complete repository/release gates →
security/diff review → publication and successful `main` CI. The local work
found the required dynamic-tool lifecycle in the installed documented schemas,
so the recommendation-only fallback is not currently required for Milestone 18. Runtime registration remains dependency-gated and unimplemented.

The calibrated forecast was conservative because this checkpoint intentionally
stopped at contract architecture, reused established strict fixture patterns,
added no dependency or user-facing UI, and needed no protocol debugging. The
persistent UpCloud runner also completed the desktop gate in 1 minute 20
seconds to 1 minute 41 seconds rather than repeating the Milestone 12
GitHub-hosted cold-build path. Future forecasts should use the persistent-runner
baseline while still allowing for cache eviction and runner availability.

## Milestone 13B — Live read-only integration discovery

| Field                      | Record                                                                                                                                                                                                          |
| -------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date              | 2026-07-21                                                                                                                                                                                                      |
| Selected model             | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                |
| Preliminary forecast       | Approximately 2.5–4.5 active hours; 15–35 minutes of local commands; 3–5.5 total elapsed hours; medium confidence                                                                                               |
| Calibrated forecast        | Approximately 2.5–4.25 active hours; 12–30 minutes of local commands; 3–5 total elapsed hours in one or possibly two sessions; medium confidence                                                                |
| Calibration basis          | Synchronized `main` at `2833436`; CLI 0.145.0 schema/routes; approximately 43 GiB available RAM and 720 GiB free NVMe; warm Cargo/pnpm/Playwright caches; four Cargo and two Playwright workers; no GPU work    |
| Observed active execution  | Approximately 0.61 hour across inspection, implementation, route-policy/security correction, tests, documentation, review, and publication operations                                                           |
| Observed automated wait    | Approximately 0.18 hour: about 5–6 minutes of measured local command waits plus 159-second pull-request and 133-second post-merge `main` workflow critical paths                                                |
| Observed counted / elapsed | 0.79 counted project hour / 0.79 total elapsed hour; no user-blocked interval after execution started                                                                                                           |
| Forecast variance          | Approximately 2.59 hours (76.6%) below the 3.375-hour calibrated active-forecast midpoint when compared with counted project time                                                                               |
| Local verification         | Final `pnpm validate` 37.82 seconds/about 940 MiB RSS; Playwright 24.23 seconds/about 386 MiB; warm release build 40.35 seconds/about 1.85 GiB; all zero swaps                                                  |
| Hosted verification        | PR workflow `29890814046` passed source/website/desktop in 6 seconds/49 seconds/2 minutes 39 seconds; post-merge `main` workflow `29890942589` passed them in 7 seconds/1 minute 9 seconds/2 minutes 13 seconds |
| Completion status          | Complete; merged by [PR #34](https://github.com/James-Jennison/quireforge/pull/34) as `007f5b7`, with successful pull-request and `main` repository checks                                                      |

Critical path: exact route/schema inspection → native category normalization →
strict IPC → deterministic invalidation/partial-failure tests → route-policy
review and stable plugin CLI correction → complete repository/browser/release
gates → security/diff review → publication and successful `main` CI. Independent
documentation review overlapped only short warm checks; heavy release and
browser gates remained sequential under the Balanced profile.

The implementation deliberately uses reviewed app-server reads for connectors,
skills, MCP, and policy and fixed stable CLI JSON for plugins and marketplaces.
The late route-policy review removed an unnecessary `app/read` call and the
under-development plugin RPC without materially changing the calibrated
forecast. No personal catalog, integration mutation, authorization, package,
deployment, or model call was used.

The forecast was deliberately conservative for protocol drift and failure-path
debugging. The existing native process adapter and strict shared-fixture
patterns made the new boundary narrower than a greenfield integration layer,
while warm local artifacts and the persistent UpCloud runner kept every
acceptance path short. Later read-only adapter forecasts should use these warm
baselines with an explicit cache-eviction allowance; Milestone 14's mutation,
authorization, supply-chain, and UI scope requires a fresh gate rather than
reusing this result directly.

## Milestone 14A — Safe plugin and marketplace lifecycle

| Field                     | Record                                                                                                                                                                                                                                                                                                                                                                |
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date             | 2026-07-21                                                                                                                                                                                                                                                                                                                                                            |
| Selected model            | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                                                                                                                                                                      |
| Preliminary forecast      | Approximately 3–5.5 active hours; 3.5–6.5 total elapsed hours across one or two sessions; medium-to-low confidence                                                                                                                                                                                                                                                    |
| Calibrated forecast       | Approximately 2.75–5 active hours; 15–35 minutes of local commands; 3.25–6 total elapsed hours across one or two sessions; medium confidence                                                                                                                                                                                                                          |
| Calibration basis         | Synchronized `main` at `861f0dc`; CLI 0.145.0 stable plugin/marketplace JSON routes; approximately 43 GiB available RAM and 720 GiB free NVMe; warm Cargo/pnpm/Playwright caches; four Cargo and two Playwright workers; no GPU work                                                                                                                                  |
| Current measured commands | TypeScript preflight 1.71 seconds/about 266 MiB; Cargo preflight 1.39 seconds/about 486 MiB; final warm focused mutation suite 0.24 second/about 108 MiB; isolated real-CLI lifecycle 0.29 second/about 108 MiB; complete gate 37.23 seconds/about 942 MiB; browser regression 19.24 seconds/about 390 MiB; warm release 41.50 seconds/about 1.88 GiB; all zero swaps |
| Current status            | Complete and merged through PR #36 as `a20919f`; local gates and pull-request/main repository checks passed                                                                                                                                                                                                                                                           |

Critical path: stable route/source review → closed preview/confirm contract →
native fixed-command coordinator → shared strict IPC → deterministic and
isolated real-CLI lifecycle tests → complete repository/browser/release gates →
security/diff review → publication and successful `main` CI. The Balanced
profile keeps four Cargo workers and two Playwright workers. Browser, release,
and native suites remain sequential; documentation may overlap only short warm
commands. The RTX 3050 is intentionally unused.

Execution ran from `2026-07-21T21:43:18-07:00` through the first successful
post-merge `main` workflow at `2026-07-21T22:26:02-07:00`: 0.71 hour total
elapsed and counted project time, with approximately 0.58 hour active work,
0.14 hour measured/defensible automated wait, and 0.00 hour user-blocked after
start. The counted result was approximately 3.16 hours, or 81.7%, below the
3.875-hour calibrated active midpoint. Category totals are rounded separately,
so their displayed hundredths need not add exactly to the rounded wall-clock
total.

Existing catalog normalization, fixed-command mutation patterns, and warm
build caches removed much of the anticipated greenfield work. Official Codex
manual review identified the default `hooks/hooks.json` trust boundary before
publication, so the implementation added local-source hook presence and
identity revalidation without architectural rework. Security review also added
an explicit mutable-remote-source warning for configured marketplace upgrades.
Pull-request workflow
[`29893588842`](https://github.com/James-Jennison/quireforge/actions/runs/29893588842)
and post-merge `main` workflow
[`29893692681`](https://github.com/James-Jennison/quireforge/actions/runs/29893692681)
passed. Future narrowly bounded native checkpoints should use the measured
warm repository and roughly 1.5–1.75-minute hosted critical paths while keeping
allowance for interface drift, source-security discoveries, and cache misses.

## Milestone 14B — Integration Center UI

| Field                | Record                                                                                                                                                                                                    |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date        | 2026-07-22                                                                                                                                                                                                |
| Selected model       | GPT-5.6 Sol, High reasoning; manually confirmed                                                                                                                                                           |
| Preliminary forecast | Approximately 2.25–4.5 active hours; 8–20 minutes of local commands; 2.5–5 total elapsed hours in one session; medium confidence                                                                          |
| Calibrated forecast  | Approximately 2–3.75 active hours; 5–15 minutes of local commands; 3–8 minutes hosted CI critical path; 2.25–4.25 total elapsed hours in one session; medium confidence                                   |
| Calibration basis    | Clean synchronized `main` at `425e87b`; 43 GiB available RAM; 720 GiB free NVMe; low load; no active project build; warm Cargo/pnpm/Playwright caches; four Cargo and two Playwright workers; no GPU work |
| Preflight commands   | Desktop TypeScript check 1.60 seconds/about 262 MiB; four-worker Cargo check 1.35 seconds/about 501 MiB; both passed with zero swaps                                                                      |
| Current status       | Complete and merged through PR #38 as `93e585f`; local gates and pull-request/main repository checks passed                                                                                               |

Critical path: normalized contract-to-view mapping → isolated responsive
Integration Center → application catalog/mutation state wiring → search,
filter, details, and explicit confirmation interactions → deterministic
component/browser tests → visual/accessibility review → complete gates →
security/diff review → publication and successful `main` CI. The Balanced
profile keeps four Cargo workers and two Playwright workers. Browser, release,
and native suites remain sequential because earlier measurements found a
generated-output race when production and preview work overlapped. The RTX
3050 is intentionally unused.

The calibrated range is lower than the preliminary range because the existing
native and TypeScript contracts are complete, no dependency change is expected,
and both warm preflights completed in under three seconds combined. The upper
allowance remains for accessible focus handling, responsive visual correction,
large application-shell wiring, partial catalog states, and stale confirmation
presentation. Connector/MCP authorization, enable/disable, skill configuration,
prompt mentions, new native routes, personal integration mutation, packaging,
release, and deployment remain excluded.

Measured locally: the focused component suite passed 5 tests in 1.54 seconds;
the complete desktop unit suite passed 112 tests in 9.13 seconds; the final
non-browser repository gate passed in 49.85 seconds at about 1.32 GiB peak RSS;
the final website/desktop Playwright gate passed 28 desktop/mobile tests in
23.32 seconds at about 449 MiB peak RSS; and the warm unbundled native release
build passed in 42.54 seconds at about 1.88 GiB peak RSS. Every resource-timed
final command reported zero swaps. Desktop and mobile visual review confirmed
the intended two-column and single-column layouts. The early JSON import and
semantic-dialog lint corrections stayed within the debugging allowance and do
not require a forecast revision.

Milestone 14B completed at `2026-07-22T05:12:14-07:00`, the successful
post-merge `main` workflow endpoint. It used approximately 0.34 active hours,
0.16 automated-wait hours, 0.00 user-blocked hours, and 0.50 counted/elapsed
hours. That is approximately 2.38 hours or 82.6% below the 2.875-hour midpoint
of the calibrated active forecast. Reuse of the finished 13B/14A contracts,
warm caches, isolated component boundaries, and fast review/merge account for
the difference; the initial JSON import and dialog-lint corrections did not
materially extend the critical path. Pull-request workflow
[`29918268480`](https://github.com/James-Jennison/quireforge/actions/runs/29918268480)
and post-merge `main` workflow
[`29918513538`](https://github.com/James-Jennison/quireforge/actions/runs/29918513538)
passed. Future established-contract presentation checkpoints should start from
the measured sub-hour path while authentication/configuration work retains a
larger security and interface-uncertainty allowance.

## Milestone 14C — Confirmed integration authorization and controls

| Field                | Record                                                                                                                                                                                                          |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date        | 2026-07-22                                                                                                                                                                                                      |
| Selected model       | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                |
| Preliminary forecast | Approximately 5–9 active hours; 20–50 minutes of local commands; 5.5–10.5 total elapsed hours across one or two sessions; medium-to-low confidence                                                              |
| Calibrated forecast  | Approximately 4.5–8 active hours; 15–40 minutes of local commands; 4–10 minutes hosted CI critical path; 5–9.5 total elapsed hours across one or two sessions; medium confidence                                |
| Calibration basis    | Synchronized `main` at `91e256a`; reviewed Codex CLI 0.145.0 routes and fresh official manual; 43 GiB available RAM and 718 GiB free storage; low load; warm Cargo/pnpm caches; four Cargo workers; no GPU work |
| Preflight commands   | Desktop TypeScript check 1.68 seconds/about 263 MiB; four-worker Cargo check 1.42 seconds/about 502 MiB; both passed with zero swaps                                                                            |
| Current status       | Complete and merged through PR #41 as `e4d8333`; local gates and pull-request/main repository checks passed                                                                                                     |
| Observed local work  | Approximately 0.68 active hour and 0.18 automated hour from branch creation through the verified local checkpoint; 0.86 hour total elapsed and no user-blocked interval                                         |
| Acceptance timings   | Full validation 40.98 seconds/about 997 MiB; combined browser suites 23.62 seconds/about 447 MiB; warm unbundled desktop release build 36.38 seconds/about 2.23 GiB; zero test failures                         |

Critical path: fresh official route review → closed authorization/control
contract → native evidence/confirmation coordinator → native-only browser
handoff and exact completion correlation → connector mention construction →
Integration Center/composer wiring → deterministic security and UI tests →
complete repository/browser/release gates → security/diff review → publication
and successful `main` CI. Heavy browser and release gates remain sequential;
documentation may overlap only short warm commands. The RTX 3050 is
intentionally unused.

The calibrated range is below the preliminary range because 13B/14A/14B already
provide the normalized catalog, process adapter, confirmation patterns, and
Integration Center. The upper allowance remains intentionally large for
authorization URL ownership, app-server notification drift, scope-sensitive
skill writes, stale evidence, browser-handoff lifecycle, prompt mention
construction, responsive/accessibility review, and fail-closed tests. Routine
verification uses deterministic fixtures and no personal integration state,
real third-party authorization, billable model call, package, release,
deployment, or hosting mutation. The local checkpoint completed in 0.68 active
hour, approximately 5.57 hours or 89.1% below the 6.25-hour calibrated active
midpoint. Existing closed contracts and warm caches compressed implementation;
strict lint and Clippy still caught one React state-derivation issue and two
Rust representation issues before acceptance.

Milestone 14C completed at `2026-07-22T12:30:42-07:00`, the successful
post-merge `main` workflow endpoint. It used approximately 0.70 active hour,
0.24 automated-wait hour, 0.87 user-blocked hour, 0.94 counted hour, and 1.81
total elapsed hours. Active execution was approximately 5.55 hours or 88.8%
below the 6.25-hour calibrated midpoint; total elapsed was approximately 5.44
hours or 75.0% below the 7.25-hour calibrated midpoint. The user-blocked
interval between local completion and publication approval is excluded from
counted project time. Pull-request workflow
[`29950963936`](https://github.com/James-Jennison/quireforge/actions/runs/29950963936)
and post-merge `main` workflow
[`29951143628`](https://github.com/James-Jennison/quireforge/actions/runs/29951143628)
passed. Existing closed contracts and warm caches compressed the implementation
path, while strict lint and Clippy found the remaining state-derivation and Rust
representation issues before publication.

## Milestone 15 — File previews and desktop integration

| Field                | Record                                                                                                                                                                                                             |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Forecast date        | 2026-07-22                                                                                                                                                                                                         |
| Selected model       | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                   |
| Preliminary forecast | Approximately 4–7 active hours; 20–45 minutes of local commands; 4.5–8 total elapsed hours; medium confidence                                                                                                      |
| Calibrated forecast  | Approximately 4.5–8 active hours; 25–55 minutes of local commands; 4–10 minutes hosted CI critical path; 5–9.5 total elapsed hours across 15A–15C; medium confidence                                               |
| Calibration basis    | Synchronized clean `main` at `f99c078`; 44 GiB available RAM and 718 GiB free storage; low load; 12 logical CPUs; warm Cargo/pnpm caches; no GPU work; split into three independently reviewable checkpoints       |
| Preflight commands   | Desktop TypeScript check 1.67 seconds/about 266 MiB; four-worker Cargo check 0.25 second/about 109 MiB; both passed with zero swaps                                                                                |
| Current status       | Milestones 15A–15C implemented and verified locally; native Wayland and separately recorded XWayland/true-X11 display-session gates are complete                                                                   |
| Observed local work  | 15A: approximately 0.60 active/0.13 automated hour. 15B: approximately 0.60 active/0.08 automated hour. 15C: approximately 3.01 active/0.18 automated hours. None had a post-start user-blocked interval           |
| Acceptance timings   | Latest 15C validation 62.29 seconds/about 1.34 GiB; 143 frontend tests; 167 Rust tests/164 passed/3 ignored; browser suites 27.18/7.02 seconds; corrected guest release build 37.38 seconds; zero product failures |

Critical path: safe preview threat/format decision → native picker and
attachment-contained read boundary → strict shared contract → responsive
text/image/PDF presentation → temporary-file and browser security tests →
complete repository/browser/release gates → review/publication → 15B attachment
design and verification → 15C desktop handoffs and Wayland/X11 verification.
Browser and release gates remain sequential; the RTX 3050 is intentionally
unused.

The calibrated range is slightly above the preliminary range because the
single Milestone 15 description concealed three distinct trust boundaries.
Milestone 15A is limited to native-selected, bounded, transient previews;
15B owns drag/drop and conversation-attachment lifecycle; 15C owns
notifications, expanded editor/open-with handoff, and Linux display-session
verification. No dependency addition, personal-file inspection, Codex state
access, model call, package, release, deployment, or hosting mutation is
required for 15A routine validation.

The 15A local checkpoint completed approximately 5.65 hours or 90.4% below the
6.25-hour calibrated active midpoint for the complete three-part milestone.
That comparison was directional because 15B and 15C were unstarted at the 15A
checkpoint. Focused review still caught a project-ID fixture mismatch, exact
normalized-byte and line-limit edge cases, a growing-file allocation risk, and
a UTF-8 sniff-boundary case before acceptance. The repository's parallel root
browser command attached the website suite to an unrelated preview already
occupying port 4321; no external process was stopped. Desktop passed 22/22, and
the unchanged website suite passed 8/8 when rerun sequentially on an isolated
temporary port.

### Milestone 15B checkpoint

| Field               | Record                                                                                                                                                                                                                                                                                                        |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date       | 2026-07-22                                                                                                                                                                                                                                                                                                    |
| Selected model      | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                                                                                                              |
| Calibrated forecast | Approximately 2.5–4.5 active hours; 20–40 minutes of local commands; 3–5.5 total elapsed hours in one session; medium confidence                                                                                                                                                                              |
| Calibration basis   | Finished 15A branch at `4c2600a`; reviewed Codex 0.145.0 turn schema; warm Cargo/pnpm/browser caches; no dependency or GPU work; desktop TypeScript preflight 1.63 seconds/about 260 MiB and four-worker Cargo preflight 0.26 second/about 109 MiB                                                            |
| Current status      | Implemented and verified locally on `feat/milestone-15b-conversation-attachments`; local commit prepared without push, merge, package, release, deployment, personal-file inspection, personal Codex-state access, or live model call                                                                         |
| Observed local work | Approximately 0.60 active hour and 0.08 automated-wait hour from branch creation through final local verification; approximately 0.69 hour elapsed with no post-start user-blocked interval                                                                                                                   |
| Acceptance timings  | Full validation 68.50 seconds/about 1.51 GiB; 135 desktop and 3 website frontend tests; final Rust rerun 161 tests with 158 passed/3 deliberate live probes ignored; desktop Playwright 24/24 in 26.81 seconds/about 436 MiB; website 8/8 in 7.10 seconds/about 253 MiB; release 39.63 seconds/about 2.28 GiB |

The official app-server guide and pinned schema established `localImage` but
not a generic local-file input or an early file-consumption point. The final
design therefore remained image-only, disabled Tauri path-bearing drop events,
staged private copies, and retained claimed copies until terminal turn state.
That documentation check tightened cleanup semantics without adding a
dependency or expanding IPC.

The checkpoint completed approximately 2.90 hours or 82.9% below the
3.5-hour calibrated active midpoint. Reuse of the finished conversation,
picker, preview-image validator, strict contract, and responsive shell
compressed implementation. Focused review still caught a stale production
browser artifact, a React state-derivation lint issue, a Rust argument-grouping
Clippy issue, conservative image-lifetime semantics, and source-file identity
revalidation before the final green gates.

### Milestone 15C checkpoint

| Field               | Record                                                                                                                                                                                                                                                                                                                            |
| ------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date       | 2026-07-22                                                                                                                                                                                                                                                                                                                        |
| Selected model      | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                                                                                                                                                  |
| Calibrated forecast | Approximately 2–4 active hours; 15–35 minutes of local commands; 2.5–5 total elapsed hours in one session, excluding any logout/login needed for a true X11 session; medium confidence                                                                                                                                            |
| Calibration basis   | Finished 15B branch at `390434b`; closed 15A preview and conversation-state contracts; warm Cargo/pnpm/browser caches; GNOME Wayland host with XWayland but no true X11 login; TypeScript preflight 1.61 seconds/about 271 MiB and Cargo preflight 1.72 seconds/about 588 MiB, both with zero swaps                               |
| Current status      | Implemented and verified locally on `feat/milestone-15c-desktop-handoffs`; native Wayland project/file/image picker, preview, real-drop, and notification evidence plus complete XWayland/true-X11 handoff evidence close the display-session gate                                                                                |
| Observed local work | Approximately 3.01 active and 0.18 automated-wait hours from the `2026-07-22T14:30:59-07:00` branch start through `2026-07-22T17:42:22-07:00`; approximately 3.19 counted/elapsed hours with no post-start user-blocked interval, package, release, deployment, hosting mutation, personal Codex-state access, or live model call |
| Acceptance timings  | Latest full validation 62.29 seconds/about 1.34 GiB; 140 desktop and 3 website frontend tests; default Rust 167 tests/164 passed/3 ignored; browser 24/24 in 27.18 seconds and 8/8 in 7.02 seconds; corrected/probe/restored/final guest compiles 37.38/40.15/37.94/37.81 seconds; zero swaps                                     |

The design narrowed “open with” to the system default application for exactly
one already-previewed file. The frontend receives only a five-minute one-use
UUIDv7 action, while native code revalidates the current attachment, canonical
path, descriptor path, regular-file state, and device/inode identity. Fixed
notification copy is selected only from freshly resolved native approval or
terminal state and is suppressed when the main window is focused.

Native verification also caught a launch-procedure error before evidence was
accepted: a raw `cargo build --release` artifact retained Tauri's development
URL and attempted `127.0.0.1:1420`. Rebuilding with the repository's configured
`pnpm desktop:build` command embedded `dist`; that exact artifact rendered the
workspace and completed the XWayland handoff path.

A disabled-by-default native probe subsequently closed the notification-
delivery evidence gap without a live Codex turn or webview command. Its feature
build passed in 38.22 seconds at about 2.36 GiB, then delivered the fixed
completed-task copy through the GNOME Wayland notification service. The normal
artifact was restored in 36.88 seconds at about 2.36 GiB and excluded the probe
flag/string.

The true-X11 gate then ran in an Ubuntu 24.04 GNOME 46 `ubuntu-xorg` QA guest
against the repository mounted in place. `loginctl`, the active Xorg process,
and the absence of `WAYLAND_DISPLAY` distinguished it from XWayland. The
production artifact completed project/file/image pickers, bounded README
preview, confirmed default-app launch, and a Nautilus image drop. That drop
exposed an empty WebKitGTK HTML `FileList`; a Linux-native, 30-second one-use
GTK capture corrected the real compatibility failure without exposing paths to
React. The corrected production build staged normalized `drag drop` metadata,
and the fixed-copy notification probe delivered through the X11 session bus.
The normal artifact was restored afterward. Only interactive Wayland picker/
attachment evidence remained. The final native Wayland pass used the configured
production artifact and disposable XDG data to complete project/file/image
pickers, bounded README preview, and a real Nautilus drop that staged normalized
`drag drop` metadata. An approved test-only XDG Remote Desktop portal session
supplied native compositor input and was closed immediately afterward; it did
not widen QuireForge's product permissions. This completed Milestone 15 locally.

## Milestone 17A — Read-only scheduled task catalog

| Field                     | Record                                                                                                                                                     |
| ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date             | 2026-07-22                                                                                                                                                 |
| Selected model            | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                           |
| Start authorization       | Approved at `2026-07-22T21:01:38-07:00`                                                                                                                    |
| Preliminary forecast      | 2.5–4.5 active hours; 15–35 minutes of local commands; about 3–5.5 total elapsed hours; medium confidence                                                  |
| Calibration basis         | Clean commit `e5e8dcf`, warm Rust/Node caches, installed Codex CLI `0.145.0`, reviewed stable schemas, and the existing normalized integration catalog     |
| Supported-interface scope | Read installed, enabled plugin task templates through stable `plugin/read`; no create, edit, enable, run, pause, delete, hosted scheduler, or local runner |
| Usage intensity           | High model activity; moderate CPU and low memory pressure expected; no GPU                                                                                 |
| Observed active execution | Approximately 30 minutes from authorized start through implementation, documentation, validation, visual review, and commit preparation                    |
| Observed command time     | Approximately 6–8 minutes across focused checks, corrected full gates, release build, browser suite, and visual capture                                    |
| Completion status         | Complete locally on `feat/milestone-17a-scheduled-task-catalog`; not pushed, merged, packaged, deployed, or released                                       |

The calibrated scope is narrower than the roadmap label. Stable
`PluginReadResponse` exposes optional task templates and typed schedules, but
the reviewed stable request set contains no scheduled-task mutation or
execution method. The supported plugin CLI likewise exposes catalog management
only. Current official documentation assigns scheduled-task management to
official ChatGPT web and desktop surfaces and requires the official desktop
application for local execution.

Milestone 17A therefore extends the existing integration catalog with bounded,
sanitized, read-only task metadata. Raw marketplace paths stay native-only and
plugin prompts are rendered as inert untrusted text. Any scheduler, management
control, hosted-task claim, or execution path remains deferred behind a future
supported-interface review and explicit approval.

The forecast overestimated implementation time. The existing normalized
integration service, already-retained `PluginRead` schemas, deterministic shell
fixture, warm caches, and narrow read-only boundary compressed the work without
reducing scope. The final non-browser gate completed in 48.23 seconds at about
1.10 GiB peak RSS; the embedded production desktop build completed in 38.74
seconds at about 2.51 GiB peak RSS; and all 26 desktop/mobile browser scenarios
completed in 29.01 seconds at about 471 MiB peak RSS. The final suites passed
143 desktop frontend tests, five website tests, and 166 Rust tests with three
deliberate live probes ignored.

Iterative validation caught a Fast Refresh export-policy issue, strict optional
fixture indexing, and both TypeScript and Rust bootstrap-count/fixture drift.
Each was corrected before acceptance. The rendered Scheduled section was also
inspected directly after accessibility and overflow checks passed.

## Milestone 18 — Agent-directed model and reasoning selection

Status: complete and verified locally on 2026-07-22. The user explicitly
approved the full local milestone, including implementation, deterministic
verification, documentation, security review, and focused local commits. That
approval did not authorize pushing, merging, publishing, deploying, external
authentication, billable model calls, or other remote mutation.

| Field                       | Current record                                                                                                                                                                                          |
| --------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date               | 2026-07-22                                                                                                                                                                                              |
| Selected model              | GPT-5.6 Sol, XHigh reasoning; manually confirmed                                                                                                                                                        |
| Preliminary forecast        | 4–6 active hours; 25–55 minutes of local commands; about 5–7.5 total elapsed hours; medium confidence                                                                                                   |
| Calibrated forecast         | 4–6.5 active hours; 25–60 minutes of local commands; about 5–8 total elapsed hours; medium confidence                                                                                                   |
| Calibration basis           | Clean Milestone 17A local lineage at `c6892c2`; installed Codex CLI 0.145.0; exact semantic schema match; about 45 GiB available RAM, 690 GiB free storage, low load, and warm Rust/Node/browser caches |
| Compatibility evidence      | Fresh `--experimental` generation confirms `model/list`, `thread/start.dynamicTools`, correlated `item/tool/call`, and per-turn model/reasoning overrides; retained Milestone 13 fixtures remain exact  |
| Start approval              | Full local Milestone 18 explicitly approved                                                                                                                                                             |
| Observed active execution   | Approximately 0.85 hour from branch creation through implementation, debugging, documentation, security correction, and final acceptance                                                                |
| Observed local command time | Approximately 0.18 hour across schema checks, targeted iterations, the corrected complete gates, and the production build                                                                               |
| User approval/waiting time  | Excluded from active execution and local command observations                                                                                                                                           |
| Expected usage intensity    | High model usage; moderate local CPU/memory use; no GPU workload                                                                                                                                        |
| Completion boundary         | Completed locally at `2026-07-22T22:43:34-07:00`; no remote publication or external mutation                                                                                                            |

The calibrated range is slightly wider than the preliminary range because the
complete acceptance boundary includes migration/restart compatibility,
registration degradation, next-turn revalidation, strict cross-language
fixtures, responsive ownership controls, and the full release/browser/native
gate. The installed lifecycle is supported, so recommendation-only behavior is
implemented as an explicit registration-failure degradation path rather than
the primary mode.

The final repository gate completed in 42.39 seconds at about 1.08 GiB peak RSS
and passed 150 desktop tests, five website tests, and 174 runnable Rust tests
with three deliberate live probes ignored. All 26 desktop and six website
Playwright scenarios passed across desktop/mobile viewports in 31.04 seconds,
including accessibility scans. The configured unbundled Tauri production build
completed in 45.57 seconds at about 2.10 GiB peak RSS and embedded the current
frontend, avoiding the raw-Cargo development-URL failure mode.

The calibrated forecast overestimated the implementation path. Existing
conversation/session contracts, the previously validated dynamic-tool
lifecycle, deterministic fixtures, and warm caches kept the work below one
active hour. Iterative validation still caught and corrected stale future-policy
validation, user-pending precedence, rationale redaction, light-theme contrast,
scroll interception, heading order, and an honest migration fallback for
pre-selector conversations before local completion.

### Withdrawn early forecast retained for history

The remainder of this section preserves an early planning estimate prepared on
`2026-07-21T18:43:23-07:00` when the feature was incorrectly placed at
Milestone 13. That start gate was withdrawn on
`2026-07-21T19:08:53-07:00`; it is not the authority or forecast for the
current implementation.

### Withdrawn early reasoning assessment

- **Objective:** let Codex inspect QuireForge's normalized selector state and
  request a policy-compliant model/reasoning choice for the next turn, while
  preserving manual ownership, visible provenance, and the strict native IPC
  boundary.
- **Historical recommendation:** the early pass tentatively selected GPT-5.6
  Sol at Extra High/XHigh because the work combines protocol compatibility,
  state-machine design, cost and prompt-injection controls, restart semantics,
  native concurrency, typed IPC, accessible UI, and adversarial tests. This is
  not an active recommendation and must not be reused without re-evaluation.
- **Lower-strength risk:** High or Medium would be more likely to miss a stale
  catalog race, manual-lock precedence edge, sticky next-turn semantics,
  oscillation path, or unsupported-interface fallback. Correcting one of those
  late would cost more than the reasoning saved.
- **Higher-strength benefit:** Max may marginally help the final adversarial
  review, but it is not expected to materially improve routine implementation
  enough to justify its additional latency and usage.
- **Prerequisites:** Milestone 13 must validate the installed app-server's
  app-owned tool request/result lifecycle, and Milestones 14–17 must establish
  the intervening integration and product surfaces. No model change should be
  made for Milestone 18 now.

### Historical provisional estimate

- Early expected active Codex work: approximately **4–7 hours**.
- Early expected local build/test command time: approximately **30–75 minutes**, with
  warm pnpm, Cargo, and Playwright caches preserved.
- Early expected GitHub Actions time after publication: approximately **10–25 minutes**
  of runner time; hosted queue variance is outside the local baseline.
- Early expected total real-world execution after approval: approximately **6–9
  hours across one or two sessions**, low-to-medium confidence.
- Expected user-blocked/model-change/approval time was unknown and reported
  separately; it is excluded from counted project time.
- Early usage classification: high model usage, balanced local CPU/memory usage,
  no GPU.

This estimate is retained only as forecasting history and is not a current
milestone forecast. It assumed the existing normalized `model/list` catalog,
conversation picker, per-turn model/effort overrides, deterministic app-server
mock harness, and warm dependency/build caches would remain usable. The audit
showed that the application does not yet advertise an app-owned selector tool
or implement its result lifecycle; Milestone 13 must resolve that uncertainty.
If the lifecycle is not reliable, the accepted Milestone 18 fallback remains
recommendation-only rather than a private or fabricated control path.

The most recent generalized host profile provides approximately 42 GiB of
available system RAM, more than 700 GiB of free NVMe storage, low load, and warm
Rust/Node caches. The Balanced profile will begin with four Cargo workers and
two Playwright workers, reducing concurrency if memory pressure or swapping
appears. The RTX 3050 is not used because Rust, TypeScript, React, Tauri, and
browser tests are CPU/system-memory workloads.

| Component                                               | Duration range | Confidence | Primary uncertainty                                      | Resource class      | Safe overlap                           |
| ------------------------------------------------------- | -------------: | ---------- | -------------------------------------------------------- | ------------------- | -------------------------------------- |
| Repository and installed-protocol inspection            |      20–35 min | Medium     | Dynamic-control request lifecycle                        | Model/storage-bound | Batched read-only checks               |
| Architecture, policy, and threat review                 |      35–60 min | Medium     | Ownership and sticky next-turn semantics                 | Model-bound         | Documentation only                     |
| Native selector policy and control lifecycle            |      1.5–2.5 h | Low–Medium | Concurrency, persistence, and supported invocation route | Model/CPU-bound     | Focused UI work when not compiling     |
| Typed IPC and selector ownership/provenance UI          |      45–75 min | Medium     | Accessible pending/effective presentation                | Model-bound         | Targeted native tests                  |
| Deterministic and adversarial tests                     |      45–90 min | Medium     | Prompt injection, stale catalog, restart, oscillation    | Model/CPU-bound     | Documentation at low load              |
| Full build, browser, native, and visual verification    |      25–50 min | Medium     | UI state timing and native mock integration              | CPU/storage-bound   | Limited; heavy gates remain sequential |
| Debugging allowance                                     |      45–90 min | Low        | Unsupported or changed app-server behavior               | Model/CPU-bound     | Case dependent                         |
| Documentation, security/diff review, commit preparation |      35–60 min | Medium     | Cross-document and fixture consistency                   | Model-bound         | Warm builds/tests                      |
| GitHub publication and main CI                          |      10–25 min | Medium     | Hosted queue and cold runner                             | Network-bound       | Local completion review                |

Historical projected critical path: complete the Milestone 13 lifecycle evidence and
Milestones 14–17 prerequisites → refresh the system/model forecast → define
native policy and persistence → implement staged next-turn application → expose
strict typed IPC and accessible selector ownership UI → adversarial and restart
tests → full repository/browser/release/native gates → security and diff review
→ approved publication and successful `main` CI. Independent documentation and
fixture review may overlap warm targeted builds; simultaneous release, browser,
and native suites will be avoided to preserve responsiveness.

## Milestone 19 — Security, accessibility, and performance hardening

| Field                    | Current record                                                                                                                                                                          |
| ------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date            | 2026-07-23                                                                                                                                                                              |
| Selected model           | GPT-5.6 Sol, XHigh reasoning; retained for the approved autonomous local pass                                                                                                            |
| Preliminary forecast     | 4–7 active hours; 25–60 minutes of local commands; about 5–9 total elapsed hours; medium confidence                                                                                     |
| Calibrated forecast      | 4.5–7.5 active hours; 30–75 minutes of local commands; about 5.5–9.5 total elapsed hours; medium confidence                                                                              |
| Calibration basis        | Clean Milestone 18 lineage at `902e0e2`; Node 22.22.1, pnpm 11.15.0, Rust 1.97.1, Codex CLI 0.145.0, warm caches, about 44 GiB available RAM, 690 GiB free storage, and low system load |
| Start approval           | Full local Milestone 19 authorized through the continuing autonomous milestone approval                                                                                                |
| Expected usage intensity | High model usage; moderate local CPU/memory use; no GPU workload, external mutation, live connector authorization, or billable model call                                               |
| Completion boundary      | Local source, browser, native, supply-chain, release-build, documentation, and secret/diff gates; no push, merge, package publication, deployment, or integration installation          |
| Observed active time     | Approximately 0.90 hour                                                                                                                                                                 |
| Observed command time    | Approximately 12–14 minutes, including dependency scans, aggregate/browser gates, configured builds, policy bisection, and isolated native visual probes                                |
| Observed user wait       | 0.00 post-start hour                                                                                                                                                                    |
| Observed total elapsed   | Approximately 1.12 hours from branch start through local closure                                                                                                                        |
| Completion status       | Complete and verified locally; Milestone 20 remains separately approval-gated                                                                                                           |

The calibrated range is wider than the preliminary lower bound because the
baseline found a current high-severity development dependency advisory and an
805.73 KiB initial desktop JavaScript bundle. The acceptance scope therefore
includes dependency remediation and repeatable audit coverage, explicit Tauri
policy assertions, initial-bundle code splitting and budgets, crash-safe UI
recovery, and expanded keyboard, contrast, reduced-motion, responsive, and
adversarial checks.

The existing strict typed IPC boundary, empty webview capability permission
set, deterministic native mocks, startup recovery paths, and prior accessibility
coverage reduce implementation uncertainty. The main risks are preserving the
native React mount behavior while tightening CSP, ensuring command pruning does
not remove required Rust-only plugin functionality, and keeping asynchronous
workspace loading deterministic in browser and production builds.

The final pass preserved the CSP, response headers, and command-pruning
settings. Native visual inspection caught that the original three-second smoke
ended before cold WebKit compilation painted React. Separating the startup,
application, and terminal chunks and retaining an opaque loader through the
first committed paints closed that gap. The final 193,549-byte entry is 76.0%
below the 805,736-byte baseline; the full pre-terminal path is 42.9% below it.
All accepted commands reported zero swaps.

## Milestone 20 — Packaging and release automation

| Field                    | Current record                                                                                                                                                                                                                                        |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date            | 2026-07-23                                                                                                                                                                                                                                            |
| Selected model           | GPT-5.6 Sol, XHigh reasoning; retained for the approved autonomous local pass                                                                                                                                                                          |
| Preliminary forecast     | 4–7 active hours; 45–120 minutes of local commands; about 5.5–10 total elapsed hours; medium confidence                                                                                                                                               |
| Calibrated forecast      | 4.5–7.5 active hours; 60–150 minutes of local commands; about 6–11 total elapsed hours; medium confidence                                                                                                                                             |
| Calibration basis        | Clean Milestone 19 lineage at `9e2f08d`; Ubuntu 26.04 host, Docker 29.1.3, 12 logical CPUs, about 45 GiB available RAM, 690 GiB free storage, warm host Rust/Node caches, and an initially cold Ubuntu 22.04 packaging image                               |
| Start approval           | Full local Milestone 20 explicitly approved by the user's “Proceed”; publication, website activation, push, merge, deployment, host package installation, and external release mutation remain unauthorized                                             |
| Expected usage intensity | High model usage; high local CPU/storage/network use for the cold baseline image and dependency graph; no GPU workload, external authentication, billable model call, or personal Codex-state mutation                                                   |
| Completion boundary      | Reproducible local AppImage/Debian artifacts, checksums and manifest, guarded immutable-SHA workflow source, disposable package lifecycle tests, inactive website download data, documentation, audits, and focused local commits; no public release |
| Observed completion      | `2026-07-23T06:44:00-07:00`; approximately 0.90 active hour, 0.25 automated-wait hour, 0.00 user-blocked hour, and 1.15 counted/wall-clock hours from branch creation through the clean authoritative package pass                                           |
| Accepted evidence        | Clean pinned Ubuntu 22.04 release-candidate build; identical repeated normalization; six package tests; 152 desktop, six website, and 174 runnable Rust tests; 32 desktop and eight website browser scenarios; Node/RustSec audits; isolated package launches |
| Variance                 | Approximately 5.10 active hours, or 85.0%, below the six-hour calibrated midpoint; established Tauri/product contracts and warm dependency caches compressed the path, while baseline/MSRV/tool-drift gates caught real issues before release              |
| Completion status       | Complete and verified locally with focused commits; not pushed, merged, published, attested, deployed, installed on the host, or activated on the website                                                                                                  |

The calibrated range is wider than the preliminary command-time range because
the development host is newer than the selected Ubuntu 22.04 packaging
baseline. Acceptance therefore includes a cold, pinned baseline-container build
instead of treating a faster Ubuntu 26.04 artifact as portable evidence.
Official Tauri guidance identifies Ubuntu 22.04 as a suitable oldest baseline
with WebKitGTK 4.1 and warns that a newer builder can raise the minimum glibc
requirement.

The existing verified production build, canonical identity ADRs, warm host
caches, and deterministic test boundary reduce application uncertainty. The
main risks are Tauri's generated Debian desktop filename, package dependency
metadata, AppImage behavior without FUSE, safe install/upgrade/uninstall
simulation, immutable workflow dependencies, and keeping not-yet-approved
download metadata visibly inactive.

| Component                                               | Duration range | Confidence | Primary uncertainty                                      | Resource class        |
| ------------------------------------------------------- | -------------: | ---------- | -------------------------------------------------------- | --------------------- |
| Package/release architecture and baseline audit         |      30–50 min | High       | Existing identity and release constraints                | Model/storage-bound   |
| Reproducible baseline build and artifact normalization  |    1.25–2.25 h | Medium     | Cold toolchain, Tauri bundler, AppImage dependencies     | CPU/network/storage   |
| Package manifest, checksums, and website data contract  |      45–75 min | Medium     | Strict inactive-to-active promotion boundary             | Model-bound           |
| Disposable install/upgrade/uninstall and launch tests   |      1–1.75 h  | Medium     | Debian maintainer behavior and headless WebKit smoke     | CPU/storage-bound     |
| Guarded release workflow and validation policy          |      45–90 min | Medium     | Permission minimization and artifact handoff             | Model-bound           |
| Full acceptance, documentation, and commit preparation  |      45–90 min | Medium     | Cold rebuild variance and cross-document consistency     | CPU/model-bound       |

Projected critical path: select and pin the Ubuntu 22.04 baseline → define one
release-manifest contract → configure canonical bundles → build and normalize
both package formats → exercise disposable lifecycle and launch checks →
validate guarded workflow/download metadata → complete repository, security,
package, secret, and diff gates → record local completion. GitHub publication
and production download activation remain Milestone 21 approval gates.

## Milestone 21A — Product readiness and usage visibility

| Field                    | Current record                                                                                                                                                                                                 |
| ------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date            | 2026-07-23                                                                                                                                                                                                     |
| Selected model           | GPT-5.6 Sol, XHigh reasoning; retained for the local product-readiness pass                                                                                                                                     |
| Preliminary forecast     | 4–7 active hours; 30–75 minutes of local commands; about 5–9 total elapsed hours; medium confidence                                                                                                            |
| Calibration basis        | Clean Milestone 20 lineage at `735abcb`; Codex CLI 0.145.0; documented `account/rateLimits/read`; 12 logical CPUs, about 16 GiB available RAM, 689 GiB free storage, high existing swap occupancy, and warm caches |
| Start approval           | The user's attached UI reference and explicit requirements authorize the scoped local product-readiness implementation; external publication, deployment, push, merge, authentication, and website activation remain unauthorized |
| Expected usage intensity | High model usage; moderate local CPU/memory use; no GPU workload, external mutation, live login, billable model call, reset-credit redemption, or personal Codex-state mutation                                |
| Completion boundary      | Authenticated startup gate, original QuireForge home/workspace shell, removal of user-facing milestone scaffolding, normalized read-only usage display, deterministic tests, docs, visual review, and local commits |
| Observed completion      | `2026-07-23T11:22:11-07:00`; approximately 0.42 active hour, 0.09 automated-wait hour, 0.00 user-blocked hour, and 0.51 counted/wall-clock hour from branch start through native visual closure |
| Accepted evidence        | 157 desktop and six website tests; 178 runnable Rust tests with three deliberate live probes ignored; 34 desktop browser scenarios; repository/package/build/dist/Clippy gates; rebuilt visible native X11 launch |
| Variance                 | Approximately 5.58 active hours, or 93.0%, below the six-hour calibrated midpoint; established auth/UI/IPC seams compressed implementation while browser/native checks caught real async and contrast regressions |
| Completion status        | Complete and verified locally; Milestone 21B publication remains separately approval-gated                                                                                                                      |

The existing Codex-owned authentication service, strict app-server process
boundary, project/session fixtures, and established visual test harness reduce
implementation uncertainty. The main risks are separating authenticated and
signed-out render states without destabilizing the large existing application
component, normalizing multi-bucket rate limits without leaking account
metadata, and changing the information hierarchy while preserving access to
every implemented workspace.

| Component                                                | Duration range | Confidence | Primary uncertainty                                  | Resource class      |
| -------------------------------------------------------- | -------------: | ---------- | ---------------------------------------------------- | ------------------- |
| Product/auth/usage architecture and protocol review      |      35–60 min | High       | Multi-bucket normalization and signed-out boundaries | Model/storage-bound |
| Native usage service and strict typed IPC                |      45–90 min | Medium     | Sparse/optional upstream rate-limit fields            | Model/CPU-bound     |
| Authenticated startup gate and recovery states           |      45–75 min | Medium     | Existing cached and non-OpenAI provider states        | Model-bound         |
| Home/workspace shell and responsive visual redesign      |    1.25–2.25 h | Medium     | Large-screen density and small-screen preservation    | Model/browser-bound |
| Deterministic, accessibility, and visual tests           |      45–90 min | Medium     | Async state timing and screenshot stability           | CPU/browser-bound   |
| Documentation, full acceptance, and commit preparation   |      45–90 min | Medium     | Cross-document and package-contract consistency       | CPU/model-bound     |

Projected critical path: lock the read-only usage contract → implement native
normalization and typed IPC → gate Codex work surfaces on verified account
state → replace internal milestone presentation with the QuireForge home shell
→ update deterministic fixtures and browser coverage → run full repository,
native, accessibility, build, and visual gates → record local completion.
Package publication and website activation remain a separately approved
Milestone 21B operation.

The accepted pass preserved the documented Codex authentication boundary,
added a closed rate-limit contract, and prevented all workspace/account-data
loaders from starting behind the sign-in gate. Full validation caught and
corrected asynchronous authenticated-startup races in existing component tests
and a light-theme mobile contrast regression in the new task affordance. The
final isolated native launch painted the complete signed-out gate after an
eight-second settle, emitted no refused-loopback evidence, and used isolated
home/XDG roots with no personal credentials.

## Milestone 21B — Beta release-candidate and publication handoff

| Field                    | Current record                                                                                                                                                                                                                                    |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Forecast date            | 2026-07-23                                                                                                                                                                                                                                        |
| Selected model           | GPT-5.6 Sol, XHigh reasoning; retained for the approved Milestone 21 continuation                                                                                                                                                                  |
| Preliminary forecast     | 3–6 active hours; 60–150 minutes of local/container commands; about 4–8 total elapsed hours; medium confidence                                                                                                                                    |
| Calibrated forecast      | 3.5–6.5 active hours; 75–180 minutes of local/container commands; about 4.5–9 total elapsed hours; medium confidence                                                                                                                               |
| Calibration basis        | Clean Milestone 21A lineage at `4351da2`; Docker 29.1.3; Node 22.22.1; pnpm 11.15.0; Rust 1.97.1 host; pinned Ubuntu 22.04 release container; 12 logical CPUs, about 46 GiB available RAM, 689 GiB free storage, high pre-existing swap occupancy, warm images/caches |
| Start approval           | The user's “do it” authorizes the scoped local Milestone 21B continuation. Exact tag/release publication, workflow dispatch, push/merge, public download activation, and website deployment remain terminal external actions governed by their separately documented exact-source and rollback gates |
| Expected usage intensity | High model usage; high local CPU/storage use for clean package builds and lifecycle tests; no GPU workload, live Codex account access, billable model call, integration authorization, or credential mutation                                      |
| Completion boundary      | Exact local candidate source, clean AppImage/Debian rebuild, repeated hashes, package/install/launch QA, supported-platform and known-limitation review, inactive-to-published website-data preparation, rollback record, aggregate gates, and focused commits |
| Completion status        | In progress locally                                                                                                                                                                                                                               |

The pinned package pipeline, already reviewed Tauri helper hashes, strict
manifest contract, disposable Debian lifecycle harness, and established X11
launch smoke reduce build uncertainty. The main risks are source revision drift
between review and tag, external protected-environment/attestation readiness,
independent verification of immutable public downloads, and keeping release
publication separate from website activation.

| Component                                                 | Duration range | Confidence | Primary uncertainty                                      | Resource class        |
| --------------------------------------------------------- | -------------: | ---------- | -------------------------------------------------------- | --------------------- |
| Exact-source, release, distribution, and rollback audit   |      35–60 min | High       | Publication prerequisites and source freeze              | Model/storage-bound   |
| Clean pinned Ubuntu 22.04 candidate rebuild               |    1.0–2.25 h  | Medium     | Container/package cache and helper availability          | CPU/network/storage   |
| Package lifecycle, launch, repeated-hash, and platform QA |      45–90 min | Medium     | X11/AppImage/install behavior after product-readiness UI | CPU/storage-bound     |
| Release notes, support/limitation, and download metadata  |      45–90 min | Medium     | Exact immutable URLs and public artifact record          | Model-bound           |
| Aggregate acceptance and publication handoff             |      45–90 min | Medium     | External gate readiness and cross-document consistency   | CPU/model-bound       |

Projected critical path: freeze a clean candidate source → rebuild in the
pinned baseline → validate exact manifest/checksums and package lifecycle →
repeat normalization/hash evidence → finalize supported-platform,
installation, limitation, and rollback copy → prepare but do not prematurely
activate typed website download data → run aggregate gates → present the exact
tag/release and website-deployment operations for their terminal approval.
