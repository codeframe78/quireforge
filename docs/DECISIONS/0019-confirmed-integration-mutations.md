# ADR 0019: Confirmed native integration mutations

- Status: Accepted
- Date: 2026-07-21

## Context

Milestone 13 established a normalized read-only integration catalog. Codex CLI
0.145.x also exposes stable JSON commands for plugin installation/removal and
marketplace add/remove/upgrade, while the richer app-server plugin lifecycle is
explicitly under development. These mutations can install executable hooks,
skills, MCP servers, or connector metadata and can fetch remote content.

A generic command bridge, raw source details in React, mutable confirmations,
or success inferred only from process exit would turn the Integration Center
into a supply-chain and confused-deputy boundary. Personal Codex state must
also remain outside deterministic tests.

## Decision

Milestone 14A introduces `IntegrationMutationService` as the only plugin and
marketplace mutation owner:

- React can request one of five closed operations: plugin install/remove or
  marketplace add/remove/upgrade. It sends an opaque normalized entry ID, or a
  bounded `owner/repository` and pinned 40- or 64-hex reference for marketplace
  add. It never sends a program, subcommand, argument vector, path, URL, or
  arbitrary JSON.
- Marketplace remove is available only for an explicitly configured source;
  built-in/default marketplace rows are not represented as removable.
- Preview forces a fresh normalized catalog/policy rebuild, verifies the
  reviewed CLI 0.145.x minor and ready capability, resolves the opaque ID to a
  fresh native CLI row, and inspects source evidence before returning only
  normalized display name, source class, permissions, and closed warnings.
- Local plugin review canonicalizes the source directory, refuses a symlinked
  or oversized manifest, and checks manifest name/version plus declared hooks,
  MCP servers, apps, skills, and the documented default `hooks/hooks.json`.
  Bundled hooks are shown as a separate-trust execution risk because install or
  enable does not itself trust them. Repository plugins require credential-free
  HTTPS and a 40- or 64-hex commit. Package sources require a bounded package
  name, exact version, and safe registry URL.
- A ready preview creates a process-local UUIDv7 confirmation that expires in
  five minutes and can be consumed once. At most 32 previews can remain
  pending. Mutations are serialized.
- Confirmation forces another catalog/policy rebuild, rechecks the CLI minor,
  exact normalized entry, effective policy, capability, and native raw source
  evidence, then runs one fixed CLI command from neutral `/` with null
  stdin/stderr, a 30-second timeout, a one-MiB stdout cap, credential removal,
  and child kill/reap handling.
- Success requires a closed operation-specific JSON result and a fresh CLI
  postcondition read. Process exit alone is insufficient. Raw CLI results,
  source URLs, install paths, marketplace roots, and errors are discarded.
- Marketplace upgrades explicitly warn that remote contents are mutable
  because the configured marketplace-list record does not identify the next
  fetched artifact. The confirmation reviews the configured source and risk;
  it does not claim artifact pinning.

The shared Rust/TypeScript contract exposes fixed preview and confirm commands
only. Authorization remains separate from installation.

## Test-state isolation

Routine mutation tests use deterministic temporary shell fixtures. The explicit
real-CLI proof is ignored by default and creates temporary `CODEX_HOME` and
`HOME` directories, a local marketplace, and a local plugin; it installs and
removes only that fixture and deletes the temporary tree. It neither reads nor
changes personal Codex configuration, authentication, connectors, plugins, or
marketplaces.

## Consequences

- Milestone 14B can build an accessible permission-review UI without gaining a
  generic native execution primitive.
- A CLI minor change, malformed response, stale catalog, expired/replayed
  confirmation, policy change, source change, failed postcondition, unsafe URL,
  or unpinned install source fails closed with a stable diagnostic.
- Marketplace upgrades remain an explicitly disclosed mutable-source action;
  users must not interpret their preview as a content hash review.
- Connector/MCP authorization, plugin enable/disable, skill configuration,
  prompt mentions, and user-facing Integration Center behavior need separate
  gates and supported-route review.
