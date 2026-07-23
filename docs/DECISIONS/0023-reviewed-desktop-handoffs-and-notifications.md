# ADR 0023: Reviewed desktop handoffs and privacy-safe notifications

- Status: Accepted
- Date: 2026-07-22

## Context

Milestone 15C needs useful Linux desktop integration without turning the
privileged webview into a generic opener or copying conversation content into
the operating system notification service. A frontend path or executable
argument would create a confused-deputy boundary. Notification titles or
bodies derived from prompts, project names, paths, model output, or raw errors
could disclose private work on the lock screen or notification history.

Milestone 15A already validates a native-picker selection before returning a
bounded preview. Tauri's official opener and
[notification plugin](https://v2.tauri.app/plugin/notification/) provide the
desktop mechanisms, so 15C can add fixed native operations without granting
either plugin directly to the webview.

## Decision

- Every ready safe preview receives one process-local UUIDv7 open action. Rust
  retains the project ID, attachment-relative path, device/inode identity, and
  five-minute creation time. React receives only the opaque action plus the
  already-reviewed relative display name.
- The open action is one-use, project-bound, capped to 16 pending entries, and
  removed on replacement, clear, expiry, or successful claim. The handoff
  command accepts no path, URL, application, executable, argument, MIME type,
  or working directory.
- Before opening, Rust reloads the attachment, revalidates directory identity,
  canonical containment, regular non-symlink state, the descriptor-resolved
  path, and the selected device/inode. Identity drift consumes the action and
  fails closed. A failed opener call may restore the same action only when no
  newer action exists for that project.
- The UI names the exact attachment-relative file and the allowlisted
  destination class, `System default application`, then requires a separate
  confirmation. No application chooser, shell command, custom editor path,
  or generic opener IPC is exposed.
- A fixed notification command accepts only an app-owned conversation UUIDv7.
  Native conversation state must freshly identify a pending approval,
  completion, block, or failure. Interrupted tasks and all other states are
  ineligible.
- Notifications are suppressed while the QuireForge window is focused and
  deduplicated by the native approval or terminal identity. Their title/body
  are fixed strings selected from a closed state enum; project names, prompts,
  paths, account/model data, output, diagnostics, and raw protocol payloads are
  never interpolated.
- The official notification plugin is initialized only for Rust use. The main
  webview capability remains empty, so React cannot call the notification or
  opener plugins directly. Notification delivery is best-effort and never
  changes conversation state.
- Manual desktop delivery uses a disabled-by-default Cargo feature and one exact
  native process flag. The probe reuses the fixed completed-task copy, accepts
  no caller content, registers no Tauri command, and is removed again by the
  normal build before acceptance.
- Routine tests use temporary files, sanitized fixtures, deterministic mock
  conversations, and a mocked notification state machine. They do not open a
  user's files, inspect personal Codex state, or start a live/billable model
  turn.

## Linux verification boundary

Automated repository, browser, and release checks cover the strict IPC and UI
contracts. Native launch, picker, handoff, notification-service availability,
and desktop application identity require display-session evidence. Wayland and
X11-family results must name the actual backend used; an XWayland smoke test
does not become a claim that a separate GNOME Xorg login was tested.

## Consequences

- QuireForge can hand one already-reviewed project file to its system default
  application without exposing a reusable project-relative or absolute-path
  opener.
- Desktop alerts can draw attention to background work without copying private
  task content into the shell notification surface.
- An atomic file replacement between preview and handoff requires a new
  preview. This is intentional identity continuity, not an automatic retry.
- The system default application remains an external trust boundary. Its
  identity and behavior are controlled by the desktop association, visibly
  disclosed before the handoff, and never represented as a QuireForge-owned
  renderer.
- Full X11-session acceptance remains open when only XWayland is available;
  compatibility records must retain that distinction.
