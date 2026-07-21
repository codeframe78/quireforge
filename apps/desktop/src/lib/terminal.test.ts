import { describe, expect, it } from "vitest";

import {
  scaffoldTerminalRegistry,
  terminalRegistrySchema,
  terminalSnapshotSchema,
  terminalStartRequestSchema,
  terminalWriteRequestSchema,
} from "./terminal";

const projectId = "018f0000-0000-7000-8000-000000000001";
const terminalId = "018f0000-0000-7000-8000-000000000002";

const runningTerminal = terminalSnapshotSchema.parse({
  schemaVersion: 1,
  state: "running",
  terminalId,
  projectId,
  title: "Terminal 1",
  live: true,
  columns: 100,
  rows: 30,
  output: [{ sequence: 1, dataBase64: "/wA=" }],
  firstSequence: 1,
  lastSequence: 1,
  truncated: false,
  hasMore: false,
  exitCode: null,
  diagnosticCode: null,
});

describe("terminal contract", () => {
  it("keeps the browser scaffold honest and process-free", () => {
    expect(scaffoldTerminalRegistry).toEqual({
      schemaVersion: 1,
      capacity: 8,
      terminals: [],
      diagnosticCode: null,
    });
  });

  it("preserves arbitrary PTY bytes through bounded base64 output", () => {
    expect(runningTerminal.output[0]?.dataBase64).toBe("/wA=");
    expect(
      terminalRegistrySchema.parse({
        schemaVersion: 1,
        capacity: 8,
        terminals: [runningTerminal],
        diagnosticCode: null,
      }).terminals,
    ).toHaveLength(1);
  });

  it("rejects native paths, process IDs, executables, and environment input", () => {
    for (const field of ["cwd", "pid", "shell", "environment"] as const) {
      expect(() =>
        terminalStartRequestSchema.parse({
          projectId,
          columns: 100,
          rows: 30,
          [field]: field === "pid" ? 1234 : "/private/value",
        }),
      ).toThrow();
    }
  });

  it("enforces input size and ordered output bounds before native IPC", () => {
    expect(() =>
      terminalWriteRequestSchema.parse({
        terminalId,
        dataBase64: "A".repeat(87_388),
      }),
    ).toThrow();
    expect(() =>
      terminalSnapshotSchema.parse({
        ...runningTerminal,
        output: [
          { sequence: 2, dataBase64: "QQ==" },
          { sequence: 1, dataBase64: "Qg==" },
        ],
        lastSequence: 2,
      }),
    ).toThrow();
  });

  it("marks recovered metadata as non-live without output history", () => {
    const recovered = terminalSnapshotSchema.parse({
      ...runningTerminal,
      state: "interrupted",
      live: false,
      output: [],
      firstSequence: 0,
      lastSequence: 0,
    });
    expect(recovered.live).toBe(false);
    expect(JSON.stringify(recovered)).not.toMatch(/cwd|pid|shell|environment/u);
  });
});
