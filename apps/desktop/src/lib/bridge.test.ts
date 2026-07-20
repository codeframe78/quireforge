import { describe, expect, it, vi } from "vitest";

import { scaffoldCodexAuth } from "./auth";
import { scaffoldCodexRuntime } from "./codex";
import {
  archiveProject,
  cancelProjectAttachment,
  cancelCodexAuth,
  CODEX_AUTH_CANCEL_COMMAND,
  CODEX_AUTH_OPEN_BROWSER_COMMAND,
  CODEX_AUTH_START_COMMAND,
  CODEX_AUTH_STATUS_COMMAND,
  CODEX_RUNTIME_PROBE_COMMAND,
  confirmProjectAttachment,
  DESKTOP_BOOTSTRAP_COMMAND,
  detachProject,
  loadCodexAuth,
  loadCodexRuntime,
  loadDesktopBootstrap,
  loadProjectWorkspace,
  openCodexAuthBrowser,
  pickProjectDirectory,
  pickProjectRelink,
  preflightProject,
  PROJECT_ARCHIVE_COMMAND,
  PROJECT_CANCEL_ATTACHMENT_COMMAND,
  PROJECT_CONFIRM_ATTACHMENT_COMMAND,
  PROJECT_DETACH_COMMAND,
  PROJECT_PICK_DIRECTORY_COMMAND,
  PROJECT_PICK_RELINK_COMMAND,
  PROJECT_PREFLIGHT_COMMAND,
  PROJECT_WORKSPACE_STATUS_COMMAND,
  startCodexAuth,
} from "./bridge";
import { scaffoldBootstrap } from "./contract";
import { scaffoldProjectWorkspace } from "./project";

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
});
