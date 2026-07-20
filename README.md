# QuireForge

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="assets/brand/quireforge-lockup-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="assets/brand/quireforge-lockup.svg">
  <img alt="QuireForge — Build boldly. Work locally." src="assets/brand/quireforge-lockup.svg" width="620">
</picture>

> **Build boldly. Work locally.**

> [!IMPORTANT]
> This is an unofficial community project. It is not made, endorsed, supported,
> or distributed by OpenAI. QuireForge is an unofficial native Linux workspace
> for Codex.

QuireForge is an early-stage open-source project building a native graphical
Codex workspace for Linux. It works toward a direct, local-first project model:
user-selected directories remain in place and Codex operates against those
original directories through supported interfaces. The original Codex
discovery, QuireForge identity, governance, and the local static website
foundation are complete. The Tauri desktop foundation also builds and launches
locally, and its versioned Rust adapter detects the installed Codex CLI and
normalizes the supported app-server model catalog and Codex-owned account
state. Browser/device onboarding, cancellation, logout, and redacted recovery
are implemented locally without storing credentials. Milestone 6's project
metadata, directory attachment, cwd preflight, and accessible workspace are
merged. Milestone 7A's native conversation runtime now starts a verified
thread/turn, normalizes its bounded stream, interrupts the exact active turn,
and stores reference-only metadata. Milestone 7B adds the responsive composer,
runtime-derived model and reasoning controls, explicit sandbox and approval
choices, normalized progress stream, and exact stop action. Milestone 8A adds
native app-reference-only resume, fork, archive/restore, authoritative session
reconciliation, and conservative crash recovery. Milestone 8B adds bounded
Codex-authoritative title search, project/fork grouping, keyboard-accessible
tabs, and the user-facing resume/fork/archive/restore interface without
persisting titles or transcript content.
Milestone 9A adds a strict native approval boundary, app-owned approval and
activity identity, turn-scoped decisions, safe pending-request cancellation,
and bounded redacted command/tool/file progress. Milestone 9B groups that
progress into selectable, in-place expanded activity rows and adds an
accessible approval card that offers only the native-advertised decisions.
Milestone 10A adds a read-only native Git boundary with branch/status review,
staged and working-tree diffs, a responsive changed-file interface, and an
explicit revalidated editor handoff. Milestone 10B adds fixed stage, unstage,
bounded revert/recovery, and commit workflows with native-held expiring
confirmations, concurrency and postcondition checks, attachment-scoped staged
paths, repository-local identity, and high-confidence secret refusal. It does
not expose arbitrary Git arguments, branches, worktrees, or remote operations.
Cloudflare Pages is the selected production host, but the site has not been
deployed. There is no application package to install yet.

## Project status

- Supported distributions: none yet; Ubuntu support is being evaluated.
- Installation: not available before the packaging milestone.
- Website: the Astro site builds and passes local responsive/accessibility
  checks for `https://quireforge.jamesjennison.net`; it is not deployed.
- Integration support: the local adapter is validated against Codex CLI
  0.144.6; account status and Codex-managed authentication are implemented,
  while integration workflows remain planned.
- Desktop: the Tauri 2, React, TypeScript, and Rust shell builds and launches
  locally with narrow typed IPC, a supervised non-billable Codex runtime and
  account-status probe, a verified native project-attachment workflow, and a
  strict native conversation runtime with a responsive task UI and native
  session-lifecycle/recovery boundary, accessible session history controls,
  the complete Milestone 9 native approval and detailed-activity interface, and
  complete Milestone 10 reviewed Git status/diff and mutation workflows.
- CI status: repository, website, and desktop quality gates are configured for
  pull requests and `main` pushes; deployment remains separately gated.
- Current milestone: Milestone 10 Git review and controlled mutation workflows
  are complete and verified locally; Milestone 11 requires a fresh gate.
- Known limitations: advanced Git/worktree/remote operations, installable
  packages, releases, public deployment, and production Lighthouse evidence do
  not exist yet.

## Discovery documents

- [Architecture](docs/ARCHITECTURE.md)
- [Codex integration findings](docs/CODEX-INTEGRATION.md)
- [Compatibility](docs/COMPATIBILITY.md)
- [Feature parity](docs/FEATURE-PARITY.md)
- [Threat model](docs/THREAT-MODEL.md)
- [Cloudflare Pages capability audit](docs/CLOUDFLARE-PAGES-CAPABILITY-AUDIT.md)
- [Cloudflare Pages deployment](docs/CLOUDFLARE-PAGES-DEPLOYMENT.md)
- [Website architecture](docs/WEBSITE.md)
- [Building](docs/BUILDING.md)
- [Testing](docs/TESTING.md)
- [Local build performance](docs/LOCAL-BUILD-PERFORMANCE.md)
- [Milestone forecasts](docs/MILESTONE-FORECASTS.md)
- [Superseded GitHub Pages plan](docs/GITHUB-PAGES.md)
- [Permanent identity decision](docs/DECISIONS/0003-permanent-quireforge-identity.md)
- [Native approval and activity decision](docs/DECISIONS/0011-native-approvals-and-activity-contract.md)
- [Reviewed Git mutation decision](docs/DECISIONS/0013-reviewed-git-mutation-boundary.md)
- [Brand sources and usage](assets/brand/README.md)
- [Roadmap](docs/ROADMAP.md)
- [Changelog](CHANGELOG.md)

The application will use only supported Codex and ChatGPT integration
mechanisms. It will not scrape ChatGPT, copy browser tokens, reverse engineer a
proprietary desktop application, or present account-specific integrations as a
guaranteed public catalog.

## Permanent identity

The repository is `codeframe78/quireforge`. Future application and
packaging work must use `quireforge` for the executable and Debian package,
`QuireForge` for the desktop display name and AppImage basename, and
`io.github.codeframe78.QuireForge` as the application identifier. Its syntax is
validated for Tauri and freedesktop application identity; functional bundle,
and packaging wiring remains an implementation-milestone test obligation. The
unbundled GTK/D-Bus runtime identity is verified locally.
The canonical desktop entry is
`io.github.codeframe78.QuireForge.desktop`; its `Exec` target remains
`quireforge`.

The production website target is
`https://quireforge.jamesjennison.net`, hosted as a static Astro site on
Cloudflare Pages. Cloudflare is authoritative DNS. GitHub remains the source,
CI, issue, and release host. GitHub Pages is disabled and is not the production
host. The website source is under `apps/website/`; project creation, DNS, and
deployment remain separately approval-gated.

## Website development

With Node 22 and pnpm 11.15.0 available:

```bash
pnpm install --frozen-lockfile
pnpm validate
pnpm test:e2e
```

See [Building](docs/BUILDING.md) and [Testing](docs/TESTING.md) for the complete
local workflow and a fallback when an older distribution Corepack cannot launch
the pinned pnpm release.

## Desktop development

With Rust 1.88 or newer and the documented Tauri Linux development packages:

```bash
pnpm desktop:dev
pnpm desktop:build
pnpm codex:schema
```

The first command launches the local development shell. The second produces an
unbundled local executable for verification; it does not create or publish an
installable package. The third refreshes the reviewed Codex app-server schema
subset and requires explicit diff review. See [Building](docs/BUILDING.md) for
prerequisites.

Application-owned files will use the XDG locations `~/.config/quireforge`,
`~/.local/share/quireforge`, `~/.cache/quireforge`, and, where needed,
`~/.local/state/quireforge`. Codex-owned authentication, configuration, and
session storage are outside this identity migration.

## Governance

QuireForge is licensed under the [Apache License 2.0](LICENSE). Contributions
follow [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md). Review [SECURITY.md](SECURITY.md) before
reporting a vulnerability and [SUPPORT.md](SUPPORT.md) before sharing sanitized
diagnostics.
