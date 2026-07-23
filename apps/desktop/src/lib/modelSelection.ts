import { z } from "zod";

export const modelSelectionProtocolChoiceSchema = z
  .string()
  .min(1)
  .max(128)
  .regex(/^[A-Za-z0-9._:/-]+$/u);

const modelSelectionReasoningSchema =
  modelSelectionProtocolChoiceSchema.max(32);
const conversationIdSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

function containsUnsafeControl(value: string): boolean {
  return [...value].some((character) => {
    const codePoint = character.codePointAt(0) ?? 0;
    return (
      codePoint <= 0x1f ||
      (codePoint >= 0x7f && codePoint <= 0x9f) ||
      (codePoint >= 0x200b && codePoint <= 0x200f) ||
      (codePoint >= 0x202a && codePoint <= 0x202e) ||
      (codePoint >= 0x2060 && codePoint <= 0x206f) ||
      codePoint === 0xfeff
    );
  });
}

export const modelSelectionChoiceSchema = z
  .object({
    modelId: modelSelectionProtocolChoiceSchema,
    reasoningEffort: modelSelectionReasoningSchema,
  })
  .strict();

export const modelSelectionOwnershipSchema = z.enum([
  "manual",
  "recommend",
  "automatic",
]);

export const modelSelectionPolicySchema = z
  .object({
    ownership: modelSelectionOwnershipSchema,
    userLocked: z.boolean(),
    allowedModelIds: z.array(modelSelectionProtocolChoiceSchema).max(32),
    reasoningCeiling: modelSelectionReasoningSchema.nullable(),
  })
  .strict()
  .superRefine((policy, context) => {
    if (
      new Set(policy.allowedModelIds).size !== policy.allowedModelIds.length
    ) {
      context.addIssue({
        code: "custom",
        message: "Automatic model boundaries must be unique",
        path: ["allowedModelIds"],
      });
    }
    if (
      policy.ownership === "automatic" &&
      policy.allowedModelIds.length === 0 &&
      policy.reasoningCeiling === null
    ) {
      context.addIssue({
        code: "custom",
        message:
          "Automatic ownership requires a model allowlist or reasoning ceiling",
      });
    }
  });

const pendingModelSelectionSchema = z
  .object({
    choice: modelSelectionChoiceSchema,
    provenance: z.enum(["user", "codex"]),
    application: z.enum(["manual", "recommendation", "automatic"]),
    rationale: z
      .string()
      .trim()
      .min(1)
      .max(240)
      .refine((value) => !containsUnsafeControl(value)),
    requestedAtMs: z.number().int().nonnegative().safe(),
  })
  .strict()
  .superRefine((pending, context) => {
    const consistent =
      (pending.provenance === "user" && pending.application === "manual") ||
      (pending.provenance === "codex" &&
        ["recommendation", "automatic"].includes(pending.application));
    if (!consistent) {
      context.addIssue({
        code: "custom",
        message: "Pending selector provenance and application are inconsistent",
      });
    }
  });

export const modelSelectionSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    availability: z.enum(["ready", "recommendation-only", "unavailable"]),
    effective: modelSelectionChoiceSchema,
    pending: pendingModelSelectionSchema.nullable(),
    policy: modelSelectionPolicySchema,
    diagnosticCode: z
      .enum([
        "invalid-request",
        "conversation-not-found",
        "metadata-unavailable",
        "catalog-unavailable",
        "model-unavailable",
        "reasoning-unavailable",
        "policy-blocked",
        "manual-ownership",
        "user-locked",
        "request-already-made",
        "control-unavailable",
      ])
      .nullable(),
  })
  .strict()
  .superRefine((snapshot, context) => {
    if (
      snapshot.policy.ownership === "automatic" &&
      snapshot.policy.allowedModelIds.length > 0 &&
      !snapshot.policy.allowedModelIds.includes(snapshot.effective.modelId)
    ) {
      context.addIssue({
        code: "custom",
        message: "Automatic policy must retain the effective model",
        path: ["policy", "allowedModelIds"],
      });
    }
  });

export const modelSelectionUpdateRequestSchema = z
  .object({
    conversationId: conversationIdSchema,
    choice: modelSelectionChoiceSchema,
    policy: modelSelectionPolicySchema,
    pendingAction: z.enum(["keep", "accept", "dismiss"]),
  })
  .strict();

export type ModelSelectionChoice = z.infer<typeof modelSelectionChoiceSchema>;
export type ModelSelectionPolicy = z.infer<typeof modelSelectionPolicySchema>;
export type ModelSelectionSnapshot = z.infer<
  typeof modelSelectionSnapshotSchema
>;
export type ModelSelectionUpdateRequest = z.infer<
  typeof modelSelectionUpdateRequestSchema
>;

export function defaultModelSelectionPolicy(): ModelSelectionPolicy {
  return {
    ownership: "manual",
    userLocked: false,
    allowedModelIds: [],
    reasoningCeiling: null,
  };
}
