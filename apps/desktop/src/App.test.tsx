import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@xterm/xterm", () => ({ Terminal: class {} }));
vi.mock("@xterm/addon-fit", () => ({ FitAddon: class {} }));

import App from "./App";
import { sharedConversationAttachmentFixture } from "./lib/attachment";
import { codexAuthSchema, scaffoldCodexAuth } from "./lib/auth";
import { scaffoldCodexRuntime } from "./lib/codex";
import { scaffoldBootstrap } from "./lib/contract";
import { sharedFilePreviewFixture } from "./lib/filePreview";
import {
  type ConversationSnapshot,
  conversationSnapshotSchema,
  scaffoldConversation,
} from "./lib/conversation";
import {
  projectWorkspaceSchema,
  scaffoldProjectWorkspace,
} from "./lib/project";
import {
  integrationCatalogSchema,
  integrationControlResultSchema,
  scaffoldIntegrationCatalog,
  scaffoldIntegrationControlPreview,
  scaffoldIntegrationControlResult,
  scaffoldIntegrationMutationPreview,
  scaffoldIntegrationMutationResult,
} from "./lib/integration";
import { sessionLifecycleSchema } from "./lib/session";
import { worktreeWorkspaceSchema } from "./lib/worktree";
import { scaffoldCodexUsage } from "./lib/usage";

const projectId = "018f0000-0000-7000-8000-000000000001";
const associationId = "018f0000-0000-7000-8000-000000000002";
const authenticatedAuth = codexAuthSchema.parse({
  ...scaffoldCodexAuth,
  state: "authenticated",
  accountKind: "chatgpt",
});
function modelSelection(reasoningEffort = "high") {
  return {
    schemaVersion: 1 as const,
    availability: "ready" as const,
    effective: { modelId: "gpt-5.6-sol", reasoningEffort },
    pending: null,
    policy: {
      ownership: "manual" as const,
      userLocked: false,
      allowedModelIds: [],
      reasoningCeiling: null,
    },
    diagnosticCode: null,
  };
}
const pendingProject = projectWorkspaceSchema.parse({
  ...scaffoldProjectWorkspace,
  pendingAttachment: {
    operation: "attach",
    projectId: null,
    displayName: "QuireForge",
    selectedDisplayPath: "~/work/quireforge-link",
    resolvedDisplayPath: "/mnt/work/quireforge",
    state: "connected-accessible",
    git: { isRepository: true, isLinkedWorktree: true },
    hasAgentsGuidance: true,
    hasCodexConfig: true,
  },
});
const attachedProject = projectWorkspaceSchema.parse({
  ...scaffoldProjectWorkspace,
  state: "ready",
  projects: [
    {
      id: projectId,
      displayName: "QuireForge",
      archived: false,
      directory: {
        associationId,
        displayPath: "~/work/quireforge-link",
        resolvedDisplayPath: "/mnt/work/quireforge",
        state: "connected-accessible",
        expectedAccess: "read-write",
        isPrimary: true,
        git: { isRepository: true, isLinkedWorktree: true },
        hasAgentsGuidance: true,
        hasCodexConfig: true,
      },
    },
  ],
});
const missingProject = projectWorkspaceSchema.parse({
  ...attachedProject,
  projects: attachedProject.projects.map((project) => ({
    ...project,
    directory: project.directory
      ? { ...project.directory, state: "missing-or-moved" as const }
      : null,
  })),
});
const pendingRelink = projectWorkspaceSchema.parse({
  ...missingProject,
  pendingAttachment: {
    operation: "relink",
    projectId,
    displayName: "QuireForge",
    selectedDisplayPath: "/media/quireforge",
    resolvedDisplayPath: "/media/quireforge",
    state: "connected-read-only",
    git: { isRepository: true, isLinkedWorktree: false },
    hasAgentsGuidance: true,
    hasCodexConfig: false,
  },
});
const readyIntegrationCatalog = integrationCatalogSchema.parse({
  ...scaffoldIntegrationCatalog,
  capabilities: scaffoldIntegrationCatalog.capabilities.map((capability) =>
    ["plugin.install", "plugin.remove", "marketplace.configure"].includes(
      capability.id,
    )
      ? {
          ...capability,
          availability: "ready",
          implementation: "ready",
          diagnosticCode: null,
        }
      : capability,
  ),
});
const controlReadyIntegrationCatalog = integrationCatalogSchema.parse({
  ...readyIntegrationCatalog,
  capabilities: readyIntegrationCatalog.capabilities.map((capability) =>
    ["connector.authorize", "skill.configure", "mcp.authorize"].includes(
      capability.id,
    )
      ? {
          ...capability,
          availability: "ready",
          implementation: "ready",
          diagnosticCode: null,
        }
      : capability,
  ),
});

describe("QuireForge desktop shell", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("renders the honest scaffold state and verifies native data", async () => {
    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadUsage={() => Promise.resolve(scaffoldCodexUsage)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
      />,
    );

    expect(
      await screen.findByRole("heading", {
        name: "What should we build today?",
      }),
    ).toBeInTheDocument();
    expect(await screen.findByText("Native IPC verified")).toBeInTheDocument();
    expect(await screen.findAllByText("Codex adapter ready")).toHaveLength(1);
    expect(screen.getAllByText("No project attached")).toHaveLength(2);
    expect(
      await screen.findByText("Codex account connected"),
    ).toBeInTheDocument();
    expect(screen.queryByText(/Milestone/u)).not.toBeInTheDocument();
    expect(screen.getAllByText("73%")).not.toHaveLength(0);
    expect(screen.getAllByText("ready")).toHaveLength(12);
    expect(screen.queryByText("planned")).not.toBeInTheDocument();
    expect(
      screen.getByText(
        /not made, endorsed, supported, or distributed by OpenAI/u,
      ),
    ).toBeInTheDocument();
  });

  it("labels a browser-only render without simulating native success", async () => {
    render(
      <App
        loadBootstrap={() => Promise.reject(new Error("no IPC"))}
        loadRuntime={() => Promise.reject(new Error("no IPC"))}
        loadAuth={() => Promise.reject(new Error("no IPC"))}
        loadProjects={() => Promise.reject(new Error("no IPC"))}
      />,
    );

    expect(
      await screen.findByText(
        "Native Codex authentication is unavailable in this browser preview.",
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText("Native IPC verified")).not.toBeInTheDocument();
    expect(screen.queryByText(/native folder picker/u)).not.toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Attach local project" }),
    ).not.toBeInTheDocument();
  });

  it("does not read workspace data before Codex authentication", async () => {
    const loadUsage = vi.fn();
    const loadProjects = vi.fn();
    const loadConversation = vi.fn();
    const loadActiveConversationTasks = vi.fn();
    const loadSessions = vi.fn();
    const loadTerminalsTask = vi.fn();
    const loadIntegrationCatalogTask = vi.fn();

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
        loadUsage={loadUsage}
        loadProjects={loadProjects}
        loadConversation={loadConversation}
        loadActiveConversationTasks={loadActiveConversationTasks}
        loadSessions={loadSessions}
        loadTerminalsTask={loadTerminalsTask}
        loadIntegrationCatalogTask={loadIntegrationCatalogTask}
      />,
    );

    expect(
      await screen.findByRole("button", { name: "Continue with ChatGPT" }),
    ).toBeInTheDocument();
    expect(loadUsage).not.toHaveBeenCalled();
    expect(loadProjects).not.toHaveBeenCalled();
    expect(loadConversation).not.toHaveBeenCalled();
    expect(loadActiveConversationTasks).not.toHaveBeenCalled();
    expect(loadSessions).not.toHaveBeenCalled();
    expect(loadTerminalsTask).not.toHaveBeenCalled();
    expect(loadIntegrationCatalogTask).not.toHaveBeenCalled();
  });

  it("persists the explicit theme choice", () => {
    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
      />,
    );

    const button = screen.getByRole("button", { name: /theme/u });
    fireEvent.click(button);

    expect(window.localStorage.getItem("quireforge-theme")).toBe(
      document.documentElement.dataset.theme,
    );
  });

  it("previews one native-selected file through an opaque project ID", async () => {
    const pickFilePreviewTask = vi
      .fn()
      .mockResolvedValue({ ...sharedFilePreviewFixture, projectId });
    const openFilePreviewTask = vi.fn().mockResolvedValue(undefined);
    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(attachedProject)}
        pickFilePreviewTask={pickFilePreviewTask}
        openFilePreviewTask={openFilePreviewTask}
      />,
    );

    const chooseProjectFile = await screen.findByRole("button", {
      name: "Choose project file",
    });
    await waitFor(() => expect(chooseProjectFile).toBeEnabled());
    fireEvent.click(chooseProjectFile);

    expect(
      await screen.findByRole("article", {
        name: "Preview of docs/preview.md",
      }),
    ).toBeInTheDocument();
    expect(pickFilePreviewTask).toHaveBeenCalledWith(projectId);

    fireEvent.click(
      screen.getByRole("button", { name: "Open with desktop app" }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Open with default app" }),
    );
    await waitFor(() =>
      expect(openFilePreviewTask).toHaveBeenCalledWith({
        openActionId: sharedFilePreviewFixture.openActionId,
      }),
    );
  });

  it("sends only reviewed opaque image IDs with an explicit task start", async () => {
    const conversationId = "018f0000-0000-7000-8000-000000000010";
    const attachmentSnapshot = {
      ...sharedConversationAttachmentFixture,
      projectId,
    };
    const pickConversationAttachmentsTask = vi
      .fn()
      .mockResolvedValue(attachmentSnapshot);
    const startConversationTask = vi.fn().mockResolvedValue(
      conversationSnapshotSchema.parse({
        ...scaffoldConversation,
        state: "running",
        conversationId,
        projectId,
        modelId: "gpt-5.6-sol",
        reasoningEffort: "medium",
        modelSelection: modelSelection("medium"),
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
        events: [{ type: "lifecycle", sequence: 1, phase: "running" }],
      }),
    );
    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(attachedProject)}
        loadConversation={() => Promise.resolve(scaffoldConversation)}
        pickConversationAttachmentsTask={pickConversationAttachmentsTask}
        startConversationTask={startConversationTask}
      />,
    );

    const chooseImages = await screen.findByRole("button", {
      name: "Choose images",
    });
    await waitFor(() => expect(chooseImages).toBeEnabled());
    fireEvent.click(chooseImages);
    expect(await screen.findByText("review.png")).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Task"), {
      target: { value: "Review the attached image." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Start task" }));

    await waitFor(() =>
      expect(startConversationTask).toHaveBeenCalledWith(
        expect.objectContaining({
          projectId,
          prompt: "Review the attached image.",
          attachmentIds: [
            sharedConversationAttachmentFixture.attachments[0]!.attachmentId,
          ],
        }),
      ),
    );
    expect(screen.queryByText("review.png")).not.toBeInTheDocument();
    expect(JSON.stringify(startConversationTask.mock.calls)).not.toContain(
      "/private/",
    );
  });

  it("renders a device-code handoff and cancels through fixed actions", async () => {
    const pending = codexAuthSchema.parse({
      ...scaffoldCodexAuth,
      state: "login-pending",
      pendingMethod: "device-code",
      handoff: {
        verificationUrl: "https://auth.openai.com/device",
        userCode: "SAFE-CODE",
      },
    });
    const startAuth = vi.fn().mockResolvedValue(pending);
    const openAuthBrowser = vi.fn().mockResolvedValue(undefined);
    const cancelAuth = vi.fn().mockResolvedValue(scaffoldCodexAuth);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
        startAuth={startAuth}
        openAuthBrowser={openAuthBrowser}
        cancelAuth={cancelAuth}
      />,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Use a device code" }),
    );
    expect(await screen.findByText("SAFE-CODE")).toBeInTheDocument();
    expect(startAuth).toHaveBeenCalledWith("device-code");
    expect(openAuthBrowser).toHaveBeenCalledOnce();

    fireEvent.click(screen.getByRole("button", { name: "Cancel sign-in" }));
    expect(
      await screen.findByRole("button", { name: "Continue with ChatGPT" }),
    ).toBeInTheDocument();
    expect(cancelAuth).toHaveBeenCalledOnce();
  });

  it("requires a second explicit action before logout", async () => {
    const authenticated = codexAuthSchema.parse({
      ...scaffoldCodexAuth,
      state: "authenticated",
      accountKind: "chatgpt",
    });
    const logoutAuth = vi.fn().mockResolvedValue(scaffoldCodexAuth);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticated)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
        logoutAuth={logoutAuth}
      />,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Sign out of Codex" }),
    );
    expect(logoutAuth).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Confirm sign out" }));
    expect(logoutAuth).toHaveBeenCalledOnce();
  });

  it("reviews selected and resolved paths before confirming attachment", async () => {
    const pickProject = vi.fn().mockResolvedValue(pendingProject);
    const confirmProject = vi.fn().mockResolvedValue(attachedProject);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
        pickProject={pickProject}
        confirmProject={confirmProject}
      />,
    );

    const attachProject = await screen.findByRole("button", {
      name: "Attach local project",
    });
    await waitFor(() => expect(attachProject).toBeEnabled());
    fireEvent.click(attachProject);
    expect(
      await screen.findByText("~/work/quireforge-link"),
    ).toBeInTheDocument();
    expect(screen.getByText("/mnt/work/quireforge")).toBeInTheDocument();
    expect(screen.getByText("Linked Git worktree")).toBeInTheDocument();
    expect(screen.getByText("AGENTS.md detected")).toBeInTheDocument();
    expect(confirmProject).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Confirm attachment" }));

    expect(await screen.findAllByText("QuireForge")).not.toHaveLength(0);
    expect(confirmProject).toHaveBeenCalledOnce();
    expect(pickProject).toHaveBeenCalledWith();
  });

  it("requires explicit confirmation and passes only an opaque ID to detach", async () => {
    const detachProjectDirectory = vi
      .fn()
      .mockResolvedValue(scaffoldProjectWorkspace);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(attachedProject)}
        detachProjectDirectory={detachProjectDirectory}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Detach" }));
    expect(detachProjectDirectory).not.toHaveBeenCalled();
    expect(
      screen.getByText(/source content will remain in place/iu),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Confirm detach" }));

    expect(detachProjectDirectory).toHaveBeenCalledWith(projectId);
    expect(
      await screen.findByRole("heading", { name: "No project attached" }),
    ).toBeInTheDocument();
  });

  it("blocks a missing cwd and relinks through an explicitly reviewed preview", async () => {
    const preflightProjectDirectory = vi.fn().mockResolvedValue({
      schemaVersion: 3,
      projectId,
      cwdReady: false,
      displayPath: "~/work/quireforge-link",
      state: "missing-or-moved",
      diagnosticCode: null,
    });
    const pickRelink = vi.fn().mockResolvedValue(pendingRelink);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(missingProject)}
        preflightProjectDirectory={preflightProjectDirectory}
        pickRelink={pickRelink}
      />,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Verify directory" }),
    );
    expect(
      await screen.findByText("Working directory blocked: Missing or moved"),
    ).toBeInTheDocument();
    expect(preflightProjectDirectory).toHaveBeenCalledWith(projectId);

    fireEvent.click(screen.getByRole("button", { name: "Relink" }));
    expect(await screen.findAllByText("/media/quireforge")).toHaveLength(2);
    expect(
      screen.getByText(/refuse it as a writable task directory/iu),
    ).toBeInTheDocument();
    expect(pickRelink).toHaveBeenCalledWith(projectId);
  });

  it("starts, polls, deduplicates, and completes a native conversation", async () => {
    const conversationId = "018f0000-0000-7000-8000-000000000010";
    const running = conversationSnapshotSchema.parse({
      schemaVersion: 3,
      state: "running",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection: modelSelection(),
      sandboxMode: "workspace-write",
      approvalPolicy: "on-request",
      pendingApproval: null,
      events: [
        {
          type: "agent-message-delta",
          sequence: 1,
          delta: "Reviewing the task.",
        },
      ],
      diagnosticCode: null,
    });
    const completed = conversationSnapshotSchema.parse({
      ...running,
      state: "completed",
      events: [
        ...running.events,
        { type: "lifecycle", sequence: 2, phase: "completed" },
      ],
    });
    const startConversationTask = vi.fn().mockResolvedValue(running);
    const pollConversationTask = vi.fn().mockResolvedValue(completed);
    const notifyConversationTask = vi.fn().mockResolvedValue({
      schemaVersion: 1 as const,
      status: "foreground" as const,
    });

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(attachedProject)}
        loadConversation={() => Promise.resolve(scaffoldConversation)}
        startConversationTask={startConversationTask}
        pollConversationTask={pollConversationTask}
        notifyConversationTask={notifyConversationTask}
      />,
    );

    const start = await screen.findByRole("button", { name: "Start task" });
    fireEvent.change(screen.getByLabelText("Task"), {
      target: { value: "Review the shell wiring." },
    });
    fireEvent.change(screen.getByLabelText("Reasoning"), {
      target: { value: "high" },
    });
    await waitFor(() => expect(start).toBeEnabled());
    fireEvent.click(start);

    await waitFor(() =>
      expect(startConversationTask).toHaveBeenCalledWith(
        expect.objectContaining({ projectId, reasoningEffort: "high" }),
      ),
    );
    await waitFor(() =>
      expect(pollConversationTask).toHaveBeenCalledWith(conversationId),
    );
    expect(await screen.findByText("Task completed")).toBeInTheDocument();
    expect(screen.getAllByText("Reviewing the task.")).toHaveLength(1);
    expect(notifyConversationTask).toHaveBeenCalledWith(conversationId);
  });

  it("cancels pending conversation polling when the shell unmounts", async () => {
    const running = conversationSnapshotSchema.parse({
      schemaVersion: 3,
      state: "running",
      conversationId: "018f0000-0000-7000-8000-000000000010",
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection: modelSelection(),
      sandboxMode: "workspace-write",
      approvalPolicy: "on-request",
      pendingApproval: null,
      events: [],
      diagnosticCode: null,
    });
    const pollConversationTask = vi.fn().mockResolvedValue(running);
    const { unmount } = render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(attachedProject)}
        loadConversation={() => Promise.resolve(running)}
        pollConversationTask={pollConversationTask}
      />,
    );

    expect(
      await screen.findByRole("button", { name: "Stop task" }),
    ).toBeInTheDocument();
    unmount();
    await new Promise((resolve) => window.setTimeout(resolve, 300));
    expect(pollConversationTask).not.toHaveBeenCalled();
  });

  it("prevents an in-flight poll from overwriting an approval decision", async () => {
    const conversationId = "018f0000-0000-7000-8000-000000000010";
    const approvalId = "018f0000-0000-7000-8000-000000000011";
    const waiting = conversationSnapshotSchema.parse({
      schemaVersion: 3,
      state: "waiting-for-approval",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection: modelSelection(),
      sandboxMode: "workspace-write",
      approvalPolicy: "on-request",
      pendingApproval: {
        approvalId,
        activityId: "018f0000-0000-7000-8000-000000000012",
        kind: "command-execution",
        title: "Run checks?",
        reason: null,
        details: [],
        decisions: ["approve", "decline"],
      },
      events: [
        {
          type: "approval-requested",
          sequence: 1,
          approvalId,
          activityId: "018f0000-0000-7000-8000-000000000012",
          kind: "command-execution",
        },
      ],
      diagnosticCode: null,
    });
    const completed = conversationSnapshotSchema.parse({
      ...waiting,
      state: "completed",
      pendingApproval: null,
      events: [
        ...waiting.events,
        {
          type: "approval-resolved",
          sequence: 2,
          approvalId,
          resolution: "approved",
        },
      ],
    });
    let resolvePoll: ((value: typeof waiting) => void) | undefined;
    const pollConversationTask = vi.fn(
      () =>
        new Promise<typeof waiting>((resolve) => {
          resolvePoll = resolve;
        }),
    );
    const decideConversationApprovalTask = vi.fn().mockResolvedValue(completed);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(attachedProject)}
        loadConversation={() => Promise.resolve(waiting)}
        pollConversationTask={pollConversationTask}
        decideConversationApprovalTask={decideConversationApprovalTask}
      />,
    );

    const approve = await screen.findByRole("button", {
      name: "Approve once",
    });
    await waitFor(() => expect(pollConversationTask).toHaveBeenCalled());
    fireEvent.click(approve);
    await waitFor(() =>
      expect(decideConversationApprovalTask).toHaveBeenCalledWith({
        conversationId,
        approvalId,
        decision: "approve",
      }),
    );
    expect(await screen.findByText("Task completed")).toBeInTheDocument();
    resolvePoll?.(waiting);
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    expect(screen.getByText("Task completed")).toBeInTheDocument();
    expect(screen.queryByText("Run checks?")).toBeNull();
  });

  it("polls distinct worktree tasks independently and aggregates their state", async () => {
    const secondProjectId = "018f0000-0000-7000-8000-000000000003";
    const secondAssociationId = "018f0000-0000-7000-8000-000000000004";
    const projects = projectWorkspaceSchema.parse({
      ...attachedProject,
      projects: [
        ...attachedProject.projects,
        {
          id: secondProjectId,
          displayName: "QuireForge parallel",
          archived: false,
          directory: {
            associationId: secondAssociationId,
            displayPath: "~/work/quireforge-parallel",
            resolvedDisplayPath: "/mnt/work/quireforge-parallel",
            state: "connected-accessible",
            expectedAccess: "read-write",
            isPrimary: true,
            git: { isRepository: true, isLinkedWorktree: true },
            hasAgentsGuidance: true,
            hasCodexConfig: false,
          },
        },
      ],
    });
    const worktrees = worktreeWorkspaceSchema.parse({
      schemaVersion: 2,
      state: "ready",
      sourceProjectId: projectId,
      worktrees: [
        {
          projectId,
          recoveryId: null,
          displayName: "QuireForge",
          displayPath: "~/work/quireforge-link",
          branchName: "main",
          ownership: "source",
          state: "ready",
          current: true,
        },
        {
          projectId: secondProjectId,
          recoveryId: null,
          displayName: "QuireForge parallel",
          displayPath: "~/work/quireforge-parallel",
          branchName: "feature/parallel",
          ownership: "managed",
          state: "ready",
          current: false,
        },
      ],
      truncated: false,
      diagnosticCode: null,
    });
    const task = (conversationId: string, taskProjectId: string) =>
      conversationSnapshotSchema.parse({
        schemaVersion: 3,
        state: "running",
        conversationId,
        projectId: taskProjectId,
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        modelSelection: modelSelection(),
        sandboxMode: "workspace-write",
        approvalPolicy: "on-request",
        pendingApproval: null,
        events: [],
        diagnosticCode: null,
      });
    const first = task("018f0000-0000-7000-8000-000000000010", projectId);
    const second = task(
      "018f0000-0000-7000-8000-000000000020",
      secondProjectId,
    );
    const pollConversationTask = vi.fn((conversationId: string) =>
      Promise.resolve(conversationId === first.conversationId ? first : second),
    );
    let resolveLegacyStatus:
      ((snapshot: ConversationSnapshot) => void) | undefined;
    const legacyStatus = new Promise<ConversationSnapshot>((resolve) => {
      resolveLegacyStatus = resolve;
    });
    const { unmount } = render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(projects)}
        loadWorktreesTask={() => Promise.resolve(worktrees)}
        loadConversation={() => legacyStatus}
        loadActiveConversationTasks={() =>
          Promise.resolve({
            schemaVersion: 1,
            capacity: 4,
            conversations: [first, second],
          })
        }
        pollConversationTask={pollConversationTask}
      />,
    );

    expect(await screen.findByText("2 of 4 active")).toBeInTheDocument();
    resolveLegacyStatus?.(first);
    await waitFor(() =>
      expect(screen.getByText("2 of 4 active")).toBeInTheDocument(),
    );
    await waitFor(() => {
      expect(pollConversationTask).toHaveBeenCalledWith(first.conversationId);
      expect(pollConversationTask).toHaveBeenCalledWith(second.conversationId);
    });
    unmount();
  });

  it("wires session search and resume through app-owned references", async () => {
    const conversationId = "018f0000-0000-7000-8000-000000000010";
    const lifecycle = sessionLifecycleSchema.parse({
      schemaVersion: 3,
      state: "ready",
      sessions: [
        {
          conversationId,
          projectId,
          parentConversationId: null,
          title: "Review session wiring",
          modelId: "gpt-5.6-sol",
          reasoningEffort: "high",
          modelSelection: modelSelection(),
          sandboxMode: "workspace-write",
          approvalPolicy: "on-request",
          state: "completed",
          createdAtMs: 1,
          updatedAtMs: 2,
        },
      ],
      diagnosticCode: null,
    });
    const running = conversationSnapshotSchema.parse({
      schemaVersion: 3,
      state: "running",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      modelSelection: modelSelection(),
      sandboxMode: "workspace-write",
      approvalPolicy: "on-request",
      pendingApproval: null,
      events: [],
      diagnosticCode: null,
    });
    const completed = conversationSnapshotSchema.parse({
      ...running,
      state: "completed",
    });
    const loadSessions = vi.fn().mockResolvedValue(lifecycle);
    const resumeConversationTask = vi.fn().mockResolvedValue(running);
    const pollConversationTask = vi.fn().mockResolvedValue(completed);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(attachedProject)}
        loadConversation={() => Promise.resolve(scaffoldConversation)}
        loadSessions={loadSessions}
        resumeConversationTask={resumeConversationTask}
        pollConversationTask={pollConversationTask}
      />,
    );

    fireEvent.change(await screen.findByLabelText("Search session titles"), {
      target: { value: "wiring" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Search" }));
    await waitFor(() =>
      expect(loadSessions).toHaveBeenCalledWith({
        projectId: null,
        searchTerm: "wiring",
      }),
    );

    const sessionWorkspace = screen
      .getByRole("heading", {
        name: "Return to work without copying its history.",
      })
      .closest("section");
    fireEvent.click(
      within(sessionWorkspace!).getByRole("button", {
        name: /Review session wiring/u,
      }),
    );
    fireEvent.change(screen.getByLabelText("Next task"), {
      target: { value: "Continue from the app-owned reference." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Resume" }));

    await waitFor(() =>
      expect(resumeConversationTask).toHaveBeenCalledWith({
        conversationId,
        prompt: "Continue from the app-owned reference.",
        attachmentIds: [],
      }),
    );
    await waitFor(() =>
      expect(pollConversationTask).toHaveBeenCalledWith(conversationId),
    );
  });

  it("wires the Integration Center through fixed catalog and mutation tasks", async () => {
    const loadIntegrationCatalogTask = vi
      .fn()
      .mockResolvedValue(readyIntegrationCatalog);
    const previewIntegrationMutationTask = vi
      .fn()
      .mockResolvedValue(scaffoldIntegrationMutationPreview);
    const confirmIntegrationMutationTask = vi
      .fn()
      .mockResolvedValue(scaffoldIntegrationMutationResult);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
        loadIntegrationCatalogTask={loadIntegrationCatalogTask}
        previewIntegrationMutationTask={previewIntegrationMutationTask}
        confirmIntegrationMutationTask={confirmIntegrationMutationTask}
      />,
    );

    expect(
      await screen.findByRole("heading", {
        name: "Inspect trust before changing state.",
      }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Integrations" })).toBeEnabled();
    fireEvent.change(screen.getByLabelText("Category"), {
      target: { value: "plugin" },
    });
    fireEvent.click(
      await screen.findByRole("button", { name: "Install plugin" }),
    );
    await waitFor(() =>
      expect(previewIntegrationMutationTask).toHaveBeenCalledWith({
        operation: "plugin-install",
        targetEntryId: "plugin:fixture-review",
        repository: null,
        reference: null,
      }),
    );
    fireEvent.click(
      await screen.findByRole("button", { name: "Confirm change" }),
    );
    await waitFor(() =>
      expect(confirmIntegrationMutationTask).toHaveBeenCalledWith({
        confirmationId: scaffoldIntegrationMutationPreview.confirmationId,
      }),
    );
    await waitFor(() =>
      expect(loadIntegrationCatalogTask).toHaveBeenCalledTimes(2),
    );
  });

  it("keeps authorization URLs native while advancing an opaque control action", async () => {
    const pending = integrationControlResultSchema.parse({
      ...scaffoldIntegrationControlResult,
      state: "pending",
      browserHandoffAvailable: false,
    });
    const completed = integrationControlResultSchema.parse({
      ...scaffoldIntegrationControlResult,
      state: "completed",
      actionId: null,
      browserHandoffAvailable: false,
      catalogRefreshRequired: true,
    });
    const loadIntegrationCatalogTask = vi
      .fn()
      .mockResolvedValue(controlReadyIntegrationCatalog);
    const previewIntegrationControlTask = vi
      .fn()
      .mockResolvedValue(scaffoldIntegrationControlPreview);
    const confirmIntegrationControlTask = vi
      .fn()
      .mockResolvedValue(scaffoldIntegrationControlResult);
    const openIntegrationControlTask = vi.fn().mockResolvedValue(pending);
    const pollIntegrationControlTask = vi.fn().mockResolvedValue(completed);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticatedAuth)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
        loadIntegrationCatalogTask={loadIntegrationCatalogTask}
        previewIntegrationControlTask={previewIntegrationControlTask}
        confirmIntegrationControlTask={confirmIntegrationControlTask}
        openIntegrationControlTask={openIntegrationControlTask}
        pollIntegrationControlTask={pollIntegrationControlTask}
      />,
    );

    await screen.findByRole("heading", {
      name: "Inspect trust before changing state.",
    });
    fireEvent.change(screen.getByLabelText("Category"), {
      target: { value: "mcp-server" },
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Authorize MCP server" }),
    );
    await waitFor(() =>
      expect(previewIntegrationControlTask).toHaveBeenCalledWith({
        operation: "mcp-authorize",
        targetEntryId: "mcp:fixture-knowledge",
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: "Confirm action" }));
    await waitFor(() =>
      expect(confirmIntegrationControlTask).toHaveBeenCalledWith({
        confirmationId: scaffoldIntegrationControlPreview.confirmationId,
      }),
    );
    fireEvent.click(
      await screen.findByRole("button", {
        name: "Open authorization in browser",
      }),
    );
    await waitFor(() =>
      expect(openIntegrationControlTask).toHaveBeenCalledWith({
        actionId: scaffoldIntegrationControlResult.actionId,
      }),
    );
    fireEvent.click(
      await screen.findByRole("button", { name: "Check authorization" }),
    );
    await waitFor(() =>
      expect(pollIntegrationControlTask).toHaveBeenCalledWith({
        actionId: scaffoldIntegrationControlResult.actionId,
      }),
    );
    expect(
      await screen.findByText(/completed and the catalog was refreshed/u),
    ).toBeInTheDocument();
    expect(loadIntegrationCatalogTask).toHaveBeenCalledTimes(2);
  });
});
