# ADR 0021: Safe project file previews

- Status: Accepted
- Date: 2026-07-22

## Context

Milestone 15 needs useful local file review without turning the privileged
webview into a general filesystem reader or active-document host. A path sent
from React could escape an attached project, race a symlink, disclose a local
absolute path, or act as a confused deputy. Rendering HTML, SVG, animated
images, or PDF bytes in the webview would also broaden the active-content and
resource-loading boundary.

The project service already owns attachment identity and accessibility
revalidation. The Tauri dialog plugin already provides a native file picker.
The preview path can therefore remain native from selection through validation,
while React receives only a closed, bounded presentation contract.

## Decision

Milestone 15A introduces a stateless native `FilePreviewService` and one fixed
`file_preview_pick` command:

- React supplies only one canonical app-owned UUIDv7 project ID. Native code
  rejects malformed identity before opening the picker; no frontend path,
  content type, size, renderer, URL, or arbitrary filesystem operation exists.
- The service reloads the project attachment, rechecks stored directory
  identity and current readable accessibility, canonicalizes the picker result,
  requires containment, rejects symlinks and non-regular files, opens with
  `O_NOFOLLOW`, and compares the opened device/inode with the selected file.
- The validated canonical attachment root remains open as a directory file
  descriptor while the relative selection is opened through that descriptor.
  Replacing or renaming the root cannot redirect the read to a new directory.
- The opened Linux file descriptor is resolved again through `/proc/self/fd`
  and must still identify the exact canonical path under the attachment. A
  parent-directory retarget or rename race therefore fails closed.
- Source files are capped at 8 MiB. Text output is valid UTF-8, normalized to
  LF, stripped of unsafe controls and bidi overrides, and capped at 128 KiB and
  2,000 lines. Only the attachment-relative display path crosses IPC.
- PNG and JPEG previews are capped at 4 MiB, 8,192 pixels per dimension, and
  16 million pixels. Dimensions and type are checked before and after the full
  bounded read; APNG is refused. The webview receives only a bounded
  `data:image/png` or `data:image/jpeg` URL under an explicit `img-src data:`
  CSP allowance.
- PDF is recognized only by its header and represented as metadata. PDF bytes
  are not sent to or rendered inside the privileged webview. HTML/SVG source
  may appear only through the normalized inert-text path; active markup, video,
  audio, archives, binary files, and unknown formats remain unsupported.
- The response is a strict versioned snapshot with closed state, kind,
  rendering, MIME, and diagnostic enums. Empty and unavailable responses carry
  no content. Preview state is process/UI transient and is never persisted.
- Browser preview reports that local selection is unavailable and never uses a
  browser file input or synthetic local content.

Drag/drop and conversation attachments remain Milestone 15B. Notifications,
editor/open-with expansion, and Wayland/X11 verification remain Milestone 15C;
they may not bypass this boundary.

## Test-state isolation

Routine tests use temporary directories and sanitized Rust/TypeScript fixtures.
They cover containment, symlink and binary refusal, malformed project identity,
dimension limits, full-file APNG detection, text normalization, strict bridge
input/output, browser honesty, responsive layout, and accessibility. They do
not inspect a user's files, access personal Codex state, open an external
application, or make a model call.

## Consequences

- A selected file can be reviewed without exposing an absolute path or general
  read primitive to React.
- The initial format set is deliberately small. PDF rendering, active
  HTML/SVG, image animation, syntax highlighting, and larger files require a
  separate parser, isolation, and resource-budget decision.
- `data:` is allowed only for image sources; scripts, frames, objects, remote
  resources, and arbitrary navigation remain excluded by the production CSP.
- Readable attached directories can be previewed even when they are unsuitable
  for writable Codex tasks. Attachment identity changes fail closed.
