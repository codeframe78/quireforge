import { useState, type FormEvent } from "react";

import {
  worktreeBranchSchema,
  type WorktreePreviewSnapshot,
  type WorktreeResultSnapshot,
  type WorktreeWorkspaceSnapshot,
} from "./lib/worktree";

type WorktreeAvailability = "checking" | "native" | "preview";

interface WorktreeWorkspaceProps {
  availability: WorktreeAvailability;
  projectName: string | null;
  snapshot: WorktreeWorkspaceSnapshot;
  preview: WorktreePreviewSnapshot | null;
  result: WorktreeResultSnapshot | null;
  busy: boolean;
  actionError: boolean;
  onRefresh: () => Promise<void>;
  onCreate: (branchName: string) => Promise<void>;
  onPickAttach: () => Promise<void>;
  onConfirm: (confirmationId: string) => Promise<void>;
  onCancel: (confirmationId: string) => Promise<void>;
  onSelectProject: (projectId: string) => void;
}

const diagnosticMessages: Record<
  NonNullable<WorktreeWorkspaceSnapshot["diagnosticCode"]>,
  string
> = {
  "metadata-unavailable": "QuireForge worktree metadata is unavailable.",
  "project-not-found": "Select an attached project before reviewing worktrees.",
  "project-busy":
    "A related project is busy. Finish its active operation first.",
  "not-repository": "The selected project is not a Git repository.",
  "directory-unavailable":
    "A required worktree directory is unavailable or read-only.",
  "identity-changed": "Repository identity changed. Refresh before continuing.",
  "picker-unavailable": "The native picker did not return a usable worktree.",
  "invalid-branch": "Use a valid new branch name of at most 96 characters.",
  "branch-exists":
    "That local branch already exists. Choose a new branch name.",
  "duplicate-directory": "That worktree is already attached as a project.",
  "not-linked-worktree":
    "Choose a linked Git worktree, not a primary checkout.",
  "different-repository": "That worktree belongs to a different repository.",
  "git-unavailable": "Git is not available through the native service.",
  "git-failed": "Git could not safely inspect or create the worktree.",
  "output-too-large": "The worktree inventory exceeded the safe display limit.",
  "confirmation-expired":
    "The confirmation expired. Preview the operation again.",
  "stale-preview": "The repository changed after preview. Review it again.",
  "worktree-remains":
    "The worktree remains on disk for explicit recovery; nothing was deleted.",
};

const stateLabels: Record<
  WorktreeWorkspaceSnapshot["worktrees"][number]["state"],
  string
> = {
  ready: "Ready",
  missing: "Missing",
  archived: "Archived project",
  locked: "Locked",
  prunable: "Prunable",
  detached: "Detached HEAD",
};

export function WorktreeWorkspace({
  availability,
  projectName,
  snapshot,
  preview,
  result,
  busy,
  actionError,
  onRefresh,
  onCreate,
  onPickAttach,
  onConfirm,
  onCancel,
  onSelectProject,
}: WorktreeWorkspaceProps) {
  const [branchName, setBranchName] = useState("");
  const branchValid = worktreeBranchSchema.safeParse(branchName).success;
  const nativeReady =
    availability === "native" && snapshot.state !== "unavailable";

  function submitCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (branchValid) void onCreate(branchName);
  }

  return (
    <section
      className="worktree-workspace"
      id="worktrees"
      aria-labelledby="worktrees-title"
    >
      <div className="worktree-workspace__heading">
        <div>
          <p className="eyebrow">Isolated workspaces · Milestone 11A</p>
          <h2 id="worktrees-title">Give each line of work its own checkout.</h2>
          <p>
            Create an app-managed Git worktree or attach one that already
            exists. Worktrees remain ordinary local projects; removal and
            cleanup are intentionally unavailable in this milestone.
          </p>
        </div>
        <button
          className="auth-button"
          type="button"
          disabled={!nativeReady || busy || projectName === null}
          onClick={() => void onRefresh()}
        >
          Refresh inventory
        </button>
      </div>

      <div className="worktree-status" aria-live="polite">
        {availability === "checking" && (
          <p>Reading the native worktree inventory.</p>
        )}
        {availability === "preview" && (
          <p>Browser preview cannot inspect or create local Git worktrees.</p>
        )}
        {snapshot.diagnosticCode && (
          <p className="project-message project-message--warning" role="alert">
            {diagnosticMessages[snapshot.diagnosticCode]}
          </p>
        )}
        {preview?.diagnosticCode && (
          <p className="project-message project-message--warning" role="alert">
            {diagnosticMessages[preview.diagnosticCode]}
          </p>
        )}
        {result?.diagnosticCode && (
          <p className="project-message project-message--warning" role="alert">
            {diagnosticMessages[result.diagnosticCode]}
            {result.recoverableDisplayPath && (
              <span className="worktree-recovery-path">
                {result.recoverableDisplayPath}
              </span>
            )}
          </p>
        )}
        {actionError && (
          <p className="project-message project-message--warning" role="alert">
            The native worktree action did not complete. QuireForge did not
            delete or clean any directory.
          </p>
        )}
      </div>

      <div className="worktree-controls">
        <form className="worktree-create" onSubmit={submitCreate}>
          <label htmlFor="worktree-branch">New branch name</label>
          <div>
            <input
              id="worktree-branch"
              name="branchName"
              type="text"
              value={branchName}
              maxLength={96}
              autoComplete="off"
              spellCheck={false}
              aria-describedby="worktree-branch-help"
              disabled={!nativeReady || busy || projectName === null}
              onChange={(event) => setBranchName(event.target.value)}
              placeholder="feature/focused-change"
            />
            <button
              className="auth-button auth-button--primary"
              type="submit"
              disabled={
                !nativeReady || busy || !branchValid || projectName === null
              }
            >
              Preview managed worktree
            </button>
          </div>
          <small id="worktree-branch-help">
            Creates a new local branch from the source checkout&apos;s current
            HEAD.
          </small>
        </form>
        <div className="worktree-attach-control">
          <span>Existing worktree</span>
          <button
            className="auth-button"
            type="button"
            disabled={!nativeReady || busy || projectName === null}
            onClick={() => void onPickAttach()}
          >
            Choose with native picker
          </button>
        </div>
      </div>

      {preview?.state === "ready" && preview.confirmationId && (
        <div
          className="attachment-review"
          aria-labelledby="worktree-preview-title"
        >
          <div className="attachment-review__heading">
            <div>
              <span className="project-kicker">Non-destructive preview</span>
              <h3 id="worktree-preview-title">
                {preview.operation === "create"
                  ? `Create ${preview.branchName}`
                  : `Attach ${preview.branchName ?? "detached worktree"}`}
              </h3>
            </div>
            <span className="directory-state directory-state--connected-accessible">
              Confirmation required
            </span>
          </div>
          <dl className="attachment-paths">
            <div>
              <dt>Native-reviewed path</dt>
              <dd>{preview.displayPath}</dd>
            </div>
            <div>
              <dt>Ownership</dt>
              <dd>{preview.ownership}</dd>
            </div>
          </dl>
          <p className="project-message">
            This operation does not remove, prune, or clean any existing
            worktree. Repository identity and branch availability will be
            checked again on confirmation.
          </p>
          <div className="project-actions">
            <button
              className="auth-button auth-button--primary"
              type="button"
              disabled={busy}
              onClick={() => void onConfirm(preview.confirmationId!)}
            >
              Confirm {preview.operation}
            </button>
            <button
              className="auth-button"
              type="button"
              disabled={busy}
              onClick={() => void onCancel(preview.confirmationId!)}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {snapshot.worktrees.length === 0 && (
        <div className="project-empty">
          <span aria-hidden="true">⑂</span>
          <div>
            <h3>No native worktree inventory</h3>
            <p>
              Attach a Git repository to inspect its source checkout and linked
              worktrees.
            </p>
          </div>
        </div>
      )}

      {snapshot.worktrees.length > 0 && (
        <div className="worktree-list" aria-label="Git worktrees">
          {snapshot.worktrees.map((worktree) => (
            <article
              className={`worktree-card ${worktree.current ? "worktree-card--current" : ""}`}
              key={`${worktree.displayPath}-${worktree.ownership}`}
            >
              <div>
                <span className="project-kicker">
                  {worktree.ownership} checkout
                </span>
                <h3>{worktree.displayName}</h3>
                <code>{worktree.displayPath}</code>
              </div>
              <div className="worktree-card__meta">
                <span>{worktree.branchName ?? "Detached HEAD"}</span>
                <span
                  className={`worktree-state worktree-state--${worktree.state}`}
                >
                  {stateLabels[worktree.state]}
                </span>
                {worktree.projectId &&
                  !worktree.current &&
                  worktree.state !== "archived" && (
                    <button
                      className="auth-button"
                      type="button"
                      disabled={busy}
                      onClick={() => onSelectProject(worktree.projectId!)}
                    >
                      Select project
                    </button>
                  )}
                {worktree.current && <strong>Current project</strong>}
              </div>
            </article>
          ))}
        </div>
      )}

      {snapshot.truncated && (
        <p className="project-message">
          Only the first 256 worktrees are shown.
        </p>
      )}
    </section>
  );
}
