import filePreviewFixture from "../../fixtures/file-preview.json";
import { z } from "zod";

const opaqueIdSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

const safePathSchema = z
  .string()
  .min(1)
  .max(4096)
  .refine(
    (value) =>
      !value.startsWith("/") &&
      !value.includes("\\") &&
      !/\p{Cc}|[\u202a-\u202e\u2066-\u2069]/u.test(value),
  )
  .refine((value) =>
    value
      .split("/")
      .every((part) => part !== "" && part !== "." && part !== ".."),
  );

function isSafePreviewCharacter(character: string): boolean {
  return (
    character === "\n" ||
    character === "\t" ||
    (!/\p{Cc}/u.test(character) &&
      !/[\u202a-\u202e\u2066-\u2069]/u.test(character))
  );
}

const previewTextSchema = z
  .string()
  .max(128 * 1024)
  .refine((value) => new TextEncoder().encode(value).byteLength <= 128 * 1024)
  .refine((value) => value.split("\n").length <= 2_000)
  .refine((value) => [...value].every(isSafePreviewCharacter));

const imageDataUrlSchema = z
  .string()
  .max(5_600_000)
  .refine((value) => {
    if (!/^data:image\/(png|jpeg);base64,[A-Za-z0-9+/]+={0,2}$/u.test(value)) {
      return false;
    }
    return value.slice(value.indexOf(",") + 1).length % 4 === 0;
  });

function imagePayloadByteLength(value: string): number {
  const payload = value.slice(value.indexOf(",") + 1);
  const padding = payload.endsWith("==") ? 2 : payload.endsWith("=") ? 1 : 0;
  return (payload.length / 4) * 3 - padding;
}

export const filePreviewDiagnosticCodeSchema = z.enum([
  "invalid-request",
  "project-not-found",
  "directory-unavailable",
  "identity-changed",
  "picker-unavailable",
  "outside-project",
  "unsafe-path",
  "unsupported-type",
  "file-too-large",
  "read-failed",
  "invalid-content",
  "image-dimensions-too-large",
  "handoff-expired",
  "open-failed",
]);

export const filePreviewHandoffRequestSchema = z
  .object({ openActionId: opaqueIdSchema })
  .strict();

export type FilePreviewHandoffRequest = z.infer<
  typeof filePreviewHandoffRequestSchema
>;

export const filePreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready", "unavailable"]),
    projectId: opaqueIdSchema.nullable(),
    displayPath: safePathSchema.nullable(),
    kind: z.enum(["text", "image", "pdf"]).nullable(),
    rendering: z
      .enum(["normalized-text", "bounded-image", "metadata-only"])
      .nullable(),
    mimeType: z
      .enum([
        "text/plain; charset=utf-8",
        "image/png",
        "image/jpeg",
        "application/pdf",
      ])
      .nullable(),
    byteSize: z
      .number()
      .int()
      .nonnegative()
      .max(8 * 1024 * 1024)
      .nullable(),
    truncated: z.boolean(),
    textContent: previewTextSchema.nullable(),
    imageDataUrl: imageDataUrlSchema.nullable(),
    imageWidth: z.number().int().positive().max(8192).nullable(),
    imageHeight: z.number().int().positive().max(8192).nullable(),
    openActionId: opaqueIdSchema.nullable(),
    diagnosticCode: filePreviewDiagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((preview, context) => {
    const contentFields = [
      preview.displayPath,
      preview.kind,
      preview.rendering,
      preview.mimeType,
      preview.byteSize,
      preview.textContent,
      preview.imageDataUrl,
      preview.imageWidth,
      preview.imageHeight,
      preview.openActionId,
    ];
    if (preview.state === "empty") {
      if (
        contentFields.some((value) => value !== null) ||
        preview.diagnosticCode !== null ||
        preview.truncated
      ) {
        context.addIssue({
          code: "custom",
          message: "Empty preview fields are inconsistent",
        });
      }
      return;
    }
    if (preview.state === "unavailable") {
      if (
        contentFields.some((value) => value !== null) ||
        preview.diagnosticCode === null ||
        preview.truncated
      ) {
        context.addIssue({
          code: "custom",
          message: "Unavailable preview fields are inconsistent",
        });
      }
      return;
    }
    if (
      preview.projectId === null ||
      preview.displayPath === null ||
      preview.kind === null ||
      preview.rendering === null ||
      preview.mimeType === null ||
      preview.byteSize === null ||
      preview.openActionId === null ||
      preview.diagnosticCode !== null
    ) {
      context.addIssue({
        code: "custom",
        message: "Ready preview metadata is incomplete",
      });
      return;
    }
    if (
      preview.kind === "text" &&
      (preview.rendering !== "normalized-text" ||
        preview.mimeType !== "text/plain; charset=utf-8" ||
        preview.textContent === null ||
        (preview.byteSize > 128 * 1024 && !preview.truncated) ||
        preview.imageDataUrl !== null ||
        preview.imageWidth !== null ||
        preview.imageHeight !== null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Text preview fields are inconsistent",
      });
    }
    if (
      preview.kind === "image" &&
      (preview.rendering !== "bounded-image" ||
        !["image/png", "image/jpeg"].includes(preview.mimeType) ||
        preview.textContent !== null ||
        preview.imageDataUrl === null ||
        !preview.imageDataUrl.startsWith(`data:${preview.mimeType};base64,`) ||
        preview.byteSize > 4 * 1024 * 1024 ||
        imagePayloadByteLength(preview.imageDataUrl) !== preview.byteSize ||
        preview.imageWidth === null ||
        preview.imageHeight === null ||
        preview.truncated ||
        (preview.imageWidth !== null &&
          preview.imageHeight !== null &&
          preview.imageWidth * preview.imageHeight > 16_000_000))
    ) {
      context.addIssue({
        code: "custom",
        message: "Image preview fields are inconsistent",
      });
    }
    if (
      preview.kind === "pdf" &&
      (preview.rendering !== "metadata-only" ||
        preview.mimeType !== "application/pdf" ||
        preview.textContent !== null ||
        preview.imageDataUrl !== null ||
        preview.imageWidth !== null ||
        preview.imageHeight !== null ||
        preview.truncated)
    ) {
      context.addIssue({
        code: "custom",
        message: "PDF preview fields are inconsistent",
      });
    }
  });

export type FilePreviewSnapshot = z.infer<typeof filePreviewSchema>;

export const scaffoldFilePreview: FilePreviewSnapshot = {
  schemaVersion: 1,
  state: "empty",
  projectId: null,
  displayPath: null,
  kind: null,
  rendering: null,
  mimeType: null,
  byteSize: null,
  truncated: false,
  textContent: null,
  imageDataUrl: null,
  imageWidth: null,
  imageHeight: null,
  openActionId: null,
  diagnosticCode: null,
};

export const sharedFilePreviewFixture =
  filePreviewSchema.parse(filePreviewFixture);
