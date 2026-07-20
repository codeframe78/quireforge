# ADR 0009: Native Session Lifecycle and Recovery

- Status: Accepted for Milestone 8A
- Date: 2026-07-19

## Context

QuireForge must resume, fork, archive, restore, and reconcile its Codex threads
without making the webview a Codex protocol boundary or creating a second
transcript store. A stale active-turn reference after a crash must not reserve a
project indefinitely. Lifecycle operations also cannot trust a frontend path,
Codex thread ID, rollout path, turn history, or runtime workspace root.

## Decision

Milestone 8A extends the serialized native `ConversationService` rather than
adding a competing process owner. Every lifecycle command accepts only an
application UUIDv7 reference. Resume and fork additionally accept a bounded
prompt that is forwarded to Codex and never persisted.

The native service reloads the reference-only SQLite row, revalidates the
attached project identity and exact cwd, initializes one supervised app-server
process, and uses only the reviewed stable lifecycle methods. It reads the
native-owned thread before mutation, then sends the stored model, reasoning,
sandbox, and approval controls. It never sends rollout paths, supplied history,
configuration objects, or runtime workspace roots. Response thread IDs and cwds
must match the expected native identity; a fork must return a distinct UUIDv7
with the expected parent.

Codex remains authoritative for archive state and thread availability. Session
reconciliation batches exact verified cwd filters through bounded current and
archived `thread/list` pages and matches only IDs already owned by QuireForge.
Unrelated Codex threads are never imported. React receives app IDs, project IDs,
parent app references, controls, timestamps, and stable states only—never Codex
IDs, paths, previews, transcripts, raw status objects, or protocol payloads.

SQLite schema version 3 adds only an optional parent application reference and
archive timestamp. On database open, stale `thread-started`, `running`, or
`stopping` rows become `interrupted` and lose their active-turn reference.
In-memory execution reservations start empty, so a crashed task cannot remain
active after restart. Archive and restore mutate metadata and Codex lifecycle
state only; they never delete thread content or project files.

## Consequences

- One serialized owner prevents lifecycle mutations from racing an active turn.
- Resume and fork remain bound to the originally attached, revalidated project.
- Startup recovery is conservative: it records interruption without claiming
  that the old process or turn is still alive.
- A missing authoritative thread is reported as a stable `missing` state; no
  alternative path or personal Codex thread is substituted.
- Title search, tabs, grouping presentation, and lifecycle controls remain the
  Milestone 8B frontend checkpoint.
- Approval decisions, command details, and diffs remain later milestones.

## Alternatives considered

- **Pass Codex thread IDs or paths through React:** rejected because the webview
  could redirect lifecycle operations and expose sensitive native identity.
- **Persist transcript/title content in QuireForge SQLite:** rejected because
  Codex owns session content and QuireForge needs only bounded references.
- **Treat stale rows as still running:** rejected because process ownership does
  not survive application restart.
- **Delete a thread when removing a UI reference:** rejected because archive,
  detach, and deletion are distinct operations and Milestone 8A implements no
  destructive deletion.
