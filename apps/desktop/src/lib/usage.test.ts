import { describe, expect, it } from "vitest";

import { codexUsageSchema, scaffoldCodexUsage } from "./usage";

describe("Codex usage contract", () => {
  it("accepts the sanitized read-only fixture", () => {
    expect(codexUsageSchema.parse(scaffoldCodexUsage)).toEqual(
      scaffoldCodexUsage,
    );
  });

  it("rejects inconsistent percentages and duplicate meters", () => {
    expect(() =>
      codexUsageSchema.parse({
        ...scaffoldCodexUsage,
        meters: [
          {
            ...scaffoldCodexUsage.meters[0],
            windows: [
              {
                ...scaffoldCodexUsage.meters[0]!.windows[0],
                remainingPercent: 74,
              },
            ],
          },
        ],
      }),
    ).toThrow();

    expect(() =>
      codexUsageSchema.parse({
        ...scaffoldCodexUsage,
        meters: [scaffoldCodexUsage.meters[0], scaffoldCodexUsage.meters[0]],
      }),
    ).toThrow();
  });

  it("rejects raw account and reset-credit fields", () => {
    expect(() =>
      codexUsageSchema.parse({
        ...scaffoldCodexUsage,
        planType: "pro",
        credits: { balance: "private" },
        rateLimitResetCredits: { availableCount: 1 },
      }),
    ).toThrow();
  });
});
