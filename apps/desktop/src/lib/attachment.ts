import attachmentFixture from "../../fixtures/conversation-attachments.json";
import { z } from "zod";

const MAX_ATTACHMENT_BYTES = 4 * 1024 * 1024;
const MAX_BASE64_BYTES = Math.ceil(MAX_ATTACHMENT_BYTES / 3) * 4;

const opaqueIdSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

const safeDisplayNameSchema = z
  .string()
  .min(1)
  .max(255)
  .refine((value) => new TextEncoder().encode(value).byteLength <= 255)
  .refine(
    (value) =>
      value !== "." &&
      value !== ".." &&
      !value.includes("/") &&
      !value.includes("\\") &&
      !/\p{Cc}|[\u202a-\u202e\u2066-\u2069]/u.test(value),
  );

export const conversationAttachmentDiagnosticSchema = z.enum([
  "invalid-request",
  "project-not-found",
  "project-unavailable",
  "project-identity-changed",
  "project-not-writable",
  "staging-unavailable",
  "too-many-files",
  "file-too-large",
  "unsupported-type",
  "invalid-content",
  "unsafe-name",
  "read-failed",
  "attachment-not-found",
  "attachment-expired",
  "cleanup-failed",
]);

const attachmentSummarySchema = z
  .object({
    attachmentId: opaqueIdSchema,
    displayName: safeDisplayNameSchema,
    source: z.enum(["native-picker", "drag-drop"]),
    mimeType: z.enum(["image/png", "image/jpeg"]),
    byteSize: z.number().int().positive().max(MAX_ATTACHMENT_BYTES),
    imageWidth: z.number().int().positive().max(8192),
    imageHeight: z.number().int().positive().max(8192),
  })
  .strict()
  .refine(
    (attachment) =>
      attachment.imageWidth * attachment.imageHeight <= 16_000_000,
  );

export const conversationAttachmentSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready", "unavailable"]),
    projectId: opaqueIdSchema.nullable(),
    attachments: z.array(attachmentSummarySchema).max(4),
    diagnosticCode: conversationAttachmentDiagnosticSchema.nullable(),
  })
  .strict()
  .superRefine((snapshot, context) => {
    if (
      snapshot.state === "unavailable" &&
      (snapshot.attachments.length !== 0 || snapshot.diagnosticCode === null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Unavailable attachment state is inconsistent",
      });
    }
    if (
      snapshot.state === "empty" &&
      (snapshot.attachments.length !== 0 || snapshot.diagnosticCode !== null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Empty attachment state is inconsistent",
      });
    }
    if (
      snapshot.state === "ready" &&
      (snapshot.projectId === null ||
        snapshot.attachments.length === 0 ||
        snapshot.diagnosticCode !== null ||
        new Set(
          snapshot.attachments.map((attachment) => attachment.attachmentId),
        ).size !== snapshot.attachments.length)
    ) {
      context.addIssue({
        code: "custom",
        message: "Ready attachment state is inconsistent",
      });
    }
  });

const droppedFileSchema = z
  .object({
    displayName: safeDisplayNameSchema,
    declaredMimeType: z.enum(["image/png", "image/jpeg"]),
    base64Data: z
      .string()
      .min(4)
      .max(MAX_BASE64_BYTES)
      .regex(/^[A-Za-z0-9+/]+={0,2}$/u)
      .refine((value) => value.length % 4 === 0),
  })
  .strict();

export const conversationAttachmentDropRequestSchema = z
  .object({
    projectId: opaqueIdSchema,
    files: z.array(droppedFileSchema).min(1).max(4),
  })
  .strict();

export const conversationAttachmentCancelRequestSchema = z
  .object({
    projectId: opaqueIdSchema,
    attachmentIds: z.array(opaqueIdSchema).min(1).max(4),
  })
  .strict()
  .refine(
    (request) =>
      new Set(request.attachmentIds).size === request.attachmentIds.length,
  );

export type ConversationAttachmentSnapshot = z.infer<
  typeof conversationAttachmentSnapshotSchema
>;
export type ConversationAttachmentDropRequest = z.infer<
  typeof conversationAttachmentDropRequestSchema
>;
export type ConversationAttachmentCancelRequest = z.infer<
  typeof conversationAttachmentCancelRequestSchema
>;

export const scaffoldConversationAttachments: ConversationAttachmentSnapshot = {
  schemaVersion: 1,
  state: "empty",
  projectId: null,
  attachments: [],
  diagnosticCode: null,
};

export const sharedConversationAttachmentFixture =
  conversationAttachmentSnapshotSchema.parse(attachmentFixture);
