# ADR 0013: Reviewed Git mutation boundary

- Status: Accepted
- Date: 2026-07-20
- Milestone: 10B

## Context

After the read-only boundary in ADR 0012, QuireForge needs deliberate stage,
unstage, revert, and commit workflows. These operations can lose uncommitted
work, commit credentials, race Codex or another Git process, run repository
extensions, or affect paths outside a subdirectory attachment. React must not
gain a generic Git, process, path, index, object, or reference API.

## Decision

Milestone 10B adds three fixed Tauri commands: operation preview, confirmation,
and one-time revert recovery. A preview accepts one app-owned project ID and a
closed operation. File operations accept one normalized attachment-relative
path; commit accepts one bounded message. Native code revalidates writable
attachment, worktree, Git-directory, and common-directory identity, reserves
the project against concurrent Codex work, captures exact Git evidence, and
retains the plan behind an expiring in-memory UUIDv7 token. Confirmation accepts
only that token, consumes it once, reacquires project ownership, and rejects
expired, moved, changed, or busy state. React cannot resubmit or broaden the
reviewed paths during confirmation.

Stage and unstage are file-level index operations. The native plan records the
exact old index entry, worktree object evidence, status, and ordinary Unix mode.
Confirmation rechecks that evidence, applies the fixed operation, checks the
new index/status state, and attempts to restore the exact prior index entry if
the postcondition fails. Symlinks, special files, filters, working-tree
encodings, conflicts, submodules, deceptive paths, and unsupported change
shapes fail closed.

Revert is limited to a reviewed tracked regular-file modification of at most
one MiB. It preserves the indexed/staged version, snapshots the worktree bytes
and mode in memory, runs a worktree-only restore, and verifies content and
status. A successful revert returns a bounded, 30-minute, single-use recovery
token. Recovery requires the reverted content to remain unchanged and restores
through a same-directory temporary file and atomic rename. Recovery tokens and
backups are intentionally not persisted and are lost on application exit.

Commit requires every staged path and any rename source to stay inside the
exact attachment. It rejects conflicts, submodules, active merge/cherry-pick/
revert state, missing repository-local `user.name` or `user.email`, and staged
content that cannot be scanned within one MiB per blob and four MiB total.
High-confidence sensitive filenames, private keys, GitHub tokens, OpenAI API
keys, and the same token patterns in the commit message block confirmation.
The scanner is a preventative boundary, not a guarantee that all secrets can
be recognized.

Commit uses Git plumbing rather than porcelain commit. Native code verifies the
reviewed index, writes its tree, acquires the exact worktree index lock,
revalidates the evidence and tree/index equality, creates the commit with
repository-local identity, and updates `HEAD` with an expected-old-value check.
The inherited environment, global/system configuration, prompts, signing, and
hooks are disabled. The final `HEAD` and clean-index postconditions are checked;
an unexpected postcondition triggers an expected-value reference rollback.

The coordinator retains at most 32 pending confirmations and 16 recoveries.
New previews replace an older pending preview for the same project. No preview,
token, staged content, commit message, object ID, backup, or recovery data is
stored in SQLite or forwarded to browser preview.

Branch creation/deletion, reset, checkout, stash, worktree management, remote
operations, push, pull, force operations, arbitrary Git commands, and source
file deletion remain outside this boundary.

## Consequences

- Every mutation has a fresh review/confirm separation and project ownership
  check without turning the webview into a Git command surface.
- Stage/unstage can be reversed explicitly; revert has bounded post-operation
  recovery; every operation has exact postcondition checks and best-effort
  rollback for unexpected results.
- Commit does not run user hooks, signing helpers, editors, credential prompts,
  or global configuration. Users who require those workflows continue to use
  their terminal or editor.
- Recovery is deliberately temporary and process-local. The UI must describe
  that limit and must never call it a durable backup.
- Conservative path, content-size, identity, attribute, and repository-state
  refusal means some valid advanced Git workflows remain unavailable.

## Alternatives considered

- A generic `git(args, cwd)` command was rejected because it grants shell-like
  repository authority to React.
- Sending paths or messages again during confirmation was rejected because the
  second request could differ from the reviewed plan.
- `git commit --no-verify` was rejected because it does not disable every hook
  or all signing/editor configuration.
- Persisting revert backups was rejected because it would put source content
  into QuireForge metadata and expand retention, encryption, and deletion
  obligations.
- Reset/checkout-based rollback was rejected because it can overwrite unrelated
  staged or working-tree changes.
