# Milestone 21B Release-Readiness Report

Status: local package/platform preflight and dormant website activation
hardening are complete on `feat/milestone-21b-beta-readiness`. External
publication is not complete. No branch or tag was pushed, no workflow was
dispatched, no release or attestation was created, no package was installed on
the host, no public download was activated, and no website or hosting state
changed.

## Outcome

The `0.1.0-beta.1` packages have passed a fresh clean Ubuntu 22.04 container
build, disposable Debian lifecycle checks, repeated byte-identical
normalization, current-host AppImage and extracted-Debian launches, and native
pixel review of the Milestone 21A account gate. Public installation and
limitation copy is staged without claiming that a release exists.

The website still commits `state: "unavailable"` and `release: null`. Its
dormant published path now validates a complete two-package record before a
build can expose it: exact version-derived filenames, positive byte sizes,
lowercase SHA-256 values, one AppImage and one Debian package, UTC publication
time, and credential-free HTTPS URLs on the approved QuireForge origin. It also
refuses published state until an approved private security-reporting URL is
configured.

## Package and platform evidence

The clean preflight manifest records:

- schema version 1 and `state: "release-candidate"`;
- source commit `65c95c109f809824f18feadfc0b50952376ebacf`;
- clean source tree;
- x86_64 Ubuntu 22.04 builder pinned by image digest; and
- exactly one AppImage and one Debian artifact plus `SHA256SUMS`.

That preflight source precedes the dormant website hardening and is not the
terminal publication source. The generated release manifest, rather than this
version-controlled report, is the authoritative terminal source/hash record. A
publication handoff is valid only when that manifest names the exact clean
commit selected for the tag and all four files still match it.

Accepted preflight behavior includes:

- both package structures, identities, dependencies, AppStream records,
  checksums, sizes, and maximum GLIBC 2.35 requirement;
- disposable Debian installation, upgrade from synthetic `0.0.0`, removal,
  and preservation of project and QuireForge metadata sentinels;
- visible Debian and AppImage X11 launches inside the pinned baseline without
  Codex credentials or a development server;
- byte-identical repeated normalization of both packages, the manifest, and
  checksum file;
- visible current-host launches of the AppImage and extracted Debian payload
  under isolated home/XDG roots;
- an eight-second AppImage pixel capture of the complete signed-out gate with
  6,149 colors, no black screen, and no `127.0.0.1` or connection-refused
  evidence; and
- a fresh Node audit with no known high-severity vulnerability plus the pinned
  Rust auditor scanning all 503 locked crates against 1,169 advisories with
  zero unaccepted vulnerability or warning.

The Rust refresh retains the same 17 exact reviewed upstream GTK3, GLib,
proc-macro, and `unic` maintenance/advisory exceptions. QuireForge does not use
the affected `glib::VariantStrIter` directly. The exact-source terminal gate
must rerun both audits; this preflight does not exempt a later dependency
change.

The declared initial beta scope remains x86_64 Ubuntu 22.04 or newer on GNOME
Wayland or X11. The evidence combines the pinned Ubuntu 22.04 package gate,
current Ubuntu package launches, prior native Wayland picker/drop/notification
acceptance, and prior true-X11 Ubuntu 24.04 interaction acceptance. It does not
establish arm64, non-Ubuntu, or non-GNOME support.

## Aggregate local acceptance

The source and dormant website activation pass completed:

- repository validation and six package-contract tests;
- 157 desktop and seven website unit/contract tests;
- 178 runnable Rust tests, with three deliberate live probes ignored;
- TypeScript/Astro checks, ESLint, Prettier, Rust formatting, warning-denying
  Clippy, and production compilation;
- website and desktop generated-artifact validation, including desktop bundle
  budgets;
- all 34 desktop and eight website Playwright desktop/mobile scenarios,
  including overflow, keyboard, reduced-motion, forced-color, and automated
  accessibility checks; and
- fresh Node and Rust dependency audits with no unaccepted finding.

The browser suite uses deterministic fixtures. No live login, personal Codex
state read, billable model call, connector authorization, or user repository
mutation occurred.

## Distribution finding

The configured source repository is private. A GitHub prerelease in that
repository is therefore an access-controlled review/provenance record, not an
anonymous public download channel. The public website must not link those
private release URLs or imply that they are publicly retrievable.

The `quireforge-release` environment now exists with one custom deployment
policy admitting only tags matching `v*`. The current GitHub plan rejected
required-reviewer and wait-timer rules, so those protections are not claimed.
The workflow's exact operation, confirmation phrase, tag ref, clean manifest,
pinned builder, checksum attestation, and prerelease flags remain mandatory.

The prepared public layout uses the owner-hosted QuireForge origin:

```text
https://quireforge.jamesjennison.net/downloads/v0.1.0-beta.1/
```

That path is a recommendation, not an authorization or deployment record. The
owner must explicitly approve the exact distribution path and operational
promotion before package bytes are placed on the Webuzo-managed origin.
Provider account names, server addresses, document roots, backup identifiers,
and credentials remain execution-time private data and must not enter source.
The production beta also needs an approved private security-reporting route;
the existing security policy resolves that route through the `codeframe78`
GitHub profile only to request a private channel without including
vulnerability details in the initial message.

## Publication handoff

The remaining terminal sequence is deliberately split:

1. Freeze and rebuild the exact clean source; record its manifest and hashes.
2. Approve the exact commit, tag `v0.1.0-beta.1`, private GitHub
   review/provenance operation, and rollback boundary before any push, tag, or
   workflow dispatch.
3. Independently verify the four resulting files against the reviewed
   manifest and `SHA256SUMS`.
4. Separately approve promotion of those exact bytes to the exact
   owner-hosted public version directory.
5. Retrieve every public URL without repository credentials, compare sizes and
   hashes, and install/launch each downloaded format through disposable QA.
6. Only then copy the public version, UTC date, manifest/checksum URLs, package
   URLs, sizes, and hashes into `apps/website/src/data/downloads.ts`.
7. Retain the approved security-contact route and verify that its public copy
   requests a private channel without soliciting vulnerability details.
8. Run the full website unit, build, artifact, desktop/mobile, accessibility,
   link, origin, and rollback gates, then obtain separate website-deployment
   approval.

No step inherits authority from the previous one. In particular, a private
GitHub prerelease does not authorize a public server upload, and a public
package upload does not authorize changing the website.

## Rollback

Before public package promotion, retain a recoverable copy of the current
website artifact and independently record the exact version-directory
manifest. If public verification fails:

1. leave or restore website download availability to `unavailable`;
2. stop promotion and identify the exact failed file/version;
3. preserve the failed evidence for diagnosis;
4. never replace bytes under the same published version;
5. restore the prior website artifact if it advertised the failed release; and
6. publish corrected packages only under a new prerelease version after new
   approval.

Removing a package, release record, server artifact, or website link is a
separate exact-target operation. Rollback never authorizes deleting attached
projects, worktrees, uncommitted changes, Codex state or credentials,
QuireForge user metadata, unrelated releases, or unrelated hosting content.
