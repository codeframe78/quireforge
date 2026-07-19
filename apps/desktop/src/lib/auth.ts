import authFixture from "../../fixtures/codex-auth.json";
import { z } from "zod";

export const authLoginMethodSchema = z.enum(["browser", "device-code"]);
export type AuthLoginMethod = z.infer<typeof authLoginMethodSchema>;

const diagnosticCodeSchema = z.enum([
  "runtime-unavailable",
  "protocol-invalid",
  "rpc-rejected",
  "timeout",
  "login-failed",
  "cancel-not-found",
]);

function isAllowedAuthUrl(value: string): boolean {
  try {
    const url = new URL(value);
    const host = url.hostname.toLowerCase();
    return (
      url.protocol === "https:" &&
      url.username === "" &&
      url.password === "" &&
      (host === "openai.com" ||
        host.endsWith(".openai.com") ||
        host === "chatgpt.com" ||
        host.endsWith(".chatgpt.com"))
    );
  } catch {
    return false;
  }
}

const handoffSchema = z
  .object({
    verificationUrl: z.string().min(1).max(2048).refine(isAllowedAuthUrl),
    userCode: z
      .string()
      .min(1)
      .max(32)
      .regex(/^[A-Za-z0-9-]+$/u)
      .nullable(),
  })
  .strict();

export const codexAuthSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum([
      "authenticated",
      "unauthenticated",
      "login-pending",
      "not-required",
      "unavailable",
    ]),
    accountKind: z
      .enum(["chatgpt", "api-key", "managed-provider", "unknown"])
      .nullable(),
    pendingMethod: authLoginMethodSchema.nullable(),
    handoff: handoffSchema.nullable(),
    diagnosticCode: diagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((snapshot, context) => {
    const authenticated =
      snapshot.state === "authenticated" &&
      snapshot.accountKind !== null &&
      snapshot.pendingMethod === null &&
      snapshot.handoff === null &&
      snapshot.diagnosticCode === null;
    const unauthenticated =
      snapshot.state === "unauthenticated" &&
      snapshot.accountKind === null &&
      snapshot.pendingMethod === null &&
      snapshot.handoff === null &&
      (snapshot.diagnosticCode === null ||
        snapshot.diagnosticCode === "login-failed" ||
        snapshot.diagnosticCode === "cancel-not-found");
    const pending =
      snapshot.state === "login-pending" &&
      snapshot.accountKind === null &&
      snapshot.pendingMethod !== null &&
      snapshot.handoff !== null &&
      snapshot.diagnosticCode === null &&
      ((snapshot.pendingMethod === "browser" &&
        snapshot.handoff.userCode === null) ||
        (snapshot.pendingMethod === "device-code" &&
          snapshot.handoff.userCode !== null));
    const notRequired =
      snapshot.state === "not-required" &&
      snapshot.accountKind === null &&
      snapshot.pendingMethod === null &&
      snapshot.handoff === null &&
      snapshot.diagnosticCode === null;
    const unavailable =
      snapshot.state === "unavailable" &&
      snapshot.accountKind === null &&
      snapshot.pendingMethod === null &&
      snapshot.handoff === null &&
      snapshot.diagnosticCode !== null;

    if (!(
      authenticated ||
      unauthenticated ||
      pending ||
      notRequired ||
      unavailable
    )) {
      context.addIssue({
        code: "custom",
        message: "Codex authentication fields are inconsistent",
      });
    }
  });

export type CodexAuthSnapshot = z.infer<typeof codexAuthSchema>;
export const scaffoldCodexAuth = codexAuthSchema.parse(authFixture);
