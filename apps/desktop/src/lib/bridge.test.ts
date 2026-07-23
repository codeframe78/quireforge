import { describe, expect, it, vi } from "vitest";

import { scaffoldCodexAuth } from "./auth";
import { scaffoldCodexUsage } from "./usage";
import { sharedConversationAttachmentFixture } from "./attachment";
import { scaffoldCodexRuntime } from "./codex";
import { scaffoldConversation } from "./conversation";
import {
  scaffoldGitDiff,
  scaffoldGitMutationPreview,
  scaffoldGitMutationResult,
  scaffoldGitWorkspace,
} from "./git";
import { scaffoldSessionLifecycle } from "./session";
import {
  archiveConversation,
  archiveProject,
  cancelFilePreview,
  cancelConversationAttachments,
  cancelProjectAttachment,
  cancelWorktree,
  cancelCodexAuth,
  CODEX_AUTH_CANCEL_COMMAND,
  CODEX_AUTH_OPEN_BROWSER_COMMAND,
  CODEX_AUTH_START_COMMAND,
  CODEX_AUTH_STATUS_COMMAND,
  CODEX_RUNTIME_PROBE_COMMAND,
  confirmProjectAttachment,
  confirmWorktree,
  confirmGitMutation,
  confirmIntegrationControl,
  confirmIntegrationMutation,
  decideConversationApproval,
  CONVERSATION_APPROVAL_DECIDE_COMMAND,
  CONVERSATION_ATTACHMENT_CANCEL_COMMAND,
  CONVERSATION_ATTACHMENT_PICK_COMMAND,
  CONVERSATION_ATTACHMENT_STAGE_DROP_COMMAND,
  CONVERSATION_ATTACHMENT_STAGE_NATIVE_DROP_COMMAND,
  CONVERSATION_ATTACHMENT_STATUS_COMMAND,
  CONVERSATION_ACTIVE_COMMAND,
  CONVERSATION_INTERRUPT_COMMAND,
  CONVERSATION_NOTIFY_COMMAND,
  CONVERSATION_ARCHIVE_COMMAND,
  CONVERSATION_FORK_COMMAND,
  CONVERSATION_POLL_COMMAND,
  CONVERSATION_RESTORE_COMMAND,
  CONVERSATION_RESUME_COMMAND,
  CONVERSATION_SESSIONS_COMMAND,
  CONVERSATION_START_COMMAND,
  CONVERSATION_STATUS_COMMAND,
  DESKTOP_BOOTSTRAP_COMMAND,
  detachProject,
  forkConversation,
  FILE_PREVIEW_CANCEL_COMMAND,
  FILE_PREVIEW_OPEN_COMMAND,
  FILE_PREVIEW_PICK_COMMAND,
  loadCodexAuth,
  loadCodexUsage,
  loadCodexRuntime,
  loadConversationStatus,
  loadActiveConversations,
  loadConversationSessions,
  loadConversationAttachments,
  loadDesktopBootstrap,
  loadGitDiff,
  loadGitStatus,
  loadProjectWorkspace,
  loadWorktreeStatus,
  notifyConversation,
  openCodexAuthBrowser,
  refreshCodexUsage,
  openFilePreview,
  openGitFile,
  pickProjectDirectory,
  pickConversationAttachments,
  pickFilePreview,
  pickProjectRelink,
  pickWorktreeAttach,
  preflightProject,
  previewGitMutation,
  previewWorktreeCreate,
  previewWorktreeRecover,
  previewWorktreeRemove,
  pollConversation,
  restoreConversation,
  resumeConversation,
  PROJECT_ARCHIVE_COMMAND,
  PROJECT_CANCEL_ATTACHMENT_COMMAND,
  PROJECT_CONFIRM_ATTACHMENT_COMMAND,
  PROJECT_DETACH_COMMAND,
  PROJECT_PICK_DIRECTORY_COMMAND,
  PROJECT_PICK_RELINK_COMMAND,
  PROJECT_PREFLIGHT_COMMAND,
  PROJECT_WORKSPACE_STATUS_COMMAND,
  WORKTREE_CANCEL_COMMAND,
  WORKTREE_CONFIRM_COMMAND,
  WORKTREE_CREATE_PREVIEW_COMMAND,
  WORKTREE_RECOVER_PREVIEW_COMMAND,
  WORKTREE_REMOVE_PREVIEW_COMMAND,
  WORKTREE_PICK_ATTACH_COMMAND,
  WORKTREE_STATUS_COMMAND,
  GIT_DIFF_COMMAND,
  GIT_OPEN_FILE_COMMAND,
  GIT_MUTATION_CONFIRM_COMMAND,
  GIT_MUTATION_PREVIEW_COMMAND,
  GIT_MUTATION_RECOVER_COMMAND,
  GIT_STATUS_COMMAND,
  recoverGitMutation,
  startCodexAuth,
  startConversation,
  stageDroppedConversationAttachments,
  interruptConversation,
  INTEGRATION_CATALOG_READ_COMMAND,
  INTEGRATION_CATALOG_REFRESH_COMMAND,
  INTEGRATION_CONTROL_CONFIRM_COMMAND,
  INTEGRATION_CONTROL_OPEN_BROWSER_COMMAND,
  INTEGRATION_CONTROL_PREVIEW_COMMAND,
  INTEGRATION_CONTROL_STATUS_COMMAND,
  INTEGRATION_MUTATION_CONFIRM_COMMAND,
  INTEGRATION_MUTATION_PREVIEW_COMMAND,
  MODEL_SELECTION_UPDATE_COMMAND,
  loadIntegrationCatalog,
  openIntegrationControlBrowser,
  pollIntegrationControl,
  previewIntegrationControl,
  previewIntegrationMutation,
  refreshIntegrationCatalog,
  updateModelSelection,
} from "./bridge";
import { scaffoldBootstrap } from "./contract";
import { sharedFilePreviewFixture } from "./filePreview";
import { scaffoldProjectWorkspace } from "./project";
import {
  scaffoldIntegrationCatalog,
  scaffoldIntegrationControlPreview,
  scaffoldIntegrationControlResult,
  scaffoldIntegrationMutationPreview,
  scaffoldIntegrationMutationResult,
} from "./integration";
import {
  scaffoldWorktreeWorkspace,
  worktreePreviewSchema,
  worktreeResultSchema,
} from "./worktree";

describe("desktop bridge", () => {
  it("invokes the one typed bootstrap command", async () => {
    const invokeFunction = vi.fn().mockResolvedValue(scaffoldBootstrap);

    await expect(loadDesktopBootstrap(invokeFunction)).resolves.toEqual(
      scaffoldBootstrap,
    );
    expect(invokeFunction).toHaveBeenCalledWith(DESKTOP_BOOTSTRAP_COMMAND);
  });

  it("does not pass malformed native data into the UI", async () => {
    const invokeFunction = vi.fn().mockResolvedValue({ schemaVersion: 1 });

    await expect(loadDesktopBootstrap(invokeFunction)).rejects.toThrow();
  });

  it("invokes and validates the normalized Codex runtime probe", async () => {
    const invoke = vi.fn().mockResolvedValue(scaffoldCodexRuntime);

    await expect(loadCodexRuntime(invoke)).resolves.toEqual(
      scaffoldCodexRuntime,
    );
    expect(invoke).toHaveBeenCalledWith(CODEX_RUNTIME_PROBE_COMMAND);
  });

  it("loads only the strict normalized integration catalog", async () => {
    const invoke = vi.fn().mockResolvedValue(scaffoldIntegrationCatalog);

    await expect(loadIntegrationCatalog(invoke)).resolves.toEqual(
      scaffoldIntegrationCatalog,
    );
    expect(invoke).toHaveBeenCalledWith(INTEGRATION_CATALOG_READ_COMMAND);

    invoke.mockResolvedValueOnce({
      ...scaffoldIntegrationCatalog,
      rawProtocolPayload: { accountId: "private" },
    });
    await expect(loadIntegrationCatalog(invoke)).rejects.toThrow();
  });

  it("uses fixed preview-confirm commands for integration mutations", async () => {
    const previewRequest = {
      operation: "plugin-install" as const,
      targetEntryId: "plugin:fixture-review",
      repository: null,
      reference: null,
    };
    const confirmationRequest = {
      confirmationId: "018f0000-0000-7000-8000-000000000014",
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(scaffoldIntegrationMutationPreview)
      .mockResolvedValueOnce(scaffoldIntegrationMutationResult);

    await expect(
      previewIntegrationMutation(previewRequest, invoke),
    ).resolves.toEqual(scaffoldIntegrationMutationPreview);
    await expect(
      confirmIntegrationMutation(confirmationRequest, invoke),
    ).resolves.toEqual(scaffoldIntegrationMutationResult);
    expect(invoke).toHaveBeenNthCalledWith(
      1,
      INTEGRATION_MUTATION_PREVIEW_COMMAND,
      { request: previewRequest },
    );
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      INTEGRATION_MUTATION_CONFIRM_COMMAND,
      { request: confirmationRequest },
    );
  });

  it("uses fixed opaque commands for confirmed integration controls", async () => {
    const previewRequest = {
      operation: "mcp-authorize" as const,
      targetEntryId: "mcp:fixture-knowledge",
    };
    const confirmationRequest = {
      confirmationId: scaffoldIntegrationControlPreview.confirmationId!,
    };
    const actionRequest = {
      actionId: scaffoldIntegrationControlResult.actionId!,
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(scaffoldIntegrationCatalog)
      .mockResolvedValueOnce(scaffoldIntegrationControlPreview)
      .mockResolvedValueOnce(scaffoldIntegrationControlResult)
      .mockResolvedValueOnce({
        ...scaffoldIntegrationControlResult,
        state: "pending",
        browserHandoffAvailable: false,
      })
      .mockResolvedValueOnce(scaffoldIntegrationControlResult);

    await refreshIntegrationCatalog(invoke);
    await previewIntegrationControl(previewRequest, invoke);
    await confirmIntegrationControl(confirmationRequest, invoke);
    await openIntegrationControlBrowser(actionRequest, invoke);
    await pollIntegrationControl(actionRequest, invoke);

    expect(invoke.mock.calls).toEqual([
      [INTEGRATION_CATALOG_REFRESH_COMMAND],
      [INTEGRATION_CONTROL_PREVIEW_COMMAND, { request: previewRequest }],
      [INTEGRATION_CONTROL_CONFIRM_COMMAND, { request: confirmationRequest }],
      [INTEGRATION_CONTROL_OPEN_BROWSER_COMMAND, { request: actionRequest }],
      [INTEGRATION_CONTROL_STATUS_COMMAND, { request: actionRequest }],
    ]);
  });

  it("rejects raw integration source data at the bridge boundary", async () => {
    const invoke = vi.fn().mockResolvedValue({
      ...scaffoldIntegrationMutationPreview,
      sourcePath: "/private/plugin/source",
    });

    await expect(
      previewIntegrationMutation(
        {
          operation: "plugin-install",
          targetEntryId: "plugin:fixture-review",
          repository: null,
          reference: null,
        },
        invoke,
      ),
    ).rejects.toThrow();

    const rejected = vi.fn();
    await expect(
      previewIntegrationMutation(
        {
          operation: "marketplace-add",
          targetEntryId: null,
          repository: "https://user:secret@example.invalid/repo",
          reference: "a".repeat(40),
        },
        rejected,
      ),
    ).rejects.toThrow();
    expect(rejected).not.toHaveBeenCalled();
  });

  it("uses fixed typed authentication commands", async () => {
    const invoke = vi.fn().mockResolvedValue(scaffoldCodexAuth);

    await expect(loadCodexAuth(invoke)).resolves.toEqual(scaffoldCodexAuth);
    await expect(startCodexAuth("device-code", invoke)).resolves.toEqual(
      scaffoldCodexAuth,
    );
    await expect(cancelCodexAuth(invoke)).resolves.toEqual(scaffoldCodexAuth);
    await openCodexAuthBrowser(invoke);

    expect(invoke).toHaveBeenNthCalledWith(1, CODEX_AUTH_STATUS_COMMAND);
    expect(invoke).toHaveBeenNthCalledWith(2, CODEX_AUTH_START_COMMAND, {
      method: "device-code",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, CODEX_AUTH_CANCEL_COMMAND);
    expect(invoke).toHaveBeenNthCalledWith(4, CODEX_AUTH_OPEN_BROWSER_COMMAND);
  });

  it("uses fixed read-only usage commands", async () => {
    const invoke = vi.fn().mockResolvedValue(scaffoldCodexUsage);

    await expect(loadCodexUsage(invoke)).resolves.toEqual(scaffoldCodexUsage);
    await expect(refreshCodexUsage(invoke)).resolves.toEqual(
      scaffoldCodexUsage,
    );
    expect(invoke).toHaveBeenNthCalledWith(1, "codex_usage_status");
    expect(invoke).toHaveBeenNthCalledWith(2, "codex_usage_refresh");
  });

  it("rejects raw authentication payloads", async () => {
    const invoke = vi.fn().mockResolvedValue({
      ...scaffoldCodexAuth,
      accountId: "private",
    });

    await expect(loadCodexAuth(invoke)).rejects.toThrow();
  });

  it("uses fixed project commands and passes only opaque IDs", async () => {
    const projectId = "018f0000-0000-7000-8000-000000000001";
    const preflight = {
      schemaVersion: 1,
      projectId,
      cwdReady: false,
      displayPath: null,
      state: "missing-or-moved",
      diagnosticCode: null,
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(scaffoldProjectWorkspace)
      .mockResolvedValueOnce(scaffoldProjectWorkspace)
      .mockResolvedValueOnce(scaffoldProjectWorkspace)
      .mockResolvedValueOnce(scaffoldProjectWorkspace)
      .mockResolvedValueOnce(scaffoldProjectWorkspace)
      .mockResolvedValueOnce(scaffoldProjectWorkspace)
      .mockResolvedValueOnce(scaffoldProjectWorkspace)
      .mockResolvedValueOnce(preflight);

    await loadProjectWorkspace(invoke);
    await pickProjectDirectory(invoke);
    await pickProjectRelink(projectId, invoke);
    await confirmProjectAttachment(invoke);
    await cancelProjectAttachment(invoke);
    await detachProject(projectId, invoke);
    await archiveProject(projectId, invoke);
    await expect(preflightProject(projectId, invoke)).resolves.toEqual(
      preflight,
    );

    expect(invoke.mock.calls).toEqual([
      [PROJECT_WORKSPACE_STATUS_COMMAND, undefined],
      [PROJECT_PICK_DIRECTORY_COMMAND, undefined],
      [PROJECT_PICK_RELINK_COMMAND, { projectId }],
      [PROJECT_CONFIRM_ATTACHMENT_COMMAND, undefined],
      [PROJECT_CANCEL_ATTACHMENT_COMMAND, undefined],
      [PROJECT_DETACH_COMMAND, { projectId }],
      [PROJECT_ARCHIVE_COMMAND, { projectId }],
      [PROJECT_PREFLIGHT_COMMAND, { projectId }],
    ]);
  });

  it("rejects project snapshots containing unreviewed fields", async () => {
    const invoke = vi.fn().mockResolvedValue({
      ...scaffoldProjectWorkspace,
      selectedPath: "/private/raw/path",
    });

    await expect(loadProjectWorkspace(invoke)).rejects.toThrow();
  });

  it("uses one fixed file picker command with only an opaque project ID", async () => {
    const projectId = "018f6f24-8b71-7c72-9b41-4e0b8ce4c61a";
    const invoke = vi.fn().mockResolvedValue(sharedFilePreviewFixture);

    await expect(pickFilePreview(projectId, invoke)).resolves.toEqual(
      sharedFilePreviewFixture,
    );
    expect(invoke).toHaveBeenCalledWith(FILE_PREVIEW_PICK_COMMAND, {
      projectId,
    });

    await expect(pickFilePreview("/private/path", invoke)).rejects.toThrow();
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("opens and cancels previews through one opaque handoff action", async () => {
    const openActionId = sharedFilePreviewFixture.openActionId!;
    const invoke = vi.fn().mockResolvedValue(true);

    await expect(
      openFilePreview({ openActionId }, invoke),
    ).resolves.toBeUndefined();
    expect(invoke).toHaveBeenNthCalledWith(1, FILE_PREVIEW_OPEN_COMMAND, {
      request: { openActionId },
    });
    await expect(cancelFilePreview({ openActionId }, invoke)).resolves.toBe(
      true,
    );
    expect(invoke).toHaveBeenNthCalledWith(2, FILE_PREVIEW_CANCEL_COMMAND, {
      request: { openActionId },
    });

    await expect(
      openFilePreview({ openActionId: "/private/file" }, invoke),
    ).rejects.toThrow();
    expect(invoke).toHaveBeenCalledTimes(2);
  });

  it("requests only a privacy-safe native notification by conversation ID", async () => {
    const conversationId = "018f0000-0000-7000-8000-000000000010";
    const invoke = vi.fn().mockResolvedValue({
      schemaVersion: 1,
      status: "sent",
    });

    await expect(notifyConversation(conversationId, invoke)).resolves.toEqual({
      schemaVersion: 1,
      status: "sent",
    });
    expect(invoke).toHaveBeenCalledWith(CONVERSATION_NOTIFY_COMMAND, {
      request: { conversationId },
    });
    await expect(notifyConversation("raw-thread-id", invoke)).rejects.toThrow();
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("stages and cancels conversation images through fixed opaque commands", async () => {
    const projectId = "018f6f24-8b71-7c72-9b41-4e0b8ce4c61a";
    const attachmentId =
      sharedConversationAttachmentFixture.attachments[0]!.attachmentId;
    const dropRequest = {
      projectId,
      files: [
        {
          displayName: "pixel.png",
          declaredMimeType: "image/png" as const,
          base64Data: "iVBORw==",
        },
      ],
    };
    const cancelRequest = { projectId, attachmentIds: [attachmentId] };
    const invoke = vi
      .fn()
      .mockResolvedValue(sharedConversationAttachmentFixture);

    await expect(
      loadConversationAttachments(projectId, invoke),
    ).resolves.toEqual(sharedConversationAttachmentFixture);
    await expect(
      pickConversationAttachments(projectId, invoke),
    ).resolves.toEqual(sharedConversationAttachmentFixture);
    await expect(
      stageDroppedConversationAttachments(dropRequest, invoke),
    ).resolves.toEqual(sharedConversationAttachmentFixture);
    await expect(
      stageDroppedConversationAttachments({ projectId, files: [] }, invoke),
    ).resolves.toEqual(sharedConversationAttachmentFixture);
    await expect(
      cancelConversationAttachments(cancelRequest, invoke),
    ).resolves.toEqual(sharedConversationAttachmentFixture);

    expect(invoke.mock.calls).toEqual([
      [CONVERSATION_ATTACHMENT_STATUS_COMMAND, { projectId }],
      [CONVERSATION_ATTACHMENT_PICK_COMMAND, { projectId }],
      [CONVERSATION_ATTACHMENT_STAGE_DROP_COMMAND, { request: dropRequest }],
      [CONVERSATION_ATTACHMENT_STAGE_NATIVE_DROP_COMMAND, { projectId }],
      [CONVERSATION_ATTACHMENT_CANCEL_COMMAND, { request: cancelRequest }],
    ]);
    expect(JSON.stringify(invoke.mock.calls)).not.toContain("/private/");
  });

  it("uses fixed worktree commands without frontend cwd, refs, or argv", async () => {
    const projectId = "018f0000-0000-7000-8000-000000000001";
    const confirmationId = "018f0000-0000-7000-8000-000000000002";
    const preview = worktreePreviewSchema.parse({
      schemaVersion: 2,
      state: "ready",
      sourceProjectId: projectId,
      operation: "create",
      branchName: "feature/bridge-fixture",
      displayPath: "~/.local/share/quireforge/worktrees/fixture",
      ownership: "managed",
      destructive: false,
      confirmationId,
      diagnosticCode: null,
    });
    const result = worktreeResultSchema.parse({
      schemaVersion: 2,
      state: "unavailable",
      sourceProjectId: projectId,
      projectId: null,
      workspace: null,
      recoverableDisplayPath: null,
      diagnosticCode: "confirmation-expired",
    });
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(scaffoldWorktreeWorkspace)
      .mockResolvedValueOnce(preview)
      .mockResolvedValueOnce(preview)
      .mockResolvedValueOnce(preview)
      .mockResolvedValueOnce(preview)
      .mockResolvedValueOnce(result)
      .mockResolvedValueOnce(true);

    await loadWorktreeStatus(projectId, invoke);
    await previewWorktreeCreate(
      { projectId, branchName: "feature/bridge-fixture" },
      invoke,
    );
    await previewWorktreeRecover(
      { projectId, recoveryId: confirmationId },
      invoke,
    );
    await previewWorktreeRemove(
      {
        projectId,
        worktreeProjectId: "018f0000-0000-7000-8000-000000000003",
      },
      invoke,
    );
    await pickWorktreeAttach(projectId, invoke);
    await confirmWorktree({ confirmationId }, invoke);
    await cancelWorktree({ confirmationId }, invoke);

    expect(invoke.mock.calls).toEqual([
      [WORKTREE_STATUS_COMMAND, { projectId }],
      [
        WORKTREE_CREATE_PREVIEW_COMMAND,
        {
          request: { projectId, branchName: "feature/bridge-fixture" },
        },
      ],
      [
        WORKTREE_RECOVER_PREVIEW_COMMAND,
        { request: { projectId, recoveryId: confirmationId } },
      ],
      [
        WORKTREE_REMOVE_PREVIEW_COMMAND,
        {
          request: {
            projectId,
            worktreeProjectId: "018f0000-0000-7000-8000-000000000003",
          },
        },
      ],
      [WORKTREE_PICK_ATTACH_COMMAND, { projectId }],
      [WORKTREE_CONFIRM_COMMAND, { request: { confirmationId } }],
      [WORKTREE_CANCEL_COMMAND, { request: { confirmationId } }],
    ]);
    await expect(
      previewWorktreeCreate(
        { projectId, branchName: "--force /tmp/outside" },
        invoke,
      ),
    ).rejects.toThrow();
    expect(invoke).toHaveBeenCalledTimes(7);
  });

  it("uses fixed Git review commands and validates paths before invocation", async () => {
    const projectId = "018f0000-0000-7000-8000-000000000001";
    const request = {
      projectId,
      path: "src/App.tsx",
      area: "worktree" as const,
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce({ ...scaffoldGitWorkspace, projectId })
      .mockResolvedValueOnce({ ...scaffoldGitDiff, ...request })
      .mockResolvedValueOnce(undefined);

    await loadGitStatus(projectId, invoke);
    await loadGitDiff(request, invoke);
    await openGitFile({ projectId, path: request.path }, invoke);

    expect(invoke.mock.calls).toEqual([
      [GIT_STATUS_COMMAND, { projectId }],
      [GIT_DIFF_COMMAND, { request }],
      [GIT_OPEN_FILE_COMMAND, { request: { projectId, path: request.path } }],
    ]);
    await expect(
      loadGitDiff({ ...request, path: "../outside" }, invoke),
    ).rejects.toThrow();
    expect(invoke).toHaveBeenCalledTimes(3);
  });

  it("confirms native-held Git mutation tokens without resubmitting paths", async () => {
    const projectId = "018f0000-0000-7000-8000-000000000001";
    const confirmationId = "018f0000-0000-7000-8000-000000000002";
    const recoveryId = "018f0000-0000-7000-8000-000000000003";
    const request = {
      projectId,
      operation: "stage" as const,
      path: "README.md",
      message: null,
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(scaffoldGitMutationPreview)
      .mockResolvedValueOnce(scaffoldGitMutationResult)
      .mockResolvedValueOnce(scaffoldGitMutationResult);

    await previewGitMutation(request, invoke);
    await confirmGitMutation({ confirmationId }, invoke);
    await recoverGitMutation({ recoveryId }, invoke);

    expect(invoke.mock.calls).toEqual([
      [GIT_MUTATION_PREVIEW_COMMAND, { request }],
      [GIT_MUTATION_CONFIRM_COMMAND, { request: { confirmationId } }],
      [GIT_MUTATION_RECOVER_COMMAND, { request: { recoveryId } }],
    ]);
    await expect(
      previewGitMutation({ ...request, path: "../outside" }, invoke),
    ).rejects.toThrow();
    await expect(
      confirmGitMutation(
        { confirmationId: projectId.replace("7", "4") },
        invoke,
      ),
    ).rejects.toThrow();
    expect(invoke).toHaveBeenCalledTimes(3);
  });

  it("uses fixed conversation commands without paths or Codex protocol IDs", async () => {
    const conversationId = "018f0000-0000-7000-8000-000000000010";
    const request = {
      projectId: "018f0000-0000-7000-8000-000000000001",
      prompt: "Review the attached project.",
      attachmentIds: [],
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      selectionPolicy: {
        ownership: "manual" as const,
        userLocked: false,
        allowedModelIds: [],
        reasoningCeiling: null,
      },
      sandboxMode: "read-only" as const,
      approvalPolicy: "untrusted" as const,
      integrationEntryIds: [],
    };
    const invoke = vi
      .fn()
      .mockImplementation((command: string) =>
        Promise.resolve(
          command === CONVERSATION_ACTIVE_COMMAND
            ? { schemaVersion: 1, capacity: 4, conversations: [] }
            : scaffoldConversation,
        ),
      );

    await loadConversationStatus(invoke);
    await loadActiveConversations(invoke);
    await startConversation(request, invoke);
    await pollConversation(conversationId, invoke);
    await interruptConversation(conversationId, invoke);
    await decideConversationApproval(
      {
        conversationId,
        approvalId: "018f0000-0000-7000-8000-000000000011",
        decision: "decline",
      },
      invoke,
    );

    expect(invoke.mock.calls).toEqual([
      [CONVERSATION_STATUS_COMMAND],
      [CONVERSATION_ACTIVE_COMMAND],
      [CONVERSATION_START_COMMAND, { request }],
      [CONVERSATION_POLL_COMMAND, { conversationId }],
      [CONVERSATION_INTERRUPT_COMMAND, { conversationId }],
      [
        CONVERSATION_APPROVAL_DECIDE_COMMAND,
        {
          request: {
            conversationId,
            approvalId: "018f0000-0000-7000-8000-000000000011",
            decision: "decline",
          },
        },
      ],
    ]);
  });

  it("updates only the closed app-owned next-turn selector contract", async () => {
    const request = {
      conversationId: "018f0000-0000-7000-8000-000000000010",
      choice: {
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
      },
      policy: {
        ownership: "manual" as const,
        userLocked: true,
        allowedModelIds: [],
        reasoningCeiling: null,
      },
      pendingAction: "dismiss" as const,
    };
    const snapshot = {
      schemaVersion: 1 as const,
      availability: "ready" as const,
      effective: request.choice,
      pending: null,
      policy: request.policy,
      diagnosticCode: null,
    };
    const invoke = vi.fn().mockResolvedValue(snapshot);

    await expect(updateModelSelection(request, invoke)).resolves.toEqual(
      snapshot,
    );
    expect(invoke).toHaveBeenCalledWith(MODEL_SELECTION_UPDATE_COMMAND, {
      request,
    });
    await expect(
      updateModelSelection({ ...request, prompt: "private" } as never, invoke),
    ).rejects.toThrow();
    expect(invoke).toHaveBeenCalledTimes(1);
  });

  it("rejects path-bearing conversation input before native invocation", async () => {
    const invoke = vi.fn().mockResolvedValue(scaffoldConversation);

    await expect(
      startConversation(
        {
          projectId: "018f0000-0000-7000-8000-000000000001",
          prompt: "Review.",
          attachmentIds: [],
          modelId: "gpt-5.6-sol",
          reasoningEffort: "high",
          sandboxMode: "read-only",
          approvalPolicy: "untrusted",
          cwd: "/private/raw/path",
        } as never,
        invoke,
      ),
    ).rejects.toThrow();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("uses fixed session lifecycle commands with app-owned IDs only", async () => {
    const conversationId = "018f0000-0000-7000-8000-000000000010";
    const projectId = "018f0000-0000-7000-8000-000000000001";
    const request = {
      conversationId,
      prompt: "Continue safely.",
      attachmentIds: [],
    };
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(scaffoldSessionLifecycle)
      .mockResolvedValueOnce(scaffoldConversation)
      .mockResolvedValueOnce(scaffoldConversation)
      .mockResolvedValueOnce(scaffoldSessionLifecycle)
      .mockResolvedValueOnce(scaffoldSessionLifecycle);

    await loadConversationSessions(
      { projectId, searchTerm: "lifecycle" },
      invoke,
    );
    await resumeConversation(request, invoke);
    await forkConversation(request, invoke);
    await archiveConversation(conversationId, invoke);
    await restoreConversation(conversationId, invoke);

    expect(invoke.mock.calls).toEqual([
      [
        CONVERSATION_SESSIONS_COMMAND,
        { request: { projectId, searchTerm: "lifecycle" } },
      ],
      [CONVERSATION_RESUME_COMMAND, { request }],
      [CONVERSATION_FORK_COMMAND, { request }],
      [CONVERSATION_ARCHIVE_COMMAND, { conversationId }],
      [CONVERSATION_RESTORE_COMMAND, { conversationId }],
    ]);
  });

  it("rejects path-bearing lifecycle input before native invocation", async () => {
    const invoke = vi.fn().mockResolvedValue(scaffoldConversation);

    await expect(
      resumeConversation(
        {
          conversationId: "018f0000-0000-7000-8000-000000000010",
          prompt: "Continue.",
          attachmentIds: [],
          cwd: "/private/raw/path",
        } as never,
        invoke,
      ),
    ).rejects.toThrow();
    expect(invoke).not.toHaveBeenCalled();
  });
});
