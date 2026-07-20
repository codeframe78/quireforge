# QuireForge Desktop

Status: desktop work through Milestone 10 is implemented and verified locally.
This package contains the Tauri shell, typed Rust/TypeScript boundary, supported
Codex app-server adapter, Codex-owned authentication handoff, direct local
project attachment, conversation/session/approval presentation, and reviewed
Git status, diff, stage, unstage, bounded revert/recovery, and commit workflows.
Terminal, advanced worktree/remote Git operations, integrations, packaging, and
release workflows remain separately gated.

Run package checks from the repository root with `pnpm validate`. Start the
native development window with `pnpm desktop:dev` after installing the Linux
packages documented in [`docs/BUILDING.md`](../../docs/BUILDING.md).

`pnpm codex:schema` refreshes only the reviewed initialize and `model/list`
schema subset for the installed Codex CLI. Review the generated diff and update
the versioned adapter before accepting a new schema snapshot.
