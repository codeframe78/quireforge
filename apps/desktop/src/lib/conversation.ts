import conversationFixture from "../../fixtures/conversation.json";
import { z } from "zod";

export const conversationIdSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

export const conversationProtocolChoiceSchema = z
  .string()
  .min(1)
  .max(128)
  .regex(/^[A-Za-z0-9._:/-]+$/u);

function containsUnsafeControl(value: string): boolean {
  return [...value].some((character) => {
    const codePoint = character.codePointAt(0) ?? 0;
    return (
      (codePoint <= 0x1f && ![0x09, 0x0a, 0x0d].includes(codePoint)) ||
      (codePoint >= 0x7f && codePoint <= 0x9f) ||
      (codePoint >= 0x200b && codePoint <= 0x200f) ||
      (codePoint >= 0x202a && codePoint <= 0x202e) ||
      (codePoint >= 0x2060 && codePoint <= 0x206f) ||
      codePoint === 0xfeff
    );
  });
}

const boundedTextSchema = (max: number) =>
  z
    .string()
    .min(1)
    .max(max)
    .refine((value) => !containsUnsafeControl(value));

export const conversationPromptSchema = boundedTextSchema(64 * 1024).refine(
  (value) => value.trim().length > 0,
);

export const conversationSandboxModeSchema = z.enum([
  "read-only",
  "workspace-write",
  "danger-full-access",
]);
export const conversationApprovalPolicySchema = z.enum([
  "untrusted",
  "on-request",
  "never",
]);

export const conversationStartRequestSchema = z
  .object({
    projectId: conversationIdSchema,
    prompt: conversationPromptSchema,
    modelId: conversationProtocolChoiceSchema,
    reasoningEffort: conversationProtocolChoiceSchema.max(32),
    sandboxMode: conversationSandboxModeSchema,
    approvalPolicy: conversationApprovalPolicySchema,
  })
  .strict()
  .superRefine((request, context) => {
    if (
      request.sandboxMode === "danger-full-access" &&
      request.approvalPolicy === "never"
    ) {
      context.addIssue({
        code: "custom",
        message: "Unrestricted execution cannot disable approval prompts",
      });
    }
  });

const sequenceSchema = z.number().int().positive().safe();
const planStepSchema = z
  .object({
    step: boundedTextSchema(4096),
    status: z.enum(["pending", "in-progress", "completed"]),
  })
  .strict();

const conversationEventSchema = z.discriminatedUnion("type", [
  z
    .object({
      type: z.literal("lifecycle"),
      sequence: sequenceSchema,
      phase: z.enum([
        "starting",
        "running",
        "stopping",
        "completed",
        "interrupted",
        "blocked",
        "failed",
      ]),
    })
    .strict(),
  z
    .object({
      type: z.literal("agent-message-delta"),
      sequence: sequenceSchema,
      delta: boundedTextSchema(64 * 1024),
    })
    .strict(),
  z
    .object({
      type: z.literal("reasoning-summary-delta"),
      sequence: sequenceSchema,
      delta: boundedTextSchema(64 * 1024),
    })
    .strict(),
  z
    .object({
      type: z.literal("plan-updated"),
      sequence: sequenceSchema,
      explanation: boundedTextSchema(4096).nullable(),
      steps: z.array(planStepSchema).max(128),
    })
    .strict(),
  z
    .object({
      type: z.literal("activity"),
      sequence: sequenceSchema,
      kind: z.enum([
        "user-message",
        "agent-message",
        "plan",
        "reasoning",
        "command-execution",
        "file-change",
        "tool-call",
        "web-search",
        "image",
        "other",
      ]),
      status: z.enum(["started", "completed"]),
    })
    .strict(),
  z
    .object({
      type: z.literal("error"),
      sequence: sequenceSchema,
      code: z.enum([
        "context-window-exceeded",
        "usage-limit-exceeded",
        "unauthorized",
        "sandbox",
        "server",
        "other",
      ]),
      willRetry: z.boolean(),
    })
    .strict(),
]);

export const conversationDiagnosticSchema = z.enum([
  "conversation-active",
  "conversation-not-found",
  "invalid-request",
  "project-unavailable",
  "project-identity-changed",
  "project-not-writable",
  "project-busy",
  "runtime-unavailable",
  "model-unavailable",
  "reasoning-unavailable",
  "metadata-unavailable",
  "approval-required",
  "process-exited",
  "transport-failed",
  "protocol-invalid",
  "rpc-rejected",
]);

export const conversationSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum([
      "empty",
      "running",
      "stopping",
      "completed",
      "interrupted",
      "blocked",
      "failed",
      "unavailable",
    ]),
    conversationId: conversationIdSchema.nullable(),
    projectId: conversationIdSchema.nullable(),
    modelId: conversationProtocolChoiceSchema.nullable(),
    reasoningEffort: conversationProtocolChoiceSchema.max(32).nullable(),
    sandboxMode: conversationSandboxModeSchema.nullable(),
    approvalPolicy: conversationApprovalPolicySchema.nullable(),
    events: z.array(conversationEventSchema).max(64),
    diagnosticCode: conversationDiagnosticSchema.nullable(),
  })
  .strict()
  .superRefine((snapshot, context) => {
    const metadata = [
      snapshot.conversationId,
      snapshot.projectId,
      snapshot.modelId,
      snapshot.reasoningEffort,
      snapshot.sandboxMode,
      snapshot.approvalPolicy,
    ];
    const hasAllMetadata = metadata.every((value) => value !== null);
    const hasNoMetadata = metadata.every((value) => value === null);
    const terminalWithoutDiagnostic = ["completed", "interrupted"].includes(
      snapshot.state,
    );
    const consistent =
      (snapshot.state === "empty" &&
        hasNoMetadata &&
        snapshot.events.length === 0 &&
        snapshot.diagnosticCode === null) ||
      (snapshot.state === "unavailable" &&
        hasNoMetadata &&
        snapshot.events.length === 0 &&
        snapshot.diagnosticCode !== null) ||
      (["running", "stopping"].includes(snapshot.state) &&
        hasAllMetadata &&
        snapshot.diagnosticCode === null) ||
      (terminalWithoutDiagnostic &&
        hasAllMetadata &&
        snapshot.diagnosticCode === null) ||
      (snapshot.state === "blocked" &&
        hasAllMetadata &&
        snapshot.diagnosticCode === "approval-required") ||
      (snapshot.state === "failed" &&
        hasAllMetadata &&
        snapshot.diagnosticCode !== null);
    if (!consistent) {
      context.addIssue({
        code: "custom",
        message: "Conversation snapshot fields are inconsistent",
      });
    }
    const sequences = snapshot.events.map((event) => event.sequence);
    if (
      sequences.some(
        (value, index) => index > 0 && value <= sequences[index - 1]!,
      )
    ) {
      context.addIssue({
        code: "custom",
        message: "Conversation event sequences must increase",
        path: ["events"],
      });
    }
  });

export type ConversationStartRequest = z.infer<
  typeof conversationStartRequestSchema
>;
export type ConversationSnapshot = z.infer<typeof conversationSnapshotSchema>;
export type ConversationEvent = ConversationSnapshot["events"][number];

export const scaffoldConversation =
  conversationSnapshotSchema.parse(conversationFixture);
