# ADR 0025: Expose a Read-Only Scheduled Task Catalog

- Status: Accepted
- Date: 2026-07-22
- Decision owners: Project owner and maintainers

## Context

Milestone 17 must implement only scheduling capabilities that the installed
Codex version exposes through documented or reviewed supported interfaces.
Inspection of Codex CLI `0.145.0` established the following boundary:

- stable app-server `plugin/read` returns optional scheduled task templates for
  a named plugin;
- each template contains a key, display name, prompt, and an hourly, daily,
  weekdays, or weekly schedule;
- the stable app-server request set has no create, update, enable, run, pause,
  or delete method for scheduled tasks;
- the supported `codex plugin` CLI surface has no scheduled-task management
  command; and
- current official Codex documentation assigns scheduled-task management to
  official ChatGPT web and desktop surfaces and requires the official desktop
  application for local task execution.

Plugin task names and prompts are supply-chain-controlled content. Marketplace
roots and plugin loader paths can contain private filesystem information.

## Decision

Milestone 17A adds only a read-only scheduled task catalog:

- QuireForge queries `plugin/read` only for installed, enabled plugins already
  discovered through the supported Codex plugin catalog.
- Marketplace roots and raw plugin lookup identifiers remain native-only.
- The native adapter strictly validates response shapes, applies item and text
  bounds, removes unsafe display controls, normalizes stable identifiers, and
  exposes only task metadata required by the UI.
- Plugin prompts are treated as untrusted inert text. QuireForge does not
  execute, interpolate, submit, persist, or interpret them as instructions.
- The webview receives a versioned normalized catalog containing source plugin
  identity, a bounded prompt preview, truncation state, and a typed schedule.
- The Scheduled workspace states that QuireForge cannot create, edit, enable,
  run, pause, or delete scheduled tasks.
- Discovery failure degrades the scheduled-task capability without exposing raw
  protocol errors or preventing unrelated integration catalog sources from
  loading.

QuireForge does not call private ChatGPT or web endpoints, scrape official
clients, deep-link to undocumented management routes, operate a local
scheduler, or claim hosted scheduling support.

## Approval boundaries

This decision authorizes repository-local contract, adapter, fixture, test, UI,
and documentation changes for read-only discovery. It does not authorize
plugin installation or removal, connector authorization, task creation or
execution, external account changes, GitHub publication, packaging, deployment,
or release publication.

Any future scheduled-task mutation or execution requires a separately reviewed
supported interface, threat model, owner approval, and architecture decision.

## Consequences

- Users can inspect task templates supplied by installed plugins without
  QuireForge becoming a scheduling authority.
- The integration catalog advances to schema version 2 while retaining its
  existing read and refresh IPC commands.
- A missing, malformed, or incompatible plugin response produces a bounded
  diagnostic and no unsafe task entry.
- Hosted scheduling, local execution, and all task mutations remain deferred.
