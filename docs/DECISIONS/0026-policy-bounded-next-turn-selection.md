# ADR 0026: Keep Agent-Directed Selection App-Owned and Next-Turn Only

- Status: Accepted
- Date: 2026-07-22
- Decision owners: Project owner and maintainers

## Context

Codex CLI 0.145.0 exposes the documented pieces needed for an app-owned
selector:

- `model/list` supplies account-visible model and reasoning choices;
- `thread/start.dynamicTools` registers a closed client-owned tool;
- `item/tool/call` invokes that tool with correlated native thread, turn, call,
  and request identity; and
- `turn/start` accepts model and reasoning overrides for the new turn.

These interfaces do not let an executing turn replace its own model. A selector
request may also be influenced by untrusted task or integration content. Silent
automatic escalation, stale catalog state, repeated requests, bypassing a user
lock, or writing Codex configuration would violate QuireForge's ownership and
credential boundaries.

## Decision

Milestone 18 implements `ModelSelectionService` as an app-owned policy boundary:

- A conversation registers only the closed `quireforge_model_selector` tool.
  The tool supports normalized inspection and one bounded request action.
- React never receives native request, thread, turn, or call identity and cannot
  send raw app-server payloads.
- Manual ownership rejects Codex requests. Recommend ownership stages a visible
  recommendation that requires user acceptance. Automatic ownership requires
  explicit opt-in and a model allowlist or reasoning ceiling.
- A user lock and a later manual choice take precedence. Only one request
  attempt is accepted per turn.
- A valid Codex request remains ephemeral until the current turn completes
  successfully. It then becomes a pending next-turn choice with bounded
  rationale and provenance.
- Resume constructs the next `turn/start` only after refreshing `model/list`
  and revalidating the pending choice. A stale or newly disallowed automatic
  request is discarded without changing the effective selection.
- Effective and pending choices are stored separately in QuireForge's metadata
  database. The stored fields contain no prompt, transcript, raw tool
  arguments, protocol identity, account identity, credential, or Codex-owned
  configuration.
- If exact dynamic-tool registration is rejected, QuireForge retries the thread
  start without the tool and exposes an honest `recommendation-only`
  availability state. It does not use a private endpoint or automate an
  official website.

## Approval boundaries

This decision authorizes repository-local contracts, migration, native
lifecycle code, typed IPC, UI, deterministic tests, documentation, and focused
local commits. It does not authorize a billable model call, authentication
change, Codex configuration edit, GitHub publication, merge, package,
deployment, or release.

## Consequences

- The current turn always retains its original effective model and reasoning.
- A pending choice can survive application restart without preserving runtime
  process ownership.
- Automatic mode is useful only inside limits explicitly selected by the user.
- Compatibility failure is visible and loses automation rather than weakening
  the trust boundary.
- Every future expansion of selector authority requires a separately reviewed
  policy and supported interface.
