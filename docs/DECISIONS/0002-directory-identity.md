# ADR 0002: Preserve selected and resolved directory identity

- Status: Accepted and implemented in Milestone 6
- Date: 2026-07-19

## Context

Users must attach an original directory in place, including symbolic links,
removable media, network filesystems, repositories, and worktrees. Live
app-server validation showed that a symlink cwd is observed by child processes
as its resolved target.

## Decision

Persist the user-selected absolute path and the last verified resolved path as
separate values. Add filesystem and Git identity evidence. Revalidate before
every task and fail closed on missing, changed, or ambiguous identity.

## Consequences

- UI can preserve the user's chosen path while accurately explaining the
  execution target.
- Moved-directory relinking can compare evidence before update.
- Device/inode identity is advisory on some network filesystems, so multiple
  signals and an `unknown` state are required.
