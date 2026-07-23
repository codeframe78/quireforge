# Releasing QuireForge

Status: Milestone 20 provides a verified local release-candidate pipeline.
Nothing in this procedure constitutes publication approval. The first beta,
website download activation, supported-platform statement, and rollback
rehearsal remain separately approval-gated Milestone 21 work.

## Release artifact contract

The Linux beta contract is:

- application version `0.1.0-beta.1`;
- x86_64 architecture only;
- `QuireForge-0.1.0-beta.1-x86_64.AppImage`;
- `quireforge_0.1.0~beta.1_amd64.deb`, using Debian's `~` prerelease ordering;
- `SHA256SUMS` covering exactly both installable artifacts; and
- `release-manifest.json` recording version, source commit/tree state, pinned
  builder identity, artifact names, formats, architecture, sizes, and hashes.

The package payload uses `quireforge` for the executable and Debian package,
`QuireForge` for display and AppImage naming, and
`io.github.codeframe78.QuireForge.desktop` for the desktop entry. Neither
format owns or removes attached project directories, Codex state, credentials,
or QuireForge's user metadata.

## Build and review locally

Run from a clean repository root:

```bash
./scripts/run_linux_package_container.sh
```

This builds inside the digest-pinned Ubuntu 22.04 container. Node, Rust, and
the base image are pinned by digest; the Tauri Linux helpers are fetched over
HTTPS and accepted only when their reviewed SHA-256 values match. The build
then validates:

- exact package names, versions, architecture, metadata, dependencies, files,
  desktop entry, AppStream data, and checksums;
- a maximum GLIBC requirement of 2.35 for the packaged executable;
- AppImage extraction and canonical desktop/AppStream layout;
- disposable Debian install, upgrade from a synthetic prior version, removal,
  and preservation of project/application data;
- stable visible Debian and AppImage windows under isolated X11 without Codex
  credentials or a local development server; and
- absence of a refused `127.0.0.1` launch path.

Review `release-manifest.json` and verify the checksum file independently:

```bash
cd target/ubuntu-22.04/release/packages
sha256sum --check SHA256SUMS
```

A worktree with tracked or untracked source changes produces
`state: local-candidate` and a diff digest. It is valid local evidence but is
never publishable.

## Verify-only GitHub workflow

`.github/workflows/linux-release.yml` is manual-only. Its default
`verify-only` operation builds the same candidates, runs the lifecycle and
launch checks, and uploads the exact four-file review set for 14 days. The
workflow does not run for pushes or pull requests and cannot publish from its
package job because the job has read-only repository permissions.

Running even the verify-only workflow changes external GitHub state by creating
a workflow run and artifact. Obtain the required approval before dispatch.

## Publication prerequisites

Do not select `publish-approved-beta` until all of these are true:

1. Milestone 21 publication approval explicitly covers the exact version,
   source commit/tag, two artifacts, distribution location, and rollback plan.
2. The source tree is clean and the reviewed tag is exactly `v0.1.0-beta.1`.
3. The tag points to the reviewed source and is available to the workflow.
4. The protected `quireforge-release` GitHub environment exists with the
   required reviewers or deployment policy.
5. GitHub artifact attestations are available for the repository.
6. Final supported-platform, installation, known-limitation, and rollback QA
   has passed.
7. Website download activation has separate approval and reviewed artifact
   URLs/hashes; publication does not silently activate the website.

The publish job additionally requires:

- operation `publish-approved-beta`;
- confirmation text exactly `PUBLISH-QUIREFORGE-BETA`;
- a `refs/tags/v...` workflow ref;
- a clean `release-candidate` manifest;
- the exact pinned Ubuntu 22.04 builder record;
- `--expected-tag` equality with `v{manifest.version}`;
- successful checksum-subject attestation; and
- the protected environment gate.

Only that job receives scoped `contents`, `id-token`, and `attestations` write
permissions. It creates a GitHub prerelease with `--verify-tag`; there is no
automatic stable release or website deployment.

## Website activation

`apps/website/src/data/downloads.ts` is the sole typed availability record. It
must remain:

```text
state: "unavailable"
release: null
```

until the release is published, externally retrievable, independently
checksummed, and website activation is approved. Activation must copy the
published version, date, manifest URL, checksum URL, artifact URLs, byte sizes,
and SHA-256 values from the reviewed release record. Run the full website
unit, build, generated-artifact, desktop/mobile, accessibility, and link gates
before a separately authorized website deployment.

## Rollback boundary

If final QA or download verification fails:

1. keep or restore website download state to unavailable;
2. stop promotion and record the exact failed artifact/tag;
3. preserve the release evidence for diagnosis rather than replacing files
   under the same version;
4. use a new prerelease version for corrected artifacts; and
5. request explicit authority before deleting a GitHub release/tag or changing
   public website state.

Package removal must remain separate from project or metadata deletion. A
rollback never authorizes deleting attached directories, Codex configuration,
credentials, sessions, QuireForge metadata, or unrelated hosting state.

## Current limitations

- Only x86_64 candidates are produced.
- Ubuntu 22.04 is the build/GLIBC baseline, not yet a complete distribution
  support matrix.
- Local artifacts are not distro-repository packages and have no distro
  repository signing.
- GitHub provenance is created only by an approved publish job; local
  candidates have checksums and manifest evidence but no external attestation.
- AppImage and Debian package publication, installation guidance, website
  activation, and public rollback verification remain Milestone 21.

References:

- [Tauri distribution overview](https://v2.tauri.app/distribute/)
- [Tauri AppImage guidance](https://v2.tauri.app/distribute/appimage/)
- [Tauri Debian guidance](https://v2.tauri.app/distribute/debian/)
- [GitHub artifact attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations)
