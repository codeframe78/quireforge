# QuireForge Desktop

Status: desktop work through local Milestone 21A is implemented and verified.
This package contains the Tauri shell, typed Rust/TypeScript boundary, supported
Codex app-server adapter, Codex-owned authentication gate and handoff, direct
local project attachment, conversation/session/approval presentation, reviewed
Git status/diff/mutation, bounded parallel worktrees, retained-worktree
recovery, explicit clean managed-worktree removal, native PTY tabs,
normalized/confirmed integration workflows, safe project-file previews,
bounded PNG/JPEG conversation attachments, and documented read-only remaining
usage. User-facing workspaces are unavailable until the normalized Codex
account state grants access. Generic file attachment, worktree prune, force
cleanup, advanced remote Git operations, public downloads, and release
publication remain separately gated.

Run package checks from the repository root with `pnpm validate`. Start the
native development window with `pnpm desktop:dev` after installing the Linux
packages documented in [`docs/BUILDING.md`](../../docs/BUILDING.md).

`pnpm codex:schema` refreshes only the reviewed initialize, `model/list`, and
account rate-limit schema subset for the installed Codex CLI. Review the
generated diff and update the versioned adapter before accepting a new schema
snapshot.
