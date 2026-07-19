# ADR 0001: Versioned Codex adapters

- Status: Accepted for implementation planning
- Date: 2026-07-19

## Context

The installed CLI exposes an app-server rich enough for desktop workflows, but
the CLI labels the server experimental and some individual methods—especially
plugin management—are explicitly under development. Stable CLI JSON commands
cover several of those gaps.

## Decision

Use a versioned `CodexBackend` boundary with app-server, CLI JSON, mock, and
unavailable adapters. Select capability routes per method, not once for the
whole application. Normalize all output before frontend delivery.

Use local stdio for app-server MVP transport. Do not depend on WebSocket
transport, TUI scraping, private ChatGPT endpoints, or Codex internal storage.

## Consequences

- The UI can disable unsupported features honestly.
- Contract fixtures are required for each supported Codex version family.
- CLI fallbacks may provide less interactive progress than app-server.
- Experimental promotion or breakage stays isolated in one adapter.
