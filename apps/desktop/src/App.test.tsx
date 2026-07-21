import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@xterm/xterm", () => ({ Terminal: class {} }));
vi.mock("@xterm/addon-fit", () => ({ FitAddon: class {} }));

import App from "./App";
import { codexAuthSchema, scaffoldCodexAuth } from "./lib/auth";
import { scaffoldCodexRuntime } from "./lib/codex";
import { scaffoldBootstrap } from "./lib/contract";
import {
  type ConversationSnapshot,
  conversationSnapshotSchema,
  scaffoldConversation,
} from "./lib/conversation";
import {
  projectWorkspaceSchema,
  scaffoldProjectWorkspace,
} from "./lib/project";
import { sessionLifecycleSchema } from "./lib/session";
import { worktreeWorkspaceSchema } from "./lib/worktree";

const projectId = "018f0000-0000-7000-8000-000000000001";
const associationId = "018f0000-0000-7000-8000-000000000002";
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
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "A quiet place for ambitious work.",
      }),
    ).toBeInTheDocument();
    expect(await screen.findByText("Native IPC verified")).toBeInTheDocument();
    expect(await screen.findAllByText("Codex adapter ready")).toHaveLength(2);
    expect(screen.getAllByText("No project attached")).toHaveLength(2);
    expect(
      await screen.findByRole("button", { name: "Continue in browser" }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("ready")).toHaveLength(6);
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

    expect(await screen.findByText("Browser preview")).toBeInTheDocument();
    expect(await screen.findAllByText("Native probe unavailable")).toHaveLength(
      2,
    );
    expect(screen.queryByText("Native IPC verified")).not.toBeInTheDocument();
    expect(
      await screen.findByText("Native authentication unavailable"),
    ).toBeInTheDocument();
    expect(
      await screen.findByText(/cannot open a native folder picker/u),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Attach local project" }),
    ).toBeDisabled();
  });

  it("persists the explicit theme choice", () => {
    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
      />,
    );

    const button = screen.getByRole("button", { name: /theme/u });
    fireEvent.click(button);

    expect(window.localStorage.getItem("quireforge-theme")).toBe(
      document.documentElement.dataset.theme,
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
      await screen.findByRole("button", { name: "Continue in browser" }),
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
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
        loadProjects={() => Promise.resolve(scaffoldProjectWorkspace)}
        pickProject={pickProject}
        confirmProject={confirmProject}
      />,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Attach local project" }),
    );
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
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
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
      schemaVersion: 2,
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
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
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
      schemaVersion: 2,
      state: "running",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
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

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
        loadProjects={() => Promise.resolve(attachedProject)}
        loadConversation={() => Promise.resolve(scaffoldConversation)}
        startConversationTask={startConversationTask}
        pollConversationTask={pollConversationTask}
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
  });

  it("cancels pending conversation polling when the shell unmounts", async () => {
    const running = conversationSnapshotSchema.parse({
      schemaVersion: 2,
      state: "running",
      conversationId: "018f0000-0000-7000-8000-000000000010",
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
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
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
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
      schemaVersion: 2,
      state: "waiting-for-approval",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
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
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
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
        schemaVersion: 2,
        state: "running",
        conversationId,
        projectId: taskProjectId,
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
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
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
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
      schemaVersion: 2,
      state: "ready",
      sessions: [
        {
          conversationId,
          projectId,
          parentConversationId: null,
          title: "Review session wiring",
          modelId: "gpt-5.6-sol",
          reasoningEffort: "high",
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
      schemaVersion: 2,
      state: "running",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
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
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
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

    fireEvent.click(
      screen.getByText("Review session wiring").closest("button")!,
    );
    fireEvent.change(screen.getByLabelText("Next task"), {
      target: { value: "Continue from the app-owned reference." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Resume" }));

    await waitFor(() =>
      expect(resumeConversationTask).toHaveBeenCalledWith({
        conversationId,
        prompt: "Continue from the app-owned reference.",
      }),
    );
    await waitFor(() =>
      expect(pollConversationTask).toHaveBeenCalledWith(conversationId),
    );
  });
});
