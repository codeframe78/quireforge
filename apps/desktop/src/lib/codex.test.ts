import runtimeFixture from "../../fixtures/codex-runtime.json";
import { describe, expect, it } from "vitest";

import { codexRuntimeSchema, scaffoldCodexRuntime } from "./codex";

describe("Codex runtime contract", () => {
  it("parses the shared normalized fixture", () => {
    expect(codexRuntimeSchema.parse(runtimeFixture)).toEqual(
      scaffoldCodexRuntime,
    );
    expect(scaffoldCodexRuntime.models[0]?.id).toBe("gpt-5.6-sol");
  });

  it("rejects raw or unbounded protocol-shaped data", () => {
    expect(() =>
      codexRuntimeSchema.parse({
        ...runtimeFixture,
        models: [{ ...runtimeFixture.models[0], accountId: "not-normalized" }],
      }),
    ).toThrow();

    expect(() =>
      codexRuntimeSchema.parse({
        ...runtimeFixture,
        models: Array.from({ length: 257 }, () => runtimeFixture.models[0]),
      }),
    ).toThrow();
  });

  it("rejects inconsistent availability and model invariants", () => {
    expect(() =>
      codexRuntimeSchema.parse({
        ...runtimeFixture,
        availability: "unavailable",
      }),
    ).toThrow();

    expect(() =>
      codexRuntimeSchema.parse({
        ...runtimeFixture,
        models: [runtimeFixture.models[0], runtimeFixture.models[0]],
      }),
    ).toThrow();

    expect(() =>
      codexRuntimeSchema.parse({
        ...runtimeFixture,
        models: [
          { ...runtimeFixture.models[0], displayName: "Safe\u202Espoofed" },
        ],
      }),
    ).toThrow();
  });
});
