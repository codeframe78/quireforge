import conversationFixture from "../../fixtures/conversation.json";
import conversationRegistryFixture from "../../fixtures/conversation-registry.json";
import { z } from "zod";

import {
  modelSelectionChoiceSchema,
  modelSelectionPolicySchema,
  modelSelectionProtocolChoiceSchema,
  modelSelectionSnapshotSchema,
} from "./modelSelection";

export const conversationIdSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

export const conversationProtocolChoiceSchema =
  modelSelectionProtocolChoiceSchema;

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
    attachmentIds: z.array(conversationIdSchema).max(4),
    integrationEntryIds: z
      .array(
        z
          .string()
          .min(11)
          .max(128)
          .regex(/^connector:[a-z0-9][a-z0-9._-]*$/u),
      )
      .max(8),
    modelId: conversationProtocolChoiceSchema,
    reasoningEffort: conversationProtocolChoiceSchema.max(32),
    selectionPolicy: modelSelectionPolicySchema,
    sandboxMode: conversationSandboxModeSchema,
    approvalPolicy: conversationApprovalPolicySchema,
  })
  .strict()
  .superRefine((request, context) => {
    if (new Set(request.attachmentIds).size !== request.attachmentIds.length) {
      context.addIssue({
        code: "custom",
        message: "Conversation attachments must be unique",
        path: ["attachmentIds"],
      });
    }
    if (
      new Set(request.integrationEntryIds).size !==
      request.integrationEntryIds.length
    ) {
      context.addIssue({
        code: "custom",
        message: "Connector mentions must be unique",
        path: ["integrationEntryIds"],
      });
    }
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

export const conversationApprovalDecisionSchema = z.enum([
  "approve",
  "decline",
  "cancel",
]);

export const conversationApprovalDecisionRequestSchema = z
  .object({
    conversationId: conversationIdSchema,
    approvalId: conversationIdSchema,
    decision: conversationApprovalDecisionSchema,
  })
  .strict();

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
      activityId: conversationIdSchema,
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
      title: boundedTextSchema(256),
      detail: boundedTextSchema(8 * 1024).nullable(),
      exitCode: z.number().int().min(-2147483648).max(2147483647).nullable(),
    })
    .strict(),
  z
    .object({
      type: z.literal("activity-output-delta"),
      sequence: sequenceSchema,
      activityId: conversationIdSchema,
      delta: boundedTextSchema(8 * 1024),
    })
    .strict(),
  z
    .object({
      type: z.literal("approval-requested"),
      sequence: sequenceSchema,
      approvalId: conversationIdSchema,
      activityId: conversationIdSchema,
      kind: z.enum(["command-execution", "file-change", "permissions"]),
    })
    .strict(),
  z
    .object({
      type: z.literal("approval-resolved"),
      sequence: sequenceSchema,
      approvalId: conversationIdSchema,
      resolution: z.enum([
        "approved",
        "declined",
        "canceled",
        "resolved-externally",
      ]),
    })
    .strict(),
  z
    .object({
      type: z.literal("model-selection-requested"),
      sequence: sequenceSchema,
      choice: modelSelectionChoiceSchema,
      application: z.enum(["manual", "recommendation", "automatic"]),
      rationale: boundedTextSchema(240),
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
  "parallel-capacity-reached",
  "conversation-not-found",
  "invalid-request",
  "project-unavailable",
  "project-identity-changed",
  "project-not-writable",
  "project-busy",
  "runtime-unavailable",
  "model-unavailable",
  "reasoning-unavailable",
  "integration-unavailable",
  "attachment-unavailable",
  "metadata-unavailable",
  "approval-required",
  "approval-not-found",
  "approval-decision-unavailable",
  "process-exited",
  "transport-failed",
  "protocol-invalid",
  "rpc-rejected",
]);

const conversationApprovalSchema = z
  .object({
    approvalId: conversationIdSchema,
    activityId: conversationIdSchema,
    kind: z.enum(["command-execution", "file-change", "permissions"]),
    title: boundedTextSchema(256),
    reason: boundedTextSchema(4096).nullable(),
    details: z
      .array(
        z
          .object({
            label: boundedTextSchema(128),
            value: boundedTextSchema(8 * 1024),
          })
          .strict(),
      )
      .max(16),
    decisions: z.array(conversationApprovalDecisionSchema).min(1).max(3),
  })
  .strict()
  .superRefine((approval, context) => {
    if (new Set(approval.decisions).size !== approval.decisions.length) {
      context.addIssue({
        code: "custom",
        message: "Approval decisions must be unique",
        path: ["decisions"],
      });
    }
  });

export const conversationSnapshotSchema = z
  .object({
    schemaVersion: z.literal(3),
    state: z.enum([
      "empty",
      "running",
      "waiting-for-approval",
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
    modelSelection: modelSelectionSnapshotSchema.nullable(),
    sandboxMode: conversationSandboxModeSchema.nullable(),
    approvalPolicy: conversationApprovalPolicySchema.nullable(),
    pendingApproval: conversationApprovalSchema.nullable(),
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
      snapshot.modelSelection,
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
        snapshot.pendingApproval === null &&
        snapshot.events.length === 0 &&
        snapshot.diagnosticCode === null) ||
      (snapshot.state === "unavailable" &&
        hasNoMetadata &&
        snapshot.pendingApproval === null &&
        snapshot.events.length === 0 &&
        snapshot.diagnosticCode !== null) ||
      (["running", "stopping"].includes(snapshot.state) &&
        hasAllMetadata &&
        snapshot.pendingApproval === null &&
        snapshot.diagnosticCode === null) ||
      (snapshot.state === "waiting-for-approval" &&
        hasAllMetadata &&
        snapshot.pendingApproval !== null &&
        (snapshot.diagnosticCode === null ||
          ["approval-not-found", "approval-decision-unavailable"].includes(
            snapshot.diagnosticCode,
          ))) ||
      (terminalWithoutDiagnostic &&
        hasAllMetadata &&
        snapshot.pendingApproval === null &&
        snapshot.diagnosticCode === null) ||
      (snapshot.state === "blocked" &&
        hasAllMetadata &&
        snapshot.pendingApproval === null &&
        snapshot.diagnosticCode === "approval-required") ||
      (snapshot.state === "failed" &&
        hasAllMetadata &&
        snapshot.pendingApproval === null &&
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

export const conversationRegistrySchema = z
  .object({
    schemaVersion: z.literal(1),
    capacity: z.literal(4),
    conversations: z
      .array(conversationSnapshotSchema)
      .max(4)
      .superRefine((conversations, context) => {
        const ids = conversations.map(
          (conversation) => conversation.conversationId,
        );
        const projectIds = conversations.map(
          (conversation) => conversation.projectId,
        );
        if (ids.some((id) => id === null) || new Set(ids).size !== ids.length) {
          context.addIssue({
            code: "custom",
            message: "Active conversation IDs must be present and unique",
          });
        }
        if (
          projectIds.some((projectId) => projectId === null) ||
          new Set(projectIds).size !== projectIds.length
        ) {
          context.addIssue({
            code: "custom",
            message: "Active project IDs must be present and unique",
          });
        }
        if (
          conversations.some(
            (conversation) =>
              !["running", "waiting-for-approval", "stopping"].includes(
                conversation.state,
              ),
          )
        ) {
          context.addIssue({
            code: "custom",
            message: "The active registry may contain only active states",
          });
        }
        if (
          conversations.some((conversation) => conversation.events.length > 0)
        ) {
          context.addIssue({
            code: "custom",
            message: "The active registry must not replay event batches",
          });
        }
      }),
  })
  .strict();

export type ConversationStartRequest = z.infer<
  typeof conversationStartRequestSchema
>;
export type ConversationApprovalDecisionRequest = z.infer<
  typeof conversationApprovalDecisionRequestSchema
>;
export type ConversationSnapshot = z.infer<typeof conversationSnapshotSchema>;
export type ConversationRegistrySnapshot = z.infer<
  typeof conversationRegistrySchema
>;
export type ConversationEvent = ConversationSnapshot["events"][number];

export const scaffoldConversation =
  conversationSnapshotSchema.parse(conversationFixture);
export const scaffoldConversationRegistry = conversationRegistrySchema.parse(
  conversationRegistryFixture,
);
