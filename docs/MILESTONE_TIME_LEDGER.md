# Milestone Real-World Time Ledger

This ledger separates model-driven project work from automated waits and manual
approval delays. It was introduced during Milestone 12, so Milestones 0–11 are
historical reconstructions rather than prospective stopwatch records. Ranges
and lower bounds are preserved where the evidence does not support a single
number.

Definitions used here:

- **Active execution** is time spent inspecting, designing, implementing,
  testing, reviewing, documenting, or preparing project changes.
- **Automated wait** is attended local build/test/install time plus required
  GitHub Actions time when publication evidence shows that the milestone waited
  for it. Concurrent runner time is not deliberately counted twice.
- **User-blocked** is time waiting for a manual model change, approval,
  prerequisite, access, or decision. Historical lower bounds include only gaps
  that can be defended from submilestone handoffs.
- **Counted project time** is active execution plus automated wait. It excludes
  user-blocked time.
- **Total elapsed** is the wall-clock interval from execution start to
  completion. For historical entries, Git reflog often provides only a
  lower-bound start, so `≥` is intentional.

## Cumulative project totals

Last updated: `2026-07-20T20:02:30-07:00`

| Measure                                   | Cumulative record                                                                                                                                           |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Total milestones planned                  | 21 (Milestones 0–20)                                                                                                                                        |
| Total milestones completed                | 13 (Milestones 0–12)                                                                                                                                        |
| Milestones in progress                    | 0                                                                                                                                                           |
| Reconstructed/estimated active execution  | Approximately 18.18–25.26 hours                                                                                                                             |
| Reconstructed automated wait              | Approximately 5.78–6.67 hours; early uninstrumented waits excluded                                                                                          |
| Reconstructed user-blocked time           | At least 10.86 hours, plus unmeasured early approvals/prerequisites                                                                                         |
| Counted project time                      | Approximately 23.95–31.92 hours                                                                                                                             |
| Total elapsed across completed milestones | At least 30.75 evidenced hours; exact historical total is unknown                                                                                           |
| Average counted time                      | Approximately 1.84–2.46 hours per completed milestone                                                                                                       |
| Median counted time                       | Approximately 1.85 hours using reconstructed range midpoints                                                                                                |
| Longest completed milestone               | Milestone 11, approximately 5.45–7.72 counted hours                                                                                                         |
| Shortest completed milestone              | Milestone 0, approximately 0.15 counted hours                                                                                                               |
| Comparable forecast versus actual         | Milestones 3–12 forecast 68.00–116.50 active hours in aggregate and used approximately 20.42–28.39 counted hours, about 58%–82% below the forecast envelope |
| Confidence classifications                | 0 Confirmed, 11 Reconstructed, 2 Estimated, 0 Unknown completed entries                                                                                     |

The cumulative active and automated ranges are derived from historical reports
and workflow evidence that were recorded with different granularity. They are
useful forecasting bounds, not a claim of minute-level precision. Total elapsed
is a separate lower bound and should not be arithmetically reconciled with the
range endpoints.

## Summary

| Milestone | Status   | Started                     | Completed                   | Forecast                     |      Active | Automated Wait |              User-Blocked | Counted Project Time | Total Elapsed | Variance                        | Confidence    |
| --------- | -------- | --------------------------- | --------------------------- | ---------------------------- | ----------: | -------------: | ------------------------: | -------------------: | ------------: | ------------------------------- | ------------- |
| 0         | Complete | `2026-07-19T08:52:30-07:00` | `2026-07-19T09:01:47-07:00` | Unknown                      |     ~0.15 h |    Unseparated |                   Unknown |              ~0.15 h |       ≥0.15 h | Unknown                         | Reconstructed |
| 1         | Complete | `2026-07-19T09:21:36-07:00` | `2026-07-19T12:37:30-07:00` | Unknown                      |     ~2.91 h |    Unseparated |                   Unknown |              ~2.91 h |       ≥3.27 h | Unknown                         | Estimated     |
| 2         | Complete | `2026-07-19T12:41:58-07:00` | `2026-07-19T13:10:15-07:00` | Unknown                      |     ~0.47 h |    Unseparated |                   Unknown |              ~0.47 h |       ≥0.47 h | Unknown                         | Reconstructed |
| 3         | Complete | `2026-07-19T13:29:01-07:00` | `2026-07-19T14:22:24-07:00` | 18–30 h active               | 0.58–0.92 h |    0.05–0.12 h |                   Unknown |          0.63–1.04 h |       ≥0.89 h | ~23.17 h / 96.5% below midpoint | Estimated     |
| 4         | Complete | `2026-07-19T14:34:03-07:00` | `2026-07-19T15:02:03-07:00` | 4–7 h active                 | 0.42–0.58 h |    0.03–0.07 h | 0 h after start evidenced |          0.45–0.65 h |       ≥0.47 h | ~4.95 h / 90.0% below midpoint  | Reconstructed |
| 5         | Complete | `2026-07-19T15:37:23-07:00` | `2026-07-19T16:05:45-07:00` | 2–4 h active                 | 0.42–0.67 h |    0.08–0.13 h | 0 h after start evidenced |          0.50–0.80 h |       ≥0.47 h | ~2.35 h / 78.3% below midpoint  | Reconstructed |
| 6         | Complete | `2026-07-19T16:24:44-07:00` | `2026-07-19T18:02:33-07:00` | 2.5–5 h active               | 1.33–2.08 h |    0.10–0.18 h |     Present; not measured |          1.43–2.26 h |       ≥1.63 h | ~1.91 h / 50.8% below midpoint  | Reconstructed |
| 7         | Complete | `2026-07-19T18:26:16-07:00` | `2026-07-19T20:11:20-07:00` | 4–7.5 h active               | 1.25–1.91 h |    0.91–0.98 h |                   ≥0.28 h |          2.16–2.89 h |       ≥1.75 h | ~3.23 h / 56.1% below midpoint  | Reconstructed |
| 8         | Complete | `2026-07-19T20:23:15-07:00` | `2026-07-19T22:17:24-07:00` | 5.5–10 h active              | 1.58–2.42 h |    0.73–0.81 h |                   ≥0.24 h |          2.31–3.23 h |       ≥1.90 h | ~4.98 h / 64.3% below midpoint  | Reconstructed |
| 9         | Complete | `2026-07-19T22:29:22-07:00` | `2026-07-20T05:00:25-07:00` | 6–11 h active                | 1.83–3.08 h |    1.07–1.17 h |                   ≥4.26 h |          2.90–4.25 h |       ≥6.52 h | ~4.93 h / 57.9% below midpoint  | Reconstructed |
| 10        | Complete | `2026-07-20T05:05:51-07:00` | `2026-07-20T12:10:43-07:00` | 8.5–13.5 h active            | 2.00–2.83 h |    1.06–1.19 h |                   ≥4.44 h |          3.06–4.02 h |       ≥7.08 h | ~7.46 h / 67.8% below midpoint  | Reconstructed |
| 11        | Complete | `2026-07-20T12:19:40-07:00` | `2026-07-20T16:56:33-07:00` | 12.5–20.5 h active           | 4.08–6.08 h |    1.37–1.64 h |                   ≥1.64 h |          5.45–7.72 h |       ≥4.61 h | ~9.91 h / 60.1% below midpoint  | Reconstructed |
| 12        | Complete | `2026-07-20T18:27:06-07:00` | `2026-07-20T19:59:04-07:00` | 5–8 h active; 6–10 h elapsed |     ~1.16 h |        ~0.38 h |                    0.00 h |               1.53 h |        1.53 h | ~4.97 h / 76.4% below midpoint  | Reconstructed |

Variance uses the midpoint of the historical forecast and counted-time ranges.
It is included only where a recorded forecast exists and should not be read as
greater precision than the underlying ranges.

## Milestone 0 — Existing project audit and feasibility

- **Objective:** Establish the feasibility, supported-interface, architecture,
  security, and hosting/deployment baseline without mutating production state.
- **Start / completion:** `2026-07-19T08:52:30-07:00` /
  `2026-07-19T09:01:47-07:00`.
- **Model and reasoning:** Unknown; no durable model record exists.
- **Original forecast:** Unknown.
- **Active intervals:** Approximately 0.15 hour between branch creation and the
  discovery completion commit.
- **Automated wait:** Not separable from the short interval.
- **User-blocked:** Unknown.
- **Counted / total elapsed:** Approximately 0.15 counted hour; at least 0.15
  elapsed hour.
- **Variance:** Unknown because no forecast was recorded.
- **Evidence:** Reflog branch creation, commit `4de62fd`, roadmap completion,
  and the Milestone 0 discovery documents.
- **Confidence:** **Reconstructed**. The timestamps are durable; category
  separation was not recorded.
- **Lesson:** Record the execution start before discovery commands and retain
  command timing independently from research time.
- **Publication:** Later included in [PR #1](https://github.com/codeframe78/quireforge/pull/1);
  no release or deployment exists.

## Milestone 1 — Identity migration and governance closure

- **Objective:** Reconcile the permanent identity and repository move, close
  governance requirements, and retain only the selected current hosting route.
- **Start / completion:** `2026-07-19T09:21:36-07:00` /
  `2026-07-19T12:37:30-07:00`.
- **Model and reasoning / original forecast:** Unknown.
- **Active intervals:** Approximately 2.91 hours across the identity/brand,
  superseded hosting-feasibility, and governance branch windows.
- **Automated wait:** Present but not separable from active intervals.
- **User-blocked:** Access/approval waiting occurred but was not measured.
- **Counted / total elapsed:** Approximately 2.91 counted hours; at least 3.27
  elapsed hours.
- **Variance:** Unknown.
- **Evidence:** Reflog branch intervals; identity, brand, audit, governance, and
  CI commits ending at `5d5ce9c`; roadmap and README status.
- **Confidence:** **Estimated** because the long audit interval contains an
  unknown mix of active, automated, and approval-bound time.
- **Lesson:** Split access-bound audits from implementation and timestamp the
  pause as soon as owner action becomes necessary.
- **Publication:** Later included in [PR #1](https://github.com/codeframe78/quireforge/pull/1).

## Milestone 2 — Brand and static website foundation

- **Objective:** Establish the local Astro/Cloudflare-compatible static site,
  shared brand use, and website quality gates without deployment.
- **Start / completion:** `2026-07-19T12:41:58-07:00` /
  `2026-07-19T13:10:15-07:00`.
- **Model and reasoning / original forecast:** Unknown.
- **Active intervals:** Approximately 0.47 hour from branch creation through
  implementation, CI, and documentation commits.
- **Automated wait / user-blocked:** Not separately instrumented / unknown.
- **Counted / total elapsed:** Approximately 0.47 counted hour; at least 0.47
  elapsed hour.
- **Variance:** Unknown.
- **Evidence:** Branch reflog and commits `b67caf0`, `8053582`, and `6ecf84c`.
- **Confidence:** **Reconstructed** for the interval, not its internal split.
- **Lesson:** Time production builds and browser gates as separate automated
  intervals even when the total milestone is short.
- **Publication:** Later included in [PR #1](https://github.com/codeframe78/quireforge/pull/1);
  the website remains undeployed.

## Milestone 3 — Desktop scaffold consolidation

- **Objective:** Produce the typed Tauri/React desktop foundation and native
  Linux identity without packaging.
- **Start / completion:** `2026-07-19T13:29:01-07:00` /
  `2026-07-19T14:22:24-07:00`.
- **Model and reasoning:** GPT-5.6 Sol, High.
- **Original forecast:** 18–30 active hours, low confidence.
- **Active / automated intervals:** Approximately 0.58–0.92 active hour across
  two work periods and 0.05–0.12 hour of dependency/build/test/browser waits.
- **User-blocked:** A manual Linux prerequisite was required; duration unknown.
- **Counted / total elapsed:** 0.63–1.04 counted hours; at least 0.89 elapsed
  hour.
- **Variance:** Approximately 23.17 hours (96.5%) below midpoint forecast.
- **Evidence:** `docs/MILESTONE-FORECASTS.md`, build-performance records,
  branch reflog, and commits `cf8825f` through `98af5d2`.
- **Confidence:** **Estimated** because the prerequisite split and true first
  work-period start were not instrumented.
- **Lesson:** Calibrate large roadmap labels against a real warm/cold build
  sample before estimating implementation time.
- **Publication:** Later included in [PR #1](https://github.com/codeframe78/quireforge/pull/1).

## Milestone 4 — Codex process adapter and contracts

- **Objective:** Add a supervised, versioned Codex adapter and normalized model
  contract without starting a billable turn.
- **Start / completion:** `2026-07-19T14:34:03-07:00` /
  `2026-07-19T15:02:03-07:00`.
- **Model and reasoning:** GPT-5.6 Sol, High.
- **Original forecast:** Calibrated 4–7 active hours.
- **Active / automated / user-blocked:** 0.42–0.58 active hour, 0.03–0.07
  automated hour, and no post-start user block evidenced.
- **Counted / total elapsed:** 0.45–0.65 counted hour; at least 0.47 elapsed
  hour.
- **Variance:** Approximately 4.95 hours (90.0%) below midpoint forecast.
- **Evidence:** Forecast/build records, reflog, and commits `eaa4e69` through
  `a835283`.
- **Confidence:** **Reconstructed** from a recorded active estimate and timed
  commands.
- **Lesson:** Generated protocol breadth does not imply equal implementation
  breadth when only reviewed schemas cross the boundary.
- **Publication:** Later included in [PR #1](https://github.com/codeframe78/quireforge/pull/1).

## Milestone 5 — Authentication and onboarding

- **Objective:** Add Codex-owned browser/device onboarding, cancellation,
  explicit logout, and redacted recovery without credential storage.
- **Start / completion:** `2026-07-19T15:37:23-07:00` /
  `2026-07-19T16:05:45-07:00`.
- **Model and reasoning:** GPT-5.6 Sol, High.
- **Original forecast:** Calibrated 2–4 active hours.
- **Active / automated / user-blocked:** 0.42–0.67 active hour, 0.08–0.13
  automated hour, and no post-start user block evidenced.
- **Counted / total elapsed:** 0.50–0.80 counted hour; at least 0.47 elapsed
  hour.
- **Variance:** Approximately 2.35 hours (78.3%) below midpoint forecast.
- **Evidence:** Forecast/build records, reflog, commits `f0a9dcd` through
  `5edddc1`, and bulk publication [PR #1](https://github.com/codeframe78/quireforge/pull/1).
- **Confidence:** **Reconstructed**; local command time is a recorded range.
- **Lesson:** Preserve warm native caches but keep release validation because it
  catches async Tauri contract failures ordinary checking can miss.

## Milestone 6 — Projects and direct directory attachment

- **Objective:** Add migrated app metadata, native directory selection,
  identity-aware preflight, relink, detach/archive, and the accessible project
  workspace without copying or deleting source content.
- **Start / completion:** `2026-07-19T16:24:44-07:00` /
  `2026-07-19T18:02:33-07:00`.
- **Model and reasoning:** GPT-5.6 Sol, XHigh for 6A and High for 6B.
- **Original forecast:** Calibrated complete milestone 2.5–5 active hours.
- **Active / automated:** 1.33–2.08 active hours and 0.10–0.18 automated hour.
- **User-blocked:** A submilestone model/start handoff occurred but was not
  timestamped reliably.
- **Counted / total elapsed:** 1.43–2.26 counted hours; at least 1.63 elapsed
  hours.
- **Variance:** Approximately 1.91 hours (50.8%) below midpoint forecast.
- **Evidence:** Forecast/build records, reflog, commits `e0c3333` through
  `dd0dad3`, [PR #3](https://github.com/codeframe78/quireforge/pull/3), and
  merge-status [PR #4](https://github.com/codeframe78/quireforge/pull/4).
- **Confidence:** **Reconstructed**; GitHub workflows at this point failed at
  startup and add no defensible runner wait.
- **Lesson:** Split native identity/storage and frontend integration so each can
  use the lowest reliable reasoning strength.

## Milestone 7 — Conversation MVP

- **Objective:** Establish exact native thread/turn ownership and then add the
  runtime-derived composer, bounded progress stream, and stop interaction.
- **Start / completion:** `2026-07-19T18:26:16-07:00` /
  `2026-07-19T20:11:20-07:00`.
- **Model and reasoning:** GPT-5.6 Sol, XHigh for 7A and High for 7B.
- **Original forecast:** Combined calibrated submilestones 4–7.5 active hours.
- **Active / automated / user-blocked:** 1.25–1.91 active hours, 0.91–0.98
  automated hour including the abnormal queued/cancelled Actions period, and at
  least 0.28 hour between gated submilestones.
- **Counted / total elapsed:** 2.16–2.89 counted hours; at least 1.75 elapsed
  hours. Some runner and active intervals overlapped, so the counted range is
  deliberately not forced to equal the elapsed lower bound.
- **Variance:** Approximately 3.23 hours (56.1%) below midpoint forecast.
- **Evidence:** Forecast/build records, reflog, commits/PRs
  [#5](https://github.com/codeframe78/quireforge/pull/5) and
  [#6](https://github.com/codeframe78/quireforge/pull/6), and Actions runs
  `29712137967`, `29712381378`, and `29713730219`.
- **Confidence:** **Reconstructed** with an overlap caveat.
- **Lesson:** Treat runner queue time separately and continue independent work
  only when that concurrency is safe and documented.

## Milestone 8 — Session lifecycle and recovery

- **Objective:** Add app-reference-only resume/fork/archive/restore, crash
  reconciliation, bounded search/grouping, and accessible session history.
- **Start / completion:** `2026-07-19T20:23:15-07:00` /
  `2026-07-19T22:17:24-07:00`.
- **Model and reasoning:** GPT-5.6 Sol, XHigh for 8A and High for 8B.
- **Original forecast:** Combined calibrated submilestones 5.5–10 active hours.
- **Active / automated / user-blocked:** 1.58–2.42 active hours, 0.73–0.81
  automated hour, and at least 0.24 hour at the submilestone gate.
- **Counted / total elapsed:** 2.31–3.23 counted hours; at least 1.90 elapsed
  hours.
- **Variance:** Approximately 4.98 hours (64.3%) below midpoint forecast.
- **Evidence:** Forecast/build records, reflog,
  [PR #7](https://github.com/codeframe78/quireforge/pull/7),
  [PR #8](https://github.com/codeframe78/quireforge/pull/8), and their
  successful PR/main repository-check runs.
- **Confidence:** **Reconstructed**; active ranges predate this ledger.
- **Lesson:** Reuse authoritative reconciliation and reference-only storage;
  gate UI/search separately from lifecycle architecture.

## Milestone 9 — Approvals and command presentation

- **Objective:** Add the strict native approval/activity contract and the
  selectable real-time activity and approval UI.
- **Start / completion:** `2026-07-19T22:29:22-07:00` /
  `2026-07-20T05:00:25-07:00`.
- **Model and reasoning:** GPT-5.6 Sol, XHigh for 9A and High for 9B.
- **Original forecast:** Combined calibrated submilestones 6–11 active hours.
- **Active / automated / user-blocked:** 1.83–3.08 active hours, 1.07–1.17
  automated hours, and at least 4.26 hours between the 9A publication checkpoint
  and the 9B branch start.
- **Counted / total elapsed:** 2.90–4.25 counted hours; at least 6.52 elapsed
  hours.
- **Variance:** Approximately 4.93 hours (57.9%) below midpoint forecast.
- **Evidence:** Forecast/build records, reflog, PRs
  [#9](https://github.com/codeframe78/quireforge/pull/9),
  [#10](https://github.com/codeframe78/quireforge/pull/10), and
  [#11](https://github.com/codeframe78/quireforge/pull/11), plus their
  successful repository-check runs.
- **Confidence:** **Reconstructed**.
- **Lesson:** Native correlation/redaction deserved XHigh; the presentation
  layer did not need to retain it automatically.

## Milestone 10 — Git review and controlled mutations

- **Objective:** Add fixed read-only Git review followed by separately confirmed
  stage, unstage, bounded revert/recovery, and commit workflows.
- **Start / completion:** `2026-07-20T05:05:51-07:00` /
  `2026-07-20T12:10:43-07:00`.
- **Model and reasoning:** GPT-5.6 Sol, High for 10A and XHigh for 10B.
- **Original forecast:** Combined calibrated submilestones 8.5–13.5 active
  hours.
- **Active / automated / user-blocked:** 2.00–2.83 active hours, 1.06–1.19
  automated hours, and at least 4.44 hours at the submilestone gate.
- **Counted / total elapsed:** 3.06–4.02 counted hours; at least 7.08 elapsed
  hours.
- **Variance:** Approximately 7.46 hours (67.8%) below midpoint forecast.
- **Evidence:** Forecast/build records, reflog,
  [PR #12](https://github.com/codeframe78/quireforge/pull/12),
  [PR #20](https://github.com/codeframe78/quireforge/pull/20), and corrective
  [PR #21](https://github.com/codeframe78/quireforge/pull/21), including the
  failed main run and successful correction runs.
- **Confidence:** **Reconstructed**.
- **Lesson:** Hosted CI can expose inode/process/filesystem behavior absent from
  a local gate; reserve correction time for mutation-sensitive milestones.

## Milestone 11 — Worktrees and parallel work

- **Objective:** Add managed worktree creation/attachment, bounded parallel
  execution, retained-checkout recovery, and clean managed-worktree cleanup.
- **Start / completion:** `2026-07-20T12:19:40-07:00` /
  `2026-07-20T16:56:33-07:00`.
- **Model and reasoning:** GPT-5.6 Sol, XHigh for 11A/11C and Extra High for
  11B.
- **Original forecast:** Combined calibrated submilestones 12.5–20.5 active
  hours.
- **Active / automated / user-blocked:** 4.08–6.08 active hours, 1.37–1.64
  automated hours, and at least 1.64 hours across gated handoffs.
- **Counted / total elapsed:** 5.45–7.72 counted hours; at least 4.61 elapsed
  hours. Historical active estimates and runner intervals overlap, so this is a
  forecasting range rather than a stopwatch total.
- **Variance:** Approximately 9.91 hours (60.1%) below midpoint forecast.
- **Evidence:** Forecast/build records, reflog, PRs
  [#22](https://github.com/codeframe78/quireforge/pull/22),
  [#23](https://github.com/codeframe78/quireforge/pull/23), and
  [#25](https://github.com/codeframe78/quireforge/pull/25), plus all successful
  PR/main repository-check runs.
- **Confidence:** **Reconstructed** with explicit overlap uncertainty.
- **Lesson:** Submilestone reuse sharply compresses large roadmap forecasts;
  keep destructive cleanup, concurrency, and foundation work separately gated.

## Milestone 12 — Integrated terminal

- **Objective:** Add bounded native Linux PTYs with verified attached-project
  cwd, byte-safe output, input/resize, tabs, background-job ownership, and
  explicit cleanup without persisting shell content.
- **Start / completion:** `2026-07-20T18:27:06-07:00` /
  `2026-07-20T19:59:04-07:00`. The start is reconstructed from the branch
  creation reflog after the approved start instruction; the completion is the
  recorded successful `main` workflow result after merge.
- **Model and reasoning:** GPT-5.6 Sol, XHigh; manually confirmed.
- **Original forecast:** 5–8 active hours, 35–80 minutes of local commands, and
  6–10 total elapsed hours in one or two sessions; low-to-medium confidence.
- **Active work intervals:** Approximately 1.16 hours across repository/system
  inspection, PTY/security design, implementation, test hardening, visual and
  native verification, documentation, review, and publication operations. The
  ledger directive arrived mid-milestone, so this split is reconstructed by
  subtracting evidenced automated intervals from the complete elapsed window.
- **Automated wait intervals:** Approximately 0.38 hour: at least 0.05 hour of
  individually timed local commands, 0.16 hour for the pull-request workflow
  (`2026-07-20T19:39:17-07:00`–`19:49:05-07:00`), and 0.16 hour for the
  successful `main` workflow (`19:49:28-07:00`–`19:59:04-07:00`). Concurrent
  hosted jobs are counted by critical-path wall time rather than summed.
- **User-blocked intervals:** 0.00 hour after the recorded execution start. All
  model/reasoning/start approvals preceded it.
- **Counted / total elapsed:** Approximately 1.53 counted hours and 1.53 total
  elapsed hours. No post-start user-blocked interval is excluded. Independently
  rounded category figures may not add exactly to the rounded total.
- **Variance:** Approximately 4.97 hours, or 76.4%, below the 6.5-hour midpoint
  of the original active forecast when compared with primary counted project
  time. The original 6–10-hour elapsed forecast was also conservative.
- **Evidence:** Branch `feat/milestone-12-integrated-terminal`; reflog; approved
  reasoning/start messages; timed command records; ADR 0017; commits `d060e50`,
  `bca2770`, and `f2e3fb2`; [PR #27](https://github.com/codeframe78/quireforge/pull/27);
  merge commit `07ba046`; successful PR workflow
  [29796398390](https://github.com/codeframe78/quireforge/actions/runs/29796398390);
  and successful `main` workflow
  [29796847052](https://github.com/codeframe78/quireforge/actions/runs/29796847052).
- **Confidence:** **Reconstructed**. Durable start/completion and hosted-runner
  timestamps exist, but active versus local-wait separation before the ledger
  directive is not a prospective stopwatch record.
- **Explanation of variance:** Existing project-reservation, typed bridge,
  SQLite, fixture, and responsive-shell patterns were more reusable than the
  low-confidence forecast assumed. The reviewed dependency set integrated
  cleanly, real PTY tests exposed only a small test-import correction, and no
  product defect required architectural rework.
- **Forecasting lessons:** PTY compilation remained below 1.82 GiB RSS and the
  Balanced four-Cargo/two-Playwright profile completed with zero swaps and no
  GPU work. The first optimized local release compile took 88.76 seconds, but
  fresh hosted runners spent about 9.5–9.8 minutes on each desktop gate. Future
  forecasts should separate implementation uncertainty from repeated cold-host
  compilation and evaluate credential-free Cargo/Playwright caches as their own
  approved CI optimization.
