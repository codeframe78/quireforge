# ADR 0003: Permanent QuireForge Identity

- Status: Accepted
- Date: 2026-07-19
- Decision owners: Project maintainers

## Context

Milestone 0 used “Codex Linux Workbench” as a temporary discovery-stage name.
The project needs a permanent identity before application, packaging, website,
integration-client, and storage identifiers become installed compatibility
contracts.

The migration must preserve the existing Git repository and history. QuireForge
must remain clearly distinct from OpenAI branding and must not imply that
OpenAI makes, endorses, supports, or distributes the project.

## Decision

The permanent identity is:

| Surface | Value |
|---|---|
| Product display name | `QuireForge` |
| Tagline | `Build boldly. Work locally.` |
| Description | `An unofficial native Linux workspace for Codex` |
| GitHub repository | `codeframe78/quireforge` |
| Executable | `quireforge` |
| Debian package | `quireforge` |
| AppImage basename | `QuireForge` |
| Desktop display name | `QuireForge` |
| Desktop entry filename | `quireforge.desktop` |
| Application identifier | `io.github.codeframe78.QuireForge` |
| Default configuration path | `~/.config/quireforge` |
| Default data path | `~/.local/share/quireforge` |
| Default cache path | `~/.cache/quireforge` |
| Default state path | `~/.local/state/quireforge` |
| GitHub Pages base | `/quireforge/` |
| GitHub Pages URL | `https://codeframe78.github.io/quireforge/` |

The home-relative storage paths are documentation shorthand. Implementations
must honor the XDG base-directory environment and APIs. QuireForge logs belong
under an appropriate application-owned state or data directory.

The application identifier remains a target until validated against the actual
Tauri, desktop-entry, D-Bus, and packaging versions selected during application
scaffolding. A required technical adjustment must receive its own documented
decision; it must not silently create inconsistent identifiers.

## Integration identity boundary

QuireForge may identify itself truthfully through supported client metadata as
`QuireForge` and, when appropriate, a product-owned token such as
`quireforge/<version>`. It must not rename or synthesize official Codex,
ChatGPT, OpenAI, plugin, connector, marketplace, MCP, API, or protocol
identifiers.

Connector credentials, Codex authentication, and Codex-owned sessions remain
outside QuireForge storage and outside this migration.

## Repository and local-path migration

The existing GitHub repository will be renamed in place only after separate,
action-specific approval. The local repository will eventually move from
`/mnt/faststorage/codex-linux-workbench` to the approved destination
`/mnt/faststorage/quireforge` as one intact working copy.

Because an active Codex session uses the old working directory, the local move
requires a controlled stop, move, reopen, and verification handoff. This ADR
does not authorize either external operation.

## Existing user data

At this decision date, the repository contains discovery documentation only.
No application manifests, released builds, old-identity XDG directories, or
old desktop entries were detected. An application-data migration is therefore
not implemented prematurely.

Before a release can create durable user data, schema migrations must reserve
an idempotent identity-migration path that:

- never overwrites newer QuireForge data;
- never deletes old data automatically;
- preserves application-owned project associations and non-secret settings;
- leaves Codex-owned authentication, configuration, integrations, and sessions
  untouched; and
- records sanitized results with a documented recovery procedure.

## Branding

QuireForge branding uses original, maintainable vector sources under
`assets/brand/`, with light and dark variants and sources designed for small
Linux application icons. It does not copy OpenAI or ChatGPT logos, iconography,
or visual systems. Production exports remain part of the consuming desktop,
website, and packaging milestones because those structures do not exist yet.

## Consequences

- All new product-owned identifiers derive from this map.
- Historical Git records are preserved rather than rewritten.
- Existing official Codex and integration terminology remains accurate.
- Repository rename, local-directory move, Pages deployment, package release,
  and cleanup remain independently approval-gated.
- Future scaffolding must include automated tests that assert these identifiers
  and the `/quireforge/` Pages base path.

## Intentionally preserved legacy references

Post-migration searches may still find the temporary display name in this ADR,
the changelog, and the roadmap because those passages explain project history.
The old absolute local path remains here solely as the verified source of the
approved future move. Git commits and branch history are immutable historical
records and are not rewritten.

The local Git remote also remains on the old repository URL until the separate
GitHub repository migration is approved and completed. These references are
not current branding and must not appear in new user-facing product surfaces.
