# QuireForge 0.1.0-beta.1 Release Notes

Status: superseded historical release. GitHub normalized the Debian asset's
tilde after upload, so its public name does not match the immutable beta 1
manifest and checksum record. Do not promote or install beta 1; use beta 2.
The production Downloads page remains the owner-hosted availability authority.

## Highlights

- Attach existing local project directories in place, retain their identity,
  and fail closed when paths, permissions, mounts, Git state, or project
  instructions change.
- Start and observe Codex conversations with distinct commentary, reasoning,
  plans, commands, approvals, file changes, results, interruption, and
  reference-only recovery states.
- Review normalized Git status and diffs, perform bounded confirmed Git
  mutations, and use up to four reserved worktree tasks without generic force,
  prune, or conflict-resolution behavior.
- Use app-owned native terminal tabs rooted only in freshly revalidated
  projects, with bounded output and explicit process-ending confirmation.
- Inspect supported apps/connectors, plugins, skills, MCP servers, and
  marketplaces through normalized Codex interfaces, with fixed reviewed
  lifecycle and authorization operations where supported.
- Preview bounded safe project files, attach reviewed PNG/JPEG conversation
  images, hand a revalidated file to its default application after
  confirmation, and receive privacy-safe fixed-copy notifications.
- Discover installed-plugin scheduled-task templates read-only and use the
  policy-bounded agent recommendation/automatic selector for the next turn.
- Enter through a Codex-owned account gate, use the original QuireForge
  Home/workspace hierarchy, and see documented remaining usage only when Codex
  returns a valid rate-limit meter.

## Security and data ownership

QuireForge owns only its migrated application metadata and temporary staged
copies required for reviewed workflows. It does not store Codex credentials,
scrape ChatGPT, import Codex sessions, collect connector passwords, expose raw
protocol or process identity to the webview, or silently move/copy an attached
project. Package removal is separate from QuireForge metadata cleanup and never
deletes attached projects, Git data, uncommitted changes, or Codex state.

The production webview has no broad plugin permission, no global Tauri object,
no asset-protocol scope, no remote frontend content, a narrowed CSP and
response-header policy, and fixed typed IPC. Node, Rust, GitHub Actions,
release-helper, package-identity, GLIBC, lifecycle, and checksum gates run
before publication.

## Initial platform contract

- x86_64 Ubuntu 22.04 or newer;
- GNOME on Wayland or X11;
- AppImage and Debian package formats;
- compatible external Codex CLI and Git installations; and
- no arm64, non-Ubuntu, non-GNOME, distribution-repository, or automatic-update
  promise in this beta.

See [QuireForge Beta Installation](BETA-INSTALLATION.md) for the staged
verification, installation, uninstall, first-launch, and FUSE-fallback copy.

## Known limitations

- QuireForge is unofficial and is not made, endorsed, supported, or
  distributed by OpenAI.
- Capability and integration availability varies with Codex version, account,
  region, workspace policy, administrator controls, platform, authentication,
  and supported interfaces.
- Remaining usage is unavailable or not metered when the documented Codex
  response does not provide a valid meter; QuireForge never estimates it.
- Scheduled-task templates are read-only. Generic scheduled-task creation,
  editing, execution, pause, enablement, and deletion are not supported.
- Active worktree tasks are capped at four. Generic force, prune, attached or
  external worktree deletion, automatic conflict resolution, and broad remote
  Git operations are unavailable.
- The AppImage has no automatic updater. The Debian package is not an APT
  repository release and has no repository signing metadata.
- The Tauri Linux dependency graph retains explicitly reviewed upstream GTK3
  maintenance/advisory exceptions. `freezePrototype` remains disabled because
  it prevents the verified Vite/React application from mounting; remote
  content and broad webview capabilities remain disabled as compensating
  controls.

## Verification boundary

The release is valid only when the production Downloads page provides the
versioned AppImage, Debian package, `SHA256SUMS`, and release manifest from the
approved owner-hosted origin and each file matches the reviewed release record.
A matching filename elsewhere is not sufficient.
