import gitDiffFixture from "../../fixtures/git-diff.json";
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

const gitDiagnosticCodeSchema = z.enum([
  "project-not-found",
  "directory-unavailable",
  "identity-changed",
  "not-repository",
  "git-unavailable",
  "git-failed",
  "output-too-large",
  "invalid-path",
  "diff-unavailable",
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

export type GitWorkspaceSnapshot = z.infer<typeof gitWorkspaceSchema>;
export type GitDiffSnapshot = z.infer<typeof gitDiffSchema>;
export type GitDiffRequest = z.infer<typeof gitDiffRequestSchema>;
export type GitOpenFileRequest = z.infer<typeof gitOpenFileRequestSchema>;

export const scaffoldGitWorkspace =
  gitWorkspaceSchema.parse(gitWorkspaceFixture);
export const scaffoldGitDiff = gitDiffSchema.parse(gitDiffFixture);
