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
- Completed Milestone 0 feasibility documentation locally; no hosting project,
  DNS record, deployment, push, or release was created by Codex.
- Refreshed account-scoped Codex discovery without publishing catalog entries
  or integration identifiers.

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

- Discovery only: no desktop application, website deployment, package, or
  release exists.
- Integration compatibility is based on Codex CLI 0.144.6 and must be probed at
  runtime.
