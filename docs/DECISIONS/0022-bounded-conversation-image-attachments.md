# ADR 0022: Bounded conversation image attachments

- Status: Accepted
- Date: 2026-07-22

## Context

Milestone 15B needs visual context in new, resumed, and forked Codex turns
without exposing native paths to React or inventing generic file support. A
normal desktop drop can reveal source paths, a browser-supplied MIME label is
not trustworthy, and retaining arbitrary user files would create a new local
data store. The reviewed Codex CLI 0.145.0 turn schema and official app-server
documentation support `localImage` user input, but do not define a generic
local-file input or guarantee that a local path is no longer needed when the
initial `turn/start` response arrives.

## Decision

Milestone 15B introduces a process-local `ConversationAttachmentService` and
five fixed commands for status, native picking, dragged-byte staging, Linux
native-drop claiming, and cancellation:

- Only PNG and JPEG are supported. PDF, text, SVG, animated/unknown images, and
  generic binary files are refused instead of being presented as Codex
  capabilities.
- Tauri's default path-bearing file-drop event is disabled. React may read
  explicitly dropped browser `File` bytes, but never receives a native path.
  On Linux, where WebKitGTK can return an empty HTML `FileList` for a real file-
  manager drop, a GTK signal captures at most five file URIs (four allowed plus
  one overflow sentinel) into a 30-second, process-local, one-use native slot.
  The drop zone claims that slot through a path-free fixed command; Rust
  performs the same path and content validation and React receives only the
  normalized snapshot. Native picker and captured drop source paths stay in
  Rust.
- Rust validates safe display names, real content type and structure,
  dimensions, 4 MiB per-file, four files, and 16 MiB aggregate. Picker sources
  must be regular non-symlink files opened with `O_NOFOLLOW`; declared browser
  type must match validated content.
- Validated bytes are copied beneath QuireForge app data in a mode-`0700`
  staging directory. Mode-`0600` files use app-owned UUIDv7 names unrelated to
  source names. React receives only project-bound UUIDv7 IDs and normalized
  name, source class, MIME, size, and dimensions.
- Drafts live only in process memory, expire after 15 minutes, and are consumed
  once by an explicit start, resume, or fork. Cancel, project switch/detach,
  failed send, expiry, and startup reconciliation remove only recognized staged
  copies; no operation deletes a selected source file.
- Claim reopens the staged file with `O_NOFOLLOW` and rechecks device, inode,
  size, type, and dimensions. Native code then maps the private path to the
  documented `{ "type": "localImage", "path": ... }` input; staged paths never
  cross IPC or enter QuireForge SQLite.
- A successfully started turn retains its claimed copies until the normalized
  conversation becomes completed, interrupted, blocked, or failed. This is a
  conservative lifecycle rule because app-server documents an initial start
  response followed by streaming, not an earlier local-path consumption point.
- Routine tests use deterministic temporary images and mock app-server
  processes. They do not read user files, inspect personal Codex state, or make
  a live/billable model call.

## Consequences

- QuireForge can supply bounded visual context through a documented Codex input
  without creating a general filesystem path bridge.
- Drag/drop copies bytes through the fixed IPC boundary and is therefore
  intentionally small when the webview supplies a `File`. The Linux native
  fallback copies from a short-lived Rust-held path and exposes no path through
  IPC. Larger or generic file workflows require a separate supported model
  interface and threat decision.
- Private copies may remain until terminal polling or the next application
  startup if the webview disappears. They are app-owned cache-like data, never
  a credential or source-of-record store.
- Interactive native picker and drag/drop behavior on supported Wayland and X11
  sessions remains a Milestone 15C verification obligation.
