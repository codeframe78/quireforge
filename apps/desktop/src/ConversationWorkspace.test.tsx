import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ConversationWorkspace } from "./ConversationWorkspace";
import { scaffoldCodexRuntime } from "./lib/codex";
import {
  conversationSnapshotSchema,
  scaffoldConversation,
} from "./lib/conversation";
import { projectWorkspaceSchema } from "./lib/project";

const projectId = "018f0000-0000-7000-8000-000000000001";
const conversationId = "018f0000-0000-7000-8000-000000000010";
const project = projectWorkspaceSchema.parse({
  schemaVersion: 1,
  state: "ready",
  projects: [
    {
      id: projectId,
      displayName: "QuireForge",
      archived: false,
      directory: {
        associationId: "018f0000-0000-7000-8000-000000000002",
        displayPath: "~/work/quireforge",
        resolvedDisplayPath: "/mnt/work/quireforge",
        state: "connected-accessible",
        expectedAccess: "read-write",
        isPrimary: true,
        git: { isRepository: true, isLinkedWorktree: false },
        hasAgentsGuidance: true,
        hasCodexConfig: false,
      },
    },
  ],
  pendingAttachment: null,
  diagnosticCode: null,
}).projects[0];

const runningConversation = conversationSnapshotSchema.parse({
  schemaVersion: 2,
  state: "running",
  conversationId,
  projectId,
  modelId: "gpt-5.6-sol",
  reasoningEffort: "high",
  sandboxMode: "workspace-write",
  approvalPolicy: "on-request",
  pendingApproval: null,
  events: [{ type: "lifecycle", sequence: 1, phase: "running" }],
  diagnosticCode: null,
});

function renderWorkspace(
  overrides: Partial<React.ComponentProps<typeof ConversationWorkspace>> = {},
) {
  const onStart = vi.fn().mockResolvedValue(runningConversation);
  const onInterrupt = vi.fn().mockResolvedValue({
    ...runningConversation,
    state: "interrupted",
    events: [{ type: "lifecycle", sequence: 2, phase: "interrupted" }],
  });
  const onDecideApproval = vi.fn().mockResolvedValue(runningConversation);
  render(
    <ConversationWorkspace
      availability="native"
      snapshot={scaffoldConversation}
      events={[]}
      runtime={scaffoldCodexRuntime}
      project={project}
      busy={false}
      actionError={false}
      onStart={onStart}
      onInterrupt={onInterrupt}
      onDecideApproval={onDecideApproval}
      {...overrides}
    />,
  );
  return { onStart, onInterrupt, onDecideApproval };
}

describe("ConversationWorkspace", () => {
  it("submits bounded runtime-derived controls for a verified project", async () => {
    const { onStart } = renderWorkspace();
    const start = screen.getByRole("button", { name: "Start task" });
    expect(start).toBeDisabled();

    fireEvent.change(screen.getByLabelText("Task"), {
      target: { value: "Review the conversation UI." },
    });
    fireEvent.change(screen.getByLabelText("Reasoning"), {
      target: { value: "high" },
    });
    fireEvent.click(start);

    await waitFor(() =>
      expect(onStart).toHaveBeenCalledWith({
        projectId,
        prompt: "Review the conversation UI.",
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
      }),
    );
    expect(screen.getByLabelText("Task")).toHaveValue("");
  });

  it("blocks an unrestricted no-approval combination before IPC", () => {
    const { onStart } = renderWorkspace();
    fireEvent.change(screen.getByLabelText("Task"), {
      target: { value: "Make a change." },
    });
    fireEvent.change(screen.getByLabelText("Filesystem access"), {
      target: { value: "danger-full-access" },
    });
    fireEvent.change(screen.getByLabelText("Approval policy"), {
      target: { value: "never" },
    });

    expect(
      screen.getByText(
        "Unrestricted execution cannot be combined with disabled approvals.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start task" })).toBeDisabled();
    expect(onStart).not.toHaveBeenCalled();
  });

  it("renders normalized events and interrupts only the app conversation ID", () => {
    const { onInterrupt } = renderWorkspace({
      snapshot: runningConversation,
      events: [
        ...runningConversation.events,
        {
          type: "agent-message-delta",
          sequence: 2,
          delta: "The UI is ready for review.",
        },
        {
          type: "activity",
          sequence: 3,
          activityId: "018f0000-0000-7000-8000-000000000011",
          kind: "command-execution",
          status: "completed",
          title: "Run command",
          detail: "pnpm check",
          exitCode: 0,
        },
        {
          type: "activity-output-delta",
          sequence: 4,
          activityId: "018f0000-0000-7000-8000-000000000011",
          delta: "Checks passed.",
        },
      ],
    });

    expect(screen.getByText("The UI is ready for review.")).toBeInTheDocument();
    const activity = screen.getByRole("button", { name: /Run command/u });
    expect(activity).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("pnpm check")).not.toBeInTheDocument();
    fireEvent.click(activity);
    expect(activity).toHaveAttribute("aria-expanded", "true");
    expect(screen.getByText("pnpm check")).toBeInTheDocument();
    expect(screen.getByText("Checks passed.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Stop task" }));
    expect(onInterrupt).toHaveBeenCalledWith(conversationId);
  });

  it("submits only the exact pending approval decision", async () => {
    const approvalId = "018f0000-0000-7000-8000-000000000011";
    const activityId = "018f0000-0000-7000-8000-000000000012";
    const waiting = conversationSnapshotSchema.parse({
      ...runningConversation,
      state: "waiting-for-approval",
      pendingApproval: {
        approvalId,
        activityId,
        kind: "command-execution",
        title: "Run this command?",
        reason: "The check needs permission.",
        details: [{ label: "Command", value: "pnpm check" }],
        decisions: ["approve", "decline", "cancel"],
      },
      events: [
        {
          type: "approval-requested",
          sequence: 2,
          approvalId,
          activityId,
          kind: "command-execution",
        },
      ],
    });
    const { onInterrupt, onDecideApproval } = renderWorkspace({
      snapshot: waiting,
      events: waiting.events,
    });

    expect(
      screen.getByText("Codex is waiting for approval"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("Approval requested for command execution."),
    ).toBeInTheDocument();
    expect(screen.getByText("The check needs permission.")).toBeInTheDocument();
    expect(screen.getByText("pnpm check")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Approve once" }));
    await waitFor(() =>
      expect(onDecideApproval).toHaveBeenCalledWith({
        conversationId,
        approvalId,
        decision: "approve",
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Stop task" }));
    expect(onInterrupt).toHaveBeenCalledWith(conversationId);
  });

  it("renders only advertised decisions and prevents duplicate submission", async () => {
    const approvalId = "018f0000-0000-7000-8000-000000000021";
    const activityId = "018f0000-0000-7000-8000-000000000022";
    const waiting = conversationSnapshotSchema.parse({
      ...runningConversation,
      state: "waiting-for-approval",
      pendingApproval: {
        approvalId,
        activityId,
        kind: "command-execution",
        title: "Allow this command?",
        reason: null,
        details: [],
        decisions: ["decline"],
      },
      events: [],
    });
    let resolveDecision:
      ((value: typeof runningConversation) => void) | undefined;
    const onDecideApproval = vi.fn(
      () =>
        new Promise<typeof runningConversation>((resolve) => {
          resolveDecision = resolve;
        }),
    );
    renderWorkspace({
      snapshot: waiting,
      onDecideApproval,
    });

    expect(screen.queryByRole("button", { name: "Approve once" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Cancel task" })).toBeNull();
    const decline = screen.getByRole("button", { name: "Decline" });
    fireEvent.click(decline);
    fireEvent.click(decline);
    expect(onDecideApproval).toHaveBeenCalledTimes(1);
    expect(screen.getByRole("button", { name: "Decline…" })).toBeDisabled();
    resolveDecision?.(runningConversation);
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Decline" })).toBeEnabled(),
    );
  });

  it("keeps browser preview honest and non-interactive", () => {
    renderWorkspace({ availability: "preview", project: undefined });
    expect(
      screen.getByText(
        "Browser preview cannot start or simulate a Codex task.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start task" })).toBeDisabled();
  });

  it("requires the advertised native conversation capability", () => {
    renderWorkspace({
      runtime: {
        ...scaffoldCodexRuntime,
        capabilities: scaffoldCodexRuntime.capabilities.filter(
          ({ id }) => id !== "conversation-runtime",
        ),
      },
    });
    fireEvent.change(screen.getByLabelText("Task"), {
      target: { value: "Review the task." },
    });

    expect(
      screen.getByText(
        "A ready Codex conversation capability and model catalog are required.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Start task" })).toBeDisabled();
  });
});
