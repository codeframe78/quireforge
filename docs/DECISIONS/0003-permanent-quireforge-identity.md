# ADR 0003: Permanent QuireForge Identity

- Status: Accepted; source location and website hosting amended on 2026-07-22
  by [ADR 0024](0024-webuzo-static-website-hosting.md), and repository
  visibility amended on 2026-07-23 by
  [ADR 0027](0027-public-source-and-runner-boundary.md)
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
| GitHub repository | `James-Jennison/quireforge` (public) |
| Executable | `quireforge` |
| Debian package | `quireforge` |
| AppImage basename | `QuireForge` |
| Desktop display name | `QuireForge` |
| Desktop entry filename | `io.github.codeframe78.QuireForge.desktop` |
| Application identifier | `io.github.codeframe78.QuireForge` |
| Default configuration path | `~/.config/quireforge` |
| Default data path | `~/.local/share/quireforge` |
| Default cache path | `~/.cache/quireforge` |
| Default state path | `~/.local/state/quireforge` |
| Production website | `https://quireforge.jamesjennison.net` |
| Website host | Webuzo origin behind Cloudflare |

The home-relative storage paths are documentation shorthand. Implementations
must honor the XDG base-directory environment and APIs. QuireForge logs belong
under an appropriate application-owned state or data directory.

The application identifier is syntactically valid for Tauri and freedesktop
application identity. Functional validation against the actual Tauri, GTK,
desktop-entry, D-Bus, and packaging versions remains mandatory during
application scaffolding. The desktop filename decision is explained in
[ADR 0004](0004-linux-desktop-entry-identity.md).
The current production-hosting boundary is explained in
[ADR 0024](0024-webuzo-static-website-hosting.md), and the public-source and
persistent-runner boundary is explained in
[ADR 0027](0027-public-source-and-runner-boundary.md). The legacy reverse-DNS
application identifier is intentionally retained as an installed compatibility
contract; transferring the source repository does not authorize changing it.

## Integration identity boundary

QuireForge identifies itself truthfully to app-server with machine name
`quireforge`, human title `QuireForge`, and the real application version. Codex
returns the upstream user-agent value after initialization. QuireForge must not
spoof that value or rename or synthesize official Codex, ChatGPT, OpenAI,
plugin, connector, marketplace, MCP, API, or protocol identifiers.

Connector credentials, Codex authentication, and Codex-owned sessions remain
outside QuireForge storage and outside this migration.

## Repository and local-path migration

The existing GitHub repository was renamed in place after separate,
action-specific approval. The local repository moved from
`/mnt/faststorage/codex-linux-workbench` to `/mnt/faststorage/quireforge` as one
intact working copy through a controlled stop, move, reopen, and verification
handoff. Its Git history, directory inode, tracked-content fingerprint, branch,
and remote were verified afterward.

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
- Authenticated hosting access, DNS/SSL changes, website deployment, package
  release, pushing, merging, and destructive cleanup remain independently
  approval-gated.
- Future scaffolding must include automated tests that assert these identifiers
  and the production website origin/base configuration.

## Intentionally preserved legacy references

Post-migration searches may still find the temporary display name in this ADR,
the changelog, and the roadmap because those passages explain project history.
The old absolute local path remains here solely as the verified source of the
completed move. The initial short desktop filename remains only in the
changelog and ADR 0004 to explain why it was superseded. Git commits and branch
history are immutable historical records and are not rewritten.

The local Git remote now uses the canonical QuireForge repository URL. The
historical references above are not current branding and must not appear in new
user-facing product surfaces.

### Milestone 1 verification

A case-insensitive tracked-source audit on 2026-07-19 found only these former
identity references:

| Location | Classification | Reason retained |
|---|---|---|
| This ADR | Historical reference | Records the temporary discovery name and verified source path of the intact move |
| `CHANGELOG.md` | Historical reference | Describes the completed rename for future maintainers |
| `docs/ROADMAP.md` | Historical reference | Preserves milestone history |
| ADR 0004 | Historical reference | Explains why the proposed short desktop filename was superseded |

No former repository URL, GitHub Pages URL presented as production, executable,
package, application ID, storage path, environment prefix, database filename,
window title, log prefix, or user-agent identity remains as a current product
contract. Accurate official Codex/OpenAI terminology and superseded hosting
records remain intentionally unchanged.

The verification also rechecked the expected old and new XDG locations and
desktop-entry paths. No released/development QuireForge data, temporary-name
application data, or installed desktop entry exists, so no user-data migration
is required at this stage. This conclusion must be reevaluated before any
release writes persistent application data.
