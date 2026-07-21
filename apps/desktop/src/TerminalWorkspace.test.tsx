import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

const xtermState = vi.hoisted(() => ({ writes: [] as Uint8Array[] }));

vi.mock("@xterm/xterm", () => ({
  Terminal: class {
    options: Record<string, unknown> = {};

    loadAddon() {}
    open() {}
    focus() {}
    dispose() {}
    onData() {
      return { dispose() {} };
    }
    onResize() {
      return { dispose() {} };
    }
    write(data: Uint8Array, callback?: () => void) {
      xtermState.writes.push(data);
      callback?.();
    }
  },
}));
vi.mock("@xterm/addon-fit", () => ({
  FitAddon: class {
    fit() {}
  },
}));

import { TerminalWorkspace } from "./TerminalWorkspace";
import { projectWorkspaceSchema } from "./lib/project";
import {
  scaffoldTerminalRegistry,
  terminalRegistrySchema,
  terminalSnapshotSchema,
} from "./lib/terminal";

const projectId = "018f0000-0000-7000-8000-000000000001";
const associationId = "018f0000-0000-7000-8000-000000000002";
const terminalId = "018f0000-0000-7000-8000-000000000003";
const projects = projectWorkspaceSchema.parse({
  schemaVersion: 1,
  state: "ready",
  projects: [
    {
      id: projectId,
      displayName: "QuireForge",
      archived: false,
      directory: {
        associationId,
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
});
const recoveredTerminal = terminalSnapshotSchema.parse({
  schemaVersion: 1,
  state: "interrupted",
  terminalId,
  projectId,
  title: "Terminal 1",
  live: false,
  columns: 100,
  rows: 30,
  output: [],
  firstSequence: 0,
  lastSequence: 0,
  truncated: false,
  hasMore: false,
  exitCode: null,
  diagnosticCode: null,
});
const liveTerminal = terminalSnapshotSchema.parse({
  ...recoveredTerminal,
  state: "running",
  live: true,
});

const handlers = {
  onStart: vi.fn().mockResolvedValue(recoveredTerminal),
  onPoll: vi.fn().mockResolvedValue(recoveredTerminal),
  onWrite: vi.fn().mockResolvedValue(recoveredTerminal),
  onResize: vi.fn().mockResolvedValue(recoveredTerminal),
  onClose: vi.fn().mockResolvedValue(scaffoldTerminalRegistry),
  onSnapshot: vi.fn(),
};

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("TerminalWorkspace", () => {
  it("starts from an app-owned project ID without accepting a cwd", async () => {
    render(
      <TerminalWorkspace
        availability="native"
        registry={scaffoldTerminalRegistry}
        projects={projects}
        busy={false}
        actionError={false}
        {...handlers}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "New terminal" }));
    await waitFor(() =>
      expect(handlers.onStart).toHaveBeenCalledWith({
        projectId,
        columns: 100,
        rows: 30,
      }),
    );
    expect(
      screen.queryByLabelText(/working directory/u),
    ).not.toBeInTheDocument();
  });

  it("explains recovery without fabricating persisted output", () => {
    const registry = terminalRegistrySchema.parse({
      ...scaffoldTerminalRegistry,
      terminals: [recoveredTerminal],
    });
    render(
      <TerminalWorkspace
        availability="native"
        registry={registry}
        projects={projects}
        busy={false}
        actionError={false}
        {...handlers}
      />,
    );

    expect(screen.getByText("Interrupted terminal")).toBeInTheDocument();
    expect(
      screen.getByText(/does not persist terminal output/u),
    ).toBeInTheDocument();
  });

  it("requires explicit confirmation before ending owned jobs", async () => {
    const registry = terminalRegistrySchema.parse({
      ...scaffoldTerminalRegistry,
      terminals: [recoveredTerminal],
    });
    render(
      <TerminalWorkspace
        availability="native"
        registry={registry}
        projects={projects}
        busy={false}
        actionError={false}
        {...handlers}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Close Terminal 1" }));
    expect(
      screen.getByText(/foreground and background jobs/u),
    ).toBeInTheDocument();
    expect(handlers.onClose).not.toHaveBeenCalled();
    fireEvent.click(
      screen.getByRole("button", { name: "End processes and close" }),
    );
    await waitFor(() =>
      expect(handlers.onClose).toHaveBeenCalledWith({ terminalId }),
    );
  });

  it("polls a live PTY and renders the exact decoded bytes", async () => {
    xtermState.writes.length = 0;
    vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
      callback(0);
      return 1;
    });
    const completed = terminalSnapshotSchema.parse({
      ...liveTerminal,
      state: "exited",
      output: [{ sequence: 1, dataBase64: "/wA=" }],
      firstSequence: 1,
      lastSequence: 1,
      exitCode: 0,
    });
    const onPoll = vi.fn().mockResolvedValue(completed);
    const registry = terminalRegistrySchema.parse({
      ...scaffoldTerminalRegistry,
      terminals: [liveTerminal],
    });
    render(
      <TerminalWorkspace
        availability="native"
        registry={registry}
        projects={projects}
        busy={false}
        actionError={false}
        {...handlers}
        onPoll={onPoll}
      />,
    );

    await waitFor(() =>
      expect(onPoll).toHaveBeenCalledWith({ terminalId, afterSequence: 0 }),
    );
    await waitFor(() =>
      expect(
        xtermState.writes.map((bytes) => Array.from(bytes)),
      ).toContainEqual([255, 0]),
    );
  });

  it("does not simulate a native shell in browser preview", () => {
    render(
      <TerminalWorkspace
        availability="preview"
        registry={scaffoldTerminalRegistry}
        projects={projects}
        busy={false}
        actionError={false}
        {...handlers}
      />,
    );

    expect(screen.getByText(/cannot start or simulate/u)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "New terminal" })).toBeDisabled();
  });
});
