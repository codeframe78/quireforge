import { describe, expect, it } from "vitest";

import { desktopNotificationResultSchema } from "./desktopIntegration";

describe("desktop notification contract", () => {
  it("accepts only a closed delivery status without task content", () => {
    expect(
      desktopNotificationResultSchema.parse({
        schemaVersion: 1,
        status: "foreground",
      }),
    ).toEqual({ schemaVersion: 1, status: "foreground" });
    expect(() =>
      desktopNotificationResultSchema.parse({
        schemaVersion: 1,
        status: "sent",
        prompt: "private task",
      }),
    ).toThrow();
  });
});
