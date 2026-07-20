# Threat Model

Status: initial Milestone 0 model with the Milestone 3 frontend/native boundary,
Milestone 4 Codex process adapter, Milestone 5 authentication controls, and
Milestone 6 native directory-attachment controls, and Milestone 7A native
conversation-runtime controls applied. It must be revisited before approvals,
integrations, packaging, and release milestones.

## Assets

- User source code, documents, Git history, and uncommitted changes.
- Codex conversations and app-owned project metadata.
- ChatGPT/Codex authentication and connector authorization state.
- Shell environment, local credentials, SSH agents, and developer tooling.
- Installed plugins, hooks, skills, MCP servers, and marketplace metadata.
- Release artifacts, checksums, update metadata, and GitHub workflows.

## Trust boundaries

1. React/webview to privileged Tauri command boundary.
2. Application core to Codex app-server/CLI processes.
3. Attached working root to all other filesystem paths.
4. Codex/project configuration to managed administrator requirements.
5. Integration metadata to executable plugin/MCP/hook behavior.
6. Local app to browser/OAuth/remote connector services.
7. Source repository to GitHub Actions runners and published artifacts.

The Milestone 3 shell exposes one versioned bootstrap command and grants the
main window no Tauri plugin permissions. Its shared Rust/TypeScript fixture and
runtime schema reject identity or shape drift before native data reaches UI
state.

Milestone 4 adds a fixed-purpose, argument-free runtime probe. Only the native
core can select the `codex` executable arguments or app-server methods. Protocol
lines, catalog size, strings, waits, and CLI output are bounded; numeric request
IDs are correlated; server requests fail closed; notification and error
payloads are not retained; and owned children are killed and waited on when
necessary. The React schema is strict and accepts only normalized capability,
model, version, backend, and diagnostic fields. This probe accepts no path,
prompt, arbitrary process, credential, configuration value, or other user
input.

Milestone 5 adds a closed login-method enum and otherwise argument-free account
commands. Rust alone retains the active login ID, filters account results to a
coarse account kind, reduces raw completion errors to stable codes, and accepts
only bounded HTTPS OpenAI/ChatGPT handoff URLs without embedded credentials.
React receives a short-lived URL and optional device code only while the exact
login is pending. A single owner task serializes completion/cancellation and
reaps its app-server child; logout requires an explicit confirmation step. The
native opener plugin is not granted direct webview permission, so React cannot
submit an arbitrary URL to it.

Milestone 7A adds a serialized native conversation owner. The webview submits
only an opaque project ID, bounded prompt, and closed control enums; native code
revalidates and reserves the project, owns cwd and Codex IDs, correlates every
stream event, and interrupts only the exact active turn. Protocol text and
event batches are bounded, raw reasoning and coarse item details are excluded,
and unexpected approval requests block and close the task without a fabricated
decision. Conversation SQLite rows contain references and lifecycle metadata,
not prompts, transcripts, outputs, diffs, or credentials.

## Principal threats and controls

### Wrong-directory execution

Threats include stale paths, same-named directories, cwd fallback, moved mounts,
symlink target changes, worktree changes, and cross-project thread reuse.

Controls:

- Stable project and association IDs.
- Separate selected/resolved paths and identity evidence.
- Preflight before every task/terminal.
- Exact thread/project/cwd correlation.
- Fail closed; never select a recent or home directory automatically.
- Explicit relink workflow with confirmation.
- Native-only picker input; later lifecycle commands accept opaque IDs rather
  than frontend-supplied paths.
- Confirmation-time reinspection detects symlink, mount, Git/worktree,
  `AGENTS.md`, and `.codex` changes before metadata is committed.
- Reserve an active project's metadata lifecycle so detach, archive, and relink
  cannot race a running task; release the reservation on every terminal path.

### Filesystem scope escalation

Threats include broad writable roots, symlink traversal, malicious project
configuration, directory-picker misunderstandings, and accidental deletion.

Controls:

- Rust-owned directory validation and explicit sandbox construction.
- One primary root by default.
- Additional roots require selection, preview, and confirmation.
- Separate detach/remove/archive/delete actions.
- No filesystem deletion as part of project removal.
- Apply Codex sandbox and managed requirements as independent enforcement.
- Store only QuireForge-owned project metadata in a `0700` application-data
  directory and `0600` SQLite file; do not persist Codex credentials, sessions,
  connector authorization, or project-file content.
- Construct conversation writable roots from the one reverified resolved cwd;
  never accept a cwd or writable root from the webview.

### Command and PTY abuse

Threats include shell injection, unsafe environment inheritance, terminal escape
sequences, unbounded output, background process leakage, and privilege
escalation.

Controls:

- Prefer argv arrays and avoid shell interpolation.
- Sanitize rendered terminal/control sequences where not handled by the terminal
  emulator.
- Bound captured output and process lifetime.
- Correlate approvals to exact command/cwd/turn.
- Keep integrated terminals separate from Codex approval semantics.
- Never offer bypass modes as an innocuous default.
- Reject the `danger-full-access` plus `never` approval combination and validate
  model/reasoning choices against the live supported catalog.
- Keep native thread/turn IDs out of IPC and use them as the sole source for
  exact interruption.

### Authentication and secret leakage

Threats include logging tokens, copying browser storage, persisting OAuth state
in SQLite, environment-variable disclosure, exported diagnostics, and malicious
connector prompts.

Controls:

- Use Codex-managed ChatGPT browser/device login.
- Use official MCP/connector authorization handoffs.
- Do not implement externally managed ChatGPT token mode.
- Store no OAuth/API/connector secret in app SQLite.
- Field-aware redaction before persistence, UI, clipboard, or support export.
- Never render raw auth URLs after completion or include them in logs.
- Treat local home paths and account identifiers as sensitive export metadata.
- Correlate completion to the exact native-held login ID; fail closed for stale
  or missing IDs.
- Allow only bounded HTTPS OpenAI/ChatGPT handoffs without URL credentials.
- Require an explicit second action before logout and never exercise it in
  routine validation.

### Untrusted integrations and marketplaces

Threats include malicious publisher metadata, path traversal, mutable Git refs,
package compromise, MCP data exfiltration, hook code execution, hidden external
domains, and misleading “verified” status.

Controls:

- Use only Codex-supported installation mechanisms.
- Show source, publisher, exact reference/version, scope, bundled components,
  requested permissions, commands/hooks, and external services before install.
- Prefer pinned Git references where trustworthy and available.
- Reject embedded URL credentials.
- Preserve Codex/administrator allowlists and policy-blocked states.
- Keep authentication separate from installation.
- Never label an integration verified without authoritative metadata.
- Require explicit trust for plugin hooks; official docs state installation does
  not automatically trust them.
- Run non-destructive health checks and confirm installed/auth state afterward.

### MCP and connector tool side effects

Threats include incorrect tool annotations, destructive actions hidden behind
an installed integration, open-world network actions, and confused-deputy
access to another account/workspace.

Controls:

- Display read/write/destructive/open-world annotations separately from
  filesystem permissions.
- Honor app/MCP approval requests and managed review requirements.
- Destructive annotated calls always require approval.
- Show service and account/workspace when safely available.
- Confirm connector accessibility through supported refreshed state.

### App-server protocol drift

Threats include renamed fields, changed semantics, experimental features
silently enabled, and partial messages interpreted as success.

Controls:

- Versioned adapters and capability negotiation.
- Default `experimentalApi` off; enable only for isolated justified features.
- Generated-schema fixtures and contract tests.
- Explicit unsupported/degraded states.
- Never interpret process exit alone as successful integration installation;
  parse and verify supported result state.
- Require UUIDv7 identity correlation on conversation responses and
  notifications; mismatch, unexpected server requests, oversized content, or
  control characters fail closed with stable diagnostics.

### Git and worktree data loss

Threats include discarding uncommitted changes, deleting worktrees, overwriting
branches, force pushes, and committing secrets.

Controls:

- Read status before mutations.
- Preview exact target and affected paths.
- No destructive Git recovery commands without approval.
- Worktree cleanup is independent and explicit.
- Secret scanning and diff review before commit/release.
- Never rewrite published history by default.

### Webview and preview content

Threats include script execution from previews, local-file disclosure, unsafe
URLs, huge/decompression-bomb files, and webview-to-native command abuse.

Controls:

- Strict Tauri capabilities and content security policy.
- Treat previews as untrusted data; no arbitrary active HTML execution.
- MIME/extension/size limits and safe text/image/PDF renderers.
- Allowlisted external URL opening with visible destination.
- No remote content receives privileged Tauri access.

### Build, hosting, and release supply chain

Threats include unpinned actions/dependencies, pull-request secrets, compromised
registries, artifact substitution, and publishing local/private data.

Controls:

- Lock dependency graphs and use dependency review/update automation.
- Pin actions to reviewed immutable SHAs and use minimum workflow permissions.
- Build PRs without deploy permissions or secrets.
- Make deployment secrets available only after a protected production-
  environment approval and only to an approved default-branch artifact.
- Use a dedicated SSH key, strict host verification, an exact destination, and
  explicit generated-file manifests; never use plain FTP or wildcard-copy the
  repository into public storage.
- For Cloudflare Pages, constrain GitHub/app or API-token permissions, prevent
  fork previews from receiving secrets, verify the custom domain before DNS
  cutover, and avoid dangling CNAME takeover risk.
- Keep the prior production state recoverable until Pages TLS, headers,
  redirects, assets, and rollback have been verified.
- Require two-factor authentication on the Cloudflare owner account before
  Pages project creation, GitHub integration, token issuance, or DNS cutover.
- Prefer versioned releases and atomic switching; preserve the previous release
  until post-deployment checks pass.
- Produce checksums and provenance/signatures in release milestones.
- Build Linux artifacts on the selected oldest compatible baseline.
- Review website content so local integrations and account data never publish.

### Identity and local-data migration

Threats include overwriting newer QuireForge data, treating Codex-owned data as
application-owned, running concurrently from old and new paths, stale desktop
entries, ambiguous repository redirects, and a partial rename that launches
against the wrong working directory.

Controls:

- Version all application-owned schema changes and make identity migrations
  idempotent.
- Never overwrite an existing newer QuireForge data store or delete the old
  store automatically.
- Detect and migrate only application-owned settings, project associations, and
  non-secret preferences; never migrate Codex authentication or session data.
- Close or reopen processes before moving an active working copy, then verify
  its Git identity and exact path.
- Update canonical repository and production-site links instead of relying
  indefinitely on redirects.
- Keep repository rename, local move, authenticated hosting access, DNS/SSL,
  staging, production deployment, and package cleanup as separate
  approval-gated actions.

## Privacy posture

- The application does not upload attached directories on its own.
- Codex network/model behavior remains governed by the selected Codex account,
  configuration, approvals, and policies.
- App-owned telemetry is off unless separately designed with explicit consent.
- Logs default to local, bounded, and redacted.
- Detaching or uninstalling the app does not delete source directories or
  Codex-owned sessions.

## Security test obligations

- Path traversal, symlink race, mount replacement, missing/moved directory, and
  writable-root tests.
- Shell argv, terminal-control, environment-redaction, and output-limit tests.
- Plugin/marketplace manifest validation, credential-in-URL, mutable-source,
  policy-block, and hook-trust tests.
- MCP OAuth state, reauthentication, destructive annotation, and startup failure
  mocks.
- SQLite migration and secret-absence tests.
- Conversation ID correlation, exact interruption, project-reservation,
  reference-only persistence, approval-block, event-bound, and child-reaping
  tests using deterministic mock processes.
- Tauri capability/CSP review and preview fuzzing.
- Git fixture tests protecting dirty worktrees.
- Workflow-permission and release-artifact verification.

## Deferred questions

- OS keyring use for app-owned non-Codex secrets, if any are ever required.
- Package signing identity and key custody.
- Update channel design and rollback.
- Formal security review and disclosure-response staffing before beta.
