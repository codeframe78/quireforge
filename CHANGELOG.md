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

### Changed

- Adopted **QuireForge** as the permanent product name, with the tagline
  “Build boldly. Work locally.” The former “Codex Linux Workbench” name was a
  temporary discovery-stage label.
- Updated the planned repository-project website to
  `https://codeframe78.github.io/quireforge/` with base path `/quireforge/`.

### Migration note

- The existing Git history and discovery work are being migrated in place; no
  replacement project or rewritten history is involved.
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
