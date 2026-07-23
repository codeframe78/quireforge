import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { SessionWorkspace } from "./SessionWorkspace";
import { scaffoldConversationAttachments } from "./lib/attachment";
import { scaffoldCodexRuntime } from "./lib/codex";
import {
  conversationSnapshotSchema,
  scaffoldConversation,
} from "./lib/conversation";
import { projectWorkspaceSchema } from "./lib/project";
import { sessionLifecycleSchema } from "./lib/session";

const projectId = "018f0000-0000-7000-8000-000000000001";
const parentId = "018f0000-0000-7000-8000-000000000010";
const forkId = "018f0000-0000-7000-8000-000000000011";
const modelSelection = {
  schemaVersion: 1 as const,
  availability: "ready" as const,
  effective: {
    modelId: "gpt-5.6-sol",
    reasoningEffort: "high",
  },
  pending: null,
  policy: {
    ownership: "manual" as const,
    userLocked: false,
    allowedModelIds: [],
    reasoningCeiling: null,
  },
  diagnosticCode: null,
};

const projects = projectWorkspaceSchema.parse({
  schemaVersion: 1,
  state: "ready",
  projects: [
    {
      id: projectId,
      displayName: "QuireForge",
      archived: false,
      directory: {
        associationId: "018f0000-0000-7000-8000-000000000002",
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
}).projects;

function session(
  conversationId: string,
  title: string | null,
  state: "completed" | "interrupted" | "archived" | "missing",
  parentConversationId: string | null = null,
) {
  return {
    conversationId,
    projectId,
    parentConversationId,
    title,
    modelId: "gpt-5.6-sol",
    reasoningEffort: "high",
    modelSelection,
    sandboxMode: "workspace-write" as const,
    approvalPolicy: "on-request" as const,
    state,
    createdAtMs: 1_700_000_000_000,
    updatedAtMs: 1_700_000_001_000,
  };
}

const readySnapshot = sessionLifecycleSchema.parse({
  schemaVersion: 3,
  state: "ready",
  sessions: [
    session(parentId, "Review lifecycle boundaries", "completed"),
    session(forkId, "Try the smaller adapter", "interrupted", parentId),
  ],
  diagnosticCode: null,
});

function renderWorkspace(
  overrides: Partial<React.ComponentProps<typeof SessionWorkspace>> = {},
) {
  const running = conversationSnapshotSchema.parse({
    ...scaffoldConversation,
    state: "running",
    conversationId: parentId,
    projectId,
    modelId: "gpt-5.6-sol",
    reasoningEffort: "high",
    modelSelection,
    sandboxMode: "workspace-write",
    approvalPolicy: "on-request",
    events: [],
  });
  const props: React.ComponentProps<typeof SessionWorkspace> = {
    availability: "native",
    snapshot: readySnapshot,
    runtime: scaffoldCodexRuntime,
    projects,
    activeConversationId: null,
    attachments: scaffoldConversationAttachments,
    busy: false,
    attachmentBusy: false,
    actionError: false,
    attachmentActionError: false,
    searchTerm: null,
    onSearch: vi.fn().mockResolvedValue(undefined),
    onRefresh: vi.fn().mockResolvedValue(undefined),
    onSelect: vi.fn(),
    onResume: vi.fn().mockResolvedValue(running),
    onFork: vi.fn().mockResolvedValue(running),
    onArchive: vi.fn().mockResolvedValue(undefined),
    onRestore: vi.fn().mockResolvedValue(undefined),
    onUpdateModelSelection: vi.fn().mockResolvedValue(modelSelection),
    onAttachmentPick: vi.fn().mockResolvedValue(undefined),
    onAttachmentDrop: vi.fn().mockResolvedValue(undefined),
    onAttachmentCancel: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };
  render(<SessionWorkspace {...props} />);
  return props;
}

describe("SessionWorkspace", () => {
  it("searches bounded titles and presents project and fork grouping", async () => {
    const props = renderWorkspace();

    expect(
      screen.getByRole("heading", { name: "QuireForge" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/Fork of Review lifecycle boundaries/u),
    ).toBeInTheDocument();
    fireEvent.change(screen.getByLabelText("Search session titles"), {
      target: { value: "  lifecycle  " },
    });
    fireEvent.click(screen.getByRole("button", { name: "Search" }));

    await waitFor(() =>
      expect(props.onSearch).toHaveBeenCalledWith({
        projectId: null,
        searchTerm: "lifecycle",
      }),
    );
  });

  it("opens keyboard-accessible tabs and resumes or forks only the app ID", async () => {
    const props = renderWorkspace();

    fireEvent.click(
      screen.getByText("Review lifecycle boundaries").closest("button")!,
    );
    const firstTab = screen.getByRole("tab", {
      name: "Review lifecycle boundaries",
    });
    expect(firstTab).toHaveAttribute("aria-selected", "true");

    fireEvent.click(
      screen.getByText("Try the smaller adapter").closest("button")!,
    );
    const secondTab = screen.getByRole("tab", {
      name: "Try the smaller adapter",
    });
    fireEvent.keyDown(secondTab, { key: "ArrowLeft" });
    expect(firstTab).toHaveFocus();

    fireEvent.change(screen.getByLabelText("Next task"), {
      target: { value: "Continue with the verified reference." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Resume" }));
    await waitFor(() =>
      expect(props.onResume).toHaveBeenCalledWith({
        conversationId: parentId,
        prompt: "Continue with the verified reference.",
        attachmentIds: [],
      }),
    );

    fireEvent.change(screen.getByLabelText("Next task"), {
      target: { value: "Try a separate safe approach." },
    });
    fireEvent.click(screen.getByRole("button", { name: "Fork" }));
    await waitFor(() =>
      expect(props.onFork).toHaveBeenCalledWith({
        conversationId: parentId,
        prompt: "Try a separate safe approach.",
        attachmentIds: [],
      }),
    );
  });

  it("keeps archive, restore, and missing-session behavior distinct", async () => {
    const archived = sessionLifecycleSchema.parse({
      schemaVersion: 3,
      state: "ready",
      sessions: [session(parentId, "Archived review", "archived")],
      diagnosticCode: null,
    });
    const archivedProps = renderWorkspace({ snapshot: archived });
    fireEvent.click(screen.getByRole("button", { name: /Archived review/u }));
    fireEvent.click(screen.getByRole("button", { name: "Restore" }));
    await waitFor(() =>
      expect(archivedProps.onRestore).toHaveBeenCalledWith(parentId),
    );
    expect(
      screen.queryByRole("button", { name: "Resume" }),
    ).not.toBeInTheDocument();

    const missing = sessionLifecycleSchema.parse({
      schemaVersion: 3,
      state: "ready",
      sessions: [session(forkId, null, "missing")],
      diagnosticCode: null,
    });
    renderWorkspace({ snapshot: missing });
    fireEvent.click(screen.getByRole("button", { name: /Untitled session/u }));
    expect(
      screen.getByText(/No substitute session will be opened/u),
    ).toBeInTheDocument();
  });

  it("is honest and non-interactive in browser preview", () => {
    renderWorkspace({ availability: "preview" });

    expect(
      screen.getByText(
        "Browser preview cannot inspect or simulate native session history.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Search" })).toBeDisabled();
    expect(
      screen.queryByRole("button", { name: "Resume" }),
    ).not.toBeInTheDocument();
  });
});
