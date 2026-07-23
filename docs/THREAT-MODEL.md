# Threat Model

Status: initial Milestone 0 model with the Milestone 3 frontend/native boundary,
Milestone 4 Codex process adapter, Milestone 5 authentication controls, and
Milestone 6 native directory-attachment controls, Milestone 7 native
conversation controls, Milestone 8A native lifecycle/recovery controls, and
Milestone 9 approval/activity controls, Milestone 10 reviewed Git controls,
Milestone 11A–11C managed-worktree/parallel-execution/cleanup controls, and
Milestone 12 native PTY controls applied, plus Milestones 13–14C normalized
integration discovery, mutation, authorization, and prompt-mention controls.
Milestones 15A–15C additionally apply bounded preview, conversation-image
staging, reviewed desktop-handoff, and privacy-safe notification controls.
Milestone 18 applies app-owned next-turn selector policy, provenance,
completion-time staging, fresh-catalog revalidation, and visible degradation
controls.
Milestone 19 applies the pre-packaging Tauri, active-content, dependency,
workflow, accessibility, performance-budget, and crash-recovery review. It
must be revisited during packaging and release milestones or before any
expansion of the supported integration-management surface.

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

Milestone 8A keeps lifecycle operations under the same serialized native owner.
React supplies an app reference and optional bounded prompt, never a Codex ID,
cwd, rollout path, history, configuration object, or runtime root. Native code
reloads the stored reference, revalidates the project cwd, reads and correlates
the exact thread, and uses bounded exact-cwd lists that match only already-owned
references. Startup clears obsolete active-turn ownership and records stale
work as interrupted. Fork failure attempts to archive an otherwise unreferenced
new thread; archive/restore never delete project files or Codex history.

Milestone 9A keeps approval authority behind that native owner. Only reviewed
command, file, and permission requests are accepted; exact native identities
are replaced with app-owned UUIDv7 values for IPC. Session acceptance, policy
amendments, unstable write-root grants, unsupported requests, stale IDs, and
unadvertised decisions cannot be approved. Permission approval is turn-scoped,
and cancel answers the pending request before exact-turn interruption. Pending
approval is not persisted or replayed after a crash.

Detailed activity discards raw tool arguments and file diffs, buffers command
output through line boundaries, strips terminal and bidirectional controls,
redacts credential-shaped values, reduces paths to project-relative or an
outside-project marker, and applies strict size/count bounds. Raw Codex
protocol and identity remain native-only.

Milestone 9B does not widen this trust boundary. React groups only normalized
events by app-owned activity ID, caps each displayed output tail, and reveals
detail only after a semantic user action. Approval buttons are generated from
the native-advertised closed decision list, guarded against duplicate
submission, and polling is suspended during the decision transition. Routine
UI and browser tests use fixtures and never authorize a real command.

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
- Resume, fork, archive, restore, and reconciliation revalidate the original
  app-owned project/thread binding; no recent thread or alternate cwd is
  substituted.
- Confirmation-time reinspection detects symlink, mount, Git/worktree,
  `AGENTS.md`, and `.codex` changes before metadata is committed.
- Reserve an active project's metadata lifecycle so detach, archive, and relink
  cannot race a running task; release the reservation on every terminal path.
- Bound concurrent ownership to four starting or active tasks, reserve each
  exact project before process creation, and reject duplicate same-project
  execution. Process I/O locks are per task rather than global.

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
- Keep PTY ownership native. React may submit only an app-owned UUIDv7, bounded
  base64 input, output cursor, or bounded dimensions; never cwd, shell,
  environment, TTY, PID, process group, session ID, signal, or executable.
- Use the stable xterm DOM renderer without link/WebGL/proposed-API addons.
  Keep terminal escape handling inside the emulator and never project terminal
  content into ordinary trusted application markup.
- Bound each live output window to one MiB/512 chunks, each poll to 128 KiB/64
  chunks, each input to 64 KiB after a pre-decode encoded-size check, terminal
  dimensions, the registry to eight entries, and cleanup phases to fixed waits.
- Clear the child environment and reconstruct only a fixed system `PATH`,
  terminal identity, and narrow desktop/session allowlist. Do not inherit
  credential/API variables, SSH/GPG agent sockets, or Codex/QuireForge process
  configuration.
- Retain the session-leader child handle and start time. Before signaling,
  enumerate only numeric `/proc` stat identity and require exact owned-session
  membership; never read arguments or environments. Send bounded HUP/TERM/KILL,
  reap the child, and keep a cleanup failure visible rather than silently
  dropping ownership.
- Persist only bounded presentation metadata. Mark stale live records
  interrupted on restart and do not persist terminal input, output, scrollback,
  shell history, cwd, environment, TTY, or process/session identity.
- Correlate approvals to exact command/cwd/turn.
- Correlate approval responses to the native request while accepting only an
  app-owned approval UUID from React.
- Never offer session acceptance or policy amendments through the one-turn
  approval contract; never approve an unstable session write-root grant.
- Buffer command output to a line boundary before redaction so credential
  assignments split across chunks are not exposed.
- Keep integrated terminals separate from Codex approval semantics.
- Warn that terminal input runs with the user's Linux-account privileges and
  require explicit confirmation before ending the owned foreground/background
  session. Closing a terminal never becomes filesystem deletion.
- Never offer bypass modes as an innocuous default.
- Reject the `danger-full-access` plus `never` approval combination and validate
  model/reasoning choices against the live supported catalog.
- Keep native thread/turn IDs out of IPC and use them as the sole source for
  exact interruption.

### Agent-directed model and reasoning selection

Threats include prompt injection that requests an expensive or weaker model,
selection of stale or unadvertised identifiers, unsupported reasoning values,
silent cost escalation, repeated model oscillation, bypass of a manual lock,
and misleading claims that the executing turn changed its own model.

Controls:

- Treat selector changes as app-owned policy decisions, never arbitrary model
  configuration writes or web-selector automation.
- Populate choices from a fresh normalized `model/list` result and revalidate
  both model and reasoning immediately before the next `turn/start`.
- Apply a requested change only after the current turn completes; keep effective
  and pending selection visibly distinct.
- Support Manual, Recommend, and explicitly opted-in Automatic ownership modes.
  Manual selection and user locks always win.
- Require an allowlist or model/reasoning ceiling in Automatic mode, record a
  bounded rationale and provenance, and permit at most one pending change per
  turn to prevent oscillation and hidden escalation.
- Persist no raw prompt, protocol payload, account identity, credential, or
  Codex-owned configuration as selector metadata.
- Degrade to recommendation-only behavior when the installed app-server does
  not expose a validated control lifecycle.
- Keep exact dynamic-tool request, thread, turn, and call identity native; React
  receives only a schema-versioned effective/pending projection and submits only
  an opaque app conversation ID plus closed policy fields.
- Commit a Codex request to metadata only after successful turn completion.
  Interrupted, blocked, or failed turns do not leave a fabricated pending
  selection.

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
- Normalize discovery into category, scope, source, installation, enablement,
  authentication, permission, requirement, policy, and health enums. Never
  pass raw app-server or CLI JSON to React.
- Track upstream availability separately from QuireForge implementation so an
  advertised experimental method cannot appear as a completed feature.
- Reduce marketplace loader failures, MCP startup failures, policy warnings,
  and unsupported schema changes to bounded diagnostic codes; preserve partial
  and unknown states instead of silently dropping entries.
- Gate live discovery to an explicitly reviewed CLI minor. Use stable bounded
  CLI JSON for plugin/marketplace reads instead of under-development plugin
  RPCs, neutral working directories for non-project catalog reads, null
  stdin/stderr, fixed arguments, output caps, timeouts, and child reaping.
- Treat invalidation notifications only as closed category-refresh reasons;
  discard app rows, MCP names/failures, config paths/details, and every other raw
  notification field before caching or IPC.
- Query scheduled templates only for installed, enabled plugins resolved from
  the supported CLI catalog and keep their marketplace roots and lookup values
  native-only. Cap lookup targets and task counts.
- Treat scheduled task names and prompts as malicious publisher content:
  normalize controls and whitespace, bound and label prompt previews as inert,
  bind every task to an existing normalized plugin, and never submit, persist,
  interpolate, or execute the prompt.
- Expose no scheduled-task mutation or runner. A template is not proof of
  eligibility, enablement, execution, hosting, or official-client state.
- Treat dynamic tool arguments as native-only untrusted input. Correlate the
  exact thread, turn, request, namespace, and tool; validate one closed
  app-owned schema; and return only bounded result content.
- Expose no generic integration command. Accept only closed plugin
  install/remove and marketplace add/remove/upgrade requests; map opaque entry
  IDs back to fresh native CLI evidence and keep programs, arguments, paths,
  URLs, install roots, and raw JSON out of IPC.
- Require a five-minute, one-use UUIDv7 confirmation; cap pending previews;
  serialize execution; and revalidate the reviewed CLI minor, effective policy,
  ready capability, exact normalized entry, and native source evidence after
  consuming the token.
- For local plugins, canonicalize the directory and refuse symlinked,
  malformed, mismatched, or oversized manifests. For remote installs, reject
  URL credentials and unpinned repositories/package versions. Require a pinned
  reference for a new repository marketplace.
- Label a configured repository marketplace upgrade as mutable-source because
  the next fetched artifact cannot be derived from list evidence. Confirmation
  acknowledges that risk; it is not represented as artifact verification.
- Treat process success as insufficient. Accept only closed operation-specific
  JSON and verify the expected fresh list postcondition; bound time/output,
  remove inherited API credentials, use neutral cwd and null streams, and reap
  every child.
- Render an Integration Center mutation only when its normalized capability is
  both upstream-available and implementation-ready. Unsupported connector,
  MCP, skill, enablement, authorization, prompt-mention, and repair flows remain
  visibly unavailable and cannot fall through to a generic route.
- Keep catalog search and filters presentation-only: they may select an opaque
  normalized entry ID but cannot derive a command, source, path, URL, or
  operation. Repository marketplace adds reuse the strict pinned-reference
  request contract.
- Before confirmation, display the native preview's source class, normalized
  permissions, warnings, destructive status, and separate hook-trust notice.
  Contain and restore focus so keyboard users cannot accidentally interact with
  obscured controls, and refresh normalized catalog state after an applied
  result.
- Accept only connector authorization, MCP authorization, skill enable, or
  skill disable as closed 14C controls. Resolve the opaque entry ID to fresh
  native app-server evidence before preview and again after consuming a
  five-minute one-use confirmation; serialize control/handoff state and cap
  pending previews.
- Never accept an authorization URL, skill path, MCP name, config key/value,
  app path, or protocol method from React. Keep the Codex-returned values only
  in process memory and open a URL only from its opaque native action ID.
- Permit credential-free HTTPS authorization URLs and loopback HTTP callbacks
  only; reject credentials, fragments, oversized values, mismatched MCP
  completion names, and raw completion errors. Never serialize, persist, log,
  or copy the URL, code, token, skill path, or MCP name.
- Treat skill mutation success as an exact effective-state response plus a
  fresh list postcondition. Treat connector completion as fresh accessibility,
  not browser return or process exit. Refresh normalized catalog state after a
  completed control.
- Accept at most eight unique normalized connector IDs on conversation start.
  Native code must re-resolve accessible, enabled, and callable state and
  construct the documented `mention`/`app://` item; the webview cannot provide
  or receive that path.

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
- Correlate MCP OAuth completion to the exact native-held server name and keep
  the OAuth URL, codes, tokens, endpoints, and raw errors outside IPC and
  SQLite.
- Do not infer a generic connector install/configuration API from a returned
  authorization URL. Unsupported management remains unavailable.

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
- Bound current/archived list pagination, require every returned cwd to match
  the native verified set, and discard unowned thread metadata rather than
  importing it.

### Git and worktree data loss

Threats include discarding uncommitted changes, deleting worktrees, overwriting
branches, force pushes, committing secrets, local Git configuration executing
helpers, deceptive paths, unbounded output, symlink traversal, and a webview
request reading outside the attached directory.

Controls:

- Read status before mutations.
- Preview exact target and affected paths.
- No destructive Git recovery commands without approval.
- Worktree cleanup is independent and explicit.
- Secret scanning and diff review before commit/release.
- Never rewrite published history by default.
- Keep read-only status/diff commands fixed and shell-free; clear the inherited
  environment, ignore global/system configuration, and disable prompts,
  optional locks, filesystem monitors, external diffs, and text conversion.
- Revalidate attachment identity and current status for every path action;
  accept no frontend cwd, absolute path, Git argument, revision, or executable.
- Reject escaping, non-UTF-8, control-bearing, directional-formatting,
  backslash-bearing, symlink, conflicted, and submodule review targets; enforce
  byte, line, change-count, and time limits.
- Return normalized status and diff-line records only. Discard object IDs, raw
  headers, stderr, and repository configuration; persist no diff content.
- Keep mutations out of the read-only bridge. Preview only one closed operation
  and exact attachment-relative target/message, retain the plan behind an
  expiring native token, and accept only that token for confirmation.
- Serialize Git writes against Codex project ownership, revalidate attachment
  and exact Git evidence at confirmation, check operation-specific
  postconditions, and attempt narrow index/reference rollback on unexpected
  results.
- Limit revert to a bounded tracked regular file; snapshot bytes/mode before the
  write and offer only an expiring, single-use, process-local atomic recovery
  while clearly stating that it is not a durable backup.
- Before commit, reject staged paths outside the attachment, conflicts,
  submodules, active repository operations, missing repository-local identity,
  unscannable blobs, sensitive filenames, and high-confidence secrets in staged
  content or the commit message.
- Create commits without hooks, signing, editors, prompts, or inherited/global/
  system configuration; lock/revalidate the index and update `HEAD` only from
  the reviewed old value.
- For managed worktree creation, generate the destination beneath private app
  storage, bind source identity and HEAD in a native-held expiring preview,
  require a bounded new branch, reserve the repository's app-owned project
  group, and disable hooks and configured checkout filters.
- Attach an existing worktree only through the native picker and require exact
  linked-worktree/common-directory identity. Discovered external worktrees have
  no selectable project ID until attachment succeeds.
- Leave a worktree intact and report it as recoverable if post-creation metadata
  persistence fails. Issue recovery IDs only for unregistered linked worktrees
  in the exact private source-project storage slot, and revalidate their complete
  identity before metadata-only registration.
- Present parallel worktree status only by joining the native active-task
  registry with normalized read-only Git snapshots. Return aggregate changed-
  file/conflict counts, never raw Git output, and provide no automatic conflict
  resolution or mutation in Milestone 11B.
- Remove only a stored `managed` worktree after an expiring preview, complete
  repository-group reservation, and confirmation-time relation, canonical path,
  common-directory, inventory, branch, `HEAD`, lock, and clean-status checks.
- Neutralize repository-configured clean/smudge/process filters for both the
  explicit status check and Git's internal removal check; continue disabling
  hooks, prompts, inherited/global/system configuration, and unbounded output.
- Never use force or delete the branch. Verify directory/inventory absence and
  branch retention before transactionally detaching and archiving project
  metadata. If that transaction fails, require a new metadata-only confirmation
  while the path and inventory entry remain absent.
- Do not expose generic prune, direct directory deletion, attached/external
  worktree deletion, arbitrary checkout/ref, reset, stash, remote, push, pull,
  or generic Git mutation in Milestone 11C.

### Parallel task confusion and resource exhaustion

Threats include one task receiving another task's approval or interruption,
late responses overwriting newer state, duplicate work in one directory,
unbounded child creation, registry locks serializing all I/O, orphaned children,
and raw native identity leaking through an aggregate dashboard.

Controls:

- Key native slots only by app-owned conversation UUIDv7 and route poll,
  approval, and interruption after an exact registry lookup. Keep Codex IDs,
  cwd, process identity, arguments, environment, and raw protocol native-only.
- Count provisional starts against the literal capacity of four and reserve the
  exact project. Failure paths clear the provisional slot and project
  reservation before returning a stable diagnostic.
- On active-task paths, hold the registry mutex only for capacity/membership
  operations and use one mutex per conversation for process I/O. Existing
  all-session reconciliation remains serialized only while no task is active.
  Remove a terminal slot only when its registered allocation still matches.
- Use per-task frontend action generations so a stale poll can be discarded for
  the affected task without pausing or overwriting unrelated tasks.
- Close and wait for each owned child exactly once on terminal paths. On
  application restart, mark stale persisted work interrupted rather than
  pretending process ownership can be recovered.
- Bound the active registry, recent event-free snapshots, event streams,
  activities, and output. Filter monitor rows through current worktree inventory
  and show only normalized state and Git counts.

### Webview and preview content

Threats include script execution from previews, local-file disclosure, unsafe
URLs, huge/decompression-bomb files, and webview-to-native command abuse.

Controls:

- Strict Tauri capabilities and content security policy.
- Keep the only capability Linux/main-window scoped with an empty permission
  set. Explicitly disable the global Tauri JavaScript API and asset protocol,
  preserve Tauri's compile-time CSP injection, and remove unused plugin
  commands from production builds.
- Set production `default-src` to `none`; admit only local scripts/fonts/styles,
  local/data images, and Tauri IPC. Deny objects, forms, frames, workers, media,
  manifests, base changes, external origins, and production code evaluation.
- Add same-origin opener/resource response policy, deny unused camera,
  display-capture, location, microphone, payment, and USB features, and disable
  MIME sniffing.
- Keep Tauri `freezePrototype` disabled for the current verified Vite/React
  bundle: enabling it prevents the application from mounting. Compensate with
  the explicit CSP, no privileged remote content, dependency locking, strict
  capability allowlisting, and narrow validated IPC instead of claiming a
  hardening control the shipped frontend cannot execute under.
- Retain `style-src 'unsafe-inline'` only for the stable xterm renderer; do not
  widen any other directive to compensate.
- Reject direct production-frontend HTML injection, string evaluation, fetch,
  XHR, and WebSocket primitives in the dependency-free repository gate.
- Treat previews as untrusted data; no arbitrary active HTML execution.
- File selection remains native; React supplies only a canonical app-owned
  UUIDv7 project ID and never a path, URL, claimed MIME type, or renderer.
- Reload and revalidate attachment identity/readability, canonical containment,
  symlink and regular-file status, then retain an identity-checked root
  directory descriptor. Open the relative target through that descriptor with
  `O_NOFOLLOW` and recheck the resolved `/proc/self/fd` path/device/inode.
- Cap source files at 8 MiB; normalize UTF-8 text to at most 128 KiB/2,000
  lines while replacing controls and bidi overrides.
- Admit only bounded PNG/JPEG data URLs after pre/post-read type, dimension,
  byte, pixel, and APNG checks. Permit `data:` under `img-src` only.
- Recognize PDF as metadata only. Do not send PDF bytes, unknown binary
  content, absolute paths, or active-document URLs to the webview. HTML/SVG
  source may cross only as normalized inert text, never active markup.
- Keep preview state transient; browser preview cannot select/read local files.
- Bind external opening to one native-held five-minute UUIDv7 action created by
  a successful preview. Require a second UI confirmation naming the relative
  file and fixed system-default-application destination; accept no frontend
  path, URL, application, executable, argument, MIME, or cwd.
- Before handoff, reload the attachment and revalidate canonical containment,
  regular non-symlink state, descriptor path, and the previewed device/inode.
  Consume the action once and cap/expire all pending handoffs.
- Disable Tauri's default path-bearing file-drop events. Treat browser drops as
  explicit bounded byte inputs. If WebKitGTK supplies an empty HTML `FileList`,
  retain at most five Linux file URIs (four allowed plus one overflow sentinel)
  in a 30-second native-only one-use slot and let the visible drop zone claim it
  through a path-free fixed command. Picker/captured paths remain native and
  are never returned to React.
- Accept only structurally validated PNG/JPEG conversation images: four files,
  4 MiB each/16 MiB aggregate, safe display names, and bounded dimensions.
  Refuse text, PDF, SVG, generic binary, mismatched declared type, symlinks, and
  malformed content.
- Stage app-owned UUIDv7 copies beneath a mode-`0700` root with mode-`0600`
  files. Return only opaque project-bound IDs and normalized metadata; persist
  no draft, source path, staged path, or bytes in SQLite.
- Expire unconsumed drafts after 15 minutes and consume IDs once. Reopen and
  revalidate device, inode, size, type, and dimensions immediately before
  constructing native-only documented `localImage` inputs.
- Retain claimed copies until the normalized turn becomes terminal because the
  app-server start response does not document completed image consumption.
  Clean on terminal poll, cancel, failed send, expiry, and next startup without
  deleting source files.
- Accept only an app-owned conversation UUIDv7 for background notifications.
  Re-resolve native approval/terminal state, suppress foreground delivery,
  deduplicate by approval/terminal identity, and use only fixed closed-enum
  copy. Never interpolate project names, prompts, paths, account/model data,
  output, diagnostics, or raw protocol into a notification.
- Keep the webview capability list empty for opener/notification plugins; Rust
  alone may invoke the fixed desktop operations.
- Keep manual notification delivery behind a disabled-by-default Cargo feature
  and exact native process flag. It may reuse only fixed production copy, accept
  no caller content, and add no Tauri command or webview permission. Replace the
  feature artifact with a normal build after the probe.
- Allowlisted external URL opening with visible destination.
- No remote content receives privileged Tauri access.

### Build, hosting, and release supply chain

Threats include unpinned actions/dependencies, pull-request secrets, compromised
registries, artifact substitution, and publishing local/private data.

Controls:

- Lock dependency graphs and use dependency review/update automation.
- Pin actions to reviewed immutable SHAs and use minimum workflow permissions.
- Run the high-severity Node audit and warning-denying RustSec audit in CI.
  Keep every RustSec exception as an exact reviewed advisory ID, fail on every
  new warning, and revisit Tauri/GTK3 maintenance exceptions on upstream
  updates and before release.
- Apply Dependabot to npm, Cargo, and GitHub Actions. Validate that every
  non-local workflow action uses a full commit SHA.
- Build PRs without deploy permissions or secrets.
- Make deployment secrets available only after a protected production-
  environment approval and only to an approved default-branch artifact.
- Use a dedicated SSH key, strict host verification, an exact destination, and
  explicit generated-file manifests; never use plain FTP or wildcard-copy the
  repository into public storage.
- Keep Webuzo authoritative for the exact domain, document root, origin SSL,
  ownership, and generated VirtualHost; do not hand-edit generated or global
  server configuration.
- Build outside public storage, deploy only a reviewed manifest-bound static
  artifact, and reject source, Git metadata, credentials, private documents,
  dependency trees, logs, and source maps.
- Keep the prior QuireForge document root and a verified Restic snapshot
  recoverable until origin and Cloudflare TLS, headers, routes, assets, and
  rollback have been verified.
- Use a least-privilege Cloudflare token only for an approved exact-host DNS
  operation; never expose the origin through a new public port or alter apex,
  mail, status, or unrelated project records.
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
- Parallel fixture tests for four-task capacity including provisional starts,
  same-project exclusion, exact per-task routing, independent progress, stale
  frontend response protection, normalized registry privacy, and complete
  multi-child reaping.
- Tauri capability/CSP review and preview fuzzing.
- Production-frontend active-content scans, generated desktop origin and bundle
  budgets, Node/Rust dependency audits, and exact advisory-exception checks.
- Keyboard skip/focus, reduced-motion CSS and scripted scrolling, forced-color
  controls, modal focus trap/restore, responsive overflow, and desktop/website
  axe-core tests.
- Render-crash tests must prove that raw error text is absent and the recovery
  action reconciles state without implying source or Codex-history deletion.
- Conversation-attachment fixture tests for strict IDs, source ownership,
  content/type/size limits, tamper and expiry refusal, path non-disclosure,
  one-use claim, terminal cleanup, browser honesty, and default-drop disabling.
- Desktop handoff/notification tests for one-use opaque actions, replacement
  races, attachment/file identity drift, explicit destination review, closed
  notification eligibility/copy/status, deduplication, and bridge path/command
  refusal; label Wayland, XWayland, and true X11 manual evidence separately.
- Git fixture tests protecting dirty worktrees, attached-subdirectory scope,
  read-only repositories, path containment, deceptive input, output bounds, and
  the no-mutation boundary.
- Workflow-permission and release-artifact verification.

## Deferred questions

- OS keyring use for app-owned non-Codex secrets, if any are ever required.
- Package signing identity and key custody.
- Update channel design and rollback.
- Formal security review and disclosure-response staffing before beta.
