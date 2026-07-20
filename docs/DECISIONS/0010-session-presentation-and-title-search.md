# ADR 0010: Session Presentation and Ephemeral Title Search

- Status: Accepted for Milestone 8B
- Date: 2026-07-19

## Context

QuireForge needs searchable session history, project grouping, fork lineage,
tabs, and lifecycle controls without becoming a second transcript store or
allowing the webview to address arbitrary Codex threads. A filtered
`thread/list` response is not a complete authoritative thread set: treating
non-matches as missing would corrupt lifecycle presentation and archive
reconciliation.

## Decision

Native reconciliation remains complete and unfiltered across the exact
revalidated project cwd set. An optional bounded title query runs as a second
`thread/list` projection on the same supervised process. Search results must be
a subset of the complete authoritative result, and both sets are intersected
with QuireForge-owned references before crossing IPC.

The normalized session contract exposes only a trimmed optional title of at
most 256 characters in addition to existing application IDs, project IDs,
parent-app lineage, controls, states, and timestamps. Titles are transient and
are never written to QuireForge SQLite. Codex IDs, cwd values, previews,
transcripts, raw status objects, and protocol payloads remain native-only.

React groups sessions by app-owned project and parent references. Open tabs,
selection, and continuation prompt text are short-lived view state. Resume,
fork, archive, and restore still invoke fixed native commands with exact
application IDs; archive remains distinct from deletion. Browser preview is
explicitly non-interactive.

## Consequences

- A title filter cannot cause an unmatched owned session to be labeled missing.
- Search costs up to two additional bounded current/archived list traversals,
  only when the user submits a query.
- Titles remain available for presentation without creating a duplicate
  content index or migration.
- Tabs do not persist transcript or prompt content and can be rebuilt from
  authoritative session summaries.
- Approval decisions and expandable real-time command/tool/process details
  remain Milestone 9 work behind a separately reviewed normalized event
  contract.

## Alternatives considered

- **Persist titles in SQLite:** rejected because the title is Codex-owned
  content and does not need a second local authority.
- **Filter the complete reconciliation request:** rejected because excluded
  rows would become indistinguishable from genuinely missing threads.
- **Send previews or raw thread records to React:** rejected because it expands
  the content and native-identity boundary without serving the milestone.
- **Implement tabs by loading transcripts:** rejected because lifecycle tabs
  need only normalized summaries and fixed actions.
