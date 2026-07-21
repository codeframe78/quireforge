import terminalRegistryFixture from "../../fixtures/terminal-registry.json";
import { z } from "zod";

const opaqueIdSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

const terminalStateSchema = z.enum([
  "running",
  "closing",
  "exited",
  "interrupted",
  "failed",
  "unavailable",
]);

const terminalDiagnosticCodeSchema = z.enum([
  "invalid-request",
  "capacity-reached",
  "terminal-not-found",
  "project-unavailable",
  "project-identity-changed",
  "project-not-writable",
  "project-busy",
  "metadata-unavailable",
  "pty-unavailable",
  "shell-unavailable",
  "input-too-large",
  "input-unavailable",
  "resize-unavailable",
  "output-unavailable",
  "cleanup-incomplete",
]);

const terminalOutputChunkSchema = z
  .object({
    sequence: z.number().int().positive(),
    dataBase64: z
      .string()
      .min(1)
      .max(16_384)
      .regex(
        /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/u,
      ),
  })
  .strict();

export const terminalSnapshotSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: terminalStateSchema,
    terminalId: opaqueIdSchema.nullable(),
    projectId: opaqueIdSchema.nullable(),
    title: z
      .string()
      .min(1)
      .max(80)
      .refine((value) => !/\p{Cc}/u.test(value))
      .nullable(),
    live: z.boolean(),
    columns: z.number().int().min(0).max(500),
    rows: z.number().int().min(0).max(200),
    output: z.array(terminalOutputChunkSchema).max(64),
    firstSequence: z.number().int().nonnegative(),
    lastSequence: z.number().int().nonnegative(),
    truncated: z.boolean(),
    hasMore: z.boolean(),
    exitCode: z.number().int().nullable(),
    diagnosticCode: terminalDiagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((terminal, context) => {
    const unavailable = terminal.state === "unavailable";
    if (
      unavailable &&
      (terminal.terminalId !== null ||
        terminal.title !== null ||
        terminal.live ||
        terminal.columns !== 0 ||
        terminal.rows !== 0 ||
        terminal.output.length !== 0 ||
        terminal.diagnosticCode === null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Unavailable terminal fields are inconsistent",
      });
    }
    if (
      !unavailable &&
      (terminal.terminalId === null ||
        terminal.projectId === null ||
        terminal.title === null ||
        terminal.columns < 2 ||
        terminal.rows < 2)
    ) {
      context.addIssue({
        code: "custom",
        message: "Available terminal identity is incomplete",
      });
    }
    const sequences = terminal.output.map((chunk) => chunk.sequence);
    if (
      sequences.some(
        (sequence, index) =>
          sequence > terminal.lastSequence ||
          (index > 0 && sequence <= sequences[index - 1]!),
      )
    ) {
      context.addIssue({
        code: "custom",
        message: "Terminal output sequence is invalid",
      });
    }
    if (
      terminal.firstSequence > terminal.lastSequence &&
      terminal.lastSequence !== 0
    ) {
      context.addIssue({
        code: "custom",
        message: "Terminal output bounds are invalid",
      });
    }
  });

export const terminalRegistrySchema = z
  .object({
    schemaVersion: z.literal(1),
    capacity: z.literal(8),
    terminals: z.array(terminalSnapshotSchema).max(8),
    diagnosticCode: terminalDiagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((registry, context) => {
    const ids = registry.terminals.flatMap((terminal) =>
      terminal.terminalId ? [terminal.terminalId] : [],
    );
    if (new Set(ids).size !== ids.length) {
      context.addIssue({
        code: "custom",
        message: "Terminal IDs must be unique",
      });
    }
    if (registry.diagnosticCode !== null && registry.terminals.length !== 0) {
      context.addIssue({
        code: "custom",
        message: "Unavailable registry must not mix terminal records",
      });
    }
  });

export const terminalStartRequestSchema = z
  .object({
    projectId: opaqueIdSchema,
    columns: z.number().int().min(2).max(500),
    rows: z.number().int().min(2).max(200),
  })
  .strict();

export const terminalPollRequestSchema = z
  .object({
    terminalId: opaqueIdSchema,
    afterSequence: z.number().int().nonnegative(),
  })
  .strict();

export const terminalWriteRequestSchema = z
  .object({
    terminalId: opaqueIdSchema,
    dataBase64: z
      .string()
      .min(1)
      .max(87_384)
      .regex(
        /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/u,
      ),
  })
  .strict();

export const terminalResizeRequestSchema = z
  .object({
    terminalId: opaqueIdSchema,
    columns: z.number().int().min(2).max(500),
    rows: z.number().int().min(2).max(200),
  })
  .strict();

export const terminalCloseRequestSchema = z
  .object({ terminalId: opaqueIdSchema })
  .strict();

export type TerminalSnapshot = z.infer<typeof terminalSnapshotSchema>;
export type TerminalRegistrySnapshot = z.infer<typeof terminalRegistrySchema>;
export type TerminalStartRequest = z.infer<typeof terminalStartRequestSchema>;
export type TerminalPollRequest = z.infer<typeof terminalPollRequestSchema>;
export type TerminalWriteRequest = z.infer<typeof terminalWriteRequestSchema>;
export type TerminalResizeRequest = z.infer<typeof terminalResizeRequestSchema>;
export type TerminalCloseRequest = z.infer<typeof terminalCloseRequestSchema>;

export const scaffoldTerminalRegistry = terminalRegistrySchema.parse(
  terminalRegistryFixture,
);
