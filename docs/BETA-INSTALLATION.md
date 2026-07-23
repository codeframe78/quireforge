# QuireForge Beta Installation

Status: reviewed beta 2 copy. The public GitHub prerelease is a secondary
artifact/provenance record; there is no owner-hosted QuireForge download yet.
Use these instructions only after the production Downloads page names an
approved version and provides the package, `SHA256SUMS`, and release manifest
from the same approved origin.

## Initial beta target

- x86_64 Ubuntu 22.04 or newer;
- GNOME on Wayland or X11;
- a compatible Codex CLI available to QuireForge;
- Git available for repository workflows; and
- either the AppImage or Debian package published by QuireForge.

The first beta is not a general Linux-distribution or desktop-environment
support claim. Arm64, non-Ubuntu distributions, non-GNOME desktops, automatic
updates, and distribution repositories remain outside the initial contract.

## Verify the download

Download `SHA256SUMS` and exactly one package into the same directory. From
that directory, run:

```bash
sha256sum --check --ignore-missing SHA256SUMS
```

Continue only if the selected filename reports `OK`. A missing entry, checksum
failure, unexpected filename, or package obtained from another origin is a
hard stop. The release manifest provides the source, builder, size, and hash
record for independent review.

## AppImage

The approved filename is:

```text
QuireForge-0.1.0-beta.2-x86_64.AppImage
```

After checksum verification:

```bash
chmod 0755 QuireForge-0.1.0-beta.2-x86_64.AppImage
./QuireForge-0.1.0-beta.2-x86_64.AppImage
```

If the system does not provide FUSE, the reviewed AppImage runtime supports:

```bash
./QuireForge-0.1.0-beta.2-x86_64.AppImage --appimage-extract-and-run
```

The AppImage is not registered with `apt`. Replace it manually when a later
approved version is published; removing the file uninstalls that copy.

## Debian package

The approved filename is:

```text
quireforge_0.1.0.beta.2_amd64.deb
```

After checksum verification, use `apt` so Ubuntu can resolve the declared GTK
and WebKitGTK dependencies:

```bash
sudo apt install ./quireforge_0.1.0.beta.2_amd64.deb
```

The dot in the downloaded filename is intentional. After installation,
`dpkg-query -W -f='${Version}\n' quireforge` reports the internal Debian version
`0.1.0~beta.2`, whose tilde keeps it ordered before the future stable `0.1.0`.

Remove the package with:

```bash
sudo apt remove quireforge
```

Package removal deletes only package-owned application files. It does not
delete attached directories, Git repositories or worktrees, uncommitted
changes, Codex configuration/authentication/sessions, or QuireForge user
metadata. Metadata cleanup is a separate operation and is not part of package
removal.

## First launch

QuireForge checks the local Codex runtime and normalized account state before
showing workspace data. For ordinary OpenAI use, Codex owns the ChatGPT
browser/device login. QuireForge does not receive or store the password, token,
email address, or account identifier. An already configured API-key or managed
provider may be reported by Codex as requiring no additional login.

If Codex is unavailable or incompatible, QuireForge stays at an honest runtime
or account gate. It does not fall back to a local web server, scrape ChatGPT,
or fabricate account access.

## Initial beta limitations

- QuireForge is an unofficial community project and is not made, endorsed,
  supported, or distributed by OpenAI.
- Codex and Git are external prerequisites; QuireForge does not bundle their
  credentials or data.
- Remaining usage appears only when the documented Codex rate-limit method
  returns a valid meter. Unmetered and unavailable states are not estimated.
- The AppImage has no automatic update channel. The Debian package is not
  published through an APT repository and has no repository signing metadata.
- Integration availability depends on the installed Codex version, account,
  policy, platform, and supported interfaces.
- Scheduled-task discovery is read-only, concurrency is bounded, and several
  advanced integration, worktree, and file-operation paths deliberately remain
  unavailable.

Use the production Downloads page as the availability authority. Do not install
a package solely because its filename matches this document.
