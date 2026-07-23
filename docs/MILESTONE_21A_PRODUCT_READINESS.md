# Milestone 21A Product-Readiness Report

Status: complete and verified locally on
`feat/milestone-21-product-readiness`. No branch was pushed or merged, no
package or release was published, no website or hosting setting changed, no
live login or account-usage read was performed, and no reset credit or personal
Codex state was mutated.

## Outcome

QuireForge now starts with a dedicated Codex account gate and exposes the
workspace only after the normalized account state grants access. The
authenticated desktop opens on an original responsive Home surface with a
stable left navigation, central task/project/action region, and right
recent-thread and usage rail. Internal roadmap milestone labels no longer
appear in product navigation or workspace copy.

This closes the local product-readiness portion of Milestone 21. It does not
publish the beta or activate public downloads. Final package/platform QA,
release approval, artifact publication, website metadata activation, download
verification, and rollback remain Milestone 21B.

## Authentication and startup

The gate uses only the existing Codex-owned authentication service:

- an existing normalized authenticated state enters the workspace;
- an unauthenticated ChatGPT account offers the official browser callback or
  device-code flow;
- pending login retains only the allowlisted short-lived handoff;
- unavailable and browser-preview states stay outside the workspace and offer
  an honest retry; and
- logout remains a separate two-step action.

Before access is granted, the React startup path runs only desktop bootstrap,
Codex runtime discovery, and account status. It does not begin project,
conversation, active-task, session, terminal, integration, Git, worktree, or
usage reads. QuireForge still neither receives nor stores passwords, OAuth
tokens, API keys, email addresses, account identifiers, or Codex credential
files.

Codex may report an already configured API-key or managed provider as
authenticated, or report that no additional OpenAI login is required.
QuireForge respects that supported normalized state rather than fabricating a
ChatGPT login requirement for a provider that Codex says does not need one.
For ordinary OpenAI use, ChatGPT browser/device login is the presented
onboarding path.

## Remaining-usage contract

The native `CodexUsageService` invokes only the documented
[`account/rateLimits/read`](https://learn.chatgpt.com/docs/app-server#6-rate-limits-chatgpt)
method through the supervised app-server process. The fixed
`codex_usage_status` and `codex_usage_refresh` commands accept no frontend
input.

Rust returns only:

- up to eight sanitized meter identifiers and labels;
- primary and secondary windows;
- integer used and remaining percentages;
- bounded window duration and Unix reset time; and
- a coarse limit-reached boolean from a reviewed closed enum.

Plan type, credit balance, spend controls, account metadata, reset-credit
inventory and IDs, and raw protocol fields are discarded. Percentages outside
0–100, impossible timestamps/durations, duplicate or malformed identifiers,
control or bidirectional label characters, unknown reached-state enums, and
oversized meter sets fail closed. Missing windows produce `not-metered`;
transport, RPC, timeout, or protocol failures produce `unavailable`. The UI
does not calculate tokens, predict quota, or present a reset-credit action.

## Product interface

The approved visual reference informed the information hierarchy without
copying another product's brand:

- permanent QuireForge identity, palette, typography, icon, and disclaimer;
- Home, New task, Projects, Threads, Scheduled, and Integrations as product
  navigation;
- Files, Changes, Worktrees, and Terminal as project-workspace navigation;
- a large central task affordance, local-project cards, and fixed quick
  actions;
- a separate recent-thread rail backed by reference-only session state;
- compact sidebar usage plus a full account usage card; and
- responsive desktop/mobile, light/dark, reduced-motion, forced-color, focus,
  overflow, and semantic accessibility behavior.

All established workspaces remain available below Home and retain their
existing fixed-purpose IPC and confirmation boundaries. Home summarizes and
navigates; it does not add a generic process, filesystem, network, integration,
or account command.

## Verification

The accepted local implementation includes:

- 157 passing desktop component/contract tests and six passing website tests;
- 178 passing runnable Rust tests, with three explicit live probes ignored;
- all 34 desktop Playwright scenarios across desktop and mobile profiles with
  no automatically detectable accessibility violations;
- strict Rust and TypeScript usage fixtures and contract tests;
- adversarial native normalization tests for percentages, timestamps, labels,
  identifiers, enum drift, meter bounds, account-metadata exclusion, and honest
  unmetered behavior;
- a component test proving no workspace/account-data loader runs before
  authentication;
- component and browser coverage for the signed-out gate, authenticated Home,
  usage refresh, milestone-label absence, existing session/project/task
  workflows, responsive behavior, and accessibility;
- reviewed Codex 0.145.0
  `v2/GetAccountRateLimitsResponse.json` schema selection and manifest hashes;
- production frontend and native compilation; and
- repository, formatting, lint, secret, link, and generated-artifact checks.

A dark desktop and light mobile visual inspection confirmed the intended
three-region hierarchy, responsive stacking, readable usage meters, and
complete QuireForge identity. The rebuilt unbundled release executable then
opened a stable native X11-backed window against isolated home/XDG roots,
painted the signed-out gate after an eight-second settle, and produced neither
a black frame nor refused-loopback evidence. Deterministic tests used sanitized
fixtures; they made no billable model call and did not access a personal
account.

## Remaining Milestone 21B work

- Select and approve the exact beta source revision and release operation.
- Rebuild final package candidates from that approved clean revision.
- Complete final supported-platform and install-from-download QA.
- Publish immutable artifacts, manifest, checksums, and provenance only after
  the separate release approval.
- Independently verify public downloads and installation guidance.
- Activate the typed website download record and deploy it under its own
  approval.
- Verify the public rollback path without deleting user data or unrelated
  hosting state.
