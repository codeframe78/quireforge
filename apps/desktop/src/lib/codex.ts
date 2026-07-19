import runtimeFixture from "../../fixtures/codex-runtime.json";
import { z } from "zod";

const identifierSchema = z
  .string()
  .min(1)
  .max(128)
  .regex(/^[A-Za-z0-9._:/-]+$/u);

function isUnsafeDisplayCharacter(character: string): boolean {
  const codePoint = character.codePointAt(0) ?? 0;
  return (
    codePoint <= 0x1f ||
    (codePoint >= 0x7f && codePoint <= 0x9f) ||
    (codePoint >= 0x200b && codePoint <= 0x200f) ||
    (codePoint >= 0x202a && codePoint <= 0x202e) ||
    (codePoint >= 0x2060 && codePoint <= 0x206f) ||
    codePoint === 0xfeff
  );
}

const displayTextSchema = z
  .string()
  .min(1)
  .max(128)
  .refine(
    (value) => ![...value].some(isUnsafeDisplayCharacter),
    "Display text contains control or directional characters",
  );

const capabilitySchema = z
  .object({
    id: identifierSchema.max(64),
    state: z.enum(["ready", "unavailable", "unsupported"]),
    route: z.enum(["app-server", "cli", "native"]),
  })
  .strict();

const modelSchema = z
  .object({
    id: identifierSchema,
    displayName: displayTextSchema,
    isDefault: z.boolean(),
    defaultReasoningEffort: identifierSchema.max(32),
    supportedReasoningEfforts: z.array(identifierSchema.max(32)).max(12),
  })
  .strict();

export const codexRuntimeSchema = z
  .object({
    schemaVersion: z.literal(1),
    adapterVersion: z.literal("codex-app-server-v2"),
    availability: z.enum(["ready", "degraded", "unavailable"]),
    backend: z.enum(["app-server-stdio", "cli-fallback", "unavailable"]),
    cliVersion: z
      .string()
      .max(32)
      .regex(/^\d+\.\d+\.\d+(?:[-+][A-Za-z0-9.-]+)?$/u)
      .nullable(),
    capabilities: z.array(capabilitySchema).max(32),
    models: z.array(modelSchema).max(256),
    diagnosticCode: z
      .enum([
        "cli-not-found",
        "cli-version-invalid",
        "process-spawn-failed",
        "process-exited",
        "transport-timeout",
        "transport-closed",
        "message-too-large",
        "invalid-protocol-message",
        "rpc-rejected",
        "unexpected-server-request",
        "invalid-model-catalog",
      ])
      .nullable(),
  })
  .strict()
  .superRefine((snapshot, context) => {
    const consistentState =
      (snapshot.availability === "ready" &&
        snapshot.backend === "app-server-stdio" &&
        snapshot.cliVersion !== null &&
        snapshot.diagnosticCode === null) ||
      (snapshot.availability === "degraded" &&
        snapshot.backend === "cli-fallback" &&
        snapshot.cliVersion !== null &&
        snapshot.models.length === 0 &&
        snapshot.diagnosticCode !== null) ||
      (snapshot.availability === "unavailable" &&
        snapshot.backend === "unavailable" &&
        snapshot.cliVersion === null &&
        snapshot.models.length === 0 &&
        snapshot.diagnosticCode !== null);

    if (!consistentState) {
      context.addIssue({
        code: "custom",
        message: "Codex runtime availability fields are inconsistent",
      });
    }

    const modelIds = new Set(snapshot.models.map((model) => model.id));
    const defaultModels = snapshot.models.filter((model) => model.isDefault);
    if (modelIds.size !== snapshot.models.length || defaultModels.length > 1) {
      context.addIssue({
        code: "custom",
        message: "Codex runtime model identities are inconsistent",
        path: ["models"],
      });
    }

    snapshot.models.forEach((model, index) => {
      if (
        !model.supportedReasoningEfforts.includes(model.defaultReasoningEffort)
      ) {
        context.addIssue({
          code: "custom",
          message: "Default reasoning effort is not advertised",
          path: ["models", index, "defaultReasoningEffort"],
        });
      }
    });
  });

export type CodexRuntimeSnapshot = z.infer<typeof codexRuntimeSchema>;

export const scaffoldCodexRuntime = codexRuntimeSchema.parse(runtimeFixture);
