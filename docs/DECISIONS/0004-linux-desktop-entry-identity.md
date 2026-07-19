# ADR 0004: Align the Linux Desktop Entry with the Application ID

- Status: Accepted
- Date: 2026-07-19
- Decision owners: Project maintainers

## Context

The initial identity proposal used `quireforge.desktop` while separately
selecting `io.github.codeframe78.QuireForge` as the Tauri application ID. The
short filename is syntactically valid, but the freedesktop specification
recommends reverse-DNS desktop filenames and derives the desktop-file ID from
that filename.

If QuireForge later enables D-Bus activation, the D-Bus well-known name must
equal the desktop filename without its `.desktop` suffix. Tauri can also apply
its identifier as the GTK application ID. Keeping one reverse-DNS identity
avoids divergent GTK, D-Bus, desktop, notification, and portal identities.

## Decision

The canonical Linux desktop entry filename is:

```text
io.github.codeframe78.QuireForge.desktop
```

Related identities remain:

- Display `Name`: `QuireForge`.
- Executable and desktop `Exec` target: `quireforge`.
- Debian package: `quireforge`.
- Tauri/GTK application ID: `io.github.codeframe78.QuireForge`.

The packaging milestone must configure or post-process Tauri output through a
tested, reproducible mechanism so the installed desktop file uses this exact
name. It must not leave both the short and reverse-DNS desktop entries installed.

## Evidence

- [Freedesktop file naming](https://specifications.freedesktop.org/desktop-entry/latest/file-naming.html)
  recommends a reverse-DNS desktop filename whose basename is a valid D-Bus
  well-known name.
- [Freedesktop D-Bus activation](https://specifications.freedesktop.org/desktop-entry/latest/dbus.html)
  requires the service name to match the desktop filename basename when D-Bus
  activation is used.
- [Tauri configuration](https://v2.tauri.app/reference/config/#identifier)
  accepts the selected reverse-DNS identifier, and `enableGTKAppId` can apply
  that identifier as the GTK application ID.

## Consequences

- The previously proposed `quireforge.desktop` value is intentionally
  superseded, not retained as an alias.
- Launcher, notification, portal, MIME, and D-Bus tests must assert the
  reverse-DNS identity during desktop and packaging milestones.
- Changing this installed filename after public release would create stale
  launcher entries, so the decision is made before an application package
  exists.
