# Building QuireForge

Status: the Milestone 2 website and Milestones 3–4 desktop shell and Codex
process adapter can be developed and built locally. No installable application
package exists.

## Supported development baseline

- Linux development host
- Node.js `22.12.0` or newer in the Node 22 line
- pnpm `11.15.0`, as pinned by the root `packageManager` field
- Rust `1.88` or newer with Cargo, rustfmt, and Clippy
- Tauri 2 Linux development packages listed below
- Python 3 for the dependency-free repository validator
- Git

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
installed integration information.

The production origin is `https://quireforge.jamesjennison.net` with base path
`/`. Local development continues to use Astro's local origin. No server runtime,
database, Pages Function, or Cloudflare adapter is required.

## Develop and build the desktop scaffold

```bash
pnpm desktop:dev
pnpm desktop:build
```

`desktop:dev` starts Vite and launches the Tauri window. `desktop:build` checks
the frontend and produces the unbundled executable `target/release/quireforge`.
The output is ignored by Git and is a local verification artifact, not a Debian
package, AppImage, release, or supported installation.

The browser-only shell preview is available with:

```bash
pnpm --filter @quireforge/desktop build
pnpm desktop:preview
```

Browser preview mode cannot call native IPC and labels itself accordingly.
The production Tauri window exposes `desktop_bootstrap` and the fixed-purpose
`codex_runtime_probe`. The latter accepts no arguments and may run only
`codex --version` plus a bounded local app-server initialize/`model/list`
exchange. No arbitrary filesystem, shell, opener, process, project, login,
thread, turn, configuration, or integration command is exposed.

## Refresh the reviewed Codex schemas

With the intended Codex CLI active:

```bash
pnpm codex:schema
```

The generator writes a versioned fixture directory containing only initialize
and `model/list` schemas plus SHA-256 hashes. It does not modify Codex
configuration or sessions. Never accept a refresh mechanically: inspect the
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
