import gitDiffFixture from "../../fixtures/git-diff.json";
import gitMutationPreviewFixture from "../../fixtures/git-mutation-preview.json";
import gitMutationResultFixture from "../../fixtures/git-mutation-result.json";
import gitWorkspaceFixture from "../../fixtures/git-workspace.json";
import { z } from "zod";

const opaqueIdSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

const safeTextSchema = z
  .string()
  .max(4096)
  .refine((value) => !/\p{Cc}|[\u202a-\u202e\u2066-\u2069]/u.test(value));

export const gitPathSchema = safeTextSchema
  .min(1)
  .refine((value) => !value.startsWith("/") && !value.includes("\\"))
  .refine((value) =>
    value
      .split("/")
      .every((part) => part !== "" && part !== "." && part !== ".."),
  );

export const gitDiagnosticCodeSchema = z.enum([
  "project-not-found",
  "directory-unavailable",
  "identity-changed",
  "not-repository",
  "git-unavailable",
  "git-failed",
  "output-too-large",
  "invalid-path",
  "diff-unavailable",
  "mutation-unavailable",
  "read-only",
  "project-busy",
  "stale-preview",
  "confirmation-expired",
  "secret-detected",
  "unscannable-content",
  "identity-unavailable",
  "outside-attachment",
  "postcondition-failed",
  "recovery-unavailable",
]);

export const gitChangeKindSchema = z.enum([
  "modified",
  "added",
  "deleted",
  "renamed",
  "copied",
  "type-changed",
  "unmerged",
  "untracked",
]);

const gitBranchSchema = z
  .object({
    head: safeTextSchema.min(1).max(256).nullable(),
    upstream: safeTextSchema.min(1).max(256).nullable(),
    ahead: z.number().int().nonnegative(),
    behind: z.number().int().nonnegative(),
    detached: z.boolean(),
  })
  .strict();

const gitFileChangeSchema = z
  .object({
    path: gitPathSchema,
    previousPath: gitPathSchema.nullable(),
    staged: gitChangeKindSchema.nullable(),
    worktree: gitChangeKindSchema.nullable(),
    conflict: z.boolean(),
    submodule: z.boolean(),
    reviewable: z.boolean(),
  })
  .strict()
  .superRefine((change, context) => {
    if (change.staged === null && change.worktree === null) {
      context.addIssue({ code: "custom", message: "A change needs an area" });
    }
    if (
      change.conflict &&
      change.staged !== "unmerged" &&
      change.worktree !== "unmerged"
    ) {
      context.addIssue({
        code: "custom",
        message: "Conflict state is inconsistent",
      });
    }
  });

export const gitWorkspaceSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["clean", "ready", "unavailable"]),
    projectId: opaqueIdSchema.nullable(),
    branch: gitBranchSchema.nullable(),
    changes: z.array(gitFileChangeSchema).max(512),
    truncated: z.boolean(),
    diagnosticCode: gitDiagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((workspace, context) => {
    const unavailable = workspace.state === "unavailable";
    if (
      unavailable !== (workspace.diagnosticCode !== null) ||
      (unavailable &&
        (workspace.branch !== null || workspace.changes.length !== 0)) ||
      (!unavailable &&
        (workspace.projectId === null || workspace.branch === null)) ||
      (workspace.state === "clean" && workspace.changes.length !== 0) ||
      (workspace.state === "ready" && workspace.changes.length === 0)
    ) {
      context.addIssue({
        code: "custom",
        message: "Git workspace fields are inconsistent",
      });
    }
  });

export const gitDiffAreaSchema = z.enum(["staged", "worktree"]);

export const gitDiffRequestSchema = z
  .object({
    projectId: opaqueIdSchema,
    path: gitPathSchema,
    area: gitDiffAreaSchema,
  })
  .strict();

export const gitOpenFileRequestSchema = z
  .object({ projectId: opaqueIdSchema, path: gitPathSchema })
  .strict();

const gitDiffLineSchema = z
  .object({
    kind: z.enum(["hunk", "context", "addition", "deletion"]),
    oldLine: z.number().int().nonnegative().nullable(),
    newLine: z.number().int().nonnegative().nullable(),
    text: safeTextSchema,
  })
  .strict();

export const gitDiffSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["ready", "unavailable"]),
    projectId: opaqueIdSchema,
    path: gitPathSchema,
    area: gitDiffAreaSchema,
    kind: z.enum(["text", "binary"]).nullable(),
    lines: z.array(gitDiffLineSchema).max(1500),
    truncated: z.boolean(),
    diagnosticCode: gitDiagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((diff, context) => {
    const unavailable = diff.state === "unavailable";
    if (
      unavailable !== (diff.diagnosticCode !== null) ||
      (unavailable && (diff.kind !== null || diff.lines.length !== 0)) ||
      (!unavailable && diff.kind === null) ||
      (diff.kind === "binary" && diff.lines.length !== 0)
    ) {
      context.addIssue({
        code: "custom",
        message: "Git diff fields are inconsistent",
      });
    }
  });

export const gitMutationOperationSchema = z.enum([
  "stage",
  "unstage",
  "revert",
  "commit",
]);

export const gitMutationPreviewRequestSchema = z
  .object({
    projectId: opaqueIdSchema,
    operation: gitMutationOperationSchema,
    path: gitPathSchema.nullable(),
    message: safeTextSchema
      .min(1)
      .max(512)
      .refine((value) => value.trim() === value)
      .nullable(),
  })
  .strict()
  .superRefine((request, context) => {
    const commit = request.operation === "commit";
    if (
      (commit && (request.path !== null || request.message === null)) ||
      (!commit && (request.path === null || request.message !== null))
    ) {
      context.addIssue({
        code: "custom",
        message: "Git mutation request fields are inconsistent",
      });
    }
  });

export const gitMutationConfirmRequestSchema = z
  .object({ confirmationId: opaqueIdSchema })
  .strict();

export const gitRecoveryRequestSchema = z
  .object({ recoveryId: opaqueIdSchema })
  .strict();

const gitMutationTargetSchema = z
  .object({
    path: gitPathSchema,
    staged: gitChangeKindSchema.nullable(),
    worktree: gitChangeKindSchema.nullable(),
  })
  .strict();

const gitSecretFindingSchema = z
  .object({
    location: z.enum(["staged-file", "commit-message"]),
    path: gitPathSchema.nullable(),
    kind: z.enum([
      "forbidden-path",
      "private-key",
      "git-hub-token",
      "open-ai-api-key",
    ]),
  })
  .strict()
  .superRefine((finding, context) => {
    if ((finding.location === "staged-file") !== (finding.path !== null)) {
      context.addIssue({
        code: "custom",
        message: "Git secret finding location is inconsistent",
      });
    }
  });

export const gitMutationPreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["ready", "blocked", "unavailable"]),
    projectId: opaqueIdSchema,
    operation: gitMutationOperationSchema,
    path: gitPathSchema.nullable(),
    targets: z.array(gitMutationTargetSchema).max(512),
    destructive: z.boolean(),
    confirmationId: opaqueIdSchema.nullable(),
    secretFindings: z.array(gitSecretFindingSchema).max(64),
    diagnosticCode: gitDiagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((preview, context) => {
    const ready = preview.state === "ready";
    const commit = preview.operation === "commit";
    if (
      ready !== (preview.confirmationId !== null) ||
      ready === (preview.diagnosticCode !== null) ||
      (ready && preview.targets.length === 0) ||
      (ready &&
        !commit &&
        (preview.targets.length !== 1 ||
          preview.targets[0]?.path !== preview.path)) ||
      preview.destructive !== (preview.operation === "revert") ||
      commit !== (preview.path === null) ||
      (preview.state !== "blocked" && preview.secretFindings.length !== 0)
    ) {
      context.addIssue({
        code: "custom",
        message: "Git mutation preview fields are inconsistent",
      });
    }
  });

export const gitMutationResultSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["applied", "unavailable"]),
    projectId: opaqueIdSchema.nullable(),
    operation: gitMutationOperationSchema.nullable(),
    recoveryId: opaqueIdSchema.nullable(),
    workspace: gitWorkspaceSchema.nullable(),
    diagnosticCode: gitDiagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((result, context) => {
    const applied = result.state === "applied";
    if (
      applied !== (result.diagnosticCode === null) ||
      (applied &&
        (result.projectId === null ||
          result.operation === null ||
          result.workspace === null)) ||
      (!applied && result.workspace !== null) ||
      (!applied && result.recoveryId !== null) ||
      (result.workspace !== null &&
        result.workspace.projectId !== result.projectId) ||
      (result.recoveryId !== null && result.operation !== "revert")
    ) {
      context.addIssue({
        code: "custom",
        message: "Git mutation result fields are inconsistent",
      });
    }
  });

export type GitWorkspaceSnapshot = z.infer<typeof gitWorkspaceSchema>;
export type GitDiffSnapshot = z.infer<typeof gitDiffSchema>;
export type GitDiffRequest = z.infer<typeof gitDiffRequestSchema>;
export type GitOpenFileRequest = z.infer<typeof gitOpenFileRequestSchema>;
export type GitMutationOperation = z.infer<typeof gitMutationOperationSchema>;
export type GitMutationPreviewRequest = z.infer<
  typeof gitMutationPreviewRequestSchema
>;
export type GitMutationConfirmRequest = z.infer<
  typeof gitMutationConfirmRequestSchema
>;
export type GitRecoveryRequest = z.infer<typeof gitRecoveryRequestSchema>;
export type GitMutationPreviewSnapshot = z.infer<
  typeof gitMutationPreviewSchema
>;
export type GitMutationResultSnapshot = z.infer<typeof gitMutationResultSchema>;

export const scaffoldGitWorkspace =
  gitWorkspaceSchema.parse(gitWorkspaceFixture);
export const scaffoldGitDiff = gitDiffSchema.parse(gitDiffFixture);
export const scaffoldGitMutationPreview = gitMutationPreviewSchema.parse(
  gitMutationPreviewFixture,
);
export const scaffoldGitMutationResult = gitMutationResultSchema.parse(
  gitMutationResultFixture,
);
