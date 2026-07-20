# ADR 0011: Native Approval and Detailed Activity Contract

- Status: Accepted and implemented through Milestone 9B
- Date: 2026-07-19

## Context

The Milestone 7 conversation boundary treated every app-server request as a
terminal block because QuireForge did not yet have a safe way to present or
answer approvals. Its activity events also disclosed only a coarse item kind
and status. Milestone 9 needs bounded real-time command, file, tool, and process
detail without passing raw app-server JSON, Codex identities, credentials,
terminal controls, diffs, tool arguments, or private paths into React.

Approval handling is security-sensitive. A response must correlate to the exact
native-owned thread, turn, request, and item. Persistent session acceptance,
execution-policy amendments, network-policy amendments, and unreviewed server
request types must not be accidentally presented as ordinary one-turn consent.

## Decision

The existing serialized `ConversationService` remains the sole owner of the
active app-server process. It accepts only the installed Codex 0.144.6 stable
command-execution, file-change, and permission approval methods. Each request
is strictly bounded and correlated to the active native thread and turn, then
projected to a new app-owned UUIDv7 approval ID and activity ID. Raw JSON-RPC,
Codex request/item/thread/turn IDs, tool arguments, diffs, and permission paths
do not cross IPC.

The public decision enum is limited to approve, decline, and cancel. Command
session acceptance and policy-amendment decisions are filtered out. Additional
per-command permissions and network targets are parsed and summarized before
approval. Permission approval echoes only the strictly parsed profile with
`scope = turn`; decline and cancel grant an empty profile. A file request that
asks for an unstable session-wide write root cannot be approved through this
contract. Cancel responds to the pending request before interrupting the exact
native-owned turn.

One pending approval may exist at a time. Its normalized presentation is
ephemeral memory state, while SQLite deliberately continues to record the
active turn as running. Existing startup recovery therefore marks a crashed
waiting turn interrupted and clears its active-turn ownership without storing
or replaying approval content. Stale IDs, unavailable decisions, mismatched
identities, duplicate requests, unsupported methods, and malformed profiles
fail closed.

Detailed activity uses app-owned stable IDs across item start, bounded output
or progress, and completion. Rust retains only the reviewed display subset:
sanitized command text, project-relative paths, tool/server/app labels, web
queries, status, and exit code. Terminal controls and bidirectional formatting
characters are removed; credential-shaped values are redacted; external or
escaping paths become `[outside project]`. Command output is held until a line
boundary so credential names and values split across protocol chunks can be
redacted together. Incomplete oversized lines are omitted, and an unfinished
tail is discarded at completion.

The TypeScript schema version advances to 2 and rejects unknown fields. A fixed
`conversation_approval_decide` command accepts only the app conversation ID,
app approval ID, and closed decision enum. React aggregates normalized activity
events by app activity ID into bounded selectable rows. The expanded panel
shows only the projected kind, detail, output tail, and exit code. The approval
card renders only decisions advertised for the exact pending request, submits
one decision at a time, and suspends polling during that state transition.

## Consequences

- Approval no longer terminates an otherwise healthy conversation.
- Project reservation remains active while user consent is pending.
- QuireForge still stores no transcript, command output, diff, approval body,
  credential, or permission profile in SQLite.
- Session-wide approval caches and policy amendments remain unavailable.
- Redaction is applied at the native boundary and repeated strict validation
  prevents raw fields from entering React state.
- Unknown or changed server-request shapes remain a compatibility failure, not
  implicit consent.
- The click/expand/decision interaction is covered by deterministic component
  and desktop/mobile browser checks; routine validation performs no real
  approval.

## Alternatives considered

- **Forward raw approval or item JSON to React:** rejected because the payloads
  contain unstable, sensitive, and authority-bearing fields.
- **Use Codex request IDs as UI IDs:** rejected because native protocol identity
  must stay behind the Tauri boundary.
- **Offer `acceptForSession` and amendment objects:** rejected because their
  persistence and policy effects exceed a bounded one-turn decision.
- **Persist pending approvals for restart:** rejected because a subprocess and
  request cannot be safely resumed after ownership is lost.
- **Render every output chunk immediately:** rejected because chunk boundaries
  can split credential names from their values and defeat field-aware
  redaction.
