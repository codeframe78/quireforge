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
not expose arbitrary Git arguments or remote operations. Milestone 11A adds a
managed-worktree foundation: bounded native inventory, app-generated
destinations, native-picker attachment, expiring confirmations, source-HEAD and
identity revalidation, and ordinary project registration. Milestone 11B adds
bounded parallel execution for up to four distinct worktree projects, an
aggregate task monitor, exact per-task controls, and normalized changed-file
and conflict counts. Selecting a task opens its live bounded activity stream;
raw Codex and process identity remains native-only. Milestone 11C adds opaque
recovery of retained app-managed checkouts and explicit removal of only clean,
inactive, app-managed worktrees after confirmation-time identity and status
revalidation. Cleanup preserves the branch and never offers force or generic
prune behavior. Milestone 12 adds up to eight app-owned native PTY tabs rooted
in freshly reverified attached projects, with byte-safe bounded output,
input/resize, controlled environment inheritance, explicit process cleanup,
and metadata-only recovery that never stores shell content or process identity.
Milestone 13A refreshed the Codex 0.145.0 protocol evidence and added strict
shared contracts for connector, plugin, marketplace, skill, MCP, policy,
requirement, scope, health, and app-owned dynamic-tool discovery. The contract
preserves blocked, degraded, and unknown states. Milestone 13B adds the live
read-only native catalog: supported app-server reads for connectors, skills,
MCP, and policy; stable CLI JSON reads for plugins and marketplaces; strict
normalization, version gating, cache invalidation, partial-failure handling, and
one narrow typed IPC command. Installation, authorization, configuration
mutation, and Integration Center UI remain Milestone 14 work. Milestone 14A
adds a native, fixed-command preview/confirm boundary for pinned plugin and
marketplace install/remove/add/upgrade operations, with one-use confirmations,
fresh catalog revalidation, and exact postcondition checks. Milestone 14B adds
the responsive Integration Center: category-preserving search, filters and
details, normalized trust review, and accessible confirmation for only those
14A operations whose capability is ready. Milestone 14C adds confirmed
connector and MCP authorization handoffs, confirmed skill enable/disable,
explicit catalog/health refresh, and native-constructed connector mentions on
new turns. URLs, skill paths, MCP names, and `app://` paths remain native-only;
unsupported generic configuration, plugin enablement, connector installation
RPCs, MCP management, and repair paths stay unavailable.
Milestone 15A adds a native-selected safe file-preview surface. Rust revalidates
the selected project attachment and file identity, keeps absolute paths native,
and sends React only bounded normalized UTF-8 text, PNG/JPEG data, or
metadata-only PDF state. Active HTML/SVG rendering, APNG, unknown binary
content, oversized files, and browser-side local selection remain unavailable;
UTF-8 markup can appear only as inert normalized text. Milestone 15B adds
explicit PNG/JPEG conversation attachments through the documented Codex
`localImage` turn input. Native picker selections, bounded browser drop bytes,
and Linux file-manager drops captured only in Rust are revalidated into
private, short-lived app-owned copies; React receives only opaque IDs and
normalized metadata. Generic file attachments and path-bearing frontend
drag/drop remain unavailable. Milestone 15C adds a separately confirmed,
one-use default-application handoff for a revalidated preview plus fixed-copy
background approval/completion/failure notifications. React receives no
absolute path or notification content input, and generic opener IPC remains
unavailable. Full Wayland/X11-session acceptance is recorded separately.
Cloudflare Pages is the selected production host, but the site has not been
deployed. There is no application package to install yet.

## Project status

- Supported distributions: none yet; Ubuntu support is being evaluated.
- Installation: not available before the packaging milestone.
- Website: the Astro site builds and passes local responsive/accessibility
  checks for `https://quireforge.jamesjennison.net`; it is not deployed.
- Integration support: runtime compatibility retains Codex CLI 0.144.6
  fixtures and now includes a reviewed 0.145.0 integration schema subset.
  Account status and Codex-managed authentication are implemented; the 13A
  catalog/dynamic-tool contract, 13B native read-only discovery service, and
  14A native plugin/marketplace lifecycle, 14B Integration Center, and 14C
  authorization/control boundary are complete, merged, and verified on `main`.
- Desktop: the Tauri 2, React, TypeScript, and Rust shell builds and launches
  locally with narrow typed IPC, a supervised non-billable Codex runtime and
  account-status probe, a verified native project-attachment workflow, and a
  strict native conversation runtime with a responsive task UI and native
  session-lifecycle/recovery boundary, accessible session history controls,
  the complete Milestone 9 native approval and detailed-activity interface, and
  complete Milestone 10 reviewed Git status/diff and mutation workflows, plus
  the Milestone 11A managed-worktree inventory/create/attach foundation and
  Milestone 11B bounded parallel task monitor, retained-worktree recovery, and
  Milestone 11C clean managed-worktree cleanup, plus the Milestone 12 native
  integrated terminal, the Milestone 13B normalized read-only integration
  catalog boundary, the Milestone 14A confirmed native plugin/marketplace
  mutation boundary, the Milestone 14B accessible Integration Center, and the
  Milestone 14C confirmed authorization/control boundary, plus the Milestone
  15A bounded project-file preview surface and Milestone 15B bounded
  conversation-image staging and explicit attachment flow, plus the Milestone
  15C reviewed default-application handoff and privacy-safe notification code
  checkpoint.
- CI status: repository, website, and desktop quality gates are configured for
  pull requests and `main` pushes; deployment remains separately gated.
- Current milestone: Milestone 15 is in progress. The 15A safe file-preview and
  15B conversation-image attachment checkpoints are implemented and verified
  locally; the 15C handoff/notification code checkpoint is implemented and its
  production Wayland launch, fixed-copy notification delivery, and complete
  XWayland and true-X11 handoff paths are verified. Interactive Wayland
  picker/attachment evidence remains open. Unsupported generic openers, file
  attachments, and integration-management paths remain unavailable.
- Known limitations: Codex-directed model/reasoning selection is not yet
  implemented and is deferred to Milestone 18 after its integration and
  advanced-feature prerequisites; the current turn cannot replace its own
  executing model. Concurrency is capped at four active worktree tasks; durable
  task recovery, automatic conflict resolution, attached-worktree cleanup,
  force/prune workflows, advanced remote operations, installable packages,
  releases, public deployment, and production Lighthouse evidence do not exist
  yet.

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
- [Milestone real-world time ledger](docs/MILESTONE_TIME_LEDGER.md)
- [Normalized integration contracts decision](docs/DECISIONS/0018-normalized-integration-contracts.md)
- [Confirmed integration mutations decision](docs/DECISIONS/0019-confirmed-integration-mutations.md)
- [Confirmed integration authorization and controls decision](docs/DECISIONS/0020-confirmed-integration-authorization-and-controls.md)
- [Safe project file previews decision](docs/DECISIONS/0021-safe-project-file-previews.md)
- [Bounded conversation image attachments decision](docs/DECISIONS/0022-bounded-conversation-image-attachments.md)
- [Superseded GitHub Pages plan](docs/GITHUB-PAGES.md)
- [Permanent identity decision](docs/DECISIONS/0003-permanent-quireforge-identity.md)
- [Native approval and activity decision](docs/DECISIONS/0011-native-approvals-and-activity-contract.md)
- [Reviewed Git mutation decision](docs/DECISIONS/0013-reviewed-git-mutation-boundary.md)
- [Managed worktree foundation decision](docs/DECISIONS/0014-managed-worktree-foundation.md)
- [Bounded parallel worktree execution decision](docs/DECISIONS/0015-bounded-parallel-worktree-execution.md)
- [Safe managed-worktree cleanup decision](docs/DECISIONS/0016-safe-managed-worktree-cleanup.md)
- [Native integrated terminal decision](docs/DECISIONS/0017-native-integrated-terminal.md)
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
