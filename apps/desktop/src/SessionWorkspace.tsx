import { useMemo, useState, type KeyboardEvent } from "react";

import { ConversationAttachmentTray } from "./ConversationAttachmentTray";
import { ModelSelectionPanel } from "./ModelSelectionPanel";
import type {
  ConversationAttachmentDropRequest,
  ConversationAttachmentSnapshot,
} from "./lib/attachment";
import type { ConversationSnapshot } from "./lib/conversation";
import type { CodexRuntimeSnapshot } from "./lib/codex";
import type {
  ModelSelectionSnapshot,
  ModelSelectionUpdateRequest,
} from "./lib/modelSelection";
import type { ProjectWorkspaceSnapshot } from "./lib/project";
import {
  conversationContinueRequestSchema,
  sessionListRequestSchema,
  type ConversationContinueRequest,
  type SessionLifecycleSnapshot,
  type SessionListRequest,
} from "./lib/session";

type SessionAvailability = "checking" | "native" | "preview";
type Session = SessionLifecycleSnapshot["sessions"][number];

interface SessionWorkspaceProps {
  availability: SessionAvailability;
  snapshot: SessionLifecycleSnapshot;
  runtime: CodexRuntimeSnapshot;
  projects: ProjectWorkspaceSnapshot["projects"];
  activeConversationId: string | null;
  attachments: ConversationAttachmentSnapshot;
  busy: boolean;
  attachmentBusy: boolean;
  actionError: boolean;
  attachmentActionError: boolean;
  searchTerm: string | null;
  onSearch: (request: SessionListRequest) => Promise<void>;
  onRefresh: () => Promise<void>;
  onSelect: (session: Session) => void;
  onResume: (
    request: ConversationContinueRequest,
  ) => Promise<ConversationSnapshot>;
  onFork: (
    request: ConversationContinueRequest,
  ) => Promise<ConversationSnapshot>;
  onArchive: (conversationId: string) => Promise<void>;
  onRestore: (conversationId: string) => Promise<void>;
  onUpdateModelSelection: (
    request: ModelSelectionUpdateRequest,
  ) => Promise<ModelSelectionSnapshot>;
  onAttachmentPick: (projectId: string) => Promise<void>;
  onAttachmentDrop: (
    request: ConversationAttachmentDropRequest,
  ) => Promise<void>;
  onAttachmentCancel: (
    projectId: string,
    attachmentId: string,
  ) => Promise<void>;
}

const stateLabels: Record<Session["state"], string> = {
  running: "Running",
  completed: "Completed",
  interrupted: "Interrupted",
  blocked: "Blocked",
  failed: "Failed",
  archived: "Archived",
  missing: "Missing in Codex",
};

function titleFor(session: Session): string {
  return session.title ?? "Untitled session";
}

function formatUpdated(timestamp: number): string {
  const date = new Date(timestamp);
  if (Number.isNaN(date.valueOf())) return "Unknown date";
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

export function SessionWorkspace({
  availability,
  snapshot,
  runtime,
  projects,
  activeConversationId,
  attachments,
  busy,
  attachmentBusy,
  actionError,
  attachmentActionError,
  searchTerm,
  onSearch,
  onRefresh,
  onSelect,
  onResume,
  onFork,
  onArchive,
  onRestore,
  onUpdateModelSelection,
  onAttachmentPick,
  onAttachmentDrop,
  onAttachmentCancel,
}: SessionWorkspaceProps) {
  const [query, setQuery] = useState(searchTerm ?? "");
  const [openIds, setOpenIds] = useState<string[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [prompt, setPrompt] = useState("");

  const sessionsById = useMemo(
    () =>
      new Map(
        snapshot.sessions.map((session) => [session.conversationId, session]),
      ),
    [snapshot.sessions],
  );
  const projectNames = useMemo(
    () => new Map(projects.map((project) => [project.id, project.displayName])),
    [projects],
  );
  const groupedSessions = useMemo(() => {
    const groups = new Map<string, Session[]>();
    for (const session of snapshot.sessions) {
      const group = groups.get(session.projectId) ?? [];
      group.push(session);
      groups.set(session.projectId, group);
    }
    return [...groups.entries()];
  }, [snapshot.sessions]);

  const visibleOpenIds = openIds.filter((id) => sessionsById.has(id));
  const effectiveSelectedId =
    selectedId && visibleOpenIds.includes(selectedId)
      ? selectedId
      : (visibleOpenIds[0] ?? null);
  const selectedSession = effectiveSelectedId
    ? sessionsById.get(effectiveSelectedId)
    : undefined;
  const attachmentIds =
    attachments.projectId === selectedSession?.projectId &&
    attachments.state === "ready"
      ? attachments.attachments.map((attachment) => attachment.attachmentId)
      : [];
  const continueRequest = selectedSession
    ? {
        conversationId: selectedSession.conversationId,
        prompt,
        attachmentIds,
      }
    : null;
  const requestValid =
    continueRequest !== null &&
    conversationContinueRequestSchema.safeParse(continueRequest).success;
  const canContinue =
    selectedSession !== undefined &&
    !["running", "archived", "missing"].includes(selectedSession.state) &&
    requestValid &&
    availability === "native" &&
    !busy;

  function openSession(session: Session) {
    setOpenIds((current) => {
      if (current.includes(session.conversationId)) return current;
      return [...current.slice(-7), session.conversationId];
    });
    setSelectedId(session.conversationId);
    setPrompt("");
    onSelect(session);
  }

  function closeSession(conversationId: string) {
    setOpenIds((current) => {
      const index = current.indexOf(conversationId);
      const next = current.filter((id) => id !== conversationId);
      if (effectiveSelectedId === conversationId) {
        setSelectedId(next[Math.min(index, next.length - 1)] ?? null);
      }
      return next;
    });
  }

  function moveTabFocus(
    event: KeyboardEvent<HTMLButtonElement>,
    index: number,
  ) {
    if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    const nextIndex =
      event.key === "Home"
        ? 0
        : event.key === "End"
          ? visibleOpenIds.length - 1
          : (index +
              (event.key === "ArrowRight" ? 1 : -1) +
              visibleOpenIds.length) %
            visibleOpenIds.length;
    const nextId = visibleOpenIds[nextIndex];
    if (!nextId) return;
    setSelectedId(nextId);
    const nextSession = sessionsById.get(nextId);
    if (nextSession) onSelect(nextSession);
    document.getElementById(`session-tab-${nextId}`)?.focus();
  }

  async function continueSession(mode: "resume" | "fork") {
    if (!canContinue || !continueRequest) return;
    try {
      const result = await (mode === "resume"
        ? onResume(continueRequest)
        : onFork(continueRequest));
      if (result.state === "running") setPrompt("");
    } catch {
      // The bounded action message is owned by App state.
    }
  }

  async function submitSearch() {
    const candidate = query.trim();
    const request = { projectId: null, searchTerm: candidate || null };
    if (!sessionListRequestSchema.safeParse(request).success) return;
    try {
      await onSearch(request);
    } catch {
      // The bounded action message is owned by App state.
    }
  }

  async function mutate(action: () => Promise<void>) {
    try {
      await action();
    } catch {
      // The bounded action message is owned by App state.
    }
  }

  return (
    <section
      className="session-workspace"
      id="sessions"
      aria-labelledby="sessions-title"
    >
      <div className="session-workspace__intro">
        <div>
          <p className="eyebrow">Session history</p>
          <h2 id="sessions-title">
            Return to work without copying its history.
          </h2>
        </div>
        <p>
          Codex remains authoritative. QuireForge shows bounded titles and
          app-owned references only; transcripts, native IDs, and paths stay
          outside React.
        </p>
      </div>

      <form
        className="session-search"
        role="search"
        onSubmit={(event) => {
          event.preventDefault();
          void submitSearch();
        }}
      >
        <label htmlFor="session-search">Search session titles</label>
        <div>
          <input
            id="session-search"
            type="search"
            maxLength={256}
            value={query}
            disabled={availability !== "native" || busy}
            placeholder="Search Codex-owned titles…"
            onChange={(event) => setQuery(event.target.value)}
          />
          <button type="submit" disabled={availability !== "native" || busy}>
            Search
          </button>
          {searchTerm && (
            <button
              type="button"
              disabled={busy}
              onClick={() => {
                setQuery("");
                void mutate(() =>
                  onSearch({ projectId: null, searchTerm: null }),
                );
              }}
            >
              Clear
            </button>
          )}
          <button
            type="button"
            disabled={availability !== "native" || busy}
            onClick={() => void mutate(onRefresh)}
          >
            Refresh
          </button>
        </div>
      </form>

      <div className="session-status" role="status" aria-live="polite">
        {availability === "checking" &&
          "Checking app-owned session references…"}
        {availability === "preview" &&
          "Browser preview cannot inspect or simulate native session history."}
        {availability === "native" &&
          snapshot.state === "unavailable" &&
          `Session history unavailable${snapshot.diagnosticCode ? `: ${snapshot.diagnosticCode}` : "."}`}
        {availability === "native" &&
          snapshot.state === "empty" &&
          (searchTerm
            ? `No session titles match “${searchTerm}”.`
            : "No app-owned sessions yet.")}
        {availability === "native" &&
          snapshot.state === "ready" &&
          `${snapshot.sessions.length} app-owned session${snapshot.sessions.length === 1 ? "" : "s"}${searchTerm ? ` matching “${searchTerm}”` : ""}.`}
        {actionError && " The last session action could not be completed."}
      </div>

      {snapshot.state === "ready" && (
        <div className="session-layout">
          <div
            className="session-history"
            aria-label="App-owned session history"
          >
            {groupedSessions.map(([projectId, sessions]) => (
              <section
                className="session-group"
                aria-labelledby={`session-group-${projectId}`}
                key={projectId}
              >
                <div className="session-group__heading">
                  <h3 id={`session-group-${projectId}`}>
                    {projectNames.get(projectId) ?? "Unavailable project"}
                  </h3>
                  <span>{sessions.length}</span>
                </div>
                <ul>
                  {sessions.map((session) => {
                    const parent = session.parentConversationId
                      ? sessionsById.get(session.parentConversationId)
                      : undefined;
                    return (
                      <li
                        key={session.conversationId}
                        data-fork={session.parentConversationId !== null}
                      >
                        <button
                          type="button"
                          onClick={() => openSession(session)}
                        >
                          <span className="session-row__title">
                            {titleFor(session)}
                          </span>
                          <span className="session-row__meta">
                            {parent ? `Fork of ${titleFor(parent)} · ` : ""}
                            {formatUpdated(session.updatedAtMs)}
                          </span>
                          <span
                            className={`session-state session-state--${session.state}`}
                          >
                            {stateLabels[session.state]}
                          </span>
                        </button>
                      </li>
                    );
                  })}
                </ul>
              </section>
            ))}
          </div>

          <div className="session-tabs-panel">
            {visibleOpenIds.length > 0 ? (
              <>
                <div className="session-tabs-bar">
                  <div
                    className="session-tabs"
                    role="tablist"
                    aria-label="Open sessions"
                  >
                    {visibleOpenIds.map((id, index) => {
                      const session = sessionsById.get(id);
                      if (!session) return null;
                      const selected = id === effectiveSelectedId;
                      return (
                        <button
                          key={id}
                          id={`session-tab-${id}`}
                          type="button"
                          role="tab"
                          aria-selected={selected}
                          aria-controls={`session-panel-${id}`}
                          tabIndex={selected ? 0 : -1}
                          onClick={() => openSession(session)}
                          onKeyDown={(event) => moveTabFocus(event, index)}
                        >
                          {titleFor(session)}
                          {activeConversationId === id ? " · current" : ""}
                        </button>
                      );
                    })}
                  </div>
                  {selectedSession && (
                    <button
                      className="session-tab-close"
                      type="button"
                      aria-label={`Close ${titleFor(selectedSession)}`}
                      onClick={() =>
                        closeSession(selectedSession.conversationId)
                      }
                    >
                      ×
                    </button>
                  )}
                </div>

                {selectedSession && (
                  <div
                    className="session-detail"
                    id={`session-panel-${selectedSession.conversationId}`}
                    role="tabpanel"
                    aria-labelledby={`session-tab-${selectedSession.conversationId}`}
                  >
                    <div className="session-detail__heading">
                      <div>
                        <span>{stateLabels[selectedSession.state]}</span>
                        <h3>{titleFor(selectedSession)}</h3>
                      </div>
                      <span>
                        {selectedSession.modelId} ·{" "}
                        {selectedSession.reasoningEffort}
                      </span>
                    </div>
                    <dl>
                      <div>
                        <dt>Project</dt>
                        <dd>
                          {projectNames.get(selectedSession.projectId) ??
                            "Unavailable project"}
                        </dd>
                      </div>
                      <div>
                        <dt>Updated</dt>
                        <dd>{formatUpdated(selectedSession.updatedAtMs)}</dd>
                      </div>
                      <div>
                        <dt>Filesystem</dt>
                        <dd>{selectedSession.sandboxMode}</dd>
                      </div>
                      <div>
                        <dt>Approvals</dt>
                        <dd>{selectedSession.approvalPolicy}</dd>
                      </div>
                    </dl>

                    <ModelSelectionPanel
                      key={[
                        selectedSession.conversationId,
                        selectedSession.modelSelection.availability,
                        selectedSession.modelSelection.effective.modelId,
                        selectedSession.modelSelection.effective
                          .reasoningEffort,
                        selectedSession.modelSelection.pending?.requestedAtMs ??
                          "none",
                        selectedSession.modelSelection.policy.ownership,
                        selectedSession.modelSelection.policy.userLocked,
                        selectedSession.modelSelection.policy.allowedModelIds.join(
                          ",",
                        ),
                        selectedSession.modelSelection.policy
                          .reasoningCeiling ?? "none",
                      ].join(":")}
                      conversationId={selectedSession.conversationId}
                      selection={selectedSession.modelSelection}
                      models={runtime.models}
                      disabled={
                        busy ||
                        availability !== "native" ||
                        selectedSession.state === "running" ||
                        runtime.availability !== "ready"
                      }
                      onUpdate={onUpdateModelSelection}
                    />

                    {!["archived", "missing", "running"].includes(
                      selectedSession.state,
                    ) && (
                      <label className="session-prompt">
                        <span>Next task</span>
                        <textarea
                          maxLength={64 * 1024}
                          value={prompt}
                          disabled={busy}
                          placeholder="Describe what Codex should do next…"
                          onChange={(event) => setPrompt(event.target.value)}
                        />
                      </label>
                    )}

                    {!["archived", "missing", "running"].includes(
                      selectedSession.state,
                    ) && (
                      <ConversationAttachmentTray
                        availability={availability}
                        projectId={selectedSession.projectId}
                        snapshot={attachments}
                        busy={attachmentBusy}
                        disabled={busy}
                        actionError={attachmentActionError}
                        onPick={onAttachmentPick}
                        onDrop={onAttachmentDrop}
                        onCancel={onAttachmentCancel}
                      />
                    )}

                    <div className="session-detail__actions">
                      {!["archived", "missing", "running"].includes(
                        selectedSession.state,
                      ) && (
                        <>
                          <button
                            type="button"
                            disabled={!canContinue}
                            onClick={() => void continueSession("resume")}
                          >
                            Resume
                          </button>
                          <button
                            type="button"
                            disabled={!canContinue}
                            onClick={() => void continueSession("fork")}
                          >
                            Fork
                          </button>
                          <button
                            type="button"
                            disabled={busy}
                            onClick={() =>
                              void mutate(() =>
                                onArchive(selectedSession.conversationId),
                              )
                            }
                          >
                            Archive
                          </button>
                        </>
                      )}
                      {selectedSession.state === "archived" && (
                        <button
                          type="button"
                          disabled={busy}
                          onClick={() =>
                            void mutate(() =>
                              onRestore(selectedSession.conversationId),
                            )
                          }
                        >
                          Restore
                        </button>
                      )}
                      {selectedSession.state === "missing" && (
                        <p>
                          This app-owned reference is no longer available from
                          Codex. No substitute session will be opened.
                        </p>
                      )}
                      {selectedSession.state === "running" && (
                        <p>
                          This session is active in the current task stream
                          below.
                        </p>
                      )}
                    </div>
                  </div>
                )}
              </>
            ) : (
              <div className="session-tabs-empty">
                Select a session to open its app-owned lifecycle controls.
              </div>
            )}
          </div>
        </div>
      )}
    </section>
  );
}
