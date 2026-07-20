import type {
  GitDiffRequest,
  GitDiffSnapshot,
  GitWorkspaceSnapshot,
} from "./lib/git";

type GitAvailability = "checking" | "native" | "preview";

interface GitWorkspaceProps {
  availability: GitAvailability;
  projectName: string | null;
  snapshot: GitWorkspaceSnapshot;
  diff: GitDiffSnapshot | null;
  selectedRequest: GitDiffRequest | null;
  busy: boolean;
  actionError: boolean;
  onRefresh: () => Promise<void>;
  onReview: (request: GitDiffRequest) => Promise<void>;
  onOpen: (projectId: string, path: string) => Promise<void>;
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
};

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
  busy,
  actionError,
  onRefresh,
  onReview,
  onOpen,
}: GitWorkspaceProps) {
  const selectedChange = selectedRequest
    ? snapshot.changes.find((change) => change.path === selectedRequest.path)
    : undefined;
  const openable =
    selectedChange?.reviewable === true &&
    selectedChange.staged !== "deleted" &&
    selectedChange.worktree !== "deleted";

  return (
    <section className="git-workspace" id="changes" aria-labelledby="git-title">
      <div className="git-workspace__heading">
        <div>
          <p className="eyebrow">Source review · read only</p>
          <h2 id="git-title">Review local changes without changing them.</h2>
          <p>
            QuireForge requests normalized status and diff records from fixed
            native Git commands. Staging, reverting, committing, and arbitrary
            arguments are not part of this milestone.
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
        </>
      )}
    </section>
  );
}
