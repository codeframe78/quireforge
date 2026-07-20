# Local Build Performance

Status: generalized baseline captured from required Milestones 3–6 work on
2026-07-19. These measurements guide local forecasts; they are not release
performance claims or a supported-hardware baseline.

## Generalized development profile

| Resource | Observed profile |
|---|---|
| Operating system | Ubuntu 26.04 LTS, Linux 7.0, x86_64 |
| CPU | AMD Ryzen 5 5600G class, 6 physical cores / 12 logical processors |
| System memory | Approximately 61 GiB total; 47 GiB available at audit |
| Swap | 8 GiB file-backed swap; about 176 MiB used; no zram configured |
| Workspace storage | NVMe-backed ext4; approximately 733 GiB available |
| GPU | NVIDIA GeForce RTX 3050, 8 GiB VRAM; driver available; CUDA toolkit absent |
| Rust | rustc/Cargo 1.97.1 with rustfmt and Clippy |
| JavaScript | Node 22.22.1 and pnpm 11.15.0 |
| Native tools | GCC 15.2.0 and pkg-config 2.5.1 |
| Optional caches/linkers | Cargo and pnpm caches present; no sccache, ccache, mold, or LLVM lld |

The audit recorded no competing QuireForge build and a load average below two.
It did not collect hardware serial numbers, network addresses, private mount
names, credentials, or unrelated process details.

## Milestone 3 measurements

Measurements came from commands already required to implement or verify the
milestone. Caches were preserved; no clean build was run for benchmarking.

| Operation | Observed wall time | Cache state | Result |
|---|---:|---|---|
| Workspace dependency installation after manifest changes | about 5 seconds | Package cache populated; lockfile updated | Passed |
| First successful native `cargo check` | about 37 seconds | Partially populated after dependency download | Passed |
| First Rust test-profile build and tests | about 44 seconds | Cold test profile | Passed |
| First Tauri unbundled release build | about 1 minute 18 seconds | Cold release profile | Passed |
| Warm `cargo check` | about 0.4 seconds | Warm | Passed |
| Warm Clippy with warnings denied | about 0.5 seconds | Warm | Passed |
| Warm Rust tests | about 4 seconds | Warm | Passed |
| Desktop Vite production transform/build | about 0.12–0.15 seconds | Warm frontend cache | Passed |
| Astro static build | about 0.35–0.37 seconds | Warm | Passed, 15 pages |
| Combined desktop and website browser suites | about 8 seconds | Browser installed; two workers per package | Passed, 14 tests |

Peak memory was not instrumented during Milestone 3. The post-build audit showed
substantial available memory, minimal pre-existing swap use, low system load,
and no evidence of memory pressure. GPU computation was not used.

## Milestone 4 measurements

The adapter work reused the existing Cargo and pnpm caches. Enabling Tokio
process/I/O features added three locked transitive packages; no clean build was
run. Resource values below come from commands already required by the milestone.

| Operation | Observed wall time | Approximate peak RSS | Result |
|---|---:|---:|---|
| Full experimental app-server schema generation to a temporary directory | about 0.33 seconds | Not instrumented | Passed, 337 files / about 3.16 MB; temporary output removed |
| First `cargo check` after Tokio feature additions | about 5.9 seconds | Not instrumented | Passed |
| First Rust test-profile compile and adapter tests | about 15.1 seconds | Not instrumented | Passed |
| Non-billable live CLI/app-server compatibility probe | about 0.32 seconds | Not instrumented | Passed; no child remained |
| Warm Clippy with warnings denied | about 0.8–1.0 seconds | Not instrumented | Passed |
| Successful unbundled release build after adapter changes | about 19.5 seconds | about 1.25 GiB | Passed |
| Full non-browser `pnpm validate` gate | about 23.1–29.7 seconds | about 584 MiB–1.02 GiB | Passed |
| Combined website and desktop browser suites | about 9.8 seconds | about 253 MiB | Passed, 14 tests |
| Desktop Vite production build | about 0.12–0.13 seconds | Included above | Passed |

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

| Operation | Observed wall time | Approximate peak RSS | Result |
|---|---:|---:|---|
| Stable authentication-schema calibration | about 0.43 seconds | about 45 MiB | Passed, 267 generated files / about 2.72 MB inspected in temporary storage |
| Reviewed authentication-schema generation | about 0.98 seconds | about 141 MiB | Passed, 13 selected schemas; repeat run was idempotent |
| First `cargo check` after opener dependencies | about 39.4 seconds | about 1.17 GiB | Passed; 39 locked transitive packages resolved |
| First authentication test-profile build and tests | about 55.6 seconds | about 1.36 GiB | Passed |
| Read-only live `account/read` compatibility probe | about 0.57 seconds | about 104 MiB | Passed; no private account data printed and no child remained |
| Unbundled Tauri release build | about 1 minute 18 seconds | about 1.37 GiB | Passed |
| Full non-browser `pnpm validate` gate | about 24.3–25.7 seconds | about 601–604 MiB | Passed, including 21 unit/integration tests and 20 Rust tests with 2 live tests ignored |
| Desktop browser suite after cache restoration | about 5.3 seconds | about 239 MiB | Passed, 6 tests with two workers |
| Final combined desktop and website browser suites | about 8.4 seconds | about 254 MiB | Passed, 14 tests with two workers per package |
| Desktop Vite production build | about 0.12–0.14 seconds | Included above | Passed |

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

| Operation | Observed wall time | Approximate peak RSS | Result |
|---|---:|---:|---|
| First dependency-expanded `cargo check` | about 14.2 seconds | about 718 MiB | Reached six ordinary compile errors after resolving dependencies |
| Warm corrected `cargo check` | about 1.5 seconds | about 452 MiB | Passed |
| Initial 10-test project suite | about 5.5 seconds | about 1.24 GiB | Passed |
| Expanded 18-test project suite | about 6.4 seconds | about 1.24 GiB | Passed |
| Final 20-test project suite | about 5.0 seconds | about 1.24 GiB | Passed |
| Warm Clippy, warnings denied | about 2.0 seconds | about 495 MiB | Passed |
| Full locked Rust suite | about 5.7 seconds | about 1.24 GiB | Passed, 38 tests; 2 deliberate live probes ignored |
| Final full non-browser repository gate | about 25.8 seconds | about 605 MiB | Passed, including 40 Rust tests; 2 deliberate live probes ignored |

Four Cargo workers preserved desktop responsiveness and were sufficient for
the small native graph. The test linker, not SQLite execution, produced the
peak memory use. No swap activity, OOM, throttling, material disk pressure, or
competing QuireForge build was observed. The RTX 3050 was correctly unused.

## Milestone 6B measurements

The project UI and integration checkpoint reused warm pnpm, Vite, Playwright,
Cargo, and Tauri caches. The Balanced profile retained four Cargo workers and
two Playwright workers. No dependency installation or clean build was needed.

| Operation | Observed wall time | Approximate peak RSS | Result |
|---|---:|---:|---|
| Desktop production build | about 2.1 seconds | about 300 MiB | Passed, 108 modules |
| Desktop-only browser suite | about 5.7 seconds | about 236 MiB | Passed, 6 tests |
| Unbundled native release build | about 67.0 seconds | about 1.36 GiB | Passed |
| Isolated native release launch | Manual smoke interval | Low runtime pressure | Exact D-Bus identity and owner-only temporary metadata permissions verified |
| Final full non-browser repository gate | about 30.0 seconds | about 1.23 GiB | Passed, including 41 Rust tests; 2 deliberate live probes ignored |
| Final combined browser gate | about 8.3 seconds | about 251 MiB | Passed, 14 tests |

The release link remained the critical local command but stayed well within
available system memory. No swap growth, OOM, throttling, cache deletion, or
competing build was observed. GPU computation remained unnecessary; native
WebKit rendering used the normal graphics stack only.

## Milestone 7A measurements

The native conversation checkpoint reused warm Rust, Tauri, pnpm, Vite, Astro,
and browser caches. The Balanced profile retained four Cargo workers and two
Playwright workers. Reviewed protocol fixtures added no dependency and no clean
build was run.

| Operation | Observed wall time | Approximate peak RSS | Result |
|---|---:|---:|---|
| Full experimental schema generation to temporary storage | about 0.32 seconds | about 45 MiB | Passed; temporary output removed after review |
| Reviewed schema-fixture generation | about 1.01 seconds | about 141 MiB | Passed, 28 selected schemas; repeat run idempotent |
| First corrected incremental `cargo check` | about 1.89 seconds | about 538 MiB | Passed |
| Conversation-focused Rust suite | about 5.2–6.3 seconds | about 1.24 GiB | Passed, 9 tests |
| Final full non-browser repository gate | about 27.81 seconds | about 660 MiB | Passed, including 50 Rust tests; 2 deliberate live probes ignored |
| Final combined browser gate | about 8.48 seconds | about 251 MiB | Passed, 14 tests |
| Warm unbundled native release build | about 28.86 seconds | about 1.45 GiB | Passed |
| Isolated native release launch | Manual smoke interval | Low runtime pressure | Exact D-Bus identity, schema v2 migration, owner-only metadata, and clean exit verified |

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

| Operation | Observed wall time | Approximate peak RSS | Result |
|---|---:|---:|---|
| Frontend type check and lint | about 8.1 seconds | Not instrumented | Passed |
| Desktop component/integration suite | about 3.84 seconds | Not instrumented | Passed, 42 tests |
| Desktop production build | about 2.37 seconds | about 316 MiB | Passed, 112 modules |
| Desktop browser suite | about 5.54 seconds | about 244 MiB | Passed, 6 tests with two workers |
| Full non-browser repository gate | about 27.48 seconds | about 696 MiB | Passed, including 45 JavaScript and 50 Rust tests; 2 live probes ignored |
| Combined desktop/website browser gate | about 8.33 seconds | about 251 MiB | Passed, 14 tests |
| Warm unbundled native release build | about 28.35 seconds | about 1.46 GiB | Passed |
| Isolated native release launch | about 1 second | Low runtime pressure | D-Bus identity, owner-only metadata, and clean child state verified |
| Desktop/mobile visual inspection | Manual review interval | Low runtime pressure | Responsive layout and preview boundaries verified |

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

| Operation | Observed wall time | Approximate peak RSS | Result |
|---|---:|---:|---|
| Reviewed Codex lifecycle schema generation | 0.48 seconds | about 45 MiB | Passed, 42 selected schemas total |
| Focused native lifecycle suite | about 5.5 seconds | Not instrumented | Passed, 6 lifecycle tests |
| Incremental native suite during implementation | about 10 seconds | Not instrumented | Passed, 57 tests; 2 live probes ignored |
| Frontend type/lint/unit checkpoint | about 9.6 seconds | Not instrumented | Passed, 48 tests |
| Full non-browser repository gate | 29.45 seconds | about 692 MiB | Passed, including 51 JavaScript and 57 Rust tests; 2 live probes ignored |
| Combined desktop/website browser gate | 8.60 seconds | about 251 MiB | Passed, 14 tests |
| Warm unbundled native release build | 25.13 seconds | about 1.49 GiB | Passed |
| Isolated native release launch | under 1 second | Low runtime pressure | D-Bus identity, schema version 3, owner-only metadata, and clean child state verified |

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

| Operation | Observed wall time | Approximate peak RSS | Result |
|---|---:|---:|---|
| Focused native lifecycle suite | 5.32 seconds compile plus 0.22 seconds tests | Not instrumented | Passed, 8 lifecycle/storage-filtered tests |
| Desktop component/integration suite | 4.76 seconds | Not instrumented | Passed, 54 tests across 11 files |
| Desktop production build | Vite phase about 0.13 seconds | Not instrumented | Passed, 115 modules |
| Full non-browser repository gate | 41.37 seconds | about 1.24 GiB | Passed, including 57 JavaScript and 58 Rust tests; 2 live probes ignored |
| Combined desktop/website browser gate | 8.81 seconds | about 265 MiB | Passed, 16 tests with two workers per package |
| Warm unbundled native release build | 30.97 seconds | about 1.48 GiB | Passed |
| Isolated native release launch | under 1 second | Low runtime pressure | Exact D-Bus identity, schema version 3, `0700`/`0600` metadata, and no Codex child verified |
| Desktop/mobile visual inspection | Manual review interval | Low runtime pressure | Grouping, tabs, lifecycle controls, responsive stacking, and no horizontal overflow verified |

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
