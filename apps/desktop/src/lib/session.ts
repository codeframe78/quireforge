import sessionLifecycleFixture from "../../fixtures/session-lifecycle.json";
import { z } from "zod";

import {
  conversationApprovalPolicySchema,
  conversationDiagnosticSchema,
  conversationIdSchema,
  conversationPromptSchema,
  conversationProtocolChoiceSchema,
  conversationSandboxModeSchema,
} from "./conversation";
import { modelSelectionSnapshotSchema } from "./modelSelection";

export const conversationContinueRequestSchema = z
  .object({
    conversationId: conversationIdSchema,
    prompt: conversationPromptSchema,
    attachmentIds: z.array(conversationIdSchema).max(4),
  })
  .strict()
  .refine(
    (request) =>
      new Set(request.attachmentIds).size === request.attachmentIds.length,
  );

export const sessionSearchTermSchema = z
  .string()
  .trim()
  .min(1)
  .max(256)
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
  );

export const sessionListRequestSchema = z
  .object({
    projectId: conversationIdSchema.nullable(),
    searchTerm: sessionSearchTermSchema.nullable(),
  })
  .strict();

const sessionReferenceSchema = z
  .object({
    conversationId: conversationIdSchema,
    projectId: conversationIdSchema,
    parentConversationId: conversationIdSchema.nullable(),
    title: sessionSearchTermSchema.nullable(),
    modelId: conversationProtocolChoiceSchema,
    reasoningEffort: conversationProtocolChoiceSchema.max(32),
    modelSelection: modelSelectionSnapshotSchema,
    sandboxMode: conversationSandboxModeSchema,
    approvalPolicy: conversationApprovalPolicySchema,
    state: z.enum([
      "running",
      "completed",
      "interrupted",
      "blocked",
      "failed",
      "archived",
      "missing",
    ]),
    createdAtMs: z.number().int().nonnegative().safe(),
    updatedAtMs: z.number().int().nonnegative().safe(),
  })
  .strict()
  .superRefine((session, context) => {
    if (session.parentConversationId === session.conversationId) {
      context.addIssue({
        code: "custom",
        message: "A session cannot be its own parent",
        path: ["parentConversationId"],
      });
    }
    if (session.updatedAtMs < session.createdAtMs) {
      context.addIssue({
        code: "custom",
        message: "Session timestamps are inconsistent",
        path: ["updatedAtMs"],
      });
    }
    if (
      session.sandboxMode === "danger-full-access" &&
      session.approvalPolicy === "never"
    ) {
      context.addIssue({
        code: "custom",
        message: "Unrestricted execution cannot disable approval prompts",
      });
    }
    if (
      session.modelId !== session.modelSelection.effective.modelId ||
      session.reasoningEffort !==
        session.modelSelection.effective.reasoningEffort
    ) {
      context.addIssue({
        code: "custom",
        message: "Session selection does not match its effective model",
        path: ["modelSelection"],
      });
    }
  });

export const sessionLifecycleSchema = z
  .object({
    schemaVersion: z.literal(3),
    state: z.enum(["empty", "ready", "unavailable"]),
    sessions: z.array(sessionReferenceSchema).max(256),
    diagnosticCode: conversationDiagnosticSchema.nullable(),
  })
  .strict()
  .superRefine((snapshot, context) => {
    const consistent =
      (snapshot.state === "empty" &&
        snapshot.sessions.length === 0 &&
        snapshot.diagnosticCode === null) ||
      (snapshot.state === "ready" &&
        snapshot.sessions.length > 0 &&
        snapshot.diagnosticCode === null) ||
      (snapshot.state === "unavailable" &&
        snapshot.sessions.length === 0 &&
        snapshot.diagnosticCode !== null);
    if (!consistent) {
      context.addIssue({
        code: "custom",
        message: "Session lifecycle snapshot fields are inconsistent",
      });
    }

    const ids = snapshot.sessions.map((session) => session.conversationId);
    if (new Set(ids).size !== ids.length) {
      context.addIssue({
        code: "custom",
        message: "Session references must be unique",
        path: ["sessions"],
      });
    }
  });

export type ConversationContinueRequest = z.infer<
  typeof conversationContinueRequestSchema
>;
export type SessionListRequest = z.infer<typeof sessionListRequestSchema>;
export type SessionLifecycleSnapshot = z.infer<typeof sessionLifecycleSchema>;

export const scaffoldSessionLifecycle = sessionLifecycleSchema.parse(
  sessionLifecycleFixture,
);
