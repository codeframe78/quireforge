import fixture from "../../fixtures/project-workspace.json";
import { describe, expect, it } from "vitest";

import {
  projectPreflightSchema,
  projectWorkspaceSchema,
  scaffoldProjectWorkspace,
} from "./project";

const projectId = "018f0000-0000-7000-8000-000000000001";
const associationId = "018f0000-0000-7000-8000-000000000002";

describe("project workspace contract", () => {
  it("parses the shared normalized empty fixture", () => {
    expect(projectWorkspaceSchema.parse(fixture)).toEqual(
      scaffoldProjectWorkspace,
    );
  });

  it("accepts a bounded attached project and ready preflight", () => {
    const workspace = projectWorkspaceSchema.parse({
      ...fixture,
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
    });
    const preflight = projectPreflightSchema.parse({
      schemaVersion: 1,
      projectId,
      cwdReady: true,
      displayPath: "~/work/quireforge",
      state: "connected-accessible",
      diagnosticCode: null,
    });

    expect(workspace.projects[0]?.directory?.git.isRepository).toBe(true);
    expect(preflight.cwdReady).toBe(true);
  });

  it("rejects raw paths, credentials, duplicate IDs, and inconsistent states", () => {
    expect(() =>
      projectWorkspaceSchema.parse({
        ...fixture,
        selectedPath: "/private/raw/path",
      }),
    ).toThrow();
    expect(() =>
      projectWorkspaceSchema.parse({
        ...fixture,
        token: "not-allowed",
      }),
    ).toThrow();
    expect(() =>
      projectWorkspaceSchema.parse({ ...fixture, state: "ready" }),
    ).toThrow();
    expect(() =>
      projectPreflightSchema.parse({
        schemaVersion: 1,
        projectId,
        cwdReady: true,
        displayPath: null,
        state: "connected-read-only",
        diagnosticCode: null,
      }),
    ).toThrow();

    const valid = projectWorkspaceSchema.parse({
      ...fixture,
      state: "ready",
      projects: [
        {
          id: projectId,
          displayName: "QuireForge",
          archived: false,
          directory: null,
        },
      ],
    });
    expect(() =>
      projectWorkspaceSchema.parse({
        ...valid,
        projects: [valid.projects[0], valid.projects[0]],
      }),
    ).toThrow();
  });

  it("requires relink previews to carry only an opaque project ID", () => {
    expect(() =>
      projectWorkspaceSchema.parse({
        ...fixture,
        pendingAttachment: {
          operation: "relink",
          projectId: null,
          displayName: "Moved project",
          selectedDisplayPath: "/mnt/moved",
          resolvedDisplayPath: "/mnt/moved",
          state: "connected-accessible",
          git: { isRepository: false, isLinkedWorktree: false },
          hasAgentsGuidance: false,
          hasCodexConfig: false,
        },
      }),
    ).toThrow();

    expect(associationId).not.toBe(projectId);
  });
});
