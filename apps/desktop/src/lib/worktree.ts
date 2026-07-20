import worktreeFixture from "../../fixtures/worktree-workspace.json";
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

const displayPathSchema = displayTextSchema.refine(
  (value) => !/[\\\p{Cf}]/u.test(value),
);

export const worktreeBranchSchema = z
  .string()
  .min(1)
  .max(96)
  .regex(
    /^(?!-)(?!HEAD$)(?!.*\.\.)(?!.*@\{)(?!.*\/\/)(?!.*\.lock$)(?!.*[/.]$)[A-Za-z0-9._/-]+$/u,
  );

const worktreeDiagnosticSchema = z.enum([
  "metadata-unavailable",
  "project-not-found",
  "project-busy",
  "not-repository",
  "directory-unavailable",
  "identity-changed",
  "picker-unavailable",
  "invalid-branch",
  "branch-exists",
  "duplicate-directory",
  "not-linked-worktree",
  "different-repository",
  "git-unavailable",
  "git-failed",
  "output-too-large",
  "confirmation-expired",
  "stale-preview",
  "worktree-remains",
]);

const worktreeOwnershipSchema = z.enum([
  "source",
  "managed",
  "attached",
  "external",
]);

const worktreeEntrySchema = z
  .object({
    projectId: opaqueIdSchema.nullable(),
    displayName: displayTextSchema.max(120),
    displayPath: displayPathSchema,
    branchName: worktreeBranchSchema.nullable(),
    ownership: worktreeOwnershipSchema,
    state: z.enum([
      "ready",
      "missing",
      "archived",
      "locked",
      "prunable",
      "detached",
    ]),
    current: z.boolean(),
  })
  .strict()
  .superRefine((entry, context) => {
    if ((entry.ownership === "external") !== (entry.projectId === null)) {
      context.addIssue({
        code: "custom",
        message: "Only external worktrees may omit a QuireForge project ID",
      });
    }
    if (entry.state === "detached" && entry.branchName !== null) {
      context.addIssue({
        code: "custom",
        message: "A detached worktree must not claim a branch",
      });
    }
  });

export const worktreeWorkspaceSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["empty", "ready", "unavailable"]),
    sourceProjectId: opaqueIdSchema.nullable(),
    worktrees: z.array(worktreeEntrySchema).max(256),
    truncated: z.boolean(),
    diagnosticCode: worktreeDiagnosticSchema.nullable(),
  })
  .strict()
  .superRefine((workspace, context) => {
    if (
      workspace.state === "unavailable" &&
      (workspace.worktrees.length !== 0 || workspace.diagnosticCode === null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Unavailable worktree fields are inconsistent",
      });
    }
    if (
      workspace.state === "empty" &&
      (workspace.worktrees.length !== 0 || workspace.diagnosticCode !== null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Empty worktree fields are inconsistent",
      });
    }
    if (
      workspace.state === "ready" &&
      (workspace.worktrees.length === 0 ||
        workspace.sourceProjectId === null ||
        workspace.diagnosticCode !== null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Ready worktree fields are inconsistent",
      });
    }
    const projectIds = workspace.worktrees.flatMap((entry) =>
      entry.projectId ? [entry.projectId] : [],
    );
    if (new Set(projectIds).size !== projectIds.length) {
      context.addIssue({
        code: "custom",
        message: "Worktree project IDs must be unique",
      });
    }
    if (
      workspace.state === "ready" &&
      workspace.worktrees.filter((entry) => entry.ownership === "source")
        .length !== 1
    ) {
      context.addIssue({
        code: "custom",
        message: "A ready inventory must contain exactly one source",
      });
    }
  });

export const worktreeCreatePreviewRequestSchema = z
  .object({
    projectId: opaqueIdSchema,
    branchName: worktreeBranchSchema,
  })
  .strict();

export const worktreeConfirmationRequestSchema = z
  .object({ confirmationId: opaqueIdSchema })
  .strict();

export const worktreePreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["ready", "cancelled", "unavailable"]),
    sourceProjectId: opaqueIdSchema,
    operation: z.enum(["create", "attach"]),
    branchName: worktreeBranchSchema.nullable(),
    displayPath: displayPathSchema.nullable(),
    ownership: z.enum(["managed", "attached"]).nullable(),
    destructive: z.literal(false),
    confirmationId: opaqueIdSchema.nullable(),
    diagnosticCode: worktreeDiagnosticSchema.nullable(),
  })
  .strict()
  .superRefine((preview, context) => {
    const ready = preview.state === "ready";
    if (
      ready !==
      (preview.displayPath !== null &&
        preview.ownership !== null &&
        preview.confirmationId !== null &&
        preview.diagnosticCode === null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Worktree preview fields are inconsistent",
      });
    }
    if (
      preview.operation === "create" &&
      ready &&
      preview.branchName === null
    ) {
      context.addIssue({
        code: "custom",
        message: "A create preview requires a branch",
      });
    }
    if (preview.state === "unavailable" && preview.diagnosticCode === null) {
      context.addIssue({
        code: "custom",
        message: "An unavailable preview requires a diagnostic",
      });
    }
  });

export const worktreeResultSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["applied", "unavailable"]),
    sourceProjectId: opaqueIdSchema.nullable(),
    projectId: opaqueIdSchema.nullable(),
    workspace: worktreeWorkspaceSchema.nullable(),
    recoverableDisplayPath: displayPathSchema.nullable(),
    diagnosticCode: worktreeDiagnosticSchema.nullable(),
  })
  .strict()
  .superRefine((result, context) => {
    if (
      result.state === "applied" &&
      (result.sourceProjectId === null ||
        result.projectId === null ||
        result.workspace === null ||
        result.diagnosticCode !== null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Applied worktree result fields are inconsistent",
      });
    }
    if (
      result.state === "unavailable" &&
      (result.projectId !== null ||
        result.workspace !== null ||
        result.diagnosticCode === null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Unavailable worktree result fields are inconsistent",
      });
    }
    if (
      (result.recoverableDisplayPath !== null) !==
      (result.diagnosticCode === "worktree-remains")
    ) {
      context.addIssue({
        code: "custom",
        message: "Recoverable paths are only reported for retained worktrees",
      });
    }
  });

export type WorktreeCreatePreviewRequest = z.infer<
  typeof worktreeCreatePreviewRequestSchema
>;
export type WorktreeConfirmationRequest = z.infer<
  typeof worktreeConfirmationRequestSchema
>;
export type WorktreePreviewSnapshot = z.infer<typeof worktreePreviewSchema>;
export type WorktreeResultSnapshot = z.infer<typeof worktreeResultSchema>;
export type WorktreeWorkspaceSnapshot = z.infer<typeof worktreeWorkspaceSchema>;

export const scaffoldWorktreeWorkspace =
  worktreeWorkspaceSchema.parse(worktreeFixture);
