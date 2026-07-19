import fixture from "../../fixtures/codex-auth.json";
import { describe, expect, it } from "vitest";

import { codexAuthSchema, scaffoldCodexAuth } from "./auth";

describe("Codex authentication contract", () => {
  it("parses the shared normalized fixture", () => {
    expect(codexAuthSchema.parse(fixture)).toEqual(scaffoldCodexAuth);
  });

  it("accepts only a bounded official pending handoff", () => {
    const pending = codexAuthSchema.parse({
      ...fixture,
      state: "login-pending",
      pendingMethod: "device-code",
      handoff: {
        verificationUrl: "https://auth.openai.com/device",
        userCode: "SAFE-CODE",
      },
    });
    expect(pending.handoff?.userCode).toBe("SAFE-CODE");

    expect(() =>
      codexAuthSchema.parse({
        ...pending,
        handoff: {
          verificationUrl: "https://openai.com.attacker.test/login",
          userCode: "SAFE-CODE",
        },
      }),
    ).toThrow();
  });

  it("rejects raw account fields and inconsistent states", () => {
    expect(() =>
      codexAuthSchema.parse({
        ...fixture,
        email: "private@example.test",
      }),
    ).toThrow();
    expect(() =>
      codexAuthSchema.parse({
        ...fixture,
        state: "authenticated",
      }),
    ).toThrow();
    expect(() =>
      codexAuthSchema.parse({
        ...fixture,
        state: "unavailable",
      }),
    ).toThrow();
  });
});
