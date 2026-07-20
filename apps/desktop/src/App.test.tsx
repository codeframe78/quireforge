import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import { codexAuthSchema, scaffoldCodexAuth } from "./lib/auth";
import { scaffoldCodexRuntime } from "./lib/codex";
import { scaffoldBootstrap } from "./lib/contract";
import {
  projectWorkspaceSchema,
  scaffoldProjectWorkspace,
} from "./lib/project";

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
    expect(screen.getAllByText("ready")).toHaveLength(5);
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
      schemaVersion: 1,
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
});
