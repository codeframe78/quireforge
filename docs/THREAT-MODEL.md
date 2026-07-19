# Threat Model

Status: initial Milestone 0 model. It must be revisited before authentication,
directory attachment, integrations, packaging, and release milestones.

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

### Build, Pages, and release supply chain

Threats include unpinned actions/dependencies, pull-request secrets, compromised
registries, artifact substitution, and publishing local/private data.

Controls:

- Lock dependency graphs and use dependency review/update automation.
- Pin third-party actions to reviewed immutable SHAs where practical; use
  official GitHub Pages actions with minimum permissions.
- Build PRs without deploy permissions or secrets.
- Deploy only default-branch artifacts through the `github-pages` environment.
- Produce checksums and provenance/signatures in release milestones.
- Build Linux artifacts on the selected oldest compatible baseline.
- Review website content so local integrations and account data never publish.

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
- Tauri capability/CSP review and preview fuzzing.
- Git fixture tests protecting dirty worktrees.
- Workflow-permission and release-artifact verification.

## Deferred questions

- OS keyring use for app-owned non-Codex secrets, if any are ever required.
- Package signing identity and key custody.
- Update channel design and rollback.
- Formal security review and disclosure-response staffing before beta.
