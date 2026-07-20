import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { GitWorkspace } from "./GitWorkspace";
import { gitDiffSchema, gitWorkspaceSchema } from "./lib/git";

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

describe("GitWorkspace", () => {
  it("reviews a normalized area and opens only the validated file", () => {
    const onReview = vi.fn().mockResolvedValue(undefined);
    const onOpen = vi.fn().mockResolvedValue(undefined);
    render(
      <GitWorkspace
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
});
