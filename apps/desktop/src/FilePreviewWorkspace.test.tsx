import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { FilePreviewWorkspace } from "./FilePreviewWorkspace";
import {
  scaffoldFilePreview,
  sharedFilePreviewFixture,
} from "./lib/filePreview";
import type { ProjectWorkspaceSnapshot } from "./lib/project";

const project: ProjectWorkspaceSnapshot["projects"][number] = {
  id: "018f6f24-8b71-7c72-9b41-4e0b8ce4c61a",
  displayName: "Preview fixture",
  archived: false,
  directory: {
    associationId: "018f6f24-8b71-7c72-9b41-4e0b8ce4c61b",
    displayPath: "/workspace/preview-fixture",
    resolvedDisplayPath: "/workspace/preview-fixture",
    state: "connected-accessible",
    expectedAccess: "read-write",
    isPrimary: true,
    git: { isRepository: true, isLinkedWorktree: false },
    hasAgentsGuidance: false,
    hasCodexConfig: false,
  },
};

describe("FilePreviewWorkspace", () => {
  it("renders bounded normalized text and invokes the native picker", () => {
    const onPick = vi.fn().mockResolvedValue(undefined);
    render(
      <FilePreviewWorkspace
        availability="native"
        project={project}
        snapshot={sharedFilePreviewFixture}
        busy={false}
        actionError={false}
        onPick={onPick}
        onOpen={vi.fn()}
        onClear={vi.fn()}
      />,
    );

    expect(screen.getByText("docs/preview.md")).toBeInTheDocument();
    expect(
      screen
        .getByRole("article", { name: "Preview of docs/preview.md" })
        .querySelector("code")?.textContent,
    ).toBe("# Safe preview\n\nPaths remain native-only.\n");
    fireEvent.click(
      screen.getByRole("button", { name: "Choose project file" }),
    );
    expect(onPick).toHaveBeenCalledWith(project.id);
  });

  it("reviews the visible default-application destination before handoff", async () => {
    const onOpen = vi.fn().mockResolvedValue(undefined);
    render(
      <FilePreviewWorkspace
        availability="native"
        project={project}
        snapshot={sharedFilePreviewFixture}
        busy={false}
        actionError={false}
        onPick={vi.fn()}
        onOpen={onOpen}
        onClear={vi.fn()}
      />,
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Open with desktop app" }),
    );
    expect(
      screen.getByText(/Destination · System default application/u),
    ).toBeInTheDocument();
    expect(onOpen).not.toHaveBeenCalled();

    fireEvent.click(
      screen.getByRole("button", { name: "Open with default app" }),
    );
    await waitFor(() =>
      expect(onOpen).toHaveBeenCalledWith({
        openActionId: sharedFilePreviewFixture.openActionId,
      }),
    );
    expect(
      screen.getByRole("button", { name: "Opened with desktop app" }),
    ).toBeDisabled();
  });

  it("keeps native file selection unavailable in browser preview", () => {
    render(
      <FilePreviewWorkspace
        availability="preview"
        project={project}
        snapshot={scaffoldFilePreview}
        busy={false}
        actionError={false}
        onPick={vi.fn()}
        onOpen={vi.fn()}
        onClear={vi.fn()}
      />,
    );

    expect(
      screen.getByRole("button", { name: "Choose project file" }),
    ).toBeDisabled();
    expect(
      screen.getByText(/cannot select or read local/u),
    ).toBeInTheDocument();
  });

  it("explains why PDF bytes are not rendered", () => {
    render(
      <FilePreviewWorkspace
        availability="native"
        project={project}
        snapshot={{
          ...sharedFilePreviewFixture,
          displayPath: "review.pdf",
          kind: "pdf",
          rendering: "metadata-only",
          mimeType: "application/pdf",
          byteSize: 1024,
          textContent: null,
        }}
        busy={false}
        actionError={false}
        onPick={vi.fn()}
        onOpen={vi.fn()}
        onClear={vi.fn()}
      />,
    );

    expect(
      screen.getByText(/active document rendering is disabled/u),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/not embedded in the privileged webview/u),
    ).toBeInTheDocument();
  });

  it("hides retained content when its project is no longer accessible", () => {
    render(
      <FilePreviewWorkspace
        availability="native"
        project={{
          ...project,
          directory: project.directory
            ? { ...project.directory, state: "identity-changed" }
            : null,
        }}
        snapshot={sharedFilePreviewFixture}
        busy={false}
        actionError={false}
        onPick={vi.fn()}
        onOpen={vi.fn()}
        onClear={vi.fn()}
      />,
    );

    expect(screen.queryByText("docs/preview.md")).not.toBeInTheDocument();
    expect(
      screen.getByText(/attach or relink an accessible project/iu),
    ).toBeInTheDocument();
  });
});
