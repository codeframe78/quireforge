import { describe, expect, it, vi } from "vitest";

import { scaffoldCodexAuth } from "./auth";
import { scaffoldCodexRuntime } from "./codex";
import {
  cancelCodexAuth,
  CODEX_AUTH_CANCEL_COMMAND,
  CODEX_AUTH_OPEN_BROWSER_COMMAND,
  CODEX_AUTH_START_COMMAND,
  CODEX_AUTH_STATUS_COMMAND,
  CODEX_RUNTIME_PROBE_COMMAND,
  DESKTOP_BOOTSTRAP_COMMAND,
  loadCodexAuth,
  loadCodexRuntime,
  loadDesktopBootstrap,
  openCodexAuthBrowser,
  startCodexAuth,
} from "./bridge";
import { scaffoldBootstrap } from "./contract";

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
});
