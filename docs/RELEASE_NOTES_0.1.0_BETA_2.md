# QuireForge 0.1.0-beta.2 Release Notes

QuireForge is an early public beta for x86_64 Ubuntu 22.04 or newer on GNOME
Wayland or X11. It is an unofficial community project and is not made,
endorsed, supported, or distributed by OpenAI.

## What beta 2 fixes

Beta 2 supersedes beta 1 because GitHub Releases normalizes `~` characters in
uploaded asset names. Beta 1's downloadable Debian filename therefore differed
from the immutable filename recorded by its manifest and checksum file.

Beta 2 makes the downloadable name GitHub-safe at build time:

```text
quireforge_0.1.0.beta.2_amd64.deb
```

The Debian package metadata still uses `0.1.0~beta.2`, preserving correct
prerelease ordering before the future stable `0.1.0`. The manifest,
`SHA256SUMS`, GitHub asset, and any later owner-hosted copy now use the same
outer filename. Beta 1 remains available as historical evidence and must not be
promoted or have its assets replaced.

## Included product surface

- A Codex-owned account gate before project, conversation, integration, or
  usage data loads.
- An original responsive QuireForge home and workspace hierarchy without
  internal milestone labels.
- Read-only remaining usage when Codex supplies a valid documented rate-limit
  meter; unavailable and unmetered states are never estimated.
- In-place local project attachment, Codex conversation and approval flows,
  reviewed Git/worktree operations, native terminal tabs, integration
  discovery and bounded controls, safe file previews, PNG/JPEG attachments,
  reviewed desktop handoffs, notifications, and read-only scheduled-template
  discovery.

## Verify before installing

Download `SHA256SUMS` and one package from the same release, then run:

```bash
sha256sum --check --ignore-missing SHA256SUMS
```

Continue only when the selected filename reports `OK`. See
[Beta Installation](BETA-INSTALLATION.md) for installation commands, the FUSE
fallback, first-launch behavior, removal boundaries, and current limitations.

## Distribution boundary

The GitHub prerelease is a secondary public artifact and provenance record.
The production Downloads page remains inactive until the exact beta 2 files
are separately promoted to and anonymously verified on the owner-hosted
QuireForge origin. A matching filename from another location is not sufficient.
