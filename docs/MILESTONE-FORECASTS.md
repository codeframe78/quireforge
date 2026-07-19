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

Preliminary planning range: approximately 2–4 active hours and 2.25–4.5 total
elapsed hours across one or two sessions, excluding approval waiting;
low-to-medium confidence. Primary uncertainty is the Codex-managed browser and
device-login lifecycle, cancellation, redaction, account events, and honest
unauthenticated recovery. Before implementation, refresh current model guidance,
inspect the relevant installed schemas and system state, recommend the model and
reasoning level, produce the full preliminary/calibrated breakdown, request
manual model confirmation, and obtain separate start approval.
