# Local Build Performance

Status: generalized baseline captured from required Milestones 3–4 work on
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
