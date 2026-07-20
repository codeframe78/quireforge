# ADR 0012: Read-only Git review boundary

- Status: Accepted
- Date: 2026-07-20
- Milestone: 10A

## Context

QuireForge needs repository status and diff review without allowing the webview
to run Git, choose a working directory, read arbitrary paths, or smuggle Git
arguments through a generic command. Attached directories may be repository
roots or subdirectories and may be read-only. Repository configuration and
filenames are untrusted inputs, while Git remains authoritative for repository
state.

Milestone 10 also includes stage, revert, and commit workflows. Those mutations
carry different approval, recovery, secret-review, and concurrency obligations,
so implementing them behind the read-only review surface would weaken the
milestone gate.

## Decision

Milestone 10 is split. Milestone 10A exposes only three fixed native commands:

- status for one app-owned attached-project ID;
- a bounded staged or working-tree diff for one path from the current status
  snapshot; and
- an explicit default-editor handoff for one revalidated changed regular file.

Rust reloads attachment metadata and revalidates selected/resolved identity on
every request. Review accepts connected writable or read-only repositories and
never substitutes a repository root, recent directory, or home directory. Git
runs without a shell, with a fixed executable search path, explicit current
directory, cleared environment, disabled prompts, pagers, optional locks,
filesystem monitors, untracked caches, external diffs, and text conversion.
Every child has bounded stdout/stderr, an eight-second timeout, and kill/wait
cleanup.

Status is limited to the attached directory. Because porcelain-v2 NUL-delimited
paths remain repository-root-relative from an attached subdirectory, Rust
derives and strips the exact prefix between the revalidated worktree and
attachment roots. It then discards object IDs, rejects absolute, escaping,
control-bearing, directional-formatting, backslash-bearing, or non-UTF-8 paths,
and caps the changed-file list. Before a diff or editor handoff, native code
obtains a fresh status and requires an exact matching path and area. Worktree
diff candidates must remain regular files inside the attachment; symlinks,
submodules, and conflicts are not reviewable. React receives only normalized
branch counts, change enums, and bounded line records. Raw Git output, Git
arguments, absolute paths, object IDs, and repository configuration do not
cross IPC.

No Git review state or diff content is persisted. Browser preview reports the
native boundary as unavailable rather than fabricating repository data.

Stage, unstage, revert, commit, branch, worktree, push, and pull operations are
not present in 10A. They require the separately reasoned and approved Milestone
10B or later milestone.

## Consequences

- Review works with the installed Git implementation and existing repository
  semantics without adding a second Git library or changing repository data.
- Large, malformed, binary, conflicted, submodule, deceptive, or unavailable
  content degrades to bounded stable states.
- A changed file is opened only after a deliberate user action and a final
  native path/type/containment check; QuireForge never accepts an absolute file
  path from React.
- Mutation design cannot reuse a generic version of these commands. It must add
  operation-specific previews, approvals, postcondition checks, recovery, and
  secret review in a new gate.

## Alternatives considered

- A generic `git(args, cwd)` IPC command was rejected because it would turn the
  webview into a shell-equivalent Git authority.
- Reading `.git` files directly was rejected because linked worktrees,
  repository configuration, index formats, and Git compatibility would be
  reimplemented incorrectly.
- Adding libgit2 was deferred because the current read-only scope needs no new
  dependency and Git CLI compatibility is already available.
- Combining review and mutation controls was rejected because it would blur
  approval and recovery boundaries.
