import projectFixture from "../../fixtures/project-workspace.json";
import { z } from "zod";

const opaqueIdSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

const displayTextSchema = z
  .string()
  .min(1)
  .max(4096)
  .refine((value) => !/\p{Cc}/u.test(value));

export const directoryAccessibilityStateSchema = z.enum([
  "connected-accessible",
  "connected-read-only",
  "missing-or-moved",
  "permission-denied",
  "removable-disconnected",
  "network-unavailable",
  "git-invalid",
  "sandbox-restricted",
  "identity-changed",
  "verification-unknown",
]);

const projectDiagnosticCodeSchema = z.enum([
  "metadata-unavailable",
  "picker-unavailable",
  "directory-unavailable",
  "duplicate-directory",
  "project-not-found",
  "attachment-not-pending",
  "identity-changed",
]);

const gitSummarySchema = z
  .object({
    isRepository: z.boolean(),
    isLinkedWorktree: z.boolean(),
  })
  .strict()
  .superRefine((git, context) => {
    if (git.isLinkedWorktree && !git.isRepository) {
      context.addIssue({
        code: "custom",
        message: "A linked worktree must also be a repository",
      });
    }
  });

const directorySummarySchema = z
  .object({
    associationId: opaqueIdSchema,
    displayPath: displayTextSchema,
    resolvedDisplayPath: displayTextSchema.nullable(),
    state: directoryAccessibilityStateSchema,
    expectedAccess: z.literal("read-write"),
    isPrimary: z.boolean(),
    git: gitSummarySchema,
    hasAgentsGuidance: z.boolean(),
    hasCodexConfig: z.boolean(),
  })
  .strict();

const projectSummarySchema = z
  .object({
    id: opaqueIdSchema,
    displayName: displayTextSchema.max(120),
    archived: z.boolean(),
    directory: directorySummarySchema.nullable(),
  })
  .strict();

const pendingAttachmentSchema = z
  .object({
    operation: z.enum(["attach", "relink"]),
    projectId: opaqueIdSchema.nullable(),
    displayName: displayTextSchema.max(120),
    selectedDisplayPath: displayTextSchema,
    resolvedDisplayPath: displayTextSchema,
    state: directoryAccessibilityStateSchema,
    git: gitSummarySchema,
    hasAgentsGuidance: z.boolean(),
    hasCodexConfig: z.boolean(),
  })
  .strict()
  .superRefine((pending, context) => {
    const validProjectId =
      (pending.operation === "attach" && pending.projectId === null) ||
      (pending.operation === "relink" && pending.projectId !== null);
    if (!validProjectId) {
      context.addIssue({
        code: "custom",
        message: "Pending attachment identity is inconsistent",
      });
    }
    if (
      !["connected-accessible", "connected-read-only"].includes(pending.state)
    ) {
      context.addIssue({
        code: "custom",
        message: "Pending directories must be inspectable",
      });
    }
  });

export const projectWorkspaceSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready", "unavailable"]),
    projects: z.array(projectSummarySchema).max(256),
    pendingAttachment: pendingAttachmentSchema.nullable(),
    diagnosticCode: projectDiagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((workspace, context) => {
    if (workspace.state === "empty" && workspace.projects.length !== 0) {
      context.addIssue({
        code: "custom",
        message: "Empty workspace has projects",
      });
    }
    if (workspace.state === "ready" && workspace.projects.length === 0) {
      context.addIssue({
        code: "custom",
        message: "Ready workspace has no projects",
      });
    }
    if (
      workspace.state === "unavailable" &&
      (workspace.projects.length !== 0 ||
        workspace.pendingAttachment !== null ||
        workspace.diagnosticCode === null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Unavailable workspace fields are inconsistent",
      });
    }
    const projectIds = workspace.projects.map((project) => project.id);
    if (new Set(projectIds).size !== projectIds.length) {
      context.addIssue({
        code: "custom",
        message: "Project IDs must be unique",
      });
    }
    const associationIds = workspace.projects.flatMap((project) =>
      project.directory ? [project.directory.associationId] : [],
    );
    if (new Set(associationIds).size !== associationIds.length) {
      context.addIssue({
        code: "custom",
        message: "Directory association IDs must be unique",
      });
    }
  });

export const projectPreflightSchema = z
  .object({
    schemaVersion: z.literal(1),
    projectId: opaqueIdSchema,
    cwdReady: z.boolean(),
    displayPath: displayTextSchema.nullable(),
    state: directoryAccessibilityStateSchema,
    diagnosticCode: projectDiagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((preflight, context) => {
    if (
      preflight.cwdReady &&
      (preflight.state !== "connected-accessible" ||
        preflight.displayPath === null ||
        preflight.diagnosticCode !== null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Ready cwd preflight fields are inconsistent",
      });
    }
    if (
      preflight.state === "identity-changed" &&
      preflight.diagnosticCode !== "identity-changed"
    ) {
      context.addIssue({
        code: "custom",
        message: "Identity change must carry its stable diagnostic",
      });
    }
  });

export type DirectoryAccessibilityState = z.infer<
  typeof directoryAccessibilityStateSchema
>;
export type ProjectWorkspaceSnapshot = z.infer<typeof projectWorkspaceSchema>;
export type ProjectPreflightSnapshot = z.infer<typeof projectPreflightSchema>;

export const scaffoldProjectWorkspace =
  projectWorkspaceSchema.parse(projectFixture);
