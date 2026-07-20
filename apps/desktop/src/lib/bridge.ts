import { invoke } from "@tauri-apps/api/core";

import {
  codexAuthSchema,
  type AuthLoginMethod,
  type CodexAuthSnapshot,
} from "./auth";
import { codexRuntimeSchema, type CodexRuntimeSnapshot } from "./codex";
import { desktopBootstrapSchema, type DesktopBootstrap } from "./contract";
import {
  projectPreflightSchema,
  projectWorkspaceSchema,
  type ProjectPreflightSnapshot,
  type ProjectWorkspaceSnapshot,
} from "./project";

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
