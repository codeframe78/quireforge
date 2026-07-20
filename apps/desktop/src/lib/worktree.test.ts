import { describe, expect, it } from "vitest";

import {
  scaffoldWorktreeWorkspace,
  worktreePreviewSchema,
  worktreeResultSchema,
  worktreeWorkspaceSchema,
} from "./worktree";

const sourceProjectId = "018f0000-0000-7000-8000-000000000001";
const linkedProjectId = "018f0000-0000-7000-8000-000000000002";

describe("worktree contract", () => {
  it("keeps the browser fixture honest and empty", () => {
    expect(scaffoldWorktreeWorkspace).toEqual({
      schemaVersion: 1,
      state: "empty",
      sourceProjectId: null,
      worktrees: [],
      truncated: false,
      diagnosticCode: null,
    });
  });

  it("validates a normalized inventory without Git object IDs", () => {
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
          projectId: linkedProjectId,
          displayName: "feature/worktrees",
          displayPath: "~/.local/share/quireforge/worktrees/managed",
          branchName: "feature/worktrees",
          ownership: "managed",
          state: "ready",
          current: false,
        },
      ],
      truncated: false,
      diagnosticCode: null,
    });

    expect(workspace.worktrees).toHaveLength(2);
    expect(JSON.stringify(workspace)).not.toMatch(
      /object|commit|HEAD [0-9a-f]/u,
    );
  });

  it("rejects raw execution controls and inconsistent external entries", () => {
    expect(() =>
      worktreeWorkspaceSchema.parse({
        ...scaffoldWorktreeWorkspace,
        cwd: "/tmp/private",
      }),
    ).toThrow();
    expect(() =>
      worktreeWorkspaceSchema.parse({
        schemaVersion: 1,
        state: "ready",
        sourceProjectId,
        worktrees: [
          {
            projectId: sourceProjectId,
            displayName: "External",
            displayPath: "/tmp/external",
            branchName: "feature/external",
            ownership: "external",
            state: "ready",
            current: false,
          },
        ],
        truncated: false,
        diagnosticCode: null,
      }),
    ).toThrow();
  });

  it("requires a one-use confirmation shape for every ready preview", () => {
    const preview = worktreePreviewSchema.parse({
      schemaVersion: 1,
      state: "ready",
      sourceProjectId,
      operation: "create",
      branchName: "feature/isolated",
      displayPath: "~/.local/share/quireforge/worktrees/isolated",
      ownership: "managed",
      destructive: false,
      confirmationId: linkedProjectId,
      diagnosticCode: null,
    });
    expect(preview.confirmationId).toBe(linkedProjectId);
    expect(() =>
      worktreePreviewSchema.parse({ ...preview, confirmationId: null }),
    ).toThrow();
  });

  it("only exposes a recovery path when a created worktree remains", () => {
    expect(() =>
      worktreeResultSchema.parse({
        schemaVersion: 1,
        state: "unavailable",
        sourceProjectId,
        projectId: null,
        workspace: null,
        recoverableDisplayPath: "/tmp/orphan",
        diagnosticCode: "git-failed",
      }),
    ).toThrow();
    expect(
      worktreeResultSchema.parse({
        schemaVersion: 1,
        state: "unavailable",
        sourceProjectId,
        projectId: null,
        workspace: null,
        recoverableDisplayPath: "~/.local/share/quireforge/worktrees/orphan",
        diagnosticCode: "worktree-remains",
      }).diagnosticCode,
    ).toBe("worktree-remains");
  });
});
