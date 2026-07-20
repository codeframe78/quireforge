# ADR 0008: Native-Owned Conversation Runtime

- Status: Accepted for Milestone 7A
- Date: 2026-07-19

## Context

QuireForge must start Codex work only in a verified attached directory, expose a
small stable frontend contract, stop the exact active task, and keep Codex
authoritative for conversation content. Raw app-server messages, paths, Codex
identifiers, prompts, command output, and approval requests are too sensitive or
unstable to pass through the webview unchanged.

Directory detach, archive, or relink during a task could also break the binding
between the user's selected project and the native process cwd. Cancellation is
unsafe if the frontend supplies the thread or turn identity to interrupt.

## Decision

Milestone 7A uses one serialized native `ConversationService` as the MVP owner
of an app-server process and its active Codex thread and turn. Before start it:

1. reserves the application project against detach, archive, and relink;
2. reloads and revalidates the active directory association;
3. discovers the model catalog on the same initialized app-server process; and
4. sends fixed thread/turn requests with the exact verified cwd and explicit
   model, reasoning, sandbox, and approval settings.

The frontend provides a project ID, bounded prompt, and closed control enums. It
never provides a cwd, command, environment, Codex thread ID, or Codex turn ID.
The service validates UUIDv7 response and notification identities, correlates
all lifecycle events with its native-owned IDs, and emits bounded normalized
events. Agent messages and reasoning summaries are allowed; raw reasoning,
commands, command output, file changes, diffs, paths, and unreviewed item fields
are excluded from this checkpoint.

Polling waits briefly for the first event, drains only immediately available
events, and returns at most 32 protocol events per call. This bounds latency and
memory without creating an unbounded background queue. Interruption sends
`turn/interrupt` with only the native-owned exact IDs, then closes and waits for
the child. An approval server request moves the task to a stable blocked state
and closes the process; Milestone 7A never guesses or auto-approves.

QuireForge SQLite stores only project association, Codex thread and active-turn
references, selected controls, lifecycle status, and timestamps. It never stores
the prompt, transcript, reasoning, command output, diffs, credentials, or Codex
session content.

## Consequences

- The verified directory remains bound to the entire active task.
- Exact interruption cannot be redirected by webview-supplied Codex IDs.
- Protocol drift and correlation failures fail closed with stable diagnostics.
- The MVP supports one active conversation at a time; concurrency is deferred
  to the worktree and parallel-work milestone.
- Crash recovery, resume/fork/archive, and reconciliation of Codex-authoritative
  sessions remain Milestone 8.
- Rendering, composer UX, task controls, and accessibility behavior remain
  Milestone 7B.
- Full approval presentation and decision handling remain Milestone 9.

## Alternatives considered

- **Pass raw app-server JSON to React:** rejected because it leaks unstable and
  sensitive fields and makes the webview a protocol/security boundary.
- **Let React supply cwd or interrupt IDs:** rejected because it permits path
  substitution and stopping a different task.
- **Persist transcripts in application SQLite:** rejected because Codex already
  owns session content and QuireForge must not create a second sensitive store.
- **Auto-approve or infer approval decisions:** rejected because scope and user
  intent cannot be preserved without the dedicated approval workflow.
