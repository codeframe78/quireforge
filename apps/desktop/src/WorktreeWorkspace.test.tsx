import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { WorktreeWorkspace } from "./WorktreeWorkspace";
import {
  scaffoldWorktreeWorkspace,
  worktreePreviewSchema,
  worktreeWorkspaceSchema,
} from "./lib/worktree";

const sourceProjectId = "018f0000-0000-7000-8000-000000000001";
const confirmationId = "018f0000-0000-7000-8000-000000000002";
const workspace = worktreeWorkspaceSchema.parse({
  schemaVersion: 1,
  state: "ready",
  sourceProjectId,
  worktrees: [
    {
      projectId: sourceProjectId,
      displayName: "QuireForge",
      displayPath: "~/work/quireforge",
      branchName: "main",
      ownership: "source",
      state: "ready",
      current: true,
    },
    {
      projectId: null,
      displayName: "feature/external",
      displayPath: "~/work/external",
      branchName: "feature/external",
      ownership: "external",
      state: "ready",
      current: false,
    },
  ],
  truncated: false,
  diagnosticCode: null,
});

const handlers = {
  onRefresh: vi.fn().mockResolvedValue(undefined),
  onCreate: vi.fn().mockResolvedValue(undefined),
  onPickAttach: vi.fn().mockResolvedValue(undefined),
  onConfirm: vi.fn().mockResolvedValue(undefined),
  onCancel: vi.fn().mockResolvedValue(undefined),
  onSelectProject: vi.fn(),
};

describe("WorktreeWorkspace", () => {
  it("accepts a bounded branch and offers no cleanup action", () => {
    render(
      <WorktreeWorkspace
        availability="native"
        projectName="QuireForge"
        snapshot={workspace}
        preview={null}
        result={null}
        busy={false}
        actionError={false}
        {...handlers}
      />,
    );

    const create = screen.getByRole("button", {
      name: "Preview managed worktree",
    });
    expect(create).toBeDisabled();
    fireEvent.change(screen.getByLabelText("New branch name"), {
      target: { value: "feature/native-foundation" },
    });
    expect(create).toBeEnabled();
    fireEvent.click(create);
    expect(handlers.onCreate).toHaveBeenCalledWith("feature/native-foundation");
    expect(
      screen.queryByRole("button", { name: /remove|delete|prune|clean/u }),
    ).not.toBeInTheDocument();
    expect(screen.getByText("external checkout")).toBeInTheDocument();
  });

  it("requires confirmation and can cancel by opaque token only", () => {
    const preview = worktreePreviewSchema.parse({
      schemaVersion: 1,
      state: "ready",
      sourceProjectId,
      operation: "create",
      branchName: "feature/confirmed",
      displayPath: "~/.local/share/quireforge/worktrees/confirmed",
      ownership: "managed",
      destructive: false,
      confirmationId,
      diagnosticCode: null,
    });
    render(
      <WorktreeWorkspace
        availability="native"
        projectName="QuireForge"
        snapshot={workspace}
        preview={preview}
        result={null}
        busy={false}
        actionError={false}
        {...handlers}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Confirm create" }));
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(handlers.onConfirm).toHaveBeenCalledWith(confirmationId);
    expect(handlers.onCancel).toHaveBeenCalledWith(confirmationId);
  });

  it("does not simulate worktrees in browser preview", () => {
    render(
      <WorktreeWorkspace
        availability="preview"
        projectName={null}
        snapshot={scaffoldWorktreeWorkspace}
        preview={null}
        result={null}
        busy={false}
        actionError={false}
        {...handlers}
      />,
    );

    expect(
      screen.getByText(/Browser preview cannot inspect or create/u),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Preview managed worktree" }),
    ).toBeDisabled();
  });
});
