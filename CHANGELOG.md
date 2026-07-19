# Changelog

All notable project changes will be documented here. The project has not
released a usable application.

## Unreleased

### Added

- Dedicated public repository and an explicit unofficial-project disclaimer.
- Milestone 0 Codex integration, compatibility, feature-parity, architecture,
  threat-model, GitHub Pages, roadmap, and architecture-decision documentation.
- Permanent QuireForge identity contract covering application, package,
  repository, GitHub Pages, integration-client, and XDG storage identifiers.
- Original path-based QuireForge mark, wordmark, light/dark lockups, favicon,
  application-icon source, social card, palette, and brand usage guidance.
- Public/local A2 Hosting capability audit, preliminary cPanel deployment
  architecture, and an explicit authenticated-access boundary.
- ADR 0005 selecting the dedicated A2-hosted production subdomain while
  retaining the earlier GitHub Pages plan as superseded history.
- Authenticated A2/cPanel capability findings and a Cloudflare Pages capability
  and deployment plan.
- Sanitized owner-mediated Cloudflare account audit covering the Free plan,
  Workers & Pages availability, DNS, managed TLS, and security settings.
- ADR 0006 selecting Cloudflare Pages as the production website host and
  authoritative DNS while A2 retains main-site and mail origins.
- Apache License 2.0 and repository-wide contributor guidance.
- Security, contribution, conduct, support, issue, and pull-request policies.
- GitHub Actions dependency updates and a minimum-permission repository-checks
  workflow pinned to a reviewed checkout revision.
- A dependency-free repository validator for required files, local links,
  QuireForge identity contracts, SVG XML, text encoding, and high-confidence
  secret patterns.
- A pinned pnpm monorepo and Astro 7 static website under `apps/website` with
  15 public pages, a custom 404, sitemap, robots policy, manifest, and canonical
  and social metadata.
- Reusable QuireForge website components, centralized light/dark design tokens,
  responsive navigation, visible keyboard focus, reduced-motion support, and
  generated favicon, application-touch, and social-card assets derived from the
  approved vector sources.
- Deterministic website type, lint, format, unit, artifact, route, responsive,
  theme, and axe-core accessibility checks in local scripts and minimum-
  permission GitHub Actions.
- Cloudflare Pages security headers with a strict static-site content policy;
  HSTS remains intentionally deferred until live HTTPS is verified.
- Website build and testing documentation with an explicit no-deployment
  boundary.
- A pinned Tauri 2, React 19, TypeScript, Vite, and Rust desktop package under
  `apps/desktop`, with the accepted Linux application identity and original
  QuireForge icon exports.
- Generalized local build-performance and milestone-forecast histories for
  system-calibrated planning.
- A responsive, accessible light/dark desktop shell and one versioned
  `desktop_bootstrap` command validated against a shared Rust/TypeScript fixture.
- Desktop type, lint, format, unit, native contract, Clippy, build, responsive,
  theme, overflow, and axe-core accessibility gates in local scripts and CI.
- A versioned `CodexBackend` boundary, fixed-command CLI detection, supervised
  JSONL app-server lifecycle, normalized capability/model contracts, and
  explicit unavailable/degraded diagnostics.
- Deterministic Codex mocks, bounded protocol failure tests, and a reproducible
  generator that commits only the reviewed initialize and `model/list` schema
  subset for Codex CLI 0.144.6.
- A narrow `codex_runtime_probe` Tauri command and strict TypeScript runtime
  schema that prevent raw app-server, account, installation, path, or user-agent
  fields from reaching React.

### Changed

- Adopted **QuireForge** as the permanent product name, with the tagline
  “Build boldly. Work locally.” The former “Codex Linux Workbench” name was a
  temporary discovery-stage label.
- Updated the planned repository-project website to
  `https://codeframe78.github.io/quireforge/` with base path `/quireforge/`.
- Replaced the initially proposed `quireforge.desktop` filename with the
  freedesktop-aligned `io.github.codeframe78.QuireForge.desktop`; the executable
  and Debian package remain `quireforge`.
- Defined app-server initialization as `clientInfo.name = "quireforge"`,
  `clientInfo.title = "QuireForge"`, and the real application version.
- Selected `https://quireforge.jamesjennison.net` as the production website;
  GitHub remains the source, CI, issue, and release host.
- Replaced A2/cPanel production hosting with Cloudflare Pages after the A2
  audit identified greater deployment and lifecycle overhead. Codex changed no
  provider, DNS, project, or production setting as part of that decision.
- Recorded the owner's separately completed move of authoritative DNS to
  Cloudflare and the temporary absence of the QuireForge hostname in the new
  zone; no DNS record was created by Codex.
- Recorded owner confirmation that Cloudflare two-factor authentication is now
  enabled without retaining factor or recovery details.
- Recorded revocation of the dedicated A2 audit key and removal of its local
  private/public files from `~/.ssh` after the read-only audit.
- Completed Milestone 0 feasibility documentation locally; no hosting project,
  DNS record, deployment, push, or release was created by Codex.
- Refreshed account-scoped Codex discovery without publishing catalog entries
  or integration identifiers.
- Reconciled the completed QuireForge path/repository migration, classified all
  remaining former-name references as intentional history, confirmed that no
  pre-release application data requires migration, and completed Milestone 1
  locally without pushing or changing repository settings.
- Completed the Milestone 2 brand and static website foundation locally without
  creating a Cloudflare project, changing DNS, deploying, pushing, or merging.
- Completed the Milestone 3 desktop scaffold locally, including an unbundled
  Wayland launch and runtime application-identity check, without implementing
  Codex workflows, packaging, pushing, or merging.
- Completed the Milestone 4 Codex process adapter locally, including a
  non-billable live app-server probe, bounded failure recovery, exact process
  cleanup, and normalized desktop status without login, conversation turns,
  configuration writes, packaging, pushing, or merging.

### Migration note

- The existing Git history and discovery work were migrated in place; no
  replacement project or rewritten history was involved.
- The GitHub repository was renamed in place to `codeframe78/quireforge`, and
  the intact working copy moved to `/mnt/faststorage/quireforge`, through
  separate approval-gated operations.
- No released or development application data was detected under the temporary
  identity, so there is currently no user configuration to move. Future
  releases must preserve old data and never modify Codex-owned authentication
  or sessions.

### Known limitations

- The desktop adapter and website are locally verified, but no authentication,
  project, conversation, installable package, website deployment, or release
  workflow exists.
- Integration compatibility is based on Codex CLI 0.144.6 and must be probed at
  runtime.
