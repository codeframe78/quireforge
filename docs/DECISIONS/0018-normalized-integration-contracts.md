# ADR 0018: Normalized integration discovery contracts

- Status: Accepted
- Date: 2026-07-21

## Context

Codex CLI 0.145.0 exposes connector, plugin, marketplace, skill, MCP, policy,
permission-profile, and client-owned dynamic-tool surfaces. The app-server is
still labeled experimental as a whole, individual methods have different
stability, and the stable plugin CLI has different output and mutation
semantics. Raw results may also contain account-scoped identifiers, local
paths, remote URLs, tool descriptions, policy details, or untrusted publisher
text that must not become an implicit frontend or persistence contract.

QuireForge needs one honest catalog that preserves unavailable, blocked,
degraded, and unknown states. It must not imply that an upstream method is
implemented merely because the installed CLI advertises it, and it must keep
discovery separate from installation, authorization, configuration, and
removal.

## Decision

Introduce `codex-integration-v1`, a category-preserving normalized contract
with these boundaries:

- `IntegrationCapability` records the domain, operation, selected official
  route, method stability, upstream availability, QuireForge implementation
  state, mutation flag, confirmation requirement, and one stable diagnostic
  code. Upstream availability and application implementation are separate.
- `IntegrationEntry` records only a bounded application identity, display
  metadata, category, scope, source class, installation/enablement/auth state,
  normalized capabilities, permissions, requirements, policy, and health.
- `IntegrationPolicySnapshot` represents effective requirements and permission
  profile availability without returning raw configuration or managed-policy
  contents.
- Health uses closed ready, degraded, blocked, unavailable, and unknown states.
  Failed sources do not disappear from the catalog and do not make unrelated
  sources look unavailable.
- Refresh notifications are invalidation signals only. The native service will
  re-run the applicable bounded discovery request instead of merging raw event
  payloads into frontend state.

The reviewed route matrix for CLI 0.145.0 is:

| Domain              | Read/discovery route                                                              | Mutating route                                                            | Milestone 13A classification                                                   |
| ------------------- | --------------------------------------------------------------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| Apps/connectors     | app-server `app/list` and `app/installed`                                         | Supported app-server configuration or official authorization handoff only | Stable methods on an experimental server; unnecessary `app/read` is not called |
| Plugins             | Stable `codex plugin list --available --json`                                     | Stable `codex plugin ... --json` fixed commands preferred                 | App-server plugin lifecycle remains experimental and disabled in production    |
| Marketplaces        | Stable `codex plugin marketplace list --json`                                     | Stable fixed `codex plugin marketplace` commands                          | Local paths remain native-only                                                 |
| Skills              | app-server `skills/list` and invalidation notification                            | app-server `skills/config/write`                                          | Stable methods on an experimental server                                       |
| MCP                 | app-server status, resource, refresh, and OAuth methods                           | Official returned authorization URL and fixed refresh/config flows        | Stable methods on an experimental server                                       |
| Policy/requirements | app-server `configRequirements/read`, `config/read`, and `permissionProfile/list` | No generic frontend configuration writer                                  | Raw managed requirements never cross IPC                                       |

Every mutation remains a separately reviewed preview/confirmation operation.
React will never receive a generic method name, argument vector, filesystem
path, configuration object, or JSON-RPC payload.

## Dynamic-tool lifecycle

The generated 0.145.0 schemas establish a supported host-owned lifecycle:

1. A client registers a bounded function or namespace through the
   `dynamicTools` field on `thread/start`.
2. Codex invokes that registered tool through the server request
   `item/tool/call`.
3. Native code correlates the JSON-RPC request ID, validates the thread, turn,
   namespace, tool, and closed arguments, performs only the app-owned action,
   and returns a bounded `DynamicToolCallResponse`.
4. The result is reduced to approved text, image, or audio content items. Raw
   arguments and native request identities do not cross IPC or persist.

This lifecycle is sufficient for the future Milestone 18 selector-control
boundary. It does not allow an executing turn to change its own model. A model
selection request can only stage an app-owned, policy-validated choice for the
next `turn/start`. QuireForge will not automate a web selector, call a private
endpoint, or edit Codex-owned configuration to provide this feature.

Milestone 13A records this lifecycle as `contract-only`. Milestone 13B
implements the read-only catalog and one fixed Tauri IPC read while keeping
dynamic-tool registration, installation, authorization, configuration
mutation, and the Integration Center UI in later milestones.

Milestone 14A subsequently implements the `plugin.install`, `plugin.remove`,
and `marketplace.configure` capabilities as fixed native preview/confirm
operations while preserving this normalized catalog boundary. The mutation
architecture and its additional source-review/revalidation rules are recorded
in [ADR 0019](0019-confirmed-integration-mutations.md). Authorization,
enable/disable configuration, the Integration Center UI, and dynamic-tool
registration remain outside 14A.

## Security and privacy invariants

- Do not persist or expose Codex account identifiers, connector credentials,
  OAuth state, authorization URLs after handoff, raw tool arguments, raw
  configuration, managed requirements, absolute paths, or marketplace loader
  messages.
- Treat all publisher, marketplace, plugin, skill, MCP, and app display text as
  untrusted. Strip control and directional characters and apply size/count
  bounds before it enters normalized state.
- Preserve administrator blocks and requirements. Application policy can be
  stricter but cannot weaken Codex or administrator enforcement.
- Installation and authorization are different actions. Installing a plugin
  does not imply trusting hooks or authorizing a connector/MCP server.
- Unknown fields and unsupported versions fail closed into stable diagnostics;
  they are not passed through for React to interpret.

## Consequences

- Deterministic Rust and TypeScript contracts can evolve independently of raw
  protocol and CLI JSON shapes.
- Capability discovery can be partially useful while one source is degraded.
- Later implementation must normalize each route separately and revalidate
  state immediately before any mutation.
- The catalog carries more explicit state, but avoids ambiguous booleans and
  prevents advertised upstream support from being mistaken for completed
  QuireForge functionality.
