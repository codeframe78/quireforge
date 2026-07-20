import { invoke } from "@tauri-apps/api/core";

import {
  codexAuthSchema,
  type AuthLoginMethod,
  type CodexAuthSnapshot,
} from "./auth";
import { codexRuntimeSchema, type CodexRuntimeSnapshot } from "./codex";
import { desktopBootstrapSchema, type DesktopBootstrap } from "./contract";
import {
  conversationSnapshotSchema,
  conversationStartRequestSchema,
  conversationIdSchema,
  type ConversationSnapshot,
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
  sessionLifecycleSchema,
  type ConversationContinueRequest,
  type SessionLifecycleSnapshot,
} from "./session";

export const CODEX_RUNTIME_PROBE_COMMAND = "codex_runtime_probe";
export const CODEX_AUTH_STATUS_COMMAND = "codex_auth_status";
export const CODEX_AUTH_REFRESH_COMMAND = "codex_auth_refresh";
export const CODEX_AUTH_START_COMMAND = "codex_auth_start";
export const CODEX_AUTH_CANCEL_COMMAND = "codex_auth_cancel";
export const CODEX_AUTH_LOGOUT_COMMAND = "codex_auth_logout";
export const CODEX_AUTH_OPEN_BROWSER_COMMAND = "codex_auth_open_browser";
export const DESKTOP_BOOTSTRAP_COMMAND = "desktop_bootstrap";
export const PROJECT_WORKSPACE_STATUS_COMMAND = "project_workspace_status";
export const PROJECT_PICK_DIRECTORY_COMMAND = "project_pick_directory";
export const PROJECT_PICK_RELINK_COMMAND = "project_pick_relink";
export const PROJECT_CONFIRM_ATTACHMENT_COMMAND = "project_confirm_attachment";
export const PROJECT_CANCEL_ATTACHMENT_COMMAND = "project_cancel_attachment";
export const PROJECT_DETACH_COMMAND = "project_detach";
export const PROJECT_ARCHIVE_COMMAND = "project_archive";
export const PROJECT_PREFLIGHT_COMMAND = "project_preflight";
export const CONVERSATION_STATUS_COMMAND = "conversation_status";
export const CONVERSATION_START_COMMAND = "conversation_start";
export const CONVERSATION_POLL_COMMAND = "conversation_poll";
export const CONVERSATION_INTERRUPT_COMMAND = "conversation_interrupt";
export const CONVERSATION_SESSIONS_COMMAND = "conversation_sessions";
export const CONVERSATION_RESUME_COMMAND = "conversation_resume";
export const CONVERSATION_FORK_COMMAND = "conversation_fork";
export const CONVERSATION_ARCHIVE_COMMAND = "conversation_archive";
export const CONVERSATION_RESTORE_COMMAND = "conversation_restore";

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

export async function loadConversationStatus(
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<ConversationSnapshot> {
  const payload = await invokeFunction(CONVERSATION_STATUS_COMMAND);
  return conversationSnapshotSchema.parse(payload);
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

export async function loadConversationSessions(
  projectId: string | null = null,
  invokeFunction: InvokeFunction = invokeTauri,
): Promise<SessionLifecycleSnapshot> {
  const reviewedProjectId =
    projectId === null ? null : conversationIdSchema.parse(projectId);
  const payload = await invokeFunction(CONVERSATION_SESSIONS_COMMAND, {
    projectId: reviewedProjectId,
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
