import { useState } from "react";

import type {
  DirectoryAccessibilityState,
  ProjectPreflightSnapshot,
  ProjectWorkspaceSnapshot,
} from "./lib/project";

type ProjectAvailability = "checking" | "native" | "preview";
type MetadataAction = "archive" | "detach";

interface ProjectWorkspaceProps {
  availability: ProjectAvailability;
  snapshot: ProjectWorkspaceSnapshot;
  busy: boolean;
  actionError: boolean;
  preflights: Record<string, ProjectPreflightSnapshot>;
  onPick: () => Promise<void>;
  onPickRelink: (projectId: string) => Promise<void>;
  onConfirm: () => Promise<void>;
  onCancel: () => Promise<void>;
  onDetach: (projectId: string) => Promise<void>;
  onArchive: (projectId: string) => Promise<void>;
  onPreflight: (projectId: string) => Promise<void>;
}

const stateLabels: Record<DirectoryAccessibilityState, string> = {
  "connected-accessible": "Ready for local work",
  "connected-read-only": "Read-only",
  "missing-or-moved": "Missing or moved",
  "permission-denied": "Permission denied",
  "removable-disconnected": "Removable drive disconnected",
  "network-unavailable": "Network location unavailable",
  "git-invalid": "Git metadata invalid",
  "sandbox-restricted": "Sandbox restricted",
  "identity-changed": "Directory identity changed",
  "verification-unknown": "Verification unavailable",
};

const diagnosticMessages: Record<
  NonNullable<ProjectWorkspaceSnapshot["diagnosticCode"]>,
  string
> = {
  "metadata-unavailable": "QuireForge project metadata is unavailable.",
  "picker-unavailable":
    "The native directory picker did not return a usable folder.",
  "directory-unavailable": "That directory could not be verified safely.",
  "duplicate-directory": "That resolved directory is already attached.",
  "project-not-found":
    "The selected project no longer exists in QuireForge metadata.",
  "project-busy":
    "That project has an active Codex task. Stop it before changing its attachment.",
  "attachment-not-pending":
    "The attachment preview expired. Select the folder again.",
  "identity-changed":
    "The directory changed after preview. Review it again before attaching.",
};

export function ProjectWorkspace({
  availability,
  snapshot,
  busy,
  actionError,
  preflights,
  onPick,
  onPickRelink,
  onConfirm,
  onCancel,
  onDetach,
  onArchive,
  onPreflight,
}: ProjectWorkspaceProps) {
  const [metadataAction, setMetadataAction] = useState<{
    action: MetadataAction;
    projectId: string;
  } | null>(null);
  const nativeReady =
    availability === "native" && snapshot.state !== "unavailable";

  async function confirmMetadataAction() {
    if (!metadataAction) return;
    if (metadataAction.action === "detach") {
      await onDetach(metadataAction.projectId);
    } else {
      await onArchive(metadataAction.projectId);
    }
    setMetadataAction(null);
  }

  return (
    <section className="project-workspace" aria-labelledby="projects-title">
      <div className="project-workspace__heading">
        <div>
          <p className="eyebrow">Local projects</p>
          <h2 id="projects-title">Work where your files already live.</h2>
          <p>
            QuireForge records directory identity and metadata only. It does not
            copy, move, upload, or delete the attached source directory.
          </p>
        </div>
        <button
          className="auth-button auth-button--primary"
          type="button"
          disabled={!nativeReady || busy}
          onClick={() => void onPick()}
        >
          Attach local project
        </button>
      </div>

      <div className="project-workspace__status" aria-live="polite">
        {availability === "checking" && (
          <p>Reading QuireForge project metadata.</p>
        )}
        {availability === "preview" && (
          <p>
            Browser preview cannot open a native folder picker or simulate an
            attached project.
          </p>
        )}
        {snapshot.diagnosticCode && (
          <p className="project-message project-message--warning" role="alert">
            {diagnosticMessages[snapshot.diagnosticCode]}
          </p>
        )}
        {actionError && (
          <p className="project-message project-message--warning" role="alert">
            The native project action did not complete. No source files were
            changed.
          </p>
        )}
      </div>

      {snapshot.pendingAttachment && (
        <div
          className="attachment-review"
          aria-labelledby="attachment-review-title"
        >
          <div className="attachment-review__heading">
            <div>
              <span className="project-kicker">
                {snapshot.pendingAttachment.operation === "attach"
                  ? "Attachment preview"
                  : "Relink preview"}
              </span>
              <h3 id="attachment-review-title">
                Confirm {snapshot.pendingAttachment.displayName}
              </h3>
            </div>
            <span
              className={`directory-state directory-state--${snapshot.pendingAttachment.state}`}
            >
              {stateLabels[snapshot.pendingAttachment.state]}
            </span>
          </div>
          <dl className="attachment-paths">
            <div>
              <dt>Selected path</dt>
              <dd>{snapshot.pendingAttachment.selectedDisplayPath}</dd>
            </div>
            <div>
              <dt>Resolved execution path</dt>
              <dd>{snapshot.pendingAttachment.resolvedDisplayPath}</dd>
            </div>
          </dl>
          <div className="project-flags" aria-label="Detected project metadata">
            <span>
              {snapshot.pendingAttachment.git.isRepository
                ? snapshot.pendingAttachment.git.isLinkedWorktree
                  ? "Linked Git worktree"
                  : "Git repository"
                : "No Git repository detected"}
            </span>
            <span>
              {snapshot.pendingAttachment.hasAgentsGuidance
                ? "AGENTS.md detected"
                : "No root AGENTS.md"}
            </span>
            <span>
              {snapshot.pendingAttachment.hasCodexConfig
                ? ".codex detected"
                : "No root .codex directory"}
            </span>
          </div>
          {snapshot.pendingAttachment.state === "connected-read-only" && (
            <p className="project-message project-message--warning">
              You may retain this project, but QuireForge will refuse it as a
              writable task directory until access changes.
            </p>
          )}
          <div className="project-actions">
            <button
              className="auth-button auth-button--primary"
              type="button"
              disabled={busy}
              onClick={() => void onConfirm()}
            >
              {snapshot.pendingAttachment.operation === "attach"
                ? "Confirm attachment"
                : "Confirm relink"}
            </button>
            <button
              className="auth-button"
              type="button"
              disabled={busy}
              onClick={() => void onCancel()}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {snapshot.projects.length === 0 && !snapshot.pendingAttachment && (
        <div className="project-empty">
          <span aria-hidden="true">↳</span>
          <div>
            <h3>No project attached</h3>
            <p>
              Choose one original local directory. QuireForge will show the
              selected and resolved paths before saving metadata.
            </p>
          </div>
        </div>
      )}

      {snapshot.projects.length > 0 && (
        <div className="project-list">
          {snapshot.projects.map((project) => {
            const directory = project.directory;
            const preflight = preflights[project.id];
            return (
              <article
                className={`project-card ${project.archived ? "project-card--archived" : ""}`}
                key={project.id}
              >
                <div className="project-card__heading">
                  <div>
                    <span className="project-kicker">
                      {project.archived
                        ? "Archived project"
                        : "Attached project"}
                    </span>
                    <h3>{project.displayName}</h3>
                  </div>
                  {directory && (
                    <span
                      className={`directory-state directory-state--${directory.state}`}
                    >
                      {stateLabels[directory.state]}
                    </span>
                  )}
                </div>

                {directory ? (
                  <>
                    <dl className="attachment-paths">
                      <div>
                        <dt>Selected path</dt>
                        <dd>{directory.displayPath}</dd>
                      </div>
                      {directory.resolvedDisplayPath &&
                        directory.resolvedDisplayPath !==
                          directory.displayPath && (
                          <div>
                            <dt>Resolved execution path</dt>
                            <dd>{directory.resolvedDisplayPath}</dd>
                          </div>
                        )}
                    </dl>
                    <div className="project-flags">
                      {directory.git.isRepository && (
                        <span>
                          {directory.git.isLinkedWorktree
                            ? "Linked Git worktree"
                            : "Git repository"}
                        </span>
                      )}
                      {directory.hasAgentsGuidance && <span>AGENTS.md</span>}
                      {directory.hasCodexConfig && <span>.codex</span>}
                      <span>{directory.expectedAccess}</span>
                    </div>
                  </>
                ) : (
                  <p className="project-message">
                    This project has no active directory. Relink it explicitly
                    before starting local work.
                  </p>
                )}

                {preflight && (
                  <p
                    className={`preflight-result ${preflight.cwdReady ? "preflight-result--ready" : ""}`}
                    role="status"
                  >
                    {preflight.cwdReady
                      ? `Working directory verified: ${preflight.displayPath}`
                      : `Working directory blocked: ${stateLabels[preflight.state]}`}
                  </p>
                )}

                <div className="project-actions">
                  {directory && (
                    <button
                      className="auth-button"
                      type="button"
                      disabled={!nativeReady || busy}
                      onClick={() => void onPreflight(project.id)}
                    >
                      Verify directory
                    </button>
                  )}
                  <button
                    className="auth-button"
                    type="button"
                    disabled={!nativeReady || busy}
                    onClick={() => void onPickRelink(project.id)}
                  >
                    Relink
                  </button>
                  {directory && (
                    <button
                      className="auth-button"
                      type="button"
                      disabled={!nativeReady || busy}
                      onClick={() =>
                        setMetadataAction({
                          action: "detach",
                          projectId: project.id,
                        })
                      }
                    >
                      Detach
                    </button>
                  )}
                  {!project.archived && (
                    <button
                      className="auth-button auth-button--danger"
                      type="button"
                      disabled={!nativeReady || busy}
                      onClick={() =>
                        setMetadataAction({
                          action: "archive",
                          projectId: project.id,
                        })
                      }
                    >
                      Archive
                    </button>
                  )}
                </div>

                {metadataAction?.projectId === project.id && (
                  <div
                    className="project-confirmation"
                    role="group"
                    aria-label="Confirm metadata action"
                  >
                    <p>
                      {metadataAction.action === "detach"
                        ? "Detach this directory from QuireForge? Source content will remain in place."
                        : "Archive this QuireForge project? Its source directory will remain untouched."}
                    </p>
                    <div className="project-actions">
                      <button
                        className="auth-button auth-button--danger"
                        type="button"
                        disabled={busy}
                        onClick={() => void confirmMetadataAction()}
                      >
                        Confirm {metadataAction.action}
                      </button>
                      <button
                        className="auth-button"
                        type="button"
                        disabled={busy}
                        onClick={() => setMetadataAction(null)}
                      >
                        Keep project
                      </button>
                    </div>
                  </div>
                )}
              </article>
            );
          })}
        </div>
      )}
    </section>
  );
}
