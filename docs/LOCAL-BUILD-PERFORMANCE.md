# Local Build Performance

Status: generalized baseline captured from required Milestones 3–6 work on
2026-07-19. These measurements guide local forecasts; they are not release
performance claims or a supported-hardware baseline.

## Generalized development profile

| Resource                | Observed profile                                                           |
| ----------------------- | -------------------------------------------------------------------------- |
| Operating system        | Ubuntu 26.04 LTS, Linux 7.0, x86_64                                        |
| CPU                     | AMD Ryzen 5 5600G class, 6 physical cores / 12 logical processors          |
| System memory           | Approximately 61 GiB total; 47 GiB available at audit                      |
| Swap                    | 8 GiB file-backed swap; about 176 MiB used; no zram configured             |
| Workspace storage       | NVMe-backed ext4; approximately 733 GiB available                          |
| GPU                     | NVIDIA GeForce RTX 3050, 8 GiB VRAM; driver available; CUDA toolkit absent |
| Rust                    | rustc/Cargo 1.97.1 with rustfmt and Clippy                                 |
| JavaScript              | Node 22.22.1 and pnpm 11.15.0                                              |
| Native tools            | GCC 15.2.0 and pkg-config 2.5.1                                            |
| Optional caches/linkers | Cargo and pnpm caches present; no sccache, ccache, mold, or LLVM lld       |

The audit recorded no competing QuireForge build and a load average below two.
It did not collect hardware serial numbers, network addresses, private mount
names, credentials, or unrelated process details.

## Milestone 3 measurements

Measurements came from commands already required to implement or verify the
milestone. Caches were preserved; no clean build was run for benchmarking.

| Operation                                                |        Observed wall time | Cache state                                   | Result           |
| -------------------------------------------------------- | ------------------------: | --------------------------------------------- | ---------------- |
| Workspace dependency installation after manifest changes |           about 5 seconds | Package cache populated; lockfile updated     | Passed           |
| First successful native `cargo check`                    |          about 37 seconds | Partially populated after dependency download | Passed           |
| First Rust test-profile build and tests                  |          about 44 seconds | Cold test profile                             | Passed           |
| First Tauri unbundled release build                      | about 1 minute 18 seconds | Cold release profile                          | Passed           |
| Warm `cargo check`                                       |         about 0.4 seconds | Warm                                          | Passed           |
| Warm Clippy with warnings denied                         |         about 0.5 seconds | Warm                                          | Passed           |
| Warm Rust tests                                          |           about 4 seconds | Warm                                          | Passed           |
| Desktop Vite production transform/build                  |   about 0.12–0.15 seconds | Warm frontend cache                           | Passed           |
| Astro static build                                       |   about 0.35–0.37 seconds | Warm                                          | Passed, 15 pages |
| Combined desktop and website browser suites              |           about 8 seconds | Browser installed; two workers per package    | Passed, 14 tests |

Peak memory was not instrumented during Milestone 3. The post-build audit showed
substantial available memory, minimal pre-existing swap use, low system load,
and no evidence of memory pressure. GPU computation was not used.

## Milestone 4 measurements

The adapter work reused the existing Cargo and pnpm caches. Enabling Tokio
process/I/O features added three locked transitive packages; no clean build was
run. Resource values below come from commands already required by the milestone.

| Operation                                                               |      Observed wall time |   Approximate peak RSS | Result                                                      |
| ----------------------------------------------------------------------- | ----------------------: | ---------------------: | ----------------------------------------------------------- |
| Full experimental app-server schema generation to a temporary directory |      about 0.33 seconds |       Not instrumented | Passed, 337 files / about 3.16 MB; temporary output removed |
| First `cargo check` after Tokio feature additions                       |       about 5.9 seconds |       Not instrumented | Passed                                                      |
| First Rust test-profile compile and adapter tests                       |      about 15.1 seconds |       Not instrumented | Passed                                                      |
| Non-billable live CLI/app-server compatibility probe                    |      about 0.32 seconds |       Not instrumented | Passed; no child remained                                   |
| Warm Clippy with warnings denied                                        |   about 0.8–1.0 seconds |       Not instrumented | Passed                                                      |
| Successful unbundled release build after adapter changes                |      about 19.5 seconds |         about 1.25 GiB | Passed                                                      |
| Full non-browser `pnpm validate` gate                                   | about 23.1–29.7 seconds | about 584 MiB–1.02 GiB | Passed                                                      |
| Combined website and desktop browser suites                             |       about 9.8 seconds |          about 253 MiB | Passed, 14 tests                                            |
| Desktop Vite production build                                           | about 0.12–0.13 seconds |         Included above | Passed                                                      |

An earlier release attempt stopped after about 12 seconds when the Tauri macro
required an async state-borrowing command to return `Result`; the corrected
build passed without repeating unchanged work. Running a production build and
its preview test concurrently also demonstrated a stale-`dist` race, so those
steps are now kept sequential. No OOM, heavy swapping, throttling, or material
disk pressure was observed. GPU computation remained unused.

## Milestone 5 measurements

Authentication work reused the existing Rust, pnpm, and browser caches. The
new native opener dependency required one lockfile-respecting dependency
resolution; no cache was deleted and no clean build was run.

| Operation                                         |        Observed wall time | Approximate peak RSS | Result                                                                                  |
| ------------------------------------------------- | ------------------------: | -------------------: | --------------------------------------------------------------------------------------- |
| Stable authentication-schema calibration          |        about 0.43 seconds |         about 45 MiB | Passed, 267 generated files / about 2.72 MB inspected in temporary storage              |
| Reviewed authentication-schema generation         |        about 0.98 seconds |        about 141 MiB | Passed, 13 selected schemas; repeat run was idempotent                                  |
| First `cargo check` after opener dependencies     |        about 39.4 seconds |       about 1.17 GiB | Passed; 39 locked transitive packages resolved                                          |
| First authentication test-profile build and tests |        about 55.6 seconds |       about 1.36 GiB | Passed                                                                                  |
| Read-only live `account/read` compatibility probe |        about 0.57 seconds |        about 104 MiB | Passed; no private account data printed and no child remained                           |
| Unbundled Tauri release build                     | about 1 minute 18 seconds |       about 1.37 GiB | Passed                                                                                  |
| Full non-browser `pnpm validate` gate             |   about 24.3–25.7 seconds |    about 601–604 MiB | Passed, including 21 unit/integration tests and 20 Rust tests with 2 live tests ignored |
| Desktop browser suite after cache restoration     |         about 5.3 seconds |        about 239 MiB | Passed, 6 tests with two workers                                                        |
| Final combined desktop and website browser suites |         about 8.4 seconds |        about 254 MiB | Passed, 14 tests with two workers per package                                           |
| Desktop Vite production build                     |   about 0.12–0.14 seconds |       Included above | Passed                                                                                  |

The release build was more than 25% slower than the Milestone 4 warm baseline
because the opener integration introduced a cold native dependency path. It
matched the earlier cold Milestone 3 release baseline. After compilation, warm
`cargo check`, Clippy, and Rust tests completed in about 0.85, 1.84, and 0.22
seconds respectively.

The pinned Playwright browser revision was absent and was restored to the
project cache without installing system packages. Production builds and preview
tests remained sequential to avoid the previously measured shared-`dist` race.
Approximately 46 GiB of system memory remained available during the final
native smoke test. No OOM, heavy swapping, throttling, orphaned app-server, or
disk pressure was observed. GPU computation remained unused.

## Milestone 6A measurements

The native project core reused existing Rust/Tauri caches. Adding bundled
SQLite, UUIDv7 generation, and the native dialog plugin locked 22 packages. No
cache was deleted and no clean or release build was run solely for timing.

| Operation                               | Observed wall time | Approximate peak RSS | Result                                                            |
| --------------------------------------- | -----------------: | -------------------: | ----------------------------------------------------------------- |
| First dependency-expanded `cargo check` | about 14.2 seconds |        about 718 MiB | Reached six ordinary compile errors after resolving dependencies  |
| Warm corrected `cargo check`            |  about 1.5 seconds |        about 452 MiB | Passed                                                            |
| Initial 10-test project suite           |  about 5.5 seconds |       about 1.24 GiB | Passed                                                            |
| Expanded 18-test project suite          |  about 6.4 seconds |       about 1.24 GiB | Passed                                                            |
| Final 20-test project suite             |  about 5.0 seconds |       about 1.24 GiB | Passed                                                            |
| Warm Clippy, warnings denied            |  about 2.0 seconds |        about 495 MiB | Passed                                                            |
| Full locked Rust suite                  |  about 5.7 seconds |       about 1.24 GiB | Passed, 38 tests; 2 deliberate live probes ignored                |
| Final full non-browser repository gate  | about 25.8 seconds |        about 605 MiB | Passed, including 40 Rust tests; 2 deliberate live probes ignored |

Four Cargo workers preserved desktop responsiveness and were sufficient for
the small native graph. The test linker, not SQLite execution, produced the
peak memory use. No swap activity, OOM, throttling, material disk pressure, or
competing QuireForge build was observed. The RTX 3050 was correctly unused.

## Milestone 6B measurements

The project UI and integration checkpoint reused warm pnpm, Vite, Playwright,
Cargo, and Tauri caches. The Balanced profile retained four Cargo workers and
two Playwright workers. No dependency installation or clean build was needed.

| Operation                              |    Observed wall time | Approximate peak RSS | Result                                                                      |
| -------------------------------------- | --------------------: | -------------------: | --------------------------------------------------------------------------- |
| Desktop production build               |     about 2.1 seconds |        about 300 MiB | Passed, 108 modules                                                         |
| Desktop-only browser suite             |     about 5.7 seconds |        about 236 MiB | Passed, 6 tests                                                             |
| Unbundled native release build         |    about 67.0 seconds |       about 1.36 GiB | Passed                                                                      |
| Isolated native release launch         | Manual smoke interval | Low runtime pressure | Exact D-Bus identity and owner-only temporary metadata permissions verified |
| Final full non-browser repository gate |    about 30.0 seconds |       about 1.23 GiB | Passed, including 41 Rust tests; 2 deliberate live probes ignored           |
| Final combined browser gate            |     about 8.3 seconds |        about 251 MiB | Passed, 14 tests                                                            |

The release link remained the critical local command but stayed well within
available system memory. No swap growth, OOM, throttling, cache deletion, or
competing build was observed. GPU computation remained unnecessary; native
WebKit rendering used the normal graphics stack only.

## Milestone 7A measurements

The native conversation checkpoint reused warm Rust, Tauri, pnpm, Vite, Astro,
and browser caches. The Balanced profile retained four Cargo workers and two
Playwright workers. Reviewed protocol fixtures added no dependency and no clean
build was run.

| Operation                                                |    Observed wall time | Approximate peak RSS | Result                                                                                  |
| -------------------------------------------------------- | --------------------: | -------------------: | --------------------------------------------------------------------------------------- |
| Full experimental schema generation to temporary storage |    about 0.32 seconds |         about 45 MiB | Passed; temporary output removed after review                                           |
| Reviewed schema-fixture generation                       |    about 1.01 seconds |        about 141 MiB | Passed, 28 selected schemas; repeat run idempotent                                      |
| First corrected incremental `cargo check`                |    about 1.89 seconds |        about 538 MiB | Passed                                                                                  |
| Conversation-focused Rust suite                          | about 5.2–6.3 seconds |       about 1.24 GiB | Passed, 9 tests                                                                         |
| Final full non-browser repository gate                   |   about 27.81 seconds |        about 660 MiB | Passed, including 50 Rust tests; 2 deliberate live probes ignored                       |
| Final combined browser gate                              |    about 8.48 seconds |        about 251 MiB | Passed, 14 tests                                                                        |
| Warm unbundled native release build                      |   about 28.86 seconds |       about 1.45 GiB | Passed                                                                                  |
| Isolated native release launch                           | Manual smoke interval | Low runtime pressure | Exact D-Bus identity, schema v2 migration, owner-only metadata, and clean exit verified |

The security review added explicit shutdown-and-wait behavior for every spawned
conversation startup failure; a deterministic mock blocks until stdin closes
and proves its exit trap completes before the service returns. The release
build was less than half the Milestone 6B time because the relevant release
graph and linker cache were warm. Approximately 46 GiB remained available
before the full gates, with no competing build, swap growth, OOM, throttling,
or material disk pressure. The RTX 3050 was correctly unused.

## Milestone 7B measurements

The conversation UI reused warm pnpm, Vite, Playwright, Rust, and Tauri caches.
The Balanced profile retained four Cargo workers and two Playwright workers. No
dependencies changed, no clean build was run, and desktop and website builds
were kept sequential where they share generated output.

| Operation                             |     Observed wall time | Approximate peak RSS | Result                                                                   |
| ------------------------------------- | ---------------------: | -------------------: | ------------------------------------------------------------------------ |
| Frontend type check and lint          |      about 8.1 seconds |     Not instrumented | Passed                                                                   |
| Desktop component/integration suite   |     about 3.84 seconds |     Not instrumented | Passed, 42 tests                                                         |
| Desktop production build              |     about 2.37 seconds |        about 316 MiB | Passed, 112 modules                                                      |
| Desktop browser suite                 |     about 5.54 seconds |        about 244 MiB | Passed, 6 tests with two workers                                         |
| Full non-browser repository gate      |    about 27.48 seconds |        about 696 MiB | Passed, including 45 JavaScript and 50 Rust tests; 2 live probes ignored |
| Combined desktop/website browser gate |     about 8.33 seconds |        about 251 MiB | Passed, 14 tests                                                         |
| Warm unbundled native release build   |    about 28.35 seconds |       about 1.46 GiB | Passed                                                                   |
| Isolated native release launch        |         about 1 second | Low runtime pressure | D-Bus identity, owner-only metadata, and clean child state verified      |
| Desktop/mobile visual inspection      | Manual review interval | Low runtime pressure | Responsive layout and preview boundaries verified                        |

Approximately 45 GiB of system memory and 726 GiB of NVMe space remained
available before the milestone gate. No swap growth, OOM, throttling, material
disk pressure, dependency download, or competing QuireForge build was observed.
The RTX 3050 was not used because React, TypeScript, Vite, and the ordinary
Tauri validation path are CPU/system-memory workloads.

## Milestone 8A measurements

The native session-lifecycle checkpoint reused the warm Cargo target, registry,
pnpm, Vite, Astro, and Playwright caches. The Balanced profile retained four
Cargo workers and two Playwright workers. No dependency install, clean build,
cache deletion, or concurrent release/browser build was required.

| Operation                                      | Observed wall time | Approximate peak RSS | Result                                                                                |
| ---------------------------------------------- | -----------------: | -------------------: | ------------------------------------------------------------------------------------- |
| Reviewed Codex lifecycle schema generation     |       0.48 seconds |         about 45 MiB | Passed, 42 selected schemas total                                                     |
| Focused native lifecycle suite                 |  about 5.5 seconds |     Not instrumented | Passed, 6 lifecycle tests                                                             |
| Incremental native suite during implementation |   about 10 seconds |     Not instrumented | Passed, 57 tests; 2 live probes ignored                                               |
| Frontend type/lint/unit checkpoint             |  about 9.6 seconds |     Not instrumented | Passed, 48 tests                                                                      |
| Full non-browser repository gate               |      29.45 seconds |        about 692 MiB | Passed, including 51 JavaScript and 57 Rust tests; 2 live probes ignored              |
| Combined desktop/website browser gate          |       8.60 seconds |        about 251 MiB | Passed, 14 tests                                                                      |
| Warm unbundled native release build            |      25.13 seconds |       about 1.49 GiB | Passed                                                                                |
| Isolated native release launch                 |     under 1 second | Low runtime pressure | D-Bus identity, schema version 3, owner-only metadata, and clean child state verified |

Before the full gate, approximately 44 GiB of system memory and 726 GiB of
NVMe space were available at a load average below 1.3. Before the release build,
approximately 43 GiB remained available. Swap stayed at about 175 MiB with no
growth observed; the measured commands reported no swaps, OOM, throttling, or
material disk pressure. The first aggregate format invocation lacked Cargo in
its transient package-runner `PATH`; explicitly preserving the already-installed
Cargo directory fixed the harness without a system or repository configuration
change.

The release smoke check used a fresh isolated XDG tree, verified schema
migrations 1–3 and `0700`/`0600` application-data permissions, observed the
exact `io.github.codeframe78.QuireForge` D-Bus name, and confirmed that merely
launching the app starts no Codex app-server child. The temporary tree was then
removed. The RTX 3050 was correctly unused because schema generation, Rust,
SQLite, TypeScript, Vite, Astro, and Playwright are CPU/system-memory workloads
for this milestone.

## Milestone 8B measurements

The session-history checkpoint reused the warm Cargo target, registry, pnpm,
Vite, Astro, and Playwright caches. The Balanced profile retained four Cargo
workers and two Playwright workers. No dependency install, clean build, cache
deletion, database migration, or simultaneous release/browser build was
required.

| Operation                             |                           Observed wall time | Approximate peak RSS | Result                                                                                       |
| ------------------------------------- | -------------------------------------------: | -------------------: | -------------------------------------------------------------------------------------------- |
| Focused native lifecycle suite        | 5.32 seconds compile plus 0.22 seconds tests |     Not instrumented | Passed, 8 lifecycle/storage-filtered tests                                                   |
| Desktop component/integration suite   |                                 4.76 seconds |     Not instrumented | Passed, 54 tests across 11 files                                                             |
| Desktop production build              |                Vite phase about 0.13 seconds |     Not instrumented | Passed, 115 modules                                                                          |
| Full non-browser repository gate      |                                41.37 seconds |       about 1.24 GiB | Passed, including 57 JavaScript and 58 Rust tests; 2 live probes ignored                     |
| Combined desktop/website browser gate |                                 8.81 seconds |        about 265 MiB | Passed, 16 tests with two workers per package                                                |
| Warm unbundled native release build   |                                30.97 seconds |       about 1.48 GiB | Passed                                                                                       |
| Isolated native release launch        |                               under 1 second | Low runtime pressure | Exact D-Bus identity, schema version 3, `0700`/`0600` metadata, and no Codex child verified  |
| Desktop/mobile visual inspection      |                       Manual review interval | Low runtime pressure | Grouping, tabs, lifecycle controls, responsive stacking, and no horizontal overflow verified |

The full gate was about 40% slower than Milestone 8A's 29.45-second baseline.
It compiled the changed native lifecycle boundary, ran the expanded suites, and
reported 3,139 major page faults and about 1.24 GiB peak RSS, but zero swaps.
The browser gate stayed near baseline despite adding native-fixture session
coverage. The release build was about 23% slower than the prior 25.13-second
measurement and remained below the forecast-update threshold.

Approximately 44 GiB of RAM and 726 GiB of NVMe space remained available after
the gates at a load average near 1.4. Swap stayed at about 175 MiB with no
measured growth, OOM, thermal issue, or material disk pressure. The RTX 3050 was
correctly unused because Rust, SQLite, React, TypeScript, Vite, Astro, and
Playwright remained CPU/system-memory workloads.

## Current execution guidance

- Default to the Balanced profile and preserve desktop responsiveness.
- Preserve Cargo, target, pnpm, Vite, Astro, and browser caches.
- Use targeted frontend tests and `cargo check` during implementation.
- Run the locked full validation and browser suites at milestone gates.
- Avoid simultaneous cold Rust release builds and other heavy workloads unless
  a current memory/load check shows sufficient headroom.
- Complete a frontend production build before starting a preview/E2E server;
  those steps share `dist` and must not overlap.
- Continue using the existing two-worker Playwright configuration.
- Select a Cargo job count for each milestone after checking current load and
  task shape; do not persist a new limit without evidence.
- Do not install sccache, an alternative linker, CUDA, or new build tooling
  without a measured need, compatibility review, and separate approval.

The RTX 3050 is reserved for genuine GPU workloads such as WebGL/WebGPU,
shader, CUDA, ML, or hardware-rendering validation. Rust, Tauri, React,
TypeScript, Vite, and Astro compilation remains CPU/system-memory work.

## Milestone 9A measurements

The native approval/activity checkpoint reused the warm Cargo target, registry,
pnpm, Vite, Astro, and Playwright caches. The Balanced profile retained four
Cargo workers and two Playwright workers. No dependency install, clean build,
cache deletion, database migration, or simultaneous release/browser build was
required.

| Operation                                          |                                                     Observed wall time |                          Approximate peak RSS | Result                                                                                                                   |
| -------------------------------------------------- | ---------------------------------------------------------------------: | --------------------------------------------: | ------------------------------------------------------------------------------------------------------------------------ |
| Reviewed Codex approval/activity schema generation |                                                           1.04 seconds |                                 about 141 MiB | Passed, 52 selected schemas total                                                                                        |
| Focused app-server and conversation suites         | about 5.5 seconds incremental compile plus under 0.3 seconds per suite |                              Not instrumented | Passed, including 13 app-server and 19 conversation tests                                                                |
| Desktop component/integration suite                |                                                           4.76 seconds |                              Not instrumented | Passed, 56 tests across 11 files                                                                                         |
| Full non-browser repository gate                   |                 37.57 seconds initially; 28.83-second final warm rerun | about 1.25 GiB initially; about 765 MiB final | Passed, including 59 JavaScript and 68 Rust tests; 2 live probes ignored                                                 |
| Combined desktop/website browser gate              |                                                           9.93 seconds |                                 about 281 MiB | Passed 18 desktop/mobile checks, including native approval/activity accessibility and overflow presentation              |
| Warm unbundled native release build                |                                                          32.03 seconds |                                about 1.55 GiB | Passed, 115 frontend modules                                                                                             |
| Isolated native release launch                     |                                                         under 1 second |                          Low runtime pressure | Exact D-Bus identity, schema version 3, `0700`/`0600` metadata, and no Codex child verified                              |
| Desktop/mobile visual inspection                   |                                                 Manual review interval |                          Low runtime pressure | Waiting state, command detail, progress, and approval-request presentation remained readable without horizontal overflow |

Approximately 44 GiB of system memory and 726 GiB of NVMe space remained
available through the final gates. Swap stayed at about 175 MiB and every
measured gate reported zero swaps. No OOM, throttling, dependency download,
cache deletion, clean build, competing QuireForge build, or material disk
pressure occurred. The RTX 3050 was correctly unused because schema generation,
Rust, React, TypeScript, Vite, Astro, and Playwright were CPU/system-memory
workloads.

Line-boundary buffering and strict native projection added security coverage
without materially changing build pressure. The warm release link remains the
longest individual local command. Milestone 9B can keep the Balanced profile,
four Cargo workers, and two Playwright workers unless its fresh preflight finds
different system load or memory availability.

## Milestone 9B measurements

The selectable activity/approval checkpoint reused the warm Cargo target,
registry, pnpm, Vite, Astro, and Playwright caches. The Balanced profile kept
four Cargo workers and two Playwright workers. No dependency installation,
clean build, schema generation, database migration, or native protocol change
was required.

| Operation                                     |                           Observed wall time | Approximate peak RSS | Result                                                                                                 |
| --------------------------------------------- | -------------------------------------------: | -------------------: | ------------------------------------------------------------------------------------------------------ |
| Parallel desktop type/lint/unit checkpoint    |                          9.1 seconds overall |     Not instrumented | Passed type checking, lint, and 61 tests across 11 files                                               |
| Desktop production build                      | 2.1 seconds overall; Vite phase 0.13 seconds |     Not instrumented | Passed, 115 modules                                                                                    |
| Focused desktop/mobile approval browser check |                      3.5 seconds after build | Low runtime pressure | Passed 2 checks with axe and overflow analysis                                                         |
| Full non-browser repository gate              |                                36.70 seconds |       about 1.25 GiB | Passed 64 JavaScript and 68 Rust tests; 2 live probes ignored                                          |
| Combined desktop/website browser gate         |                                 9.99 seconds |        about 283 MiB | Passed all 18 desktop/mobile checks                                                                    |
| Warm unbundled native release build           |                                30.73 seconds |       about 1.55 GiB | Passed, including a 26.77-second release compile/link                                                  |
| Isolated native release launch                |                      3-second bounded launch | Low runtime pressure | Schema migrations 1–3, owner-only metadata, and no remaining child verified                            |
| Desktop rendered-state inspection             |                     3.5-second focused trace | Low runtime pressure | Waiting approval, expanded activity/output, decision transition, and completed state remained readable |

Approximately 44 GiB of RAM remained available around the gates. Swap stayed
near 175 MiB, and timed commands reported zero swaps. The first aggregate gate
ended after 16.54 seconds when its transient environment omitted
the user-local Cargo bin directory from `PATH`; the rerun explicitly preserved the
already-installed tool path and passed without changing persistent
configuration. There was no OOM, throttling, dependency download, cache
deletion, clean build, or competing QuireForge build.

The feature remained frontend-bound: Rust and the reviewed Codex schema subset
were unchanged. CPU, system RAM, NVMe, and warm caches were the relevant
resources. The RTX 3050 remained unused because neither React aggregation nor
the browser accessibility checks were a genuine GPU-compute workload.

## Milestone 10A measurements

The read-only Git-review checkpoint reused warm Cargo, Git, pnpm, Vite, Astro,
and Playwright caches. No dependency installation, clean build, database
migration, cache deletion, or local linker/tool change was required. The
Balanced profile kept four Cargo workers and two Playwright workers.

| Operation                                          |                                        Observed wall time | Approximate peak RSS | Result                                                                                               |
| -------------------------------------------------- | --------------------------------------------------------: | -------------------: | ---------------------------------------------------------------------------------------------------- |
| Focused native Git parser/command tests            |       about 2.5–4.4 seconds including incremental compile |     Not instrumented | Passed shared fixtures, normalized parsing, binary/deceptive paths, and temporary-repository review  |
| Desktop TypeScript and component/integration suite |                     about 9.5 seconds combined check/test |     Not instrumented | Passed 67 tests across 13 files                                                                      |
| Desktop production build                           |   about 3.2 seconds overall; Vite phase 0.12–0.14 seconds |     Not instrumented | Passed, 119 modules                                                                                  |
| Focused desktop/mobile Git browser check           | 6.55 seconds including build on the first corrected rerun | Low runtime pressure | Passed 2 checks with axe and overflow analysis                                                       |
| Complete non-browser repository gate               |                                             31.72 seconds |        about 804 MiB | Passed 70 JavaScript and 78 Rust tests; 2 live probes ignored                                        |
| Combined desktop/website browser gate              |                                             11.64 seconds |        about 315 MiB | Passed all 20 desktop/mobile checks                                                                  |
| Warm unbundled native release build                |                                             31.18 seconds |       about 1.61 GiB | Passed, including 27.16-second release compile/link                                                  |
| Isolated native release launch                     |                                           about 2 seconds | Low runtime pressure | Exact D-Bus identity, schema version 3, `0700`/`0600` metadata, and no Codex child verified          |
| Desktop/mobile visual inspection                   |                            Focused rendered-state capture | Low runtime pressure | Two-column desktop and stacked mobile status/diff layouts remained legible after contrast correction |

The complete gate was about 14% faster than Milestone 9B's 36.70-second
baseline, the browser gate about 17% slower than 9.99 seconds, and the release
build about 1% slower than 30.73 seconds. None crossed the 25% forecast-update
threshold. The first browser fixture exposed a light-theme diff contrast issue;
the corrected focused and complete checks passed. Aggregate validation also
found and safely stopped at effect-scheduling, rustfmt, and Clippy test-literal
issues before the final passing gate.

The final native regression also established that porcelain-v2 NUL-delimited
paths are repository-root-relative when the attached project is a subdirectory.
QuireForge now derives and strips the exact verified attachment prefix rather
than relying on Git's relative-path display setting, while the pathspec still
excludes repository changes outside the attachment.

Approximately 43 GiB of RAM and 726 GiB of NVMe space remained available.
Swap stayed near 175 MiB, every timed gate reported zero swaps, and no OOM,
throttling, dependency download, cache deletion, clean build, competing project
build, user-repository Git mutation, or material disk pressure occurred. Git,
Rust, React, TypeScript, Vite, Astro, and Playwright remained CPU/system-memory
workloads; the RTX 3050 was correctly unused.

## Milestone 10B measurements

The reviewed Git-mutation checkpoint reused warm Cargo, Git, pnpm, Vite, Astro,
and Playwright caches. No dependency installation, clean build, database
migration, cache deletion, linker/tool change, or GPU workload was required.
The Balanced profile kept four Cargo workers and two Playwright workers.

| Operation                                          |                                 Observed wall time | Approximate peak RSS | Result                                                                                                               |
| -------------------------------------------------- | -------------------------------------------------: | -------------------: | -------------------------------------------------------------------------------------------------------------------- |
| Focused native mutation tests                      | about 3–5 seconds including incremental recompiles |       about 1.11 GiB | Passed strict fixtures/tokens, lock ownership, exact stage/unstage, revert/recovery, commit, scope, and secret cases |
| Desktop type/lint/component and bridge checkpoints |                       about 12–15 seconds combined |     Not instrumented | Passed 70 tests across 13 files plus strict type and lint checks                                                     |
| Desktop production build                           |       about 2–3 seconds overall; Vite phase 0.13 s |     Not instrumented | Passed, 121 modules                                                                                                  |
| Complete non-browser repository gate               |                                      43.16 seconds |       about 1.33 GiB | Passed 73 JavaScript and 86 Rust tests; 2 live probes ignored                                                        |
| Combined desktop/website browser gate              |                                      12.31 seconds |        about 318 MiB | Passed all 20 desktop/mobile checks with axe and overflow analysis                                                   |
| Warm unbundled native release build                |                                      34.56 seconds |       about 1.77 GiB | Passed, including 30.45-second release compile/link                                                                  |
| Isolated native release launch                     |                            3-second bounded launch | Low runtime pressure | Schema migrations 1–3, final `0700`/`0600` metadata, and no remaining QuireForge/Codex child                         |
| Desktop/mobile rendered-state inspection           |                         Focused full-page captures | Low runtime pressure | Confirmation target and applied mutation state remained legible in two-column desktop and stacked mobile layouts     |
| Post-merge index-lock correction gate              |                                      11.35 seconds |     Not instrumented | Focused replacement-lock regression, all 88 native tests, rustfmt, and Clippy passed                                 |

The complete gate was 36% slower than 10A's 31.72-second baseline and crossed
the 25% measurement threshold. The changed assumption was that the final native
security edit forced an incremental crate/test link inside the aggregate gate;
the 7.29-second Rust test compile accounted for much of the difference. It did
not reveal more product scope, exceed the calibrated 20–45-minute command
allowance, or justify changing the Balanced profile. The combined browser gate
was about 7% slower and the release build about 11% slower than 10A, below the
update threshold.

The first full gate stopped in 0.73 seconds when the repository validator found
an intentionally secret-shaped test literal. Building the synthetic value from
non-secret fragments kept the scanner test effective without adding an
exception. The rerun passed without a clean build or cache reset.

PR #20's hosted desktop gate passed in 9 minutes 41 seconds. The identical merge
commit's first `main` desktop job failed after 4 minutes 16 seconds when its
ephemeral filesystem immediately reused the unlinked lock inode, exposing a
real cleanup race that the previous runner did not reproduce. Keeping the
original lock file handle open until `IndexLock` cleanup prevents inode reuse;
the focused regression and complete local native gate passed before the
corrective publication. This added hosted waiting and diagnosis but no product
scope, dependency, clean build, or execution-profile change.

Approximately 42 GiB RAM and 725 GiB NVMe remained available after the gates.
Swap stayed near 176 MiB, all timed commands reported zero swaps, and there was
no OOM, throttling, dependency download, cache deletion, clean build, competing
project build, user-repository mutation, or material disk pressure. Git, Rust,
React, TypeScript, Vite, Astro, and Playwright remained CPU/system-memory
workloads. The RTX 3050 was correctly unused.

## Milestone 11A measurements

The managed-worktree checkpoint reused warm Cargo, Git, pnpm, Vite, Astro, and
Playwright caches. It added no dependency, clean build, cache reset,
linker/tool, driver, CUDA, swap, or zram change. The Balanced profile retained
four Cargo workers and two Playwright workers; Git fixture operations remained
short, serialized, CPU/storage work.

| Operation                                 |                                    Observed wall time | Approximate peak RSS | Result                                                                                                      |
| ----------------------------------------- | ----------------------------------------------------: | -------------------: | ----------------------------------------------------------------------------------------------------------- |
| Focused native worktree suite             | about 7–10 seconds including incremental compile/link |     Not instrumented | Passed inventory, source/subdirectory, create/attach, stale HEAD, token, hook/filter, and recovery fixtures |
| Desktop type/component/bridge suite       |            about 6–10 seconds per combined checkpoint |     Not instrumented | Passed 79 tests across 15 files                                                                             |
| Complete non-browser repository gate      |                                         45.99 seconds |       about 1.27 GiB | Passed 82 JavaScript tests and 95 Rust tests; 2 deliberate live probes ignored                              |
| Combined desktop/website browser gate     |                                         14.59 seconds |        about 337 MiB | Passed all 22 desktop/mobile checks with axe and overflow analysis                                          |
| Warm unbundled native release build       |                                         33.98 seconds |       about 1.73 GiB | Passed, including a 29.93-second release compile/link                                                       |
| Isolated native release launch            |                                       about 2 seconds | Low runtime pressure | Exact D-Bus identity, schema migrations 1–4, `0700`/`0600` metadata, and no remaining app/Codex process     |
| Desktop/mobile worktree visual inspection |                        Focused rendered-state capture | Low runtime pressure | Create/attach controls, confirmation, source/external inventory, and stacked mobile layout remained legible |

The complete gate was about 7% slower than Milestone 10B's 43.16-second
baseline, the combined browser gate about 19% slower than 12.31 seconds after
adding two worktree checks, and the release build effectively unchanged from
34.56 seconds. None crossed the 25% forecast-update threshold.

One aggregate attempt stopped after 18.04 seconds at rustfmt and the next after
38.63 seconds at a Clippy-only boolean assertion. Both reruns reused unchanged
caches. The first focused browser attempt served the preceding production
bundle; rebuilding once exposed one old unscoped Git text locator after the new
worktree section introduced a second branch label. The corrected combined gate
passed without a product change or cache reset.

Approximately 42 GiB RAM and 725 GiB NVMe remained available. Swap stayed near
187 MiB, all timed gates reported zero swaps, and there was no OOM, throttling,
dependency download, clean build, competing project build, user-repository
mutation, live model call, or GPU-compute workload. The RTX 3050 was correctly
unused.

## Milestone 11B measurements

The bounded-parallel checkpoint reused warm Cargo, Git, pnpm, Vite, Astro, and
Playwright caches. It added no dependency, schema migration, clean build, cache
reset, linker/tool, driver, CUDA, swap, or zram change. The Balanced profile
retained four Cargo workers and two Playwright workers; the RTX 3050 remained
irrelevant to Rust, TypeScript, browser, and process-fixture work.

| Operation                                  |                                              Observed wall time | Approximate peak RSS | Result                                                                                                                         |
| ------------------------------------------ | --------------------------------------------------------------: | -------------------: | ------------------------------------------------------------------------------------------------------------------------------ |
| Focused native conversation suite          | 0.46 seconds final warm; about 7.6 seconds with earlier compile |        about 106 MiB | Passed 23 conversation/lifecycle/presentation tests, including exact interruption, provisional/four-task capacity, and reaping |
| Desktop component/contract suite           |                                               about 6.5 seconds |     Not instrumented | Passed 83 tests across 15 files                                                                                                |
| Complete non-browser repository gate       |                                                   42.19 seconds |       about 1.27 GiB | Passed 86 JavaScript tests and 101 Rust tests; 2 deliberate live probes ignored                                                |
| Combined desktop/website browser gate      |                                                   19.29 seconds |        about 372 MiB | Passed all 24 desktop/mobile checks with expanded activity, axe, and overflow analysis                                         |
| Warm unbundled native release build        |                                                   35.73 seconds |       about 1.73 GiB | Passed, including a 31.53-second release compile/link                                                                          |
| Isolated native release launch             |                                                 about 3 seconds | Low runtime pressure | Schema migrations 1–4, `0700`/`0600` metadata, no task start, and no remaining QuireForge process                              |
| Desktop parallel-monitor visual inspection |    Focused rendered-state capture plus a 4.6-second browser run | Low runtime pressure | Approval/conflict monitor and expanded normalized live command detail were legible                                             |

The complete gate was about 8% faster than Milestone 11A's 45.99-second
baseline, and the release build about 5% slower than 33.98 seconds. The final
browser gate was about 32% slower than the 14.59-second baseline after adding
two parallel-monitor checks and expanding live activity in both viewports. It
crossed the dynamic-update threshold, so 19–20 seconds replaces the old warm
browser baseline; it did not reveal additional scope or alter the overall
milestone forecast.

The first focused browser run correctly exposed two light-theme contrast
failures in the new conflict text and an existing approval button when rendered
in the new scenario. CSS corrections passed the desktop/mobile rerun. A
targeted stale-poll test also preserved the existing per-conversation action
generation after the multi-task state refactor. These reruns reused warm output
and required no cache deletion.

Timed gates reported zero swaps, and the largest observed command used about
1.73 GiB RSS. There was no OOM, throttling, dependency download, clean build,
competing project build, user-repository mutation, live model call, or GPU-
compute workload. Temporary launch and visual-capture files were removed after
inspection; project and dependency caches were preserved.

## Milestone 11C measurements

The safe-cleanup checkpoint reused warm Cargo, Git, pnpm, Vite, Astro, and
Playwright caches. It added no dependency, schema migration, clean build, cache
reset, linker/tool, driver, CUDA, swap, or zram change. The Balanced profile
retained four Cargo workers and two Playwright workers; filesystem-sensitive
Git fixtures stayed serialized. The RTX 3050 was not applicable.

| Operation                                |                                   Observed wall time | Approximate peak RSS | Result                                                                                                                              |
| ---------------------------------------- | ---------------------------------------------------: | -------------------: | ----------------------------------------------------------------------------------------------------------------------------------- |
| Focused native worktree suite            | about 7.6 seconds including incremental compile/link |     Not instrumented | Passed 19 worktree tests plus matched project coverage, including recovery, removal, races, filters, partial failure, and ownership |
| Desktop component/contract suite         |                                    about 6.4 seconds |     Not instrumented | Passed 84 tests across 15 files                                                                                                     |
| Complete non-browser repository gate     |                                        33.38 seconds |        about 891 MiB | Passed 87 JavaScript tests and 108 Rust tests; 106 Rust tests passed and 2 deliberate live probes were ignored                      |
| Combined desktop/website browser gate    |                                        17.12 seconds |        about 370 MiB | Passed all 24 desktop/mobile checks with recovery/cleanup, axe, and overflow analysis                                               |
| Warm unbundled native release build      |                                        31.31 seconds |       about 1.98 GiB | Passed, including a 27.20-second optimized compile/link                                                                             |
| Isolated native release launch           |                                      about 3 seconds | Low runtime pressure | Private isolated metadata, no task start, expected timeout, and no remaining QuireForge process                                     |
| Desktop/mobile cleanup visual inspection |  Focused trace capture plus a 4.2-second browser run | Low runtime pressure | Destructive hierarchy, branch-preservation copy, recovery separation, controls, and stacked mobile layout were legible              |

The final complete gate was about 21% faster than Milestone 11B's 42.19-second
baseline, the browser gate about 11% faster than 19.29 seconds, and the release
build about 12% faster than 35.73 seconds. Release peak RSS increased by about
15% but remained below 2 GiB with ample headroom. No measurement crossed the
25% dynamic forecast-update threshold.

One aggregate attempt stopped after 18.07 seconds because the non-interactive
command environment omitted Cargo from `PATH`; the corrected, command-local
environment passed without changing machine configuration. An early focused
browser attempt served a stale desktop bundle, confirming that production build
and preview validation must remain sequential. Native testing also established
that `git worktree remove` performs its own clean-status conversion, so the
fixed remove command must neutralize configured clean, smudge, and process
filters as well as the explicit status check. The adversarial fixture proves
that no repository-controlled filter or hook executes.

Approximately 40 GiB RAM and 725 GiB NVMe remained available before the final
gates. Timed commands reported zero swaps; there was no OOM, throttling,
dependency download, clean build, cache reset, competing project build,
user-repository mutation, live model call, or GPU-compute workload. Temporary
launch and visual-capture files were removed after inspection; project and
dependency caches were preserved.

## Milestone 12 measurements

The integrated-terminal milestone uses the Balanced profile with four Cargo
workers and two Playwright workers. It preserves the warm Cargo, pnpm, Vite,
Astro, and Playwright caches. The first locked dependency operation added
`portable-pty`, base64/libc support, and stable xterm packages without changing
the package manager, linker, compiler, swap, zram, driver, or GPU configuration.

| Operation                                     | Observed wall time |                                           Approximate peak RSS | Result                                                                                                                |
| --------------------------------------------- | -----------------: | -------------------------------------------------------------: | --------------------------------------------------------------------------------------------------------------------- |
| Warm desktop TypeScript preflight             |       1.66 seconds |                                                  about 265 MiB | Passed before implementation                                                                                          |
| Warm native `cargo check` preflight           |        0.30 second |                                                  about 225 MiB | Passed before implementation                                                                                          |
| Locked dependency installation                |       6.38 seconds |                                                 about 1.04 GiB | Added only reviewed locked terminal dependencies; cache preserved                                                     |
| Focused terminal native suite after hardening |       6.40 seconds |                 Not separately instrumented on the final rerun | Passed 9 real/local PTY and contract tests; incremental compile dominated                                             |
| Complete desktop type/lint/unit pass          | about 17.7 seconds | Typecheck subprocess about 142 MiB; aggregate not instrumented | Passed 97 tests across 18 files plus typecheck and lint                                                               |
| Complete repository gate                      |      39.01 seconds |                                                  about 929 MiB | Passed 100 JavaScript tests and 119 Rust tests; 117 passed and 2 deliberate live probes were ignored                  |
| Combined browser gate                         |      19.24 seconds |                                                  about 393 MiB | Passed all 26 desktop/mobile checks, including the mounted xterm tab, axe, and overflow analysis                      |
| Warm unbundled native release build           |      88.76 seconds |                                                 about 1.82 GiB | Passed after the first optimized compile of newly added PTY dependency edges                                          |
| Isolated native PTY launch and visual review  |    about 6 seconds |                                           Low runtime pressure | Private metadata at mode `0600`, no GVFS noise, no remaining process; desktop/mobile terminal composition was legible |

One focused native attempt failed after about 5.8 seconds at roughly 1.31 GiB
RSS because newly added tests omitted three test-module imports. The corrected
suite passed; this was a source-level test wiring issue, not OOM, swapping, PTY
failure, or a product runtime defect. A later security review added native
pre-decode input bounds and canonical UUIDv7 checks before the final focused
rerun. The RTX 3050 remains inapplicable: the selected DOM renderer and all
ordinary Rust/TypeScript work use CPU, system RAM, and storage.

The release build was about 184% slower than Milestone 11C's 31.31-second warm
baseline and crossed the dynamic-update threshold. The changed assumption was
release-cache warmth after introducing native PTY and GTK dependency edges; the
1.82 GiB peak remained below the earlier 1.98 GiB peak, and the operation still
fit inside the approved 35–80-minute aggregate command allowance. The overall
milestone forecast therefore remained unchanged with medium confidence.

Final validation reused the warmed dependency graph: the complete repository
gate took 39.01 seconds at about 929 MiB and the browser gate took 19.24 seconds
at about 393 MiB. All timed final gates reported zero swaps. The isolated
release launch emitted no earlier GVFS warning, created only private test state,
and left no QuireForge process. Controlled desktop/mobile fixture inspection
showed a legible project selector, Linux-account warning, tab state, close
affordance, cursor, terminal surface, and stacked mobile layout with no
horizontal overflow. Temporary launch and capture data were removed.

Publication used fresh GitHub-hosted runners. Pull-request workflow
`29796398390` completed its source, website, and desktop jobs in 6 seconds, 1
minute 56 seconds, and 9 minutes 46 seconds respectively. The corresponding
`main` workflow `29796847052` completed them in 5 seconds, 1 minute 50 seconds,
and 9 minutes 33 seconds. pnpm dependency caching was already enabled, while
Cargo build outputs and Playwright browser downloads were not restored between
hosts. Those two cold-host costs are candidates for a separately reviewed cache
change; they are not evidence of local cache or resource pressure.

## Milestone 13A measurements

The protocol/contract checkpoint used the Balanced profile with four Cargo
workers. It retained warm Cargo, pnpm, Vite, Astro, and dependency caches and
added no package, schema migration, clean build, cache reset, linker, driver,
CUDA, swap, or zram change. Generated schema work and ordinary Rust/TypeScript
validation are CPU/system-memory tasks; the RTX 3050 was correctly unused.

| Operation | Observed wall time | Approximate peak RSS | Result |
| --- | ---: | ---: | --- |
| Local 0.145.0 schema inventory generation | 0.36 second | about 45 MiB | Generated the full temporary protocol inventory for read-only inspection |
| Reviewed 0.145.0 fixture refresh | 0.42 second | about 46 MiB | Generated and hashed 95 selected schemas while retaining 0.144.6 |
| Contract frontend suite | 8.65 seconds | about 264 MiB | Passed 100 desktop tests across 19 files, including 3 integration contract tests |
| Contract native suite | 11.72 seconds | about 1.44 GiB | Passed the shared Rust contract after incremental compilation |
| Repository/type/lint/Clippy preflight | 17.36 seconds | about 937 MiB | Passed with warnings denied |
| Final complete non-browser repository gate | 37.77 seconds | about 935 MiB | Passed 103 JavaScript tests and 121 Rust tests; 119 Rust tests passed and 2 deliberate live probes were ignored |
| Final warm unbundled native release build | 38.41 seconds | about 1.81 GiB | Passed, including a 34.22-second optimized compile/link |

All final timed gates reported zero swaps. About 43 GiB RAM and 720 GiB NVMe
were available before the complete gate. There was no OOM, throttling,
dependency download, personal integration read, user-repository mutation, live
model call, browser run, or GPU workload. Browser, native launch, and visual
checks were deliberately omitted because Milestone 13A introduces no
user-facing or runtime integration surface.
