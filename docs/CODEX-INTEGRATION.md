# Codex Integration Findings

Status: Milestone 0 discovery with implementation through Milestone 18,
including the native read-only catalog, confirmed plugin/marketplace lifecycle,
user-facing Integration Center, and confirmed authorization/control boundary
validated locally
Observed: initial discovery 2026-07-19; protocol refresh 2026-07-21
Installed CLI evidence: `codex-cli 0.144.6` baseline and `codex-cli 0.145.0`
current refresh
Platform: Ubuntu 26.04 LTS, x86_64, GNOME Wayland

This document records observed interfaces. It is not a promise that every
capability remains available in other Codex versions, accounts, regions, or
workspaces.

## QuireForge client identity

QuireForge is the intended client identity wherever an official Codex or MCP
interface accepts client metadata. A versioned adapter sends only truthful,
documented fields and must not impersonate an official OpenAI client or claim
OpenAI verification.

Integration-facing identifiers follow these rules:

- App-server machine identity: `clientInfo.name = "quireforge"`.
- App-server human title: `clientInfo.title = "QuireForge"`.
- App-server version: `clientInfo.version` is the real QuireForge application
  version.
- The upstream user-agent string is the value returned by Codex after
  initialization; QuireForge does not spoof or independently construct it.
- Official plugin, connector, marketplace, MCP tool, Codex protocol, and API
  identifiers remain unchanged.
- Connector authorization uses only official returned URLs and callback
  behavior; the rename does not invent or rewrite callback contracts.
- Enterprise registration, allowlisting, or compliance metadata must describe
  QuireForge as an unofficial third-party client and may require future
  coordination with the relevant administrator or OpenAI-supported process.

Client metadata and logs must not contain local paths, account identifiers,
tokens, or repository contents.

The current [Codex app-server documentation](https://learn.chatgpt.com/docs/app-server)
requires `clientInfo` during initialization and states that `clientInfo.name`
identifies the integration in the OpenAI Compliance Logs Platform. An
enterprise-targeted QuireForge deployment may therefore require coordination
through OpenAI's supported known-client process.

## Evidence sources

- Installed CLI help and redacted diagnostics.
- Generated app-server JSON Schema from
  `codex app-server generate-json-schema --experimental`.
- Live, non-mutating app-server requests against CLI 0.144.6.
- [Official Codex app-server documentation](https://learn.chatgpt.com/docs/app-server).
- [Official Codex plugin documentation](https://learn.chatgpt.com/docs/plugins)
  and [plugin build specification](https://learn.chatgpt.com/docs/build-plugins).
- [Official Codex project documentation](https://learn.chatgpt.com/docs/projects).
- [Official managed-configuration documentation](https://learn.chatgpt.com/docs/enterprise/managed-configuration).
- [Official open-source app-server source documentation](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md).

The application must probe the installed CLI at runtime and must not treat this
snapshot as a permanent API contract.

## Milestone 13A protocol refresh

The 0.145.0 refresh generated 95 reviewed schema files while retaining the
0.144.6 compatibility baseline. The selected evidence now includes app
catalog/read/installed state, plugins and marketplaces, skills, MCP status and
OAuth, configuration requirements, permission profiles, invalidation events,
and the client-owned dynamic-tool lifecycle. Schema generation reads no Codex
account data and makes no model call.

The stable plugin CLI continues to provide JSON for catalog, install, remove,
and marketplace operations. The app-server provides richer read/status events,
but plugin management and `app/read` remain experimental. QuireForge therefore
selects a route per operation and records method stability separately from
upstream availability and application implementation.

The accepted `codex-integration-v1` contract keeps connector, plugin,
marketplace, skill, and MCP-server categories distinct. It exposes bounded
scope, source, installation, enablement, authentication, permission,
requirement, policy, and health values; it never exposes raw protocol/CLI JSON,
absolute paths, account identity, authorization state, credentials, managed
configuration, or tool arguments. All 13A capability rows are deliberately
`contract-only`; live discovery and IPC remain Milestone 13B.

The refreshed `ThreadStartParams` schema accepts bounded function or namespace
definitions in `dynamicTools`. The server invokes a registered client-owned
tool with `item/tool/call`, including a correlated request ID plus native
thread/turn/call identity, and accepts a bounded success/content response. This
is the supported dependency for Milestone 18. It can stage an app-owned model
selection for the next turn but cannot replace the currently executing model.
See [ADR 0018](DECISIONS/0018-normalized-integration-contracts.md).

## Milestone 13B read-only runtime

The native `IntegrationCatalogService` implements the read-only rows of
`codex-integration-v1` and exposes only `integration_catalog_read`. It accepts no
frontend path, URL, command, account value, or arbitrary protocol input. CLI
0.145.x is the exact reviewed compatibility minor; other minors fail closed
with an unavailable catalog until their schemas and routes are reviewed.

Route selection follows the established adapter policy:

- `app/list` and `app/installed` normalize connector availability, enabled
  state, accessibility, and callable health.
- `skills/list` uses a neutral temporary cwd so a catalog read does not attach
  or inspect a user project.
- `mcpServerStatus/list` requests only tools/auth detail, then discards endpoint,
  resource, and tool definitions.
- `configRequirements/read`, `permissionProfile/list`, and `config/read` retain
  only bounded effective-policy state.
- Stable `codex plugin list --available --json` and `codex plugin marketplace
list --json` provide production plugin/marketplace discovery. The
  under-development plugin app-server RPCs are not called.

All CLI output is size/time bounded and strictly normalized. Marketplace roots,
plugin sources, URLs, paths, config contents, MCP details, raw notification
payloads, and upstream error text are discarded. App, skill, MCP-startup,
config-warning, and account notifications trigger category refresh through
closed reason enums rather than carrying upstream payloads into React.
Independent failures preserve other healthy categories and mark only the
affected capability degraded. Milestone 13B includes no mutation or UI.

## Milestone 14A plugin and marketplace lifecycle

`IntegrationMutationService` implements only the stable CLI-backed plugin
install/remove and marketplace add/remove/upgrade routes. The frontend can call
fixed `integration_mutation_preview` and `integration_mutation_confirm`
commands with a closed operation and opaque catalog ID; marketplace add instead
accepts a bounded `owner/repository` plus a 40- or 64-hex pinned reference.
Programs, subcommands, argument vectors, paths, URLs, and arbitrary JSON are
not frontend inputs. Default/built-in marketplace rows cannot be removed;
remove applies only to an explicitly configured source.

Preview forces a fresh 0.145.x catalog/policy read, inspects native source
evidence, returns normalized permissions and supply-chain warnings, and stores
the exact evidence behind a five-minute one-use UUIDv7. Confirmation consumes
the token, serializes mutations, refreshes and revalidates policy/capability/
entry/source evidence, executes the one selected fixed command, validates its
closed JSON response, and performs a fresh postcondition list. CLI commands use
neutral `/`, null stdin/stderr, removed `OPENAI_API_KEY`, a 30-second timeout,
one-MiB stdout cap, and explicit child reaping. Upstream errors, install paths,
marketplace roots, sources, and URLs are discarded.

Local plugin manifests are bounded and cannot be symlinked; their name,
version, hook (including default `hooks/hooks.json`), MCP, app, and skill
declarations are reviewed. Hook execution is labeled as requiring separate
trust rather than implied by installation. Remote plugin
installs require safe credential-free HTTPS and a pinned commit or exact package
version. Marketplace adds require a pinned repository reference. Marketplace
upgrade retains an explicit mutable-remote-source warning because list evidence
does not identify the next fetched artifact. Installation remains separate
from connector/MCP authorization. See
[ADR 0019](DECISIONS/0019-confirmed-integration-mutations.md).

Routine tests use deterministic temporary fixtures. The separately invoked
real-CLI proof uses temporary `CODEX_HOME` and `HOME` with one local fixture
marketplace/plugin and does not touch personal Codex state. Plugin
enable/disable and unsupported generic management remain later gated work.
Connector/MCP authorization, skill configuration, prompt mentions, and
explicit refresh are handled only by the separately confirmed 14C boundary.

## Milestone 14B Integration Center

The React Integration Center consumes only the strict 13B catalog and 14A
preview/result contracts. It keeps connector, plugin, marketplace, skill, and
MCP-server categories visible; offers bounded name/summary/publisher search and
category/health filters; and presents normalized source, scope, installation,
enablement, authentication, policy, version, permissions, requirements, and
health without raw protocol or CLI fields.

Actions appear only for the fixed plugin install/remove and marketplace
add/remove/upgrade operations whose capability reports both upstream
availability and QuireForge implementation readiness. A repository marketplace
add accepts only the existing bounded `owner/repository` plus 40- or 64-hex
pinned reference contract. Preview preserves the native-selected operation and
shows source, permissions, warnings, destructive status, and separate hook
trust before the one-use confirmation is submitted. Applied results trigger a
fresh catalog read.

Unsupported management is explicitly unavailable rather than mapped to a
generic command. Browser preview uses the sanitized catalog fixture; routine
component and browser tests do not inspect or mutate personal Codex integration
state.

## Milestone 14C authorization and integration controls

`IntegrationControlService` exposes four closed native operations:
`connector-authorize`, `mcp-authorize`, `skill-enable`, and `skill-disable`.
React sends only one operation and opaque normalized entry ID. It cannot send a
URL, app path, skill path, MCP name, configuration key/value, protocol method,
or raw result.

Preview requires the matching capability to be both upstream-available and
implemented, checks eligible normalized state and policy, then resolves fresh
native evidence through the reviewed 0.145.x app-server routes. A ready preview
stores exact evidence behind a five-minute one-use UUIDv7. Confirmation
re-resolves that evidence and then:

- opens a connector authorization URL only when `app/list` returned it;
- starts MCP OAuth only with `mcpServer/oauth/login` and accepts only the exact
  `mcpServer/oauthLogin/completed` server-name correlation; or
- writes one skill's effective enabled state with `skills/config/write`, then
  requires both the exact effective-state response and a fresh `skills/list`
  postcondition.

Authorization URLs are bounded credential-free HTTPS, or loopback HTTP for a
callback, with no fragment. The URL remains native-only, is opened by Tauri
from a process-local opaque action ID, and is never logged, stored in SQLite,
or serialized to React. Completion refreshes the normalized catalog; users can
also invoke a fixed-purpose catalog/health refresh without mutation.

The conversation composer lists only normalized connectors that are connected,
enabled, and healthy. It can submit up to eight unique opaque entry IDs. Native
code rechecks `app/list` and `app/installed`, requires accessible, enabled, and
callable state, and constructs the documented `mention` item with its `app://`
path. The webview never supplies or receives that path.

Generic connector install/configuration, plugin enable/disable, MCP
add/remove/logout/configuration, arbitrary health repair, and generic config
editing remain unavailable. See
[ADR 0020](DECISIONS/0020-confirmed-integration-authorization-and-controls.md).

## Required CLI inspection

The following commands were run successfully:

```text
codex --version
codex --help
codex login --help
codex exec --help
codex app-server --help
codex resume --help
codex fork --help
codex archive --help
codex review --help
codex mcp --help
codex plugin --help
codex plugin marketplace --help
codex doctor --help
codex features list
```

Observed top-level capabilities include `exec`, `review`, `login`, `logout`,
`mcp`, `plugin`, `app-server`, `resume`, `fork`, `archive`, `unarchive`,
`delete`, `doctor`, and feature inspection. `app-server` is labeled
experimental by the CLI even though many v2 methods have stable method-level
status.

## Adapter strategy

The desktop application will use a versioned `CodexBackend` boundary:

1. `AppServerBackend` for rich interactive sessions and stable v2 methods.
2. `CliJsonBackend` for stable command fallbacks, especially plugin and
   marketplace operations whose app-server RPCs are under development.
3. `MockCodexBackend` for deterministic tests and offline demonstrations.
4. An unsupported capability result when neither official interface exists.

Raw JSON-RPC and CLI JSON must never cross into the frontend. Each adapter maps
data into normalized domain events and capability records.

### Implemented Milestone 4 subset

The current adapter intentionally implements only the smallest non-billable
compatibility slice:

- fixed `codex --version` discovery with bounded output and strict version
  normalization;
- owned `codex app-server --listen stdio://` lifecycle;
- truthful `initialize` metadata and correlated `model/list` request;
- normalized model IDs, display names, default marker, and advertised reasoning
  efforts;
- explicit ready, degraded, and unavailable capability states;
- deterministic mock/failure processes and a hashed generated-schema subset.

The Milestone 4 probe receives no user path, prompt, arbitrary argument, or
frontend input. Raw notifications, user-agent, Codex-home, account,
installation, remote-control, and RPC error payloads are discarded. Later
milestones extend the adapter through separate fixed-purpose services rather
than broadening this probe.

### Implemented Milestone 5 subset

Authentication extends the same adapter without exposing raw account protocol:

- cached `account/read` with `refreshToken: false`, normalized to connection
  state and coarse account kind without email, plan, or identifiers;
- `account/login/start` for only `chatgpt` and `chatgptDeviceCode`;
- one native-held pending login ID and one owner task for completion,
  cancellation, shutdown, and exact child reaping;
- bounded HTTPS handoffs restricted to OpenAI/ChatGPT hosts with no embedded
  URL credentials, plus a bounded device code when required;
- `account/login/cancel`, `account/login/completed`, `account/updated`, and
  explicit `account/logout` handling with stable local diagnostic codes;
- a native browser command that accepts no frontend URL and an opener plugin
  that receives no direct webview permission; and
- strict Rust/TypeScript fixtures and deterministic success, failure, stale-ID,
  cancellation, URL, redaction, and UI tests.

The external-token and API-key login variants are deliberately not offered. A
live validation invoked only `account/read`; real login, browser handoff, and
logout remain user-driven and were not exercised by Codex.

### Implemented Milestone 7A subset

The native conversation checkpoint adds only the reviewed start, stream, and
interrupt lifecycle needed for the MVP:

- live model/reasoning validation and fixed `thread/start` plus `turn/start`
  requests against the exact reverified project cwd;
- explicit closed sandbox and approval enums, including rejection of the
  `danger-full-access` plus `never` combination;
- native ownership and UUIDv7 correlation of the active thread and turn;
- bounded normalized lifecycle, agent-message, reasoning-summary, plan,
  coarse-activity, completion, and stable-error events;
- exact `turn/interrupt` using native-held IDs; and
- reference-only QuireForge metadata with no prompt, transcript, raw reasoning,
  command output, file change, diff, path, credential, or Codex-owned session
  content.

The frontend supplies neither cwd nor Codex IDs. Unexpected server requests,
including approval requests, fail closed; an approval request produces a stable
blocked state and child shutdown rather than an inferred decision. All routine
tests use deterministic app-server fixtures. No live model turn was run while
implementing or verifying this checkpoint.

## Local working-directory behavior

Both the interactive CLI and `codex exec` expose `--cd <DIR>` and
`--add-dir <DIR>`. The app-server accepts absolute `cwd` values on
`thread/start`, `thread/resume`, `thread/fork`, `turn/start`, `command/exec`,
and process APIs.

A local app-server validation used a temporary original directory and a
symbolic-link path:

- `command/exec` started with the symlink as `cwd`.
- The child process observed the resolved target as `PWD`, `pwd -L`, and
  `pwd -P`.
- A `readOnly` sandbox rejected a file creation with a read-only-filesystem
  error.
- A `workspaceWrite` sandbox with the resolved target in `writableRoots`
  allowed the same operation.
- The temporary test directory was removed after verification.

Consequences for project attachment:

- Store the exact path selected by the user separately from its resolved path.
- Record filesystem identity using platform metadata such as device and inode,
  plus Git identity where present.
- Re-resolve and compare identity before every task.
- Tell the user when Codex will operate on a symlink target rather than imply
  that the lexical link path is preserved by the process runtime.
- Pass the verified resolved working root to Codex and retain the selected path
  for display and relinking.
- Add extra writable roots only through explicit user approval and supported
  `writableRoots` or `--add-dir` controls.

## Stable app-server surface observed

The following v2 methods are documented without an experimental warning at the
method level in the inspected release:

- Models: `model/list`.
- Threads: start, resume, read, list, fork, archive, unarchive, interrupt, and
  metadata operations.
- Turns: start, steer, interrupt, streamed items, plans, diffs, usage, and
  completion events.
- Approvals: command, file change, MCP/app tool, permission, and MCP elicitation
  flows.
- Review: `review/start`.
- Sandboxed commands: `command/exec` and its PTY follow-up methods.
- Filesystem invalidation: absolute-path `fs/watch` and `fs/unwatch`.
- Skills: `skills/list`, change notifications, and enable/disable writes.
- Apps/connectors: `app/list`, app mentions, and app configuration RPCs.
- MCP: server status, tools/auth, OAuth login, resource reads, refresh, and
  startup-status events.
- Configuration and policy: `config/read`, writes, and
  `configRequirements/read`.
- Authentication: account read, Codex-owned ChatGPT browser/device flow,
  logout, and account events.

The application will still version-gate every method because the app-server
executable as a whole remains labeled experimental.

## Local image turn input

The reviewed Codex CLI 0.145.0 `TurnStartParams` schema includes the documented
`localImage` user input with a native path. The official app-server guide also
describes user-message content as text, image, or `localImage`. No generic
local-file input appears in the reviewed turn contract, so QuireForge must not
represent text, PDF, or arbitrary files as supported turn attachments.

App-server documents `turn/start` as returning the initial turn and then
streaming lifecycle/item events through terminal `turn/completed`. It does not
specify an earlier point when a `localImage` path may be removed. Milestone 15B
therefore constructs the path only from a private native-held staged copy and
retains that copy until QuireForge observes a terminal normalized turn. Source
and staging paths never cross IPC or enter QuireForge metadata.

## Model discovery

Live `model/list` returned picker-visible models rather than requiring hardcoded
model IDs. GPT-5.6 Sol, Terra, Luna, GPT-5.5, and GPT-5.3 Codex Spark were
visible for the inspected account. Each row carried supported reasoning
efforts, default effort, modalities, display metadata, and default-model state.

The UI must populate model and reasoning controls only from `model/list` (or an
official fallback exposed by that Codex version). It must never assume a model
or reasoning effort exists.

## Agent-directed model selection boundary

The reviewed `turn/start` contract accepts model and effort overrides for that
turn and subsequent turns. That creates a supported next-turn application
point, not a way for the executing model to replace itself in the middle of a
turn. Milestone 18 lets Codex inspect only a normalized picker catalog,
current effective selection, pending selection, and app-owned policy, then
request at most one bounded model/reasoning change per turn with a short
rationale.

Native code must refresh and revalidate the requested model and effort before
the next `turn/start`. Manual selection and a user lock always take precedence.
Recommend mode displays a proposal without applying it; Automatic mode requires
explicit opt-in plus an allowlist or model/reasoning ceiling. The UI must show
effective and pending values separately and label Codex-requested provenance.
No account identifier, credential, raw prompt, or raw app-server payload belongs
in the selector policy or audit record.

The implementation registers `quireforge_model_selector` through
`thread/start.dynamicTools`, accepts only exactly correlated `item/tool/call`
requests, and returns bounded normalized content. One request attempt is
allowed per turn. A successful request remains ephemeral until the turn
completes, then becomes pending metadata; resume refreshes `model/list` and
revalidates before constructing `turn/start`. If registration is rejected,
QuireForge retries without the tool and exposes recommendation-only behavior.
It does not automate a website selector, call a private endpoint, or claim
automatic control. See
[ADR 0026](DECISIONS/0026-policy-bounded-next-turn-selection.md).

## Sessions and conversation lifecycle

`thread/start` accepts an absolute `cwd`, model, approval policy, sandbox,
personality, and optional service name. `turn/start` supports turn-level `cwd`,
sandbox policy, model, effort, summary, and structured output settings.

Supported lifecycle building blocks include:

- New thread: `thread/start`.
- Continue: `thread/resume`.
- Fork: `thread/fork`, optionally through a specific turn.
- Search/list: stable `thread/list` supports cwd and title-fragment filters;
  richer item/turn pagination is experimental.
- Archive/restore: `thread/archive` and `thread/unarchive`.
- Stop: `turn/interrupt`.
- Read without loading: `thread/read`.
- Stream: `turn/*`, `item/*`, `turn/diff/updated`, and
  `turn/plan/updated` notifications.

Codex remains authoritative for these threads. Application SQLite stores only
references, grouping, view state, and project association metadata.

Milestone 7A implements new-thread/new-turn start, a reviewed normalized stream,
and exact interruption. Milestone 8A adds native-owned `thread/list`,
`thread/read`, `thread/resume`, `thread/fork`, `thread/archive`, and
`thread/unarchive` use. Every operation begins from a QuireForge application
reference, revalidates its exact attached cwd, and keeps Codex IDs, paths,
previews, transcripts, and raw status objects out of React. Listing is bounded,
uses exact cwd filters, and matches only already-owned references rather than
importing unrelated Codex threads. Startup marks stale active references
interrupted because process ownership does not survive a crash. Milestone 8B
keeps complete reconciliation separate from an optional bounded `searchTerm`
projection, accepts only matching IDs already present in the complete result,
and exposes trimmed transient titles without persisting them. Project/fork
grouping, keyboard-accessible tabs, and lifecycle controls use app-owned IDs.
Approval presentation, decisions, and expandable real-time process details
remain Milestone 9.

### Implemented Milestone 9A subset

The native adapter now accepts the reviewed stable
`item/commandExecution/requestApproval`, `item/fileChange/requestApproval`, and
`item/permissions/requestApproval` methods. It also consumes reviewed command
output, MCP progress, and server-request resolution notifications. Client
response correlation checks the method before the ID so an inbound server
request cannot be mistaken for a response even if their numeric IDs coincide.

All requests must match the active native-owned thread and turn. QuireForge
retains the native request ID only in memory and exposes a new app UUIDv7.
Command `acceptForSession`, execution-policy amendments, and network-policy
amendments are filtered out. Additional per-command permissions and network
targets are strictly parsed and summarized. Permission grants echo only the
validated profile with turn scope; denial and cancellation grant an empty
profile. Unstable file write-root grants cannot be accepted through the
Milestone 9A contract.

Detailed item normalization retains no tool arguments, file diff body,
aggregated raw output, or raw protocol payload. Commands, paths, labels, web
queries, output lines, status, and exit code cross the boundary only after
control stripping, credential redaction, path reduction, and size bounds.
Command chunks are joined through a line boundary before display redaction.
Milestone 9B remains responsible for the polished selectable/expanded activity
and approval UI.

## Authentication boundary

Preferred authentication is Codex-managed ChatGPT login:

- `account/read` discovers status.
- `account/login/start` with `type: "chatgpt"` returns an official browser URL
  and hosts the callback.
- `chatgptDeviceCode` is an official alternative.
- `account/login/cancel`, completion notifications, and `account/logout` cover
  lifecycle management.

The project will not use the experimental external-token mode, inspect browser
storage, or persist ChatGPT/OAuth tokens. It will not store API keys or
connector credentials in application SQLite. Diagnostic output must be
redacted before logging or display.

The implemented UI shows a login URL and optional one-time device code only
while the exact attempt is pending. Completion or cancellation clears both.
Email, plan, login ID, and raw completion errors never cross the native
boundary. Logout requires a second explicit action.

## Apps and connectors

`app/list` is the supported discovery source for apps available to the current
account. Live validation returned catalog and account-aware rows with fields
including ID, name, description, icons, distribution channel, metadata,
install URL, `isAccessible`, and `isEnabled`.

A paginated 2026-07-19 snapshot returned a multi-page directory across the
default OpenAI catalog and ecosystem directory, while only a small subset
reported `isAccessible`. Catalog-wide `isEnabled` and install-URL values
demonstrate why those fields must not be presented as installed, authorized,
healthy, or eligible state. The repository does not publish the account-
specific entries or counts.

The Integration Center may therefore:

- Display only rows actually returned by Codex.
- Distinguish accessible from merely discoverable entries.
- Attach an app to a prompt with a documented `mention` item and `app://` path.
- Use returned official install URLs for browser handoff.
- Display tool annotations and require approvals for side effects.

There is no basis to claim that every ChatGPT app works in Codex. Availability
remains account-, plan-, region-, workspace-, and policy-dependent. The app
must not scrape ChatGPT or call private marketplace endpoints.

Connector installation/authorization is not represented as a general stable
`app/install` RPC in the inspected documentation. Where Codex returns an
official install or authorization URL, the desktop client opens it externally
and confirms completion by refreshing `app/list` or supported health state.

## Plugins and marketplaces

CLI 0.144.6 exposes stable-looking command surfaces with JSON output:

```text
codex plugin list --available --json
codex plugin add PLUGIN@MARKETPLACE --json
codex plugin remove PLUGIN --marketplace MARKETPLACE --json
codex plugin marketplace list --json
codex plugin marketplace add SOURCE --ref REF --json
codex plugin marketplace upgrade [NAME] --json
codex plugin marketplace remove NAME --json
```

The local snapshot validated separate configured-marketplace and
available/installed-plugin collections. Their account-specific rows and counts
are not published as a guaranteed catalog.

App-server schemas include marketplace and plugin management methods, but the
official documentation explicitly marks `plugin/list`, `plugin/read`,
`plugin/install`, and `plugin/uninstall` as under development and says not to
call them from production clients. Therefore:

- Milestone 13B production discovery uses the supported CLI JSON list commands;
  Milestone 14A confirmed management uses their matching fixed CLI commands.
- The app-server plugin adapter stays disabled unless a later Codex release
  promotes the methods and contract tests pass.
- Marketplace writes and plugin install/remove actions always show a preview
  and require confirmation.
- CLI output is sanitized and normalized before display.

Official plugin structure uses required `.codex-plugin/plugin.json` plus
optional `skills/`, `hooks/`, `.app.json`, `.mcp.json`, and `assets/`. Marketplace
metadata may be personal or repository-scoped at
`.agents/plugins/marketplace.json`. Git, local, and npm-backed sources are
documented; npm installation does not run package lifecycle scripts.

## Skills

`skills/list` supports cwd-scoped discovery, forced refresh, and extra roots.
Rows include enabled state, interface metadata, and dependency metadata.
`skills/config/write` enables or disables a skill by absolute manifest path.

The QuireForge cwd returned enabled, scope, interface, and dependency metadata
for visible skills. The environment-specific rows and count are not website
content.

The desktop app must preserve scope and provenance: built-in, personal,
project, repository-provided, or plugin-bundled. A project suggestion must not
silently enable a global skill.

Milestone 14C uses only the exact native path returned by `skills/list`, behind
a fresh preview and one-use confirmation. React receives scope and normalized
state but never the manifest path.

## MCP servers

The CLI provides list/get/add/remove/login/logout commands. App-server provides
status, tools, resources, auth state, OAuth login URL, completion notification,
and startup/reauthentication failure state.

The configured-server collection was validated without recording its rows.
Names, counts, commands, working directories, URLs, and authentication details
are intentionally absent from this repository.

MCP OAuth stays owned by Codex. The desktop app may open an authorization URL
and render status, but it must not log codes/tokens or save them to SQLite.
Local MCP commands and remote endpoints are executable/security-sensitive and
must be presented separately from connector and filesystem permissions.

Milestone 14C implements that OAuth handoff with native URL ownership and exact
completion-name correlation. It does not expose MCP endpoint configuration,
add/remove/logout, tool definitions, or credentials.

## Managed policy

`configRequirements/read` returns effective administrator requirements or
`null`. Official managed configuration supports:

- Allowed approval and sandbox policies.
- Feature requirements.
- Network restrictions.
- Managed hooks and command rules.
- Marketplace source allowlists.

The inspected account returned no effective requirements. That does not permit
the client to assume other accounts are unmanaged. Compatibility evaluation
must represent `policy-blocked` independently of technical compatibility.

## Experimental or deferred surfaces

- The app-server process is experimental as a product surface and must remain
  behind a versioned adapter.
- WebSocket transport is experimental/unsupported; local stdio is the MVP
  transport.
- Plugin management RPCs are under development; use CLI fallbacks.
- Process-control APIs run outside the Codex sandbox and are inappropriate as
  the default integrated-terminal backend.
- Fine-grained permission profiles and some pagination/search operations are
  experimental.
- Hosted scheduling is deferred unless an official, eligible interface is
  discovered at implementation time.
- No private ChatGPT endpoint, browser-token extraction, or Windows-app
  reverse engineering is permitted.

## Redaction requirements

The redacted `codex doctor --json` report still includes local paths,
authentication mode, account/provider state, and runtime inventory. App-server
MCP status can include tool schemas and account-profile metadata. Logs must
therefore apply field-aware redaction, bounded output, and an allowlist before
being persisted or copied to the clipboard.

At minimum, redact tokens, authorization codes, credentials, emails, account
IDs, link IDs, signed asset URLs, home-directory details when exporting, and
environment variables with secret-like names.
