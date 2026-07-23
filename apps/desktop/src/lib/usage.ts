import usageFixture from "../../fixtures/codex-usage.json";
import { z } from "zod";

const usageWindowSchema = z
  .object({
    kind: z.enum(["primary", "secondary"]),
    usedPercent: z.number().int().min(0).max(100),
    remainingPercent: z.number().int().min(0).max(100),
    windowDurationMinutes: z.number().int().positive().max(525_600).nullable(),
    resetsAt: z.number().int().min(0).max(32_503_680_000).nullable(),
  })
  .strict()
  .refine(
    (window) => window.usedPercent + window.remainingPercent === 100,
    "Usage percentages must total 100",
  );

const usageMeterSchema = z
  .object({
    label: z
      .string()
      .trim()
      .min(1)
      .max(80)
      .refine(
        (value) =>
          ![...value].some((character) => {
            const codePoint = character.codePointAt(0) ?? 0;
            return (
              codePoint <= 0x1f ||
              (codePoint >= 0x7f && codePoint <= 0x9f) ||
              (codePoint >= 0x200b && codePoint <= 0x200f) ||
              (codePoint >= 0x202a && codePoint <= 0x202e) ||
              (codePoint >= 0x2060 && codePoint <= 0x206f) ||
              codePoint === 0xfeff
            );
          }),
      ),
    limitId: z
      .string()
      .min(1)
      .max(64)
      .regex(/^[A-Za-z0-9_.-]+$/u),
    windows: z
      .array(usageWindowSchema)
      .min(1)
      .max(2)
      .refine(
        (windows) =>
          new Set(windows.map((window) => window.kind)).size === windows.length,
        "Usage window kinds must be unique",
      ),
    limited: z.boolean(),
  })
  .strict();

const usageDiagnosticSchema = z.enum([
  "runtime-unavailable",
  "protocol-invalid",
  "rpc-rejected",
  "timeout",
  "no-usage-windows",
]);

export const codexUsageSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["ready", "not-metered", "unavailable"]),
    meters: z.array(usageMeterSchema).max(8),
    diagnosticCode: usageDiagnosticSchema.nullable(),
  })
  .strict()
  .superRefine((snapshot, context) => {
    const ready =
      snapshot.state === "ready" &&
      snapshot.meters.length > 0 &&
      snapshot.diagnosticCode === null;
    const notMetered =
      snapshot.state === "not-metered" &&
      snapshot.meters.length === 0 &&
      snapshot.diagnosticCode === "no-usage-windows";
    const unavailable =
      snapshot.state === "unavailable" &&
      snapshot.meters.length === 0 &&
      snapshot.diagnosticCode !== null &&
      snapshot.diagnosticCode !== "no-usage-windows";
    if (!ready && !notMetered && !unavailable) {
      context.addIssue({
        code: "custom",
        message: "Codex usage fields are inconsistent",
      });
    }

    const ids = snapshot.meters.map((meter) => meter.limitId);
    if (new Set(ids).size !== ids.length) {
      context.addIssue({
        code: "custom",
        message: "Codex usage meter identifiers must be unique",
        path: ["meters"],
      });
    }
  });

export type CodexUsageSnapshot = z.infer<typeof codexUsageSchema>;
export type CodexUsageWindow =
  CodexUsageSnapshot["meters"][number]["windows"][number];
export const scaffoldCodexUsage = codexUsageSchema.parse(usageFixture);
