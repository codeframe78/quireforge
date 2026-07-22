import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";

import {
  gitDiffRequestSchema,
  gitDiffSchema,
  gitMutationConfirmRequestSchema,
  gitMutationPreviewRequestSchema,
  gitMutationPreviewSchema,
  gitMutationResultSchema,
  gitOpenFileRequestSchema,
  gitRecoveryRequestSchema,
  gitWorkspaceSchema,
  type GitDiffRequest,
  type GitDiffSnapshot,
  type GitMutationConfirmRequest,
  type GitMutationPreviewRequest,
  type GitMutationPreviewSnapshot,
  type GitMutationResultSnapshot,
  type GitOpenFileRequest,
  type GitRecoveryRequest,
  type GitWorkspaceSnapshot,
} from "./git";
import {
  codexAuthSchema,
  type AuthLoginMethod,
  type CodexAuthSnapshot,
} from "./auth";
import { codexRuntimeSchema, type CodexRuntimeSnapshot } from "./codex";
import { desktopBootstrapSchema, type DesktopBootstrap } from "./contract";
import {
  integrationCatalogSchema,
  integrationMutationConfirmRequestSchema,
  integrationMutationPreviewRequestSchema,
  integrationMutationPreviewSchema,
  integrationMutationResultSchema,
  type IntegrationCatalogSnapshot,
  type IntegrationMutationConfirmRequest,
  type IntegrationMutationPreviewRequest,
  type IntegrationMutationPreviewSnapshot,
  type IntegrationMutationResultSnapshot,
} from "./integration";
import {
  conversationSnapshotSchema,
  conversationRegistrySchema,
  conversationStartRequestSchema,
  conversationApprovalDecisionRequestSchema,
  conversationIdSchema,
  type ConversationApprovalDecisionRequest,
  type ConversationSnapshot,
  type ConversationRegistrySnapshot,
  type ConversationStartRequest,
} from "./conversation";
import {
  projectPreflightSchema,
  projectWorkspaceSchema,
  type ProjectPreflightSnapshot,
  type ProjectWorkspaceSnapshot,
} from "./project";
import {
  conversationContinueRequestSchema,
  sessionListRequestSchema,
  sessionLifecycleSchema,
  type ConversationContinueRequest,
  type SessionListRequest,
  type SessionLifecycleSnapshot,
} from "./session";
import {
  worktreeConfirmationRequestSchema,
  worktreeCreatePreviewRequestSchema,
  worktreePreviewSchema,
  worktreeRecoverPreviewRequestSchema,
  worktreeRemovePreviewRequestSchema,
  worktreeResultSchema,
  worktreeWorkspaceSchema,
  type WorktreeConfirmationRequest,
  type WorktreeCreatePreviewRequest,
  type WorktreePreviewSnapshot,
  type WorktreeRecoverPreviewRequest,
  type WorktreeRemovePreviewRequest,
  type WorktreeResultSnapshot,
  type WorktreeWorkspaceSnapshot,
} from "./worktree";
import {
  terminalCloseRequestSchema,
  terminalPollRequestSchema,
  terminalRegistrySchema,
  terminalResizeRequestSchema,
  terminalSnapshotSchema,
  terminalStartRequestSchema,
  terminalWriteRequestSchema,
  type TerminalCloseRequest,
  type TerminalPollRequest,
  type TerminalRegistrySnapshot,
  type TerminalResizeRequest,
  type TerminalSnapshot,
  type TerminalStartRequest,
  type TerminalWriteRequest,
} from "./terminal";

export const CODEX_RUNTIME_PROBE_COMMAND = "codex_runtime_probe";
export const CODEX_AUTH_STATUS_COMMAND = "codex_auth_status";
export const CODEX_AUTH_REFRESH_COMMAND = "codex_auth_refresh";
export const CODEX_AUTH_START_COMMAND = "codex_auth_start";
export const CODEX_AUTH_CANCEL_COMMAND = "codex_auth_cancel";
export const CODEX_AUTH_LOGOUT_COMMAND = "codex_auth_logout";
export const CODEX_AUTH_OPEN_BROWSER_COMMAND = "codex_auth_open_browser";
export const DESKTOP_BOOTSTRAP_COMMAND = "desktop_bootstrap";
export const INTEGRATION_CATALOG_READ_COMMAND = "integration_catalog_read";
export const INTEGRATION_MUTATION_PREVIEW_COMMAND =
  "integration_mutation_preview";
export const INTEGRATION_MUTATION_CONFIRM_COMMAND =
  "integration_mutation_confirm";
export const PROJECT_WORKSPACE_STATUS_COMMAND = "project_workspace_status";
export const PROJECT_PICK_DIRECTORY_COMMAND = "project_pick_directory";
export const PROJECT_PICK_RELINK_COMMAND = "project_pick_relink";
export const PROJECT_CONFIRM_ATTACHMENT_COMMAND = "project_confirm_attachment";
export const PROJECT_CANCEL_ATTACHMENT_COMMAND = "project_cancel_attachment";
export const PROJECT_DETACH_COMMAND = "project_detach";
export const PROJECT_ARCHIVE_COMMAND = "project_archive";
export const PROJECT_PREFLIGHT_COMMAND = "project_preflight";
export const WORKTREE_STATUS_COMMAND = "worktree_status";
export const WORKTREE_CREATE_PREVIEW_COMMAND = "worktree_create_preview";
export const WORKTREE_RECOVER_PREVIEW_COMMAND = "worktree_recover_preview";
export const WORKTREE_REMOVE_PREVIEW_COMMAND = "worktree_remove_preview";
export const WORKTREE_PICK_ATTACH_COMMAND = "worktree_pick_attach";
export const WORKTREE_CONFIRM_COMMAND = "worktree_confirm";
export const WORKTREE_CANCEL_COMMAND = "worktree_cancel";
export const GIT_STATUS_COMMAND = "git_status";
export const GIT_DIFF_COMMAND = "git_diff";
export const GIT_OPEN_FILE_COMMAND = "git_open_file";
export const GIT_MUTATION_PREVIEW_COMMAND = "git_mutation_preview";
export const GIT_MUTATION_CONFIRM_COMMAND = "git_mutation_confirm";
export const GIT_MUTATION_RECOVER_COMMAND = "git_mutation_recover";
export const CONVERSATION_STATUS_COMMAND = "conversation_status";
export const CONVERSATION_ACTIVE_COMMAND = "conversation_active";
export const CONVERSATION_START_COMMAND = "conversation_start";
export const CONVERSATION_POLL_COMMAND = "conversation_poll";
export const CONVERSATION_INTERRUPT_COMMAND = "conversation_interrupt";
export const CONVERSATION_APPROVAL_DECIDE_COMMAND =
  "conversation_approval_decide";
export const CONVERSATION_SESSIONS_COMMAND = "conversation_sessions";
export const CONVERSATION_RESUME_COMMAND = "conversation_resume";
export const CONVERSATION_FORK_COMMAND = "conversation_fork";
export const CONVERSATION_ARCHIVE_COMMAND = "conversation_archive";
export const CONVERSATION_RESTORE_COMMAND = "conversation_restore";
export const TERMINAL_STATUS_COMMAND = "terminal_status";
export const TERMINAL_START_COMMAND = "terminal_start";
export const TERMINAL_POLL_COMMAND = "terminal_poll";
export const TERMINAL_WRITE_COMMAND = "terminal_write";
export const TERMINAL_RESIZE_COMMAND = "terminal_resize";
export const TERMINAL_CLOSE_COMMAND = "terminal_close";

export type InvokeFunction = (
  command: string,
  args?: Record<string, unknown>,
) => Promise<unknown>;

const invokeTauri: InvokeFunction = (command, args) =>
  invoke<unknown>(command, args);

export async function loadDesktopBootstrap(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<DesktopBootstrap> {
  const payload = await invokeFunction(DESKTOP_BOOTSTRAP_COMMAND);
  return desktopBootstrapSchema.parse(payload);
}

export async function loadCodexRuntime(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexRuntimeSnapshot> {
  const payload = await invokeFunction(CODEX_RUNTIME_PROBE_COMMAND);
  return codexRuntimeSchema.parse(payload);
}

export async function loadIntegrationCatalog(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationCatalogSnapshot> {
  const payload = await invokeFunction(INTEGRATION_CATALOG_READ_COMMAND);
  return integrationCatalogSchema.parse(payload);
}

export async function previewIntegrationMutation(
  request: IntegrationMutationPreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationMutationPreviewSnapshot> {
  const reviewedRequest =
    integrationMutationPreviewRequestSchema.parse(request);
  const payload = await invokeFunction(INTEGRATION_MUTATION_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return integrationMutationPreviewSchema.parse(payload);
}

export async function confirmIntegrationMutation(
  request: IntegrationMutationConfirmRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<IntegrationMutationResultSnapshot> {
  const reviewedRequest =
    integrationMutationConfirmRequestSchema.parse(request);
  const payload = await invokeFunction(INTEGRATION_MUTATION_CONFIRM_COMMAND, {
    request: reviewedRequest,
  });
  return integrationMutationResultSchema.parse(payload);
}

export async function loadCodexAuth(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_STATUS_COMMAND);
  return codexAuthSchema.parse(payload);
}

export async function refreshCodexAuth(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_REFRESH_COMMAND);
  return codexAuthSchema.parse(payload);
}

export async function startCodexAuth(
  method: AuthLoginMethod,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_START_COMMAND, { method });
  return codexAuthSchema.parse(payload);
}

export async function cancelCodexAuth(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_CANCEL_COMMAND);
  return codexAuthSchema.parse(payload);
}

export async function logoutCodexAuth(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<CodexAuthSnapshot> {
  const payload = await invokeFunction(CODEX_AUTH_LOGOUT_COMMAND);
  return codexAuthSchema.parse(payload);
}

export async function openCodexAuthBrowser(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<void> {
  await invokeFunction(CODEX_AUTH_OPEN_BROWSER_COMMAND);
}

async function invokeProjectWorkspace(
  command: string,
  args?: Record<string, unknown>,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  const payload = await invokeFunction(command, args);
  return projectWorkspaceSchema.parse(payload);
}

export function loadProjectWorkspace(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_WORKSPACE_STATUS_COMMAND,
    undefined,
    invokeFunction,
  );
}

export function pickProjectDirectory(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_PICK_DIRECTORY_COMMAND,
    undefined,
    invokeFunction,
  );
}

export function pickProjectRelink(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_PICK_RELINK_COMMAND,
    { projectId },
    invokeFunction,
  );
}

export function confirmProjectAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_CONFIRM_ATTACHMENT_COMMAND,
    undefined,
    invokeFunction,
  );
}

export function cancelProjectAttachment(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_CANCEL_ATTACHMENT_COMMAND,
    undefined,
    invokeFunction,
  );
}

export function detachProject(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_DETACH_COMMAND,
    { projectId },
    invokeFunction,
  );
}

export function archiveProject(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectWorkspaceSnapshot> {
  return invokeProjectWorkspace(
    PROJECT_ARCHIVE_COMMAND,
    { projectId },
    invokeFunction,
  );
}

export async function preflightProject(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ProjectPreflightSnapshot> {
  const payload = await invokeFunction(PROJECT_PREFLIGHT_COMMAND, {
    projectId,
  });
  return projectPreflightSchema.parse(payload);
}

export async function loadWorktreeStatus(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreeWorkspaceSnapshot> {
  const reviewedProjectId =
    worktreeCreatePreviewRequestSchema.shape.projectId.parse(projectId);
  const payload = await invokeFunction(WORKTREE_STATUS_COMMAND, {
    projectId: reviewedProjectId,
  });
  return worktreeWorkspaceSchema.parse(payload);
}

export async function previewWorktreeCreate(
  request: WorktreeCreatePreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreePreviewSnapshot> {
  const reviewedRequest = worktreeCreatePreviewRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_CREATE_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return worktreePreviewSchema.parse(payload);
}

export async function previewWorktreeRecover(
  request: WorktreeRecoverPreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreePreviewSnapshot> {
  const reviewedRequest = worktreeRecoverPreviewRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_RECOVER_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return worktreePreviewSchema.parse(payload);
}

export async function previewWorktreeRemove(
  request: WorktreeRemovePreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreePreviewSnapshot> {
  const reviewedRequest = worktreeRemovePreviewRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_REMOVE_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return worktreePreviewSchema.parse(payload);
}

export async function pickWorktreeAttach(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreePreviewSnapshot> {
  const reviewedProjectId =
    worktreeCreatePreviewRequestSchema.shape.projectId.parse(projectId);
  const payload = await invokeFunction(WORKTREE_PICK_ATTACH_COMMAND, {
    projectId: reviewedProjectId,
  });
  return worktreePreviewSchema.parse(payload);
}

export async function confirmWorktree(
  request: WorktreeConfirmationRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<WorktreeResultSnapshot> {
  const reviewedRequest = worktreeConfirmationRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_CONFIRM_COMMAND, {
    request: reviewedRequest,
  });
  return worktreeResultSchema.parse(payload);
}

export async function cancelWorktree(
  request: WorktreeConfirmationRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<boolean> {
  const reviewedRequest = worktreeConfirmationRequestSchema.parse(request);
  const payload = await invokeFunction(WORKTREE_CANCEL_COMMAND, {
    request: reviewedRequest,
  });
  return z.boolean().parse(payload);
}

export async function loadGitStatus(
  projectId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitWorkspaceSnapshot> {
  const reviewedId = gitDiffRequestSchema.shape.projectId.parse(projectId);
  const payload = await invokeFunction(GIT_STATUS_COMMAND, {
    projectId: reviewedId,
  });
  return gitWorkspaceSchema.parse(payload);
}

export async function loadGitDiff(
  request: GitDiffRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitDiffSnapshot> {
  const reviewedRequest = gitDiffRequestSchema.parse(request);
  const payload = await invokeFunction(GIT_DIFF_COMMAND, {
    request: reviewedRequest,
  });
  return gitDiffSchema.parse(payload);
}

export async function openGitFile(
  request: GitOpenFileRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<void> {
  const reviewedRequest = gitOpenFileRequestSchema.parse(request);
  await invokeFunction(GIT_OPEN_FILE_COMMAND, { request: reviewedRequest });
}

export async function previewGitMutation(
  request: GitMutationPreviewRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitMutationPreviewSnapshot> {
  const reviewedRequest = gitMutationPreviewRequestSchema.parse(request);
  const payload = await invokeFunction(GIT_MUTATION_PREVIEW_COMMAND, {
    request: reviewedRequest,
  });
  return gitMutationPreviewSchema.parse(payload);
}

export async function confirmGitMutation(
  request: GitMutationConfirmRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitMutationResultSnapshot> {
  const reviewedRequest = gitMutationConfirmRequestSchema.parse(request);
  const payload = await invokeFunction(GIT_MUTATION_CONFIRM_COMMAND, {
    request: reviewedRequest,
  });
  return gitMutationResultSchema.parse(payload);
}

export async function recoverGitMutation(
  request: GitRecoveryRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<GitMutationResultSnapshot> {
  const reviewedRequest = gitRecoveryRequestSchema.parse(request);
  const payload = await invokeFunction(GIT_MUTATION_RECOVER_COMMAND, {
    request: reviewedRequest,
  });
  return gitMutationResultSchema.parse(payload);
}

export async function loadConversationStatus(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const payload = await invokeFunction(CONVERSATION_STATUS_COMMAND);
  return conversationSnapshotSchema.parse(payload);
}

export async function loadActiveConversations(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationRegistrySnapshot> {
  const payload = await invokeFunction(CONVERSATION_ACTIVE_COMMAND);
  return conversationRegistrySchema.parse(payload);
}

export async function startConversation(
  request: ConversationStartRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const reviewedRequest = conversationStartRequestSchema.parse(request);
  const payload = await invokeFunction(CONVERSATION_START_COMMAND, {
    request: reviewedRequest,
  });
  return conversationSnapshotSchema.parse(payload);
}

export async function pollConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const reviewedId = conversationIdSchema.parse(conversationId);
  const payload = await invokeFunction(CONVERSATION_POLL_COMMAND, {
    conversationId: reviewedId,
  });
  return conversationSnapshotSchema.parse(payload);
}

export async function interruptConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const reviewedId = conversationIdSchema.parse(conversationId);
  const payload = await invokeFunction(CONVERSATION_INTERRUPT_COMMAND, {
    conversationId: reviewedId,
  });
  return conversationSnapshotSchema.parse(payload);
}

export async function decideConversationApproval(
  request: ConversationApprovalDecisionRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const reviewedRequest =
    conversationApprovalDecisionRequestSchema.parse(request);
  const payload = await invokeFunction(CONVERSATION_APPROVAL_DECIDE_COMMAND, {
    request: reviewedRequest,
  });
  return conversationSnapshotSchema.parse(payload);
}

export async function loadConversationSessions(
  request: SessionListRequest = { projectId: null, searchTerm: null },
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<SessionLifecycleSnapshot> {
  const reviewedRequest = sessionListRequestSchema.parse(request);
  const payload = await invokeFunction(CONVERSATION_SESSIONS_COMMAND, {
    request: reviewedRequest,
  });
  return sessionLifecycleSchema.parse(payload);
}

async function continueConversation(
  command: string,
  request: ConversationContinueRequest,
  invokeFunction: InvokeFunction,
): Promise<ConversationSnapshot> {
  const reviewedRequest = conversationContinueRequestSchema.parse(request);
  const payload = await invokeFunction(command, { request: reviewedRequest });
  return conversationSnapshotSchema.parse(payload);
}

export function resumeConversation(
  request: ConversationContinueRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  return continueConversation(
    CONVERSATION_RESUME_COMMAND,
    request,
    invokeFunction,
  );
}

export function forkConversation(
  request: ConversationContinueRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  return continueConversation(
    CONVERSATION_FORK_COMMAND,
    request,
    invokeFunction,
  );
}

async function setConversationArchived(
  command: string,
  conversationId: string,
  invokeFunction: InvokeFunction,
): Promise<SessionLifecycleSnapshot> {
  const reviewedId = conversationIdSchema.parse(conversationId);
  const payload = await invokeFunction(command, { conversationId: reviewedId });
  return sessionLifecycleSchema.parse(payload);
}

export function archiveConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<SessionLifecycleSnapshot> {
  return setConversationArchived(
    CONVERSATION_ARCHIVE_COMMAND,
    conversationId,
    invokeFunction,
  );
}

export function restoreConversation(
  conversationId: string,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<SessionLifecycleSnapshot> {
  return setConversationArchived(
    CONVERSATION_RESTORE_COMMAND,
    conversationId,
    invokeFunction,
  );
}

export async function loadTerminalStatus(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalRegistrySnapshot> {
  const payload = await invokeFunction(TERMINAL_STATUS_COMMAND);
  return terminalRegistrySchema.parse(payload);
}

export async function startTerminal(
  request: TerminalStartRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalSnapshot> {
  const reviewedRequest = terminalStartRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_START_COMMAND, {
    request: reviewedRequest,
  });
  return terminalSnapshotSchema.parse(payload);
}

export async function pollTerminal(
  request: TerminalPollRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalSnapshot> {
  const reviewedRequest = terminalPollRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_POLL_COMMAND, {
    request: reviewedRequest,
  });
  return terminalSnapshotSchema.parse(payload);
}

export async function writeTerminal(
  request: TerminalWriteRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalSnapshot> {
  const reviewedRequest = terminalWriteRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_WRITE_COMMAND, {
    request: reviewedRequest,
  });
  return terminalSnapshotSchema.parse(payload);
}

export async function resizeTerminal(
  request: TerminalResizeRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalSnapshot> {
  const reviewedRequest = terminalResizeRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_RESIZE_COMMAND, {
    request: reviewedRequest,
  });
  return terminalSnapshotSchema.parse(payload);
}

export async function closeTerminal(
  request: TerminalCloseRequest,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<TerminalRegistrySnapshot> {
  const reviewedRequest = terminalCloseRequestSchema.parse(request);
  const payload = await invokeFunction(TERMINAL_CLOSE_COMMAND, {
    request: reviewedRequest,
  });
  return terminalRegistrySchema.parse(payload);
}
