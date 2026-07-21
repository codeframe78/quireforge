import { describe, expect, it, vi } from "vitest";

import {
  closeTerminal,
  loadTerminalStatus,
  pollTerminal,
  resizeTerminal,
  startTerminal,
  TERMINAL_CLOSE_COMMAND,
  TERMINAL_POLL_COMMAND,
  TERMINAL_RESIZE_COMMAND,
  TERMINAL_START_COMMAND,
  TERMINAL_STATUS_COMMAND,
  TERMINAL_WRITE_COMMAND,
  writeTerminal,
} from "./bridge";
import { scaffoldTerminalRegistry, terminalSnapshotSchema } from "./terminal";

const projectId = "018f0000-0000-7000-8000-000000000001";
const terminalId = "018f0000-0000-7000-8000-000000000002";
const terminal = terminalSnapshotSchema.parse({
  schemaVersion: 1,
  state: "running",
  terminalId,
  projectId,
  title: "Terminal 1",
  live: true,
  columns: 100,
  rows: 30,
  output: [],
  firstSequence: 0,
  lastSequence: 0,
  truncated: false,
  hasMore: false,
  exitCode: null,
  diagnosticCode: null,
});

describe("terminal bridge", () => {
  it("uses fixed commands and app-owned identifiers only", async () => {
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(scaffoldTerminalRegistry)
      .mockResolvedValueOnce(terminal)
      .mockResolvedValueOnce(terminal)
      .mockResolvedValueOnce(terminal)
      .mockResolvedValueOnce(terminal)
      .mockResolvedValueOnce(scaffoldTerminalRegistry);

    await loadTerminalStatus(invoke);
    await startTerminal({ projectId, columns: 100, rows: 30 }, invoke);
    await pollTerminal({ terminalId, afterSequence: 0 }, invoke);
    await writeTerminal({ terminalId, dataBase64: "bHMNCg==" }, invoke);
    await resizeTerminal({ terminalId, columns: 120, rows: 40 }, invoke);
    await closeTerminal({ terminalId }, invoke);

    expect(invoke.mock.calls).toEqual([
      [TERMINAL_STATUS_COMMAND],
      [
        TERMINAL_START_COMMAND,
        { request: { projectId, columns: 100, rows: 30 } },
      ],
      [TERMINAL_POLL_COMMAND, { request: { terminalId, afterSequence: 0 } }],
      [
        TERMINAL_WRITE_COMMAND,
        { request: { terminalId, dataBase64: "bHMNCg==" } },
      ],
      [
        TERMINAL_RESIZE_COMMAND,
        { request: { terminalId, columns: 120, rows: 40 } },
      ],
      [TERMINAL_CLOSE_COMMAND, { request: { terminalId } }],
    ]);
  });

  it("rejects path-bearing start input before native invocation", async () => {
    const invoke = vi.fn().mockResolvedValue(terminal);
    await expect(
      startTerminal(
        { projectId, columns: 100, rows: 30, cwd: "/private" } as never,
        invoke,
      ),
    ).rejects.toThrow();
    expect(invoke).not.toHaveBeenCalled();
  });

  it("rejects native output containing an unreviewed process identifier", async () => {
    const invoke = vi.fn().mockResolvedValue({ ...terminal, pid: 1234 });
    await expect(
      pollTerminal({ terminalId, afterSequence: 0 }, invoke),
    ).rejects.toThrow();
  });
});
