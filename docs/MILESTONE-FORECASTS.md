# Milestone Forecasts

Forecasts use ranges because model latency, downloads, failures, approvals, and
external services vary. Each future milestone receives a preliminary forecast
before manual model selection and a calibrated forecast after the selected
model, current system state, caches, and any approved preflight are verified.

Approval waiting time is reported separately from active Codex and local
command time. Performance measurements come from required project work; caches
are not deleted merely to create benchmarks.

## Milestone 3 — Desktop Scaffold Consolidation

| Field | Record |
|---|---|
| Forecast date | 2026-07-19 |
| Recommended model | GPT-5.6 Sol, High reasoning |
| Preliminary forecast | 18–30 active engineering hours; low confidence |
| Calibrated forecast | Not produced; system-calibrated forecasting was introduced during final validation |
| Observed active execution | Approximately 35–55 minutes across two active work periods; approval delay excluded and exact end-to-end time not instrumented |
| User approval/waiting time | Present for milestone confirmation and sudo prerequisite installation; not measured reliably |
| Sessions | Two active work periods separated by the host-prerequisite checkpoint |
| Usage intensity | High model/tool activity; moderate local CPU use during cold Rust builds |
| Completion status | Complete locally; not pushed, merged, packaged, or released |

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

| Field | Record |
|---|---|
| Forecast date | 2026-07-19 |
| Selected model | GPT-5.6 Sol, High reasoning; manually confirmed |
| Preliminary forecast | 2–4 active hours; about 2.25–4.5 total elapsed hours; low-to-medium confidence |
| Calibrated forecast | 4–7 active hours; about 4.5–8 total elapsed hours across 2–3 sessions; low-to-medium confidence |
| Calibration cause | The generated experimental protocol bundle contained 337 files / about 3.16 MB, and process/redaction failure paths were broader than the preliminary assumption |
| Observed active execution | Approximately 25–35 minutes from branch creation through implementation, required validation, documentation, and commit preparation; approval delay excluded |
| Local command time | Approximately 2–4 minutes across required dependency, compile, test, build, browser, live-probe, and repeated failure-correction commands |
| User approval/waiting time | Manual model and start confirmations occurred before branch creation; waiting time was not included |
| Sessions | One active implementation session with recoverable contract, transport, UI, and validation checkpoints |
| Usage intensity | High model/tool activity; low-to-moderate local CPU and memory use |
| Completion status | Complete locally; not pushed, merged, authenticated, packaged, deployed, or released |

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

| Field | Record |
|---|---|
| Forecast date | 2026-07-19 |
| Selected model | GPT-5.6 Sol, High reasoning; manually confirmed |
| Preliminary forecast | 2–4 active hours; about 2.25–4.5 total elapsed hours across 1–2 sessions; low-to-medium confidence |
| Calibrated forecast | 2–4 active hours; about 2.25–4.5 total elapsed hours across 1–2 sessions; medium confidence |
| Calibration basis | Stable installed authentication schemas, ample current memory, warm Milestone 4 caches, six Cargo workers, and two Playwright workers |
| Observed active execution | Approximately 25–40 minutes from branch creation through implementation, required validation, visual checks, documentation, and commit preparation; approval delay excluded |
| Local command time | Approximately 5–8 minutes across schema inspection, dependency resolution, compile, test, browser-cache restoration, release build, smoke test, and validation commands |
| User approval/waiting time | Manual model, reasoning-strength, audit, and start confirmations occurred before branch creation; waiting time was not included |
| Sessions | One active implementation session with recoverable native-service, UI, validation, and documentation checkpoints |
| Usage intensity | High model/tool activity; moderate local CPU use and low memory pressure during cold native builds |
| Completion status | Complete locally; not pushed, merged, logged in, logged out, packaged, deployed, or released |

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

| Field | Record |
|---|---|
| Forecast date | 2026-07-19 |
| Selected model | GPT-5.6 Sol, XHigh reasoning; manually confirmed for Milestone 6A |
| Preliminary forecast | 3–6 active hours; about 3.5–7 total elapsed hours across 1–2 sessions; low-to-medium confidence |
| Calibrated complete-milestone forecast | 2.5–5 active hours; 15–35 minutes of local commands; about 3–6 total elapsed hours; medium confidence |
| Calibrated Milestone 6A forecast | 1.5–3 active hours; 8–20 minutes of local commands; about 1.75–3.5 total elapsed hours; medium confidence |
| Observed Milestone 6A execution | Approximately 45–70 active minutes through implementation, security review, expanded native tests, documentation, final validation, and checkpoint publication; approval delay excluded |
| Observed local command time | Approximately 3–5 minutes across dependency-expanded checking, targeted/full Rust gates, and repeated full repository validation after correcting stale expectations; file reads excluded |
| User approval/waiting time | Manual model, reasoning, audit, and start confirmations occurred before implementation; waiting time was excluded |
| Sessions | One active native-core work period; frontend/integration work remains a separately gated checkpoint |
| Usage intensity | High model/tool activity; moderate CPU and low memory pressure |
| Completion status | Milestone 6A native storage/identity checkpoint complete locally; Milestone 6B frontend/integration pending |

The Milestone 6A forecast overestimated implementation and command time. The
accepted architecture, fast bundled-SQLite compilation, warm Tauri caches,
small transaction surface, and deterministic temporary-directory fixtures
compressed the critical path without reducing scope. XHigh reasoning was
useful for migration invariants, path identity, TOCTOU handling, permission
boundaries, and fail-closed diagnostics. The planned frontend/integration
checkpoint should be reevaluated at High reasoning rather than automatically
retaining XHigh.

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
