# ADR 0014: Managed worktree foundation

- Status: Accepted for Milestone 11A
- Date: 2026-07-20

## Context

QuireForge needs to create and attach Git worktrees without turning the
frontend into a general-purpose Git or process launcher. Worktrees must remain
ordinary directories that existing project, conversation, and reviewed Git
boundaries can revalidate. A failed metadata write must not cause QuireForge to
delete user work, and merely detaching project metadata must never remove a Git
worktree.

## Decision

Each worktree known to QuireForge is also an ordinary QuireForge project. An
app-owned metadata relation records its source project and whether QuireForge
created (`managed`) or only attached (`attached`) the worktree. The relation
does not store credentials, Git object IDs, conversation content, or command
output.

The native service owns all worktree paths and Git invocations:

- creation destinations are generated beneath the app data directory;
- existing worktrees are selected only through the native directory picker;
- the frontend supplies a bounded branch name and opaque project ID, never a
  working directory, executable, arbitrary ref, or argument list;
- Git is invoked directly with fixed argument arrays, a sanitized environment,
  bounded output, and a timeout;
- create and attach previews issue one-use, expiring confirmation IDs;
- confirmation revalidates the source repository and selected directory before
  changing Git or metadata;
- inventory uses `git worktree list --porcelain -z` and never exposes object
  IDs.

Milestone 11A intentionally provides no remove, prune, or filesystem-cleanup
command. If Git creates a worktree but metadata persistence fails, the service
reports the remaining worktree as recoverable and leaves it in place. Cleanup
and recovery require the separately gated Milestone 11C design.

## Consequences

Existing per-project conversation and reviewed Git safeguards apply to linked
worktrees without a second execution model. Externally discovered worktrees
remain visible but cannot be selected as QuireForge projects until explicitly
attached. Source and worktree projects are reserved during confirmation so
other QuireForge mutations cannot race the operation.

App-owned storage may accumulate an orphan after a partial failure. This is a
deliberate recoverability tradeoff: automatic cleanup would make a database
error capable of destroying uncommitted work.
