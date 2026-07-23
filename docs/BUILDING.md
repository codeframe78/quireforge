# Building QuireForge

Status: the website and desktop code through Milestone 20 can be
developed and built locally, including Codex/authentication, project and
conversation lifecycle, reviewed Git/worktree workflows, the native terminal,
normalized/confirmed integration workflows, bounded project-file previews, and
private conversation-image attachments, reviewed default-application handoffs,
privacy-safe background notifications, policy-bounded next-turn selection,
hardening, and local Linux packaging. Local installable candidates exist; no
public release or activated website download exists.

## Supported development baseline

- Linux development host
- Node.js `22.12.0` or newer in the Node 22 line
- pnpm `11.15.0`, as pinned by the root `packageManager` field
- Rust `1.95` or newer with Cargo, rustfmt, and Clippy
- Tauri 2 Linux development packages listed below
- Python 3 for the dependency-free repository validator
- Git

The integrated terminal is Linux-specific. Its native tests require `/proc`
process identity and a working local PTY; they use temporary directories and do
not need a live Codex session, network access, CUDA, or GPU rendering.

Do not install dependencies with npm or commit an additional lockfile. The
workspace uses the root `pnpm-lock.yaml` and rejects unreviewed dependency build
scripts. Only `esbuild` and `sharp` are allowed to build during installation.

## Install dependencies

From the repository root:

```bash
pnpm install --frozen-lockfile
```

If the distribution-provided Corepack cannot launch the pinned pnpm version,
use the non-persistent fallback used during Milestone 2:

```bash
npx --yes pnpm@11.15.0 install --frozen-lockfile
```

Do not use `--ignore-scripts` as a substitute for the committed pnpm build
allowlist; Astro's approved native dependencies need their normal install
steps.

## Install Linux desktop prerequisites

On Ubuntu or Debian development hosts, install Tauri's WebKitGTK 4.1 toolchain:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

Install Rust through the official rustup workflow, then ensure `cargo`,
`rustfmt`, and Clippy are available. Do not commit Cargo registry content,
toolchain caches, or generated `target/` output.

## Develop and build the website

```bash
pnpm dev
pnpm build
pnpm preview
```

The generated static artifact is `apps/website/dist/`. It is ignored by Git and
must not contain credentials, local account data, Codex state, or locally
installed integration information. It contains the domain-scoped Apache
`.htaccess` used by the Webuzo-managed production origin.

The production origin is `https://quireforge.jamesjennison.net` with base path
`/`. Local development continues to use Astro's local origin. No database,
persistent process, reverse proxy, Pages Function, Cloudflare adapter, or Astro
server adapter is required. Cloudflare remains the public edge, not the origin
host.

## Develop and build the desktop scaffold

```bash
pnpm desktop:dev
pnpm desktop:build
```

`desktop:dev` starts Vite and launches the Tauri window. `desktop:build` checks
the frontend and produces the unbundled executable `target/release/quireforge`.
The output is ignored by Git and is a local verification artifact, not a Debian
package, AppImage, release, or supported installation.

Use `pnpm desktop:build` before treating that executable as a production
artifact. A direct `cargo build --release` does not run Tauri's frontend build
hook or embed `dist`; launching that diagnostic binary can retain the configured
development URL and fail at `127.0.0.1:1420` when Vite is not running.

After a production build, launch `./target/release/quireforge` and verify that
the rendered workspace—not only the native title bar and dark background—is
visible. QuireForge intentionally keeps Tauri's `freezePrototype` option at its
documented default of `false`: the current Vite/React production bundle assigns
an own `toString` property during startup and otherwise fails before mounting.
The explicit production CSP, restricted Tauri capability, and narrow typed IPC
remain the primary webview/native controls.

## Build local Linux package candidates

The authoritative package build uses Docker, not the newer discovery host:

```bash
./scripts/run_linux_package_container.sh
```

The script builds the digest-pinned Ubuntu 22.04 image, uses isolated ignored
caches, fetches only checksum-reviewed Tauri Linux tools, builds both Tauri
bundles, normalizes their identity and timestamps, and validates metadata,
checksums, GLIBC 2.35 compatibility, disposable package lifecycle, and visible
X11 launches. It does not install either package on the host.

Successful candidates are written to:

```text
target/ubuntu-22.04/release/packages/
├── QuireForge-0.1.0-beta.1-x86_64.AppImage
├── quireforge_0.1.0~beta.1_amd64.deb
├── release-manifest.json
└── SHA256SUMS
```

The directory is ignored by Git. A dirty source tree deliberately produces a
`local-candidate` manifest; only a clean exact-tag build can pass the separate
publication validator. Building candidates does not authorize a GitHub
release, website edit, deployment, package installation, or signing action.
See [Releasing](RELEASING.md) for the guarded handoff.

### Native-only notification probe

The manual Milestone 15 notification check has a disabled-by-default Cargo
feature. It adds no Tauri command or webview permission, accepts only the exact
native process flag, and sends the same fixed completed-task copy used by the
production notification state machine:

```bash
pnpm desktop:build:notification-probe
QUIRE_FORGE_PROBE_ROOT="$(mktemp -d /tmp/quireforge-notification-probe-XXXXXX)"
mkdir -p \
  "$QUIRE_FORGE_PROBE_ROOT/config" \
  "$QUIRE_FORGE_PROBE_ROOT/data" \
  "$QUIRE_FORGE_PROBE_ROOT/cache"
env \
  GDK_BACKEND=wayland \
  XDG_CONFIG_HOME="$QUIRE_FORGE_PROBE_ROOT/config" \
  XDG_DATA_HOME="$QUIRE_FORGE_PROBE_ROOT/data" \
  XDG_CACHE_HOME="$QUIRE_FORGE_PROBE_ROOT/cache" \
  ./target/release/quireforge --manual-notification-probe
```

Use `GDK_BACKEND=x11` only from a confirmed true X11 login when recording X11
evidence. The probe accepts no title, body, project, conversation, path, or
protocol data. After the manual check, run `pnpm desktop:build` again; that
normal artifact excludes the feature and its flag.

The browser-only shell preview is available with:

```bash
pnpm --filter @quireforge/desktop build
pnpm desktop:preview
```

Browser preview mode cannot call native IPC and labels itself accordingly.
The production Tauri window exposes `desktop_bootstrap`, the fixed-purpose
`codex_runtime_probe`, narrow `codex_auth_*` commands, and fixed-purpose
`project_*` lifecycle commands. It also exposes fixed `conversation_status`,
`conversation_start`, `conversation_poll`, and `conversation_interrupt`
commands. Runtime probing accepts no arguments and may
run only `codex --version` plus a bounded local app-server
initialize/`model/list` exchange. Authentication accepts only a closed
browser/device method; browser opening takes no frontend URL and uses the
validated native-held handoff. Project directory paths can enter only through
the native folder picker; later actions accept opaque project IDs, and no
source-deletion or general filesystem command is exposed. No arbitrary shell,
process, configuration, or integration command is exposed. Conversation start
accepts only an opaque project ID, bounded prompt, and closed model/reasoning,
sandbox, and approval values plus up to eight normalized connector catalog IDs;
cwd, `app://` paths, and native Codex IDs never enter from the webview. Poll and
interrupt accept only QuireForge's application conversation ID.

Conversation start/resume/fork may additionally carry at most four opaque
attachment UUIDv7s. `conversation_attachment_pick` owns its native picker path;
`conversation_attachment_stage_drop` accepts only bounded PNG/JPEG bytes and
safe display metadata. On Linux,
`conversation_attachment_stage_native_drop` claims only the current one-use,
30-second GTK file-manager capture for an opaque project ID; no source path is
an IPC field or response. Status/cancel commands expose no path. Native code
revalidates private staged copies and constructs documented `localImage` turn
inputs. Tauri default drag/drop path events remain disabled. Generic file
attachments and arbitrary filesystem reads are not exposed.

`file_preview_pick` also accepts only an opaque project ID. The file path comes
from the native picker and remains in Rust while attachment containment,
identity, file type, size, and content limits are checked. The response contains
only a relative display path and bounded normalized text or PNG/JPEG data; PDF
is metadata-only. Browser preview cannot use this command or a browser file
input to simulate local access.

The integration surface is also fixed-purpose:
`integration_catalog_read`/`integration_catalog_refresh`, closed
preview/confirm plugin-marketplace mutation commands, and closed 14C control
preview/confirm/browser/status commands. The browser command accepts only an
opaque UUIDv7 action ID; its validated authorization URL remains native-held.
Skill paths, MCP names, raw app IDs, configuration values, and arbitrary
commands are not frontend inputs. The main window retains an empty direct
plugin-permission list.

## Refresh the reviewed Codex schemas

With the intended Codex CLI active:

```bash
pnpm codex:schema
```

The generator writes a versioned fixture directory containing only initialize,
`model/list`, stable account-lifecycle, and reviewed Milestone 7A thread/turn
schemas plus SHA-256 hashes. It does not modify Codex configuration,
authentication, or sessions. Never accept a refresh mechanically: inspect the
CLI version, generated diff, field semantics, adapter normalization, tests, and
compatibility documentation before committing it. Do not commit the complete
multi-megabyte experimental schema bundle.

## Full non-browser validation

```bash
pnpm validate
```

Browser and accessibility checks are documented separately in
[Testing](TESTING.md). Deployment remains a separate approval-gated operation;
building either artifact does not authorize packaging, release publication,
Cloudflare project creation, custom-domain changes, DNS, or deployment.
