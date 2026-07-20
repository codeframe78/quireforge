import { useEffect, useRef, useState } from "react";

import type {
  GitDiffRequest,
  GitDiffSnapshot,
  GitMutationPreviewRequest,
  GitMutationPreviewSnapshot,
  GitMutationResultSnapshot,
  GitWorkspaceSnapshot,
} from "./lib/git";

type GitAvailability = "checking" | "native" | "preview";

interface GitWorkspaceProps {
  availability: GitAvailability;
  projectName: string | null;
  snapshot: GitWorkspaceSnapshot;
  diff: GitDiffSnapshot | null;
  selectedRequest: GitDiffRequest | null;
  mutationPreview: GitMutationPreviewSnapshot | null;
  mutationResult: GitMutationResultSnapshot | null;
  busy: boolean;
  actionError: boolean;
  onRefresh: () => Promise<void>;
  onReview: (request: GitDiffRequest) => Promise<void>;
  onOpen: (projectId: string, path: string) => Promise<void>;
  onPreviewMutation: (request: GitMutationPreviewRequest) => Promise<void>;
  onConfirmMutation: (confirmationId: string) => Promise<void>;
  onCancelMutation: () => void;
  onRecoverMutation: (recoveryId: string) => Promise<void>;
}

const diagnosticLabels: Record<
  NonNullable<GitWorkspaceSnapshot["diagnosticCode"]>,
  string
> = {
  "project-not-found": "Select an attached project to review.",
  "directory-unavailable": "The attached directory is not currently available.",
  "identity-changed":
    "The directory identity changed. Relink it before reviewing changes.",
  "not-repository": "This attached directory is not inside a Git repository.",
  "git-unavailable": "Git is not available through the native runtime.",
  "git-failed": "Git could not produce a bounded status snapshot.",
  "output-too-large": "The Git result exceeded the safe review limit.",
  "invalid-path": "Git returned a path outside the review contract.",
  "diff-unavailable": "That diff is not available for review.",
  "mutation-unavailable":
    "That change is not safe to apply through this workflow.",
  "read-only": "This attached directory is read-only.",
  "project-busy":
    "Codex or another Git operation is currently using this project.",
  "stale-preview":
    "The repository changed after review. Preview the operation again.",
  "confirmation-expired":
    "This confirmation expired. Preview the operation again.",
  "secret-detected":
    "The staged content contains a likely secret and cannot be committed.",
  "unscannable-content":
    "The staged content exceeds the bounded secret-review limits.",
  "identity-unavailable":
    "Set repository-local user.name and user.email before committing.",
  "outside-attachment":
    "The index contains staged changes outside this attached directory.",
  "postcondition-failed":
    "Git did not reach the reviewed state; QuireForge attempted rollback.",
  "recovery-unavailable": "That one-time recovery is no longer available.",
};

const operationLabels = {
  stage: "Stage change",
  unstage: "Unstage change",
  revert: "Revert working-tree change",
  commit: "Commit staged changes",
} as const;

const secretLabels = {
  "forbidden-path": "sensitive filename",
  "private-key": "private key",
  "git-hub-token": "GitHub token",
  "open-ai-api-key": "OpenAI API key",
} as const;

function branchLabel(snapshot: GitWorkspaceSnapshot): string {
  if (!snapshot.branch) return "No branch data";
  return snapshot.branch.detached
    ? "Detached HEAD"
    : (snapshot.branch.head ?? "Unborn branch");
}

function areaLabel(area: GitDiffRequest["area"]): string {
  return area === "staged" ? "Staged" : "Working tree";
}

export function GitWorkspace({
  availability,
  projectName,
  snapshot,
  diff,
  selectedRequest,
  mutationPreview,
  mutationResult,
  busy,
  actionError,
  onRefresh,
  onReview,
  onOpen,
  onPreviewMutation,
  onConfirmMutation,
  onCancelMutation,
  onRecoverMutation,
}: GitWorkspaceProps) {
  const [commitMessage, setCommitMessage] = useState("");
  const confirmationRef = useRef<HTMLElement>(null);
  const mutationBusyRef = useRef(busy);
  const cancelMutationRef = useRef(onCancelMutation);
  const selectedChange = selectedRequest
    ? snapshot.changes.find((change) => change.path === selectedRequest.path)
    : undefined;
  const openable =
    selectedChange?.reviewable === true &&
    selectedChange.staged !== "deleted" &&
    selectedChange.worktree !== "deleted";
  const validCommitMessage =
    commitMessage.length > 0 &&
    commitMessage.length <= 512 &&
    commitMessage.trim() === commitMessage;

  useEffect(() => {
    mutationBusyRef.current = busy;
  }, [busy]);

  useEffect(() => {
    cancelMutationRef.current = onCancelMutation;
  }, [onCancelMutation]);

  useEffect(() => {
    if (!mutationPreview) return;
    const previous = document.activeElement as HTMLElement | null;
    const handleKeyDown = (event: globalThis.KeyboardEvent) => {
      if (event.key === "Escape" && !mutationBusyRef.current) {
        event.preventDefault();
        cancelMutationRef.current();
        return;
      }
      if (event.key !== "Tab") return;
      const controls = Array.from(
        confirmationRef.current?.querySelectorAll<HTMLElement>(
          "button:not(:disabled), [href], input:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex='-1'])",
        ) ?? [],
      );
      if (controls.length === 0) {
        event.preventDefault();
        confirmationRef.current?.focus();
        return;
      }
      const first = controls[0]!;
      const last = controls.at(-1)!;
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    const frame = window.requestAnimationFrame(() => {
      confirmationRef.current
        ?.querySelector<HTMLElement>("button:not(:disabled)")
        ?.focus();
    });
    return () => {
      window.cancelAnimationFrame(frame);
      document.removeEventListener("keydown", handleKeyDown);
      if (previous?.isConnected) previous.focus();
    };
  }, [mutationPreview]);

  function requestFileMutation(
    operation: "stage" | "unstage" | "revert",
    path: string,
  ) {
    if (!snapshot.projectId) return;
    void onPreviewMutation({
      projectId: snapshot.projectId,
      operation,
      path,
      message: null,
    });
  }

  return (
    <section className="git-workspace" id="changes" aria-labelledby="git-title">
      <div className="git-workspace__heading">
        <div>
          <p className="eyebrow">Source control · reviewed operations</p>
          <h2 id="git-title">Review each Git change before applying it.</h2>
          <p>
            QuireForge uses fixed native workflows for status, diffs, staging,
            unstaging, reverting, and commits. Every write receives a fresh
            preview and a separate confirmation; arbitrary Git arguments are
            never accepted.
          </p>
        </div>
        <button
          className="auth-button"
          type="button"
          disabled={availability !== "native" || projectName === null || busy}
          onClick={() => void onRefresh()}
        >
          {busy ? "Reviewing…" : "Refresh changes"}
        </button>
      </div>

      {availability === "checking" && (
        <p className="git-message" role="status">
          Checking the selected project…
        </p>
      )}
      {availability === "preview" && (
        <p className="git-message" role="status">
          Native Git review is unavailable in browser preview. No repository
          data is simulated.
        </p>
      )}
      {actionError && (
        <p className="git-message git-message--warning" role="alert">
          The review action failed before a validated result was available.
        </p>
      )}

      {mutationResult?.state === "applied" && (
        <div className="git-mutation-result" role="status">
          <div>
            <strong>{operationLabels[mutationResult.operation!]}</strong>
            <span>The repository was updated and revalidated.</span>
          </div>
          {mutationResult.recoveryId && (
            <button
              type="button"
              disabled={busy}
              onClick={() => void onRecoverMutation(mutationResult.recoveryId!)}
            >
              Restore reverted content
            </button>
          )}
        </div>
      )}
      {mutationResult?.state === "unavailable" && (
        <p className="git-message git-message--warning" role="alert">
          {mutationResult.diagnosticCode
            ? diagnosticLabels[mutationResult.diagnosticCode]
            : "The Git operation was not applied."}
        </p>
      )}

      {availability === "native" && snapshot.state === "unavailable" && (
        <div className="git-empty">
          <span aria-hidden="true">±</span>
          <div>
            <h3>{projectName ?? "No project selected"}</h3>
            <p>
              {snapshot.diagnosticCode
                ? diagnosticLabels[snapshot.diagnosticCode]
                : "Git review is unavailable."}
            </p>
          </div>
        </div>
      )}

      {availability === "native" && snapshot.state !== "unavailable" && (
        <>
          <div className="git-summary" aria-live="polite">
            <div>
              <span>Project</span>
              <strong>{projectName}</strong>
            </div>
            <div>
              <span>Branch</span>
              <strong>{branchLabel(snapshot)}</strong>
            </div>
            <div>
              <span>Sync</span>
              <strong>
                ↑{snapshot.branch?.ahead ?? 0} ↓{snapshot.branch?.behind ?? 0}
              </strong>
            </div>
            <div>
              <span>Changes</span>
              <strong>{snapshot.changes.length}</strong>
            </div>
          </div>

          {snapshot.state === "clean" ? (
            <div className="git-empty">
              <span aria-hidden="true">✓</span>
              <div>
                <h3>Working tree clean</h3>
                <p>
                  No staged or working-tree changes are present in the attached
                  directory.
                </p>
              </div>
            </div>
          ) : (
            <div className="git-review-layout">
              <div className="git-change-list" aria-label="Changed files">
                {snapshot.changes.map((change) => (
                  <article className="git-change" key={change.path}>
                    <div>
                      <strong title={change.path}>{change.path}</strong>
                      {change.previousPath && (
                        <span>from {change.previousPath}</span>
                      )}
                      <small>
                        {change.conflict
                          ? "Conflict"
                          : change.submodule
                            ? "Submodule"
                            : "File"}
                      </small>
                    </div>
                    <div className="git-change__areas">
                      {change.staged && (
                        <button
                          type="button"
                          aria-pressed={
                            selectedRequest?.path === change.path &&
                            selectedRequest.area === "staged"
                          }
                          disabled={!change.reviewable || busy}
                          onClick={() =>
                            void onReview({
                              projectId: snapshot.projectId!,
                              path: change.path,
                              area: "staged",
                            })
                          }
                        >
                          Staged · {change.staged}
                        </button>
                      )}
                      {change.worktree && (
                        <button
                          type="button"
                          aria-pressed={
                            selectedRequest?.path === change.path &&
                            selectedRequest.area === "worktree"
                          }
                          disabled={!change.reviewable || busy}
                          onClick={() =>
                            void onReview({
                              projectId: snapshot.projectId!,
                              path: change.path,
                              area: "worktree",
                            })
                          }
                        >
                          Working · {change.worktree}
                        </button>
                      )}
                    </div>
                    {change.reviewable && (
                      <div
                        className="git-change__mutations"
                        aria-label={`Actions for ${change.path}`}
                      >
                        {change.worktree &&
                          [
                            "modified",
                            "added",
                            "deleted",
                            "untracked",
                          ].includes(change.worktree) && (
                            <button
                              type="button"
                              disabled={busy}
                              onClick={() =>
                                requestFileMutation("stage", change.path)
                              }
                            >
                              Stage
                            </button>
                          )}
                        {change.staged &&
                          ["modified", "added", "deleted"].includes(
                            change.staged,
                          ) && (
                            <button
                              type="button"
                              disabled={busy}
                              onClick={() =>
                                requestFileMutation("unstage", change.path)
                              }
                            >
                              Unstage
                            </button>
                          )}
                        {change.worktree === "modified" && (
                          <button
                            className="git-danger-button"
                            type="button"
                            disabled={busy}
                            onClick={() =>
                              requestFileMutation("revert", change.path)
                            }
                          >
                            Revert
                          </button>
                        )}
                      </div>
                    )}
                  </article>
                ))}
                {snapshot.truncated && (
                  <p className="git-limit">
                    Additional changes were omitted at the safety limit.
                  </p>
                )}
              </div>

              <div className="git-diff" aria-live="polite">
                {!selectedRequest && (
                  <div className="git-diff__empty">
                    <strong>Select a change</strong>
                    <span>
                      Choose its staged or working-tree record to inspect a
                      bounded diff.
                    </span>
                  </div>
                )}
                {selectedRequest && !diff && (
                  <div className="git-diff__empty">
                    <strong>
                      Loading {areaLabel(selectedRequest.area).toLowerCase()}{" "}
                      diff…
                    </strong>
                  </div>
                )}
                {selectedRequest && diff?.state === "unavailable" && (
                  <div className="git-diff__empty">
                    <strong>Diff unavailable</strong>
                    <span>
                      {diff.diagnosticCode
                        ? diagnosticLabels[diff.diagnosticCode]
                        : "No normalized diff was returned."}
                    </span>
                  </div>
                )}
                {selectedRequest && diff?.state === "ready" && (
                  <>
                    <div className="git-diff__heading">
                      <div>
                        <span>{areaLabel(diff.area)}</span>
                        <strong title={diff.path}>{diff.path}</strong>
                      </div>
                      <button
                        type="button"
                        disabled={!openable || busy}
                        onClick={() => void onOpen(diff.projectId, diff.path)}
                      >
                        Open in default editor
                      </button>
                    </div>
                    {diff.kind === "binary" ? (
                      <div className="git-diff__empty">
                        <strong>Binary change</strong>
                        <span>
                          Binary contents are intentionally not forwarded to the
                          interface.
                        </span>
                      </div>
                    ) : (
                      <div
                        className="git-diff__lines"
                        role="table"
                        aria-label={`Diff for ${diff.path}`}
                      >
                        {diff.lines.map((line, index) => (
                          <div
                            className={`git-line git-line--${line.kind}`}
                            role="row"
                            key={`${index}-${line.oldLine}-${line.newLine}`}
                          >
                            <span role="cell">{line.oldLine ?? ""}</span>
                            <span role="cell">{line.newLine ?? ""}</span>
                            <code role="cell">{line.text}</code>
                          </div>
                        ))}
                        {diff.lines.length === 0 && (
                          <div className="git-diff__empty">
                            <span>
                              No textual lines were returned for this change.
                            </span>
                          </div>
                        )}
                      </div>
                    )}
                    {diff.truncated && (
                      <p className="git-limit">
                        The diff was truncated at the safety limit.
                      </p>
                    )}
                  </>
                )}
              </div>
            </div>
          )}

          {snapshot.changes.some((change) => change.staged !== null) && (
            <form
              className="git-commit"
              onSubmit={(event) => {
                event.preventDefault();
                if (!snapshot.projectId || !validCommitMessage) return;
                void onPreviewMutation({
                  projectId: snapshot.projectId,
                  operation: "commit",
                  path: null,
                  message: commitMessage,
                });
              }}
            >
              <div>
                <label htmlFor="git-commit-message">Commit message</label>
                <span>
                  Only reviewed staged files inside this attachment can be
                  committed.
                </span>
              </div>
              <textarea
                id="git-commit-message"
                maxLength={512}
                rows={3}
                value={commitMessage}
                disabled={busy}
                onChange={(event) => setCommitMessage(event.target.value)}
              />
              <button type="submit" disabled={busy || !validCommitMessage}>
                Preview commit
              </button>
            </form>
          )}
        </>
      )}

      {mutationPreview && (
        <div className="git-confirmation-backdrop">
          <section
            ref={confirmationRef}
            className="git-confirmation"
            role={mutationPreview.destructive ? "alertdialog" : "dialog"}
            aria-modal="true"
            aria-labelledby="git-confirmation-title"
            aria-busy={busy}
            tabIndex={-1}
          >
            <p className="eyebrow">
              {mutationPreview.state === "ready"
                ? "Confirmation required"
                : "Operation unavailable"}
            </p>
            <h3 id="git-confirmation-title">
              {operationLabels[mutationPreview.operation]}
            </h3>
            {mutationPreview.state === "ready" ? (
              <>
                <p>
                  Confirm only after reviewing these exact targets. The native
                  preview expires and cannot be reused.
                </p>
                <ul>
                  {mutationPreview.targets.map((target) => (
                    <li key={target.path}>{target.path}</li>
                  ))}
                </ul>
                {mutationPreview.destructive && (
                  <p className="git-confirmation__warning">
                    Revert replaces the working-tree file with its indexed
                    content. A bounded, one-time recovery is offered after a
                    successful operation.
                  </p>
                )}
              </>
            ) : (
              <>
                <p className="git-confirmation__warning">
                  {mutationPreview.diagnosticCode
                    ? diagnosticLabels[mutationPreview.diagnosticCode]
                    : "This operation cannot be confirmed."}
                </p>
                {mutationPreview.secretFindings.length > 0 && (
                  <ul>
                    {mutationPreview.secretFindings.map((finding) => (
                      <li
                        key={`${finding.location}-${finding.path}-${finding.kind}`}
                      >
                        {finding.path ?? "Commit message"} ·{" "}
                        {secretLabels[finding.kind]}
                      </li>
                    ))}
                  </ul>
                )}
              </>
            )}
            <div className="git-confirmation__actions">
              <button type="button" disabled={busy} onClick={onCancelMutation}>
                {mutationPreview.state === "ready" ? "Cancel" : "Close"}
              </button>
              {mutationPreview.state === "ready" && (
                <button
                  className={
                    mutationPreview.destructive ? "git-danger-button" : ""
                  }
                  type="button"
                  disabled={busy}
                  onClick={() =>
                    void onConfirmMutation(mutationPreview.confirmationId!)
                  }
                >
                  {busy ? "Applying…" : `Confirm ${mutationPreview.operation}`}
                </button>
              )}
            </div>
          </section>
        </div>
      )}
    </section>
  );
}
