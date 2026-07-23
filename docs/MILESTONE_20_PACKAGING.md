# Milestone 20 Packaging and Release-Automation Report

Status: complete and verified locally on `feat/milestone-20-packaging`. No
branch was pushed or merged, no GitHub workflow was dispatched, no release or
attestation was created, neither package was installed on the host, website
download data remains inactive, and no deployment or hosting setting changed.

## Outcome

QuireForge now produces local x86_64 `0.1.0-beta.1` AppImage and Debian
candidates from a pinned Ubuntu 22.04 baseline. Both packages carry the
permanent executable, package, desktop-entry, application, icon, and AppStream
identity. An exact checksum file and strict JSON manifest accompany the
artifacts.

Milestone 20 establishes a repeatable candidate pipeline and guarded release
source. It does not claim a published or generally supported application.
Milestone 21 remains responsible for final supported-platform QA, approval,
publication, public installation guidance, website activation, download
verification, and rollback.

## Baseline and supply-chain controls

- Ubuntu 22.04 is pinned by image digest and supplies WebKitGTK 4.1 with GLIBC
  2.35.
- Node 22.22.1 and Rust 1.95 container stages are pinned by image digest.
- Rust 1.95 is the honest project minimum because the locked SQLite dependency
  uses `cfg_select!`, stabilized in Rust 1.95.
- Tauri's AppRun, linuxdeploy, GTK/GStreamer plugins, AppImage tool, and
  AppImage runtime are recorded in a six-entry SHA-256 manifest.
- The fetcher rejects non-HTTPS sources, oversized responses, missing files,
  and checksum drift. Tauri's documented three-byte linuxdeploy AppImage-marker
  clearing is accepted post-build only when reconstructing those exact bytes
  yields the reviewed hash.
- AppImage repacking uses a checksum-pinned runtime rather than an unreviewed
  downloaded runtime.
- GitHub workflow dependencies use immutable commit revisions and minimum
  permissions.

The Ubuntu package repositories intentionally provide current security updates
for the pinned base rather than a byte-frozen archive snapshot. Artifact
normalization is deterministic from the same raw bundles and source epoch; the
accepted repeated pass produced identical hashes for both packages, the
manifest, and `SHA256SUMS`.

## Package contract

The normalizer converts Tauri's raw bundles into:

```text
QuireForge-0.1.0-beta.1-x86_64.AppImage
quireforge_0.1.0~beta.1_amd64.deb
release-manifest.json
SHA256SUMS
```

Debian prereleases use `~` so the final `0.1.0` orders after the beta. The
Debian package name is exactly `quireforge`, while release architecture names
use `x86_64` and Debian control/file naming uses `amd64`.

Both payloads contain:

- executable `usr/bin/quireforge`;
- desktop entry
  `usr/share/applications/io.github.codeframe78.QuireForge.desktop`;
- icon name `quireforge`;
- AppStream component `io.github.codeframe78.QuireForge`;
- Apache-2.0 package license and unofficial-project description; and
- no maintainer scripts, credential store, Codex state, project content, or
  package-owned user metadata.

## Validation evidence

The accepted local pass completed:

- six dependency-free package-contract unit tests;
- exact release-manifest, artifact-set, sizes, hashes, and checksum-file
  validation;
- Debian control, dependency, payload, desktop-file, AppStream, md5sums, and
  no-maintainer-script inspection;
- AppImage extraction, canonical desktop/AppStream layout, and executable
  inspection;
- `readelf` verification that the packaged executable does not require GLIBC
  newer than 2.35;
- a disposable Debian install → upgrade from `0.0.0` → uninstall sequence;
- proof that uninstall removes package-owned executable/desktop files while
  retaining an attached-project sentinel and QuireForge metadata sentinel;
- stable visible X11 launches for the extracted Debian executable and
  AppImage, using isolated home/XDG roots, no Codex credentials, and no Vite
  server;
- a current-session AppImage pixel capture that reached the complete workspace
  with `Native IPC verified`, `v0.1.0-beta.1`, and more than fifteen thousand
  colors rather than the reported black frame;
- the complete repository gate: 152 desktop unit tests, six website unit
  tests, and 174 runnable Rust tests, with three deliberate live probes
  ignored;
- all 32 desktop and eight website Playwright desktop/mobile scenarios;
- website and desktop production builds and generated-artifact budgets; and
- fresh high-severity Node and warning-denying RustSec audits with no findings.

The package visual probe found and corrected a real prerelease issue:
TypeScript's bootstrap schema originally rejected prerelease versions, causing
the packaged native response to fall back to browser-preview identity
`v0.0.0`. The schema and shared Rust/TypeScript fixture now accept and verify
`0.1.0-beta.1`; the packaged UI reports native IPC and the correct beta.

## Release and website boundary

The release workflow defaults to verify-only and runs only by manual dispatch.
Its package job is read-only and uploads review artifacts. Publication requires
an exact approved operation, confirmation phrase, tag ref, clean manifest,
matching `v{version}` tag, protected `quireforge-release` environment,
attestation, and scoped publish job.

The website has a typed unavailable/published union, but the committed value is
deliberately unavailable with `release: null`. No package URL, hash, size, or
release claim is exposed. The validator rejects accidental activation during
Milestone 20.

## Remaining Milestone 21 work

- Approve the exact beta source/tag and publication operation.
- Complete final supported-platform and install-from-download QA.
- Configure/confirm the protected release environment and attestation support.
- Publish the prerelease without replacing artifacts under an existing
  version.
- Independently download and verify the manifest, checksums, AppImage, and
  Debian package.
- Review and approve public installation, known-limitations, and rollback copy.
- Activate typed website download data and deploy it under separate approval.
- Rehearse the public rollback path without deleting user data or unrelated
  hosting state.
