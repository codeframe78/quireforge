import { useMemo, useRef, useState } from "react";

import type { CodexRuntimeSnapshot } from "./lib/codex";
import {
  conversationStartRequestSchema,
  type ConversationApprovalDecisionRequest,
  type ConversationEvent,
  type ConversationSnapshot,
  type ConversationStartRequest,
} from "./lib/conversation";
import {
  buildConversationActivityViews,
  type ConversationActivityView,
} from "./lib/conversationView";
import type { ProjectWorkspaceSnapshot } from "./lib/project";

type ConversationAvailability = "checking" | "native" | "preview";
type Project = ProjectWorkspaceSnapshot["projects"][number];

interface ConversationWorkspaceProps {
  availability: ConversationAvailability;
  snapshot: ConversationSnapshot;
  events: ConversationEvent[];
  runtime: CodexRuntimeSnapshot;
  project: Project | undefined;
  busy: boolean;
  actionError: boolean;
  onStart: (request: ConversationStartRequest) => Promise<ConversationSnapshot>;
  onInterrupt: (conversationId: string) => Promise<ConversationSnapshot>;
  onDecideApproval: (
    request: ConversationApprovalDecisionRequest,
  ) => Promise<ConversationSnapshot>;
}

const sandboxOptions = [
  { value: "read-only", label: "Read only" },
  { value: "workspace-write", label: "Workspace write" },
  { value: "danger-full-access", label: "Unrestricted" },
] as const;

const approvalOptions = [
  { value: "untrusted", label: "Ask for untrusted actions" },
  { value: "on-request", label: "Ask when Codex requests" },
  { value: "never", label: "Never ask" },
] as const;

const stateLabels: Record<ConversationSnapshot["state"], string> = {
  empty: "Ready for a task",
  running: "Codex is working",
  "waiting-for-approval": "Codex is waiting for approval",
  stopping: "Stopping safely",
  completed: "Task completed",
  interrupted: "Task stopped",
  blocked: "Approval required",
  failed: "Task could not continue",
  unavailable: "Conversation runtime unavailable",
};

const diagnosticMessages: Record<
  NonNullable<ConversationSnapshot["diagnosticCode"]>,
  string
> = {
  "conversation-active": "Another task is already active.",
  "conversation-not-found": "This task is no longer available.",
  "invalid-request": "Review the task settings and try again.",
  "project-unavailable": "The attached project is unavailable.",
  "project-identity-changed":
    "The project identity changed. Verify it before continuing.",
  "project-not-writable": "The project is not writable.",
  "project-busy": "This project is already in use by another task.",
  "runtime-unavailable": "The native Codex runtime is unavailable.",
  "model-unavailable": "The selected model is no longer available.",
  "reasoning-unavailable": "The selected reasoning level is unavailable.",
  "metadata-unavailable":
    "QuireForge could not read its conversation metadata.",
  "approval-required": "Codex needs an approval before it can continue.",
  "approval-not-found": "That approval is no longer pending.",
  "approval-decision-unavailable":
    "That decision is not available for this approval.",
  "process-exited": "The Codex process exited before the task finished.",
  "transport-failed": "The connection to the Codex process was interrupted.",
  "protocol-invalid":
    "Codex returned data QuireForge could not safely display.",
  "rpc-rejected": "Codex rejected the requested operation.",
};

const activityLabels: Record<
  Extract<ConversationEvent, { type: "activity" }>["kind"],
  string
> = {
  "user-message": "Task submitted",
  "agent-message": "Response",
  plan: "Plan",
  reasoning: "Reasoning summary",
  "command-execution": "Command",
  "file-change": "File change",
  "tool-call": "Tool",
  "web-search": "Web search",
  image: "Image",
  other: "Activity",
};

const decisionLabels: Record<
  ConversationApprovalDecisionRequest["decision"],
  string
> = {
  approve: "Approve once",
  decline: "Decline",
  cancel: "Cancel task",
};

function ActivityCard({
  activity,
  expanded,
  onToggle,
}: {
  activity: ConversationActivityView;
  expanded: boolean;
  onToggle: () => void;
}) {
  const panelId = `conversation-activity-${activity.activityId}`;
  const label = activity.title || activityLabels[activity.kind];
  return (
    <article className="conversation-activity">
      <button
        className="conversation-activity__toggle"
        type="button"
        aria-expanded={expanded}
        aria-controls={panelId}
        onClick={onToggle}
      >
        <span
          className="conversation-activity__status"
          data-status={activity.status}
        >
          <span aria-hidden="true" />
          {activity.status}
        </span>
        <strong>{label}</strong>
        <span className="conversation-activity__chevron" aria-hidden="true">
          ›
        </span>
      </button>
      {expanded && (
        <div className="conversation-activity__panel" id={panelId}>
          <span className="conversation-activity__kind">
            {activityLabels[activity.kind]}
          </span>
          {activity.detail && (
            <div>
              <strong>Details</strong>
              <pre>{activity.detail}</pre>
            </div>
          )}
          {activity.output && (
            <div>
              <strong>Live output</strong>
              <pre aria-label={`${label} live output`}>{activity.output}</pre>
            </div>
          )}
          {activity.exitCode !== null && (
            <small>Exit code {activity.exitCode}</small>
          )}
          {!activity.detail &&
            !activity.output &&
            activity.exitCode === null && (
              <p>No additional normalized detail is available yet.</p>
            )}
        </div>
      )}
    </article>
  );
}

function EventCard({ event }: { event: ConversationEvent }) {
  if (event.type === "agent-message-delta") {
    return <p className="conversation-event__message">{event.delta}</p>;
  }
  if (event.type === "reasoning-summary-delta") {
    return (
      <details className="conversation-event__reasoning">
        <summary>Reasoning summary</summary>
        <p>{event.delta}</p>
      </details>
    );
  }
  if (event.type === "plan-updated") {
    return (
      <div className="conversation-event__plan">
        <strong>Plan updated</strong>
        {event.explanation && <p>{event.explanation}</p>}
        <ol>
          {event.steps.map((step, index) => (
            <li data-state={step.status} key={`${event.sequence}-${index}`}>
              {step.step}
            </li>
          ))}
        </ol>
      </div>
    );
  }
  if (event.type === "activity" || event.type === "activity-output-delta")
    return null;
  if (event.type === "approval-requested") {
    return (
      <p className="conversation-event__approval">
        Approval requested for {event.kind.split("-").join(" ")}.
      </p>
    );
  }
  if (event.type === "approval-resolved") {
    return (
      <p className="conversation-event__approval">
        Approval {event.resolution.split("-").join(" ")}.
      </p>
    );
  }
  if (event.type === "error") {
    return (
      <p className="conversation-event__error" role="alert">
        {event.code.split("-").join(" ")}
        {event.willRetry ? " — retrying" : ""}
      </p>
    );
  }
  if (event.type === "lifecycle")
    return (
      <p className="conversation-event__lifecycle">
        {event.phase.split("-").join(" ")}
      </p>
    );
  return null;
}

export function ConversationWorkspace({
  availability,
  snapshot,
  events,
  runtime,
  project,
  busy,
  actionError,
  onStart,
  onInterrupt,
  onDecideApproval,
}: ConversationWorkspaceProps) {
  const defaultModel =
    runtime.models.find((model) => model.isDefault) ?? runtime.models[0];
  const [prompt, setPrompt] = useState("");
  const [modelId, setModelId] = useState(defaultModel?.id ?? "");
  const [reasoningEffort, setReasoningEffort] = useState(
    defaultModel?.defaultReasoningEffort ?? "",
  );
  const [sandboxMode, setSandboxMode] =
    useState<ConversationStartRequest["sandboxMode"]>("workspace-write");
  const [approvalPolicy, setApprovalPolicy] =
    useState<ConversationStartRequest["approvalPolicy"]>("on-request");
  const [expandedActivities, setExpandedActivities] = useState<Set<string>>(
    new Set(),
  );
  const [pendingDecision, setPendingDecision] = useState<
    ConversationApprovalDecisionRequest["decision"] | null
  >(null);
  const decisionInFlight = useRef(false);

  const activities = useMemo(
    () => buildConversationActivityViews(events),
    [events],
  );
  const activitiesByFirstSequence = useMemo(
    () =>
      new Map(activities.map((activity) => [activity.firstSequence, activity])),
    [activities],
  );

  const selectedModel =
    runtime.models.find((model) => model.id === modelId) ?? defaultModel;
  const effectiveModelId = selectedModel?.id ?? "";
  const effectiveReasoningEffort =
    selectedModel?.supportedReasoningEfforts.includes(reasoningEffort)
      ? reasoningEffort
      : (selectedModel?.defaultReasoningEffort ?? "");

  const projectReady =
    project !== undefined &&
    !project.archived &&
    project.directory?.state === "connected-accessible";
  const runtimeReady =
    runtime.availability === "ready" &&
    runtime.models.length > 0 &&
    runtime.capabilities.some(
      (capability) =>
        capability.id === "conversation-runtime" &&
        capability.state === "ready",
    );
  const active = ["running", "waiting-for-approval", "stopping"].includes(
    snapshot.state,
  );
  const unsafeCombination =
    sandboxMode === "danger-full-access" && approvalPolicy === "never";
  const request = useMemo(
    () => ({
      projectId: project?.id ?? "",
      prompt,
      modelId: effectiveModelId,
      reasoningEffort: effectiveReasoningEffort,
      sandboxMode,
      approvalPolicy,
    }),
    [
      approvalPolicy,
      effectiveModelId,
      effectiveReasoningEffort,
      project?.id,
      prompt,
      sandboxMode,
    ],
  );
  const requestValid =
    conversationStartRequestSchema.safeParse(request).success;
  const canStart =
    availability === "native" &&
    projectReady &&
    runtimeReady &&
    snapshot.state !== "unavailable" &&
    !active &&
    !busy &&
    requestValid;

  async function beginTask() {
    if (!canStart) return;
    try {
      const result = await onStart(request);
      if (result.state === "running") setPrompt("");
    } catch {
      // The bounded action message is owned by App state.
    }
  }

  async function stopTask() {
    if (
      !snapshot.conversationId ||
      !["running", "waiting-for-approval"].includes(snapshot.state) ||
      busy
    )
      return;
    try {
      await onInterrupt(snapshot.conversationId);
    } catch {
      // The bounded action message is owned by App state.
    }
  }

  async function decideApproval(
    decision: ConversationApprovalDecisionRequest["decision"],
  ) {
    const approval = snapshot.pendingApproval;
    if (
      decisionInFlight.current ||
      busy ||
      !snapshot.conversationId ||
      !approval ||
      !approval.decisions.includes(decision)
    )
      return;

    decisionInFlight.current = true;
    setPendingDecision(decision);
    try {
      await onDecideApproval({
        conversationId: snapshot.conversationId,
        approvalId: approval.approvalId,
        decision,
      });
    } catch {
      // The bounded action message is owned by App state.
    } finally {
      decisionInFlight.current = false;
      setPendingDecision(null);
    }
  }

  function toggleActivity(activityId: string) {
    setExpandedActivities((current) => {
      const next = new Set(current);
      if (next.has(activityId)) next.delete(activityId);
      else next.add(activityId);
      return next;
    });
  }

  return (
    <section
      className="conversation-workspace"
      id="conversation"
      aria-labelledby="conversation-title"
    >
      <div className="conversation-workspace__intro">
        <div>
          <p className="eyebrow">Native conversation</p>
          <h2 id="conversation-title">Start a focused Codex task.</h2>
        </div>
        <p>
          Work stays scoped to the attached directory. QuireForge displays a
          normalized event stream and does not persist transcript content.
        </p>
      </div>

      <div className="conversation-layout">
        <form
          className="conversation-composer"
          onSubmit={(event) => {
            event.preventDefault();
            void beginTask();
          }}
        >
          <label htmlFor="conversation-prompt">Task</label>
          <textarea
            id="conversation-prompt"
            maxLength={64 * 1024}
            placeholder="Describe the change, investigation, or review…"
            value={prompt}
            disabled={active || busy}
            onChange={(event) => setPrompt(event.target.value)}
          />
          <div className="conversation-controls">
            <label>
              <span>Model</span>
              <select
                aria-label="Model"
                value={effectiveModelId}
                disabled={active || busy || !runtimeReady}
                onChange={(event) => {
                  const nextModel = runtime.models.find(
                    (model) => model.id === event.target.value,
                  );
                  setModelId(event.target.value);
                  if (nextModel) {
                    setReasoningEffort(nextModel.defaultReasoningEffort);
                  }
                }}
              >
                {runtime.models.map((model) => (
                  <option value={model.id} key={model.id}>
                    {model.displayName}
                    {model.isDefault ? " — default" : ""}
                  </option>
                ))}
              </select>
            </label>
            <label>
              <span>Reasoning</span>
              <select
                aria-label="Reasoning"
                value={effectiveReasoningEffort}
                disabled={active || busy || !selectedModel}
                onChange={(event) => setReasoningEffort(event.target.value)}
              >
                {selectedModel?.supportedReasoningEfforts.map((effort) => (
                  <option value={effort} key={effort}>
                    {effort}
                  </option>
                ))}
              </select>
            </label>
            <label>
              <span>Filesystem</span>
              <select
                aria-label="Filesystem access"
                value={sandboxMode}
                disabled={active || busy}
                onChange={(event) =>
                  setSandboxMode(
                    event.target
                      .value as ConversationStartRequest["sandboxMode"],
                  )
                }
              >
                {sandboxOptions.map((option) => (
                  <option value={option.value} key={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
            <label>
              <span>Approvals</span>
              <select
                aria-label="Approval policy"
                value={approvalPolicy}
                disabled={active || busy}
                onChange={(event) =>
                  setApprovalPolicy(
                    event.target
                      .value as ConversationStartRequest["approvalPolicy"],
                  )
                }
              >
                {approvalOptions.map((option) => (
                  <option value={option.value} key={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>
          </div>

          <div className="conversation-prerequisite" aria-live="polite">
            {availability === "checking" &&
              "Checking the native conversation runtime…"}
            {availability === "preview" &&
              "Browser preview cannot start or simulate a Codex task."}
            {availability === "native" &&
              !projectReady &&
              "Attach and verify a writable project before starting a task."}
            {availability === "native" &&
              projectReady &&
              !runtimeReady &&
              "A ready Codex conversation capability and model catalog are required."}
            {unsafeCombination &&
              "Unrestricted execution cannot be combined with disabled approvals."}
          </div>

          <div className="conversation-actions">
            {!active ? (
              <button
                className="conversation-start"
                type="submit"
                disabled={!canStart}
              >
                Start task
              </button>
            ) : (
              <button
                className="conversation-stop"
                type="button"
                disabled={snapshot.state === "stopping" || busy}
                onClick={() => void stopTask()}
              >
                {snapshot.state === "stopping" ? "Stopping…" : "Stop task"}
              </button>
            )}
            <span>
              {projectReady ? project.displayName : "No runnable project"}
            </span>
          </div>
        </form>

        <div
          className="conversation-stream"
          aria-labelledby="conversation-stream-title"
        >
          <div className="conversation-stream__header">
            <div>
              <span>Current task</span>
              <strong
                id="conversation-stream-title"
                role="status"
                aria-live="polite"
              >
                {stateLabels[snapshot.state]}
              </strong>
            </div>
            {(snapshot.state === "running" ||
              snapshot.state === "waiting-for-approval" ||
              snapshot.state === "stopping") && (
              <span className="conversation-pulse" aria-hidden="true" />
            )}
          </div>

          <div
            className="conversation-events"
            aria-live="polite"
            aria-relevant="additions"
          >
            {snapshot.pendingApproval && (
              <section
                className="conversation-approval"
                aria-labelledby={`approval-${snapshot.pendingApproval.approvalId}`}
              >
                <p className="eyebrow">Action required</p>
                <h3 id={`approval-${snapshot.pendingApproval.approvalId}`}>
                  {snapshot.pendingApproval.title}
                </h3>
                <span className="conversation-approval__kind">
                  {snapshot.pendingApproval.kind.split("-").join(" ")} approval
                </span>
                {snapshot.pendingApproval.reason && (
                  <p>{snapshot.pendingApproval.reason}</p>
                )}
                {snapshot.pendingApproval.details.length > 0 && (
                  <dl>
                    {snapshot.pendingApproval.details.map((detail) => (
                      <div key={detail.label}>
                        <dt>{detail.label}</dt>
                        <dd>{detail.value}</dd>
                      </div>
                    ))}
                  </dl>
                )}
                <div className="conversation-approval__actions">
                  {snapshot.pendingApproval.decisions.map((decision) => (
                    <button
                      key={decision}
                      type="button"
                      data-decision={decision}
                      disabled={busy || pendingDecision !== null}
                      onClick={() => void decideApproval(decision)}
                    >
                      {pendingDecision === decision
                        ? `${decisionLabels[decision]}…`
                        : decisionLabels[decision]}
                    </button>
                  ))}
                </div>
                <small>
                  Approval applies only to this requested action. Declining
                  keeps broader access unchanged.
                </small>
              </section>
            )}
            {events.length === 0 ? (
              <div className="conversation-empty">
                <span aria-hidden="true">›</span>
                <p>Normalized progress and response text will appear here.</p>
              </div>
            ) : (
              events.map((event) => {
                if (
                  event.type === "activity" ||
                  event.type === "activity-output-delta"
                ) {
                  const activity = activitiesByFirstSequence.get(
                    event.sequence,
                  );
                  if (!activity) return null;
                  return (
                    <ActivityCard
                      key={activity.activityId}
                      activity={activity}
                      expanded={expandedActivities.has(activity.activityId)}
                      onToggle={() => toggleActivity(activity.activityId)}
                    />
                  );
                }
                return (
                  <article
                    className={`conversation-event conversation-event--${event.type}`}
                    key={event.sequence}
                  >
                    <EventCard event={event} />
                  </article>
                );
              })
            )}
          </div>

          {snapshot.diagnosticCode && (
            <p className="conversation-diagnostic" role="alert">
              {diagnosticMessages[snapshot.diagnosticCode]}
            </p>
          )}
          {actionError && (
            <p className="conversation-diagnostic" role="alert">
              The native conversation action did not complete. No raw native
              error was exposed to the interface.
            </p>
          )}
        </div>
      </div>
    </section>
  );
}
