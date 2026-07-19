# QuireForge Desktop

Status: Milestone 4 process adapter implemented and verified locally. This
package establishes the native shell, a small typed Tauri command boundary, and
a versioned Codex compatibility layer with CLI detection, supervised app-server
stdio, normalized model metadata, bounded failures, and deterministic mocks.
Authentication, project attachment, conversations, persistence, terminal, Git,
and integration workflows are not implemented here yet.

Run package checks from the repository root with `pnpm validate`. Start the
native development window with `pnpm desktop:dev` after installing the Linux
packages documented in [`docs/BUILDING.md`](../../docs/BUILDING.md).

`pnpm codex:schema` refreshes only the reviewed initialize and `model/list`
schema subset for the installed Codex CLI. Review the generated diff and update
the versioned adapter before accepting a new schema snapshot.
