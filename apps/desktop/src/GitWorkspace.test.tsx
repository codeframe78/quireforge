import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { GitWorkspace } from "./GitWorkspace";
import {
  gitDiffSchema,
  gitMutationPreviewSchema,
  gitWorkspaceSchema,
} from "./lib/git";

const projectId = "018f0000-0000-7000-8000-000000000001";
const workspace = gitWorkspaceSchema.parse({
  schemaVersion: 1,
  state: "ready",
  projectId,
  branch: {
    head: "feature/review",
    upstream: "origin/feature/review",
    ahead: 1,
    behind: 0,
    detached: false,
  },
  changes: [
    {
      path: "src/App.tsx",
      previousPath: null,
      staged: null,
      worktree: "modified",
      conflict: false,
      submodule: false,
      reviewable: true,
    },
  ],
  truncated: false,
  diagnosticCode: null,
});
const diff = gitDiffSchema.parse({
  schemaVersion: 1,
  state: "ready",
  projectId,
  path: "src/App.tsx",
  area: "worktree",
  kind: "text",
  lines: [
    { kind: "hunk", oldLine: null, newLine: null, text: "@@ -1 +1 @@" },
    { kind: "deletion", oldLine: 1, newLine: null, text: "old" },
    { kind: "addition", oldLine: null, newLine: 1, text: "new" },
  ],
  truncated: false,
  diagnosticCode: null,
});
const mutationProps = {
  mutationPreview: null,
  mutationResult: null,
  onPreviewMutation: vi.fn().mockResolvedValue(undefined),
  onConfirmMutation: vi.fn().mockResolvedValue(undefined),
  onCancelMutation: vi.fn(),
  onRecoverMutation: vi.fn().mockResolvedValue(undefined),
};

describe("GitWorkspace", () => {
  it("reviews a normalized area and opens only the validated file", () => {
    const onReview = vi.fn().mockResolvedValue(undefined);
    const onOpen = vi.fn().mockResolvedValue(undefined);
    render(
      <GitWorkspace
        {...mutationProps}
        availability="native"
        projectName="QuireForge"
        snapshot={workspace}
        diff={diff}
        selectedRequest={{ projectId, path: "src/App.tsx", area: "worktree" }}
        busy={false}
        actionError={false}
        onRefresh={vi.fn().mockResolvedValue(undefined)}
        onReview={onReview}
        onOpen={onOpen}
      />,
    );

    expect(screen.getByText("feature/review")).toBeInTheDocument();
    expect(screen.getByText("new")).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: /Working · modified/u }),
    );
    expect(onReview).toHaveBeenCalledWith({
      projectId,
      path: "src/App.tsx",
      area: "worktree",
    });
    fireEvent.click(
      screen.getByRole("button", { name: "Open in default editor" }),
    );
    expect(onOpen).toHaveBeenCalledWith(projectId, "src/App.tsx");
  });

  it("does not simulate Git data in browser preview", () => {
    render(
      <GitWorkspace
        {...mutationProps}
        availability="preview"
        projectName="QuireForge"
        snapshot={workspace}
        diff={null}
        selectedRequest={null}
        busy={false}
        actionError={false}
        onRefresh={vi.fn().mockResolvedValue(undefined)}
        onReview={vi.fn().mockResolvedValue(undefined)}
        onOpen={vi.fn().mockResolvedValue(undefined)}
      />,
    );
    expect(
      screen.getByText(/No repository data is simulated/u),
    ).toBeInTheDocument();
    expect(screen.queryByText("feature/review")).not.toBeInTheDocument();
  });

  it("previews a fixed file operation before any mutation", () => {
    const onPreviewMutation = vi.fn().mockResolvedValue(undefined);
    render(
      <GitWorkspace
        {...mutationProps}
        availability="native"
        projectName="QuireForge"
        snapshot={workspace}
        diff={null}
        selectedRequest={null}
        mutationPreview={null}
        busy={false}
        actionError={false}
        onRefresh={vi.fn().mockResolvedValue(undefined)}
        onReview={vi.fn().mockResolvedValue(undefined)}
        onOpen={vi.fn().mockResolvedValue(undefined)}
        onPreviewMutation={onPreviewMutation}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Stage" }));
    expect(onPreviewMutation).toHaveBeenCalledWith({
      projectId,
      operation: "stage",
      path: "src/App.tsx",
      message: null,
    });
    expect(onPreviewMutation).toHaveBeenCalledTimes(1);
  });

  it("confirms only the opaque native token and labels destructive review", () => {
    const confirmationId = "018f0000-0000-7000-8000-000000000002";
    const onConfirmMutation = vi.fn().mockResolvedValue(undefined);
    const onCancelMutation = vi.fn();
    const mutationPreview = gitMutationPreviewSchema.parse({
      schemaVersion: 1,
      state: "ready",
      projectId,
      operation: "revert",
      path: "src/App.tsx",
      targets: [
        {
          path: "src/App.tsx",
          staged: null,
          worktree: "modified",
        },
      ],
      destructive: true,
      confirmationId,
      secretFindings: [],
      diagnosticCode: null,
    });
    render(
      <GitWorkspace
        {...mutationProps}
        availability="native"
        projectName="QuireForge"
        snapshot={workspace}
        diff={null}
        selectedRequest={null}
        mutationPreview={mutationPreview}
        busy={false}
        actionError={false}
        onRefresh={vi.fn().mockResolvedValue(undefined)}
        onReview={vi.fn().mockResolvedValue(undefined)}
        onOpen={vi.fn().mockResolvedValue(undefined)}
        onConfirmMutation={onConfirmMutation}
        onCancelMutation={onCancelMutation}
      />,
    );

    expect(screen.getByRole("alertdialog")).toHaveTextContent(
      "A bounded, one-time recovery is offered",
    );
    fireEvent.click(screen.getByRole("button", { name: "Confirm revert" }));
    expect(onConfirmMutation).toHaveBeenCalledWith(confirmationId);
    expect(onConfirmMutation).toHaveBeenCalledTimes(1);
    fireEvent.keyDown(screen.getByRole("alertdialog"), { key: "Escape" });
    expect(onCancelMutation).toHaveBeenCalledTimes(1);
  });
});
