import integrationFixture from "../../fixtures/integration-catalog.json";
import integrationMutationFixture from "../../fixtures/integration-mutation.json";
import { z } from "zod";

const identifierSchema = z
  .string()
  .min(1)
  .max(128)
  .regex(/^[a-z0-9][a-z0-9._:-]*$/u);

const protocolIdentifierSchema = z
  .string()
  .min(1)
  .max(128)
  .regex(/^[A-Za-z0-9._:/-]+$/u);

function isUnsafeDisplayCharacter(character: string): boolean {
  const codePoint = character.codePointAt(0) ?? 0;
  return (
    codePoint <= 0x1f ||
    (codePoint >= 0x7f && codePoint <= 0x9f) ||
    (codePoint >= 0x200b && codePoint <= 0x200f) ||
    (codePoint >= 0x202a && codePoint <= 0x202e) ||
    (codePoint >= 0x2060 && codePoint <= 0x206f) ||
    codePoint === 0xfeff
  );
}

const displayTextSchema = (maximum: number) =>
  z
    .string()
    .min(1)
    .max(maximum)
    .refine(
      (value) => ![...value].some(isUnsafeDisplayCharacter),
      "Display text contains control or directional characters",
    );

const availabilitySchema = z.enum([
  "ready",
  "degraded",
  "blocked",
  "unavailable",
  "unknown",
]);

const diagnosticCodeSchema = identifierSchema.max(64);

const capabilitySchema = z
  .object({
    id: identifierSchema,
    domain: z.enum([
      "connector",
      "plugin",
      "marketplace",
      "skill",
      "mcp",
      "policy",
      "dynamic-tool",
    ]),
    operation: z.enum([
      "discover",
      "inspect",
      "install",
      "remove",
      "configure",
      "authorize",
      "health",
      "invoke",
    ]),
    route: z.enum(["app-server", "cli", "native"]),
    stability: z.enum([
      "stable",
      "stable-method-experimental-server",
      "experimental",
    ]),
    availability: availabilitySchema,
    implementation: z.enum(["contract-only", "ready", "unsupported"]),
    mutating: z.boolean(),
    requiresConfirmation: z.boolean(),
    diagnosticCode: diagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((capability, context) => {
    if (capability.mutating !== capability.requiresConfirmation) {
      context.addIssue({
        code: "custom",
        message: "Mutating integration capabilities require confirmation",
        path: ["requiresConfirmation"],
      });
    }

    if (
      (capability.availability === "ready" &&
        capability.diagnosticCode !== null) ||
      ((["degraded", "blocked", "unavailable"] as const).includes(
        capability.availability as "degraded" | "blocked" | "unavailable",
      ) &&
        capability.diagnosticCode === null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Integration capability diagnostics are inconsistent",
        path: ["diagnosticCode"],
      });
    }
  });

const permissionSchema = z
  .object({
    kind: z.enum([
      "filesystem",
      "network",
      "account",
      "tool",
      "hook",
      "unknown",
    ]),
    access: z.enum([
      "read",
      "write",
      "execute",
      "authorize",
      "connect",
      "unknown",
    ]),
    target: displayTextSchema(160),
    required: z.boolean(),
  })
  .strict();

const requirementSchema = z
  .object({
    kind: z.enum([
      "binary",
      "configuration",
      "network",
      "platform",
      "policy",
      "authentication",
      "unknown",
    ]),
    name: displayTextSchema(128),
    state: z.enum(["satisfied", "missing", "blocked", "unknown"]),
    detail: displayTextSchema(240).nullable(),
  })
  .strict()
  .superRefine((requirement, context) => {
    if (
      (requirement.state === "missing" || requirement.state === "blocked") &&
      requirement.detail === null
    ) {
      context.addIssue({
        code: "custom",
        message: "Missing or blocked requirements need a normalized detail",
        path: ["detail"],
      });
    }
  });

const entrySchema = z
  .object({
    id: identifierSchema,
    kind: z.enum(["connector", "plugin", "marketplace", "skill", "mcp-server"]),
    displayName: displayTextSchema(128),
    summary: displayTextSchema(320),
    scope: z.enum([
      "account",
      "user",
      "project",
      "managed",
      "remote",
      "unknown",
    ]),
    source: z.enum([
      "official",
      "marketplace",
      "local",
      "repository",
      "configuration",
      "unknown",
    ]),
    installation: z.enum([
      "available",
      "installed",
      "not-applicable",
      "unknown",
    ]),
    enablement: z.enum([
      "enabled",
      "disabled",
      "blocked",
      "not-applicable",
      "unknown",
    ]),
    authentication: z.enum([
      "connected",
      "not-connected",
      "required",
      "not-applicable",
      "unknown",
    ]),
    version: protocolIdentifierSchema.max(64).nullable(),
    publisher: displayTextSchema(128).nullable(),
    capabilityIds: z.array(identifierSchema).max(32),
    permissions: z.array(permissionSchema).max(64),
    requirements: z.array(requirementSchema).max(64),
    policy: z
      .object({
        state: z.enum(["allowed", "approval-required", "blocked", "unknown"]),
        managed: z.boolean(),
        reason: displayTextSchema(240).nullable(),
      })
      .strict(),
    health: z
      .object({
        state: availabilitySchema,
        diagnosticCodes: z.array(diagnosticCodeSchema).max(16),
      })
      .strict(),
  })
  .strict()
  .superRefine((entry, context) => {
    if (new Set(entry.capabilityIds).size !== entry.capabilityIds.length) {
      context.addIssue({
        code: "custom",
        message: "Integration entry capability IDs must be unique",
        path: ["capabilityIds"],
      });
    }
    const diagnosticsExpected =
      entry.health.state !== "ready" && entry.health.state !== "unknown";
    if (
      (entry.health.state === "ready" &&
        entry.health.diagnosticCodes.length !== 0) ||
      (diagnosticsExpected && entry.health.diagnosticCodes.length === 0)
    ) {
      context.addIssue({
        code: "custom",
        message: "Integration health diagnostics are inconsistent",
        path: ["health", "diagnosticCodes"],
      });
    }
  });

const dynamicToolSchema = z
  .object({
    state: availabilitySchema,
    route: z.literal("app-server"),
    registrationMethod: z.literal("thread/start"),
    invocationMethod: z.literal("item/tool/call"),
    responseCorrelation: z.literal("json-rpc-request-id"),
    registrationScope: z.literal("thread"),
    supportsNamespaces: z.boolean(),
    outputContentKinds: z
      .array(z.enum(["text", "image", "audio"]))
      .min(1)
      .max(3),
    currentTurnModelMutable: z.literal(false),
    diagnosticCode: diagnosticCodeSchema.nullable(),
  })
  .strict()
  .superRefine((contract, context) => {
    if (
      new Set(contract.outputContentKinds).size !==
      contract.outputContentKinds.length
    ) {
      context.addIssue({
        code: "custom",
        message: "Dynamic-tool output content kinds must be unique",
        path: ["outputContentKinds"],
      });
    }
    if (
      (contract.state === "ready" && contract.diagnosticCode !== null) ||
      (contract.state !== "ready" &&
        contract.state !== "unknown" &&
        contract.diagnosticCode === null)
    ) {
      context.addIssue({
        code: "custom",
        message: "Dynamic-tool diagnostics are inconsistent",
        path: ["diagnosticCode"],
      });
    }
  });

export const integrationCatalogSchema = z
  .object({
    schemaVersion: z.literal(1),
    adapterVersion: z.literal("codex-integration-v1"),
    cliVersion: z
      .string()
      .max(32)
      .regex(/^\d+\.\d+\.\d+(?:[-+][A-Za-z0-9.-]+)?$/u),
    catalogState: availabilitySchema,
    capabilities: z.array(capabilitySchema).max(128),
    entries: z.array(entrySchema).max(512),
    policy: z
      .object({
        state: availabilitySchema,
        source: z.enum([
          "config-requirements",
          "user-configuration",
          "unknown",
        ]),
        permissionProfiles: availabilitySchema,
        managedRequirementsPresent: z.boolean(),
        mutationConfirmationRequired: z.literal(true),
        installation: z.enum([
          "allowed",
          "approval-required",
          "blocked",
          "unknown",
        ]),
      })
      .strict(),
    dynamicTool: dynamicToolSchema,
    refreshReasons: z
      .array(
        z.enum([
          "app-list-updated",
          "skills-changed",
          "mcp-status-updated",
          "config-warning",
        ]),
      )
      .max(4),
  })
  .strict()
  .superRefine((snapshot, context) => {
    const capabilityIds = new Set(
      snapshot.capabilities.map((capability) => capability.id),
    );
    const entryIds = new Set(snapshot.entries.map((entry) => entry.id));

    if (capabilityIds.size !== snapshot.capabilities.length) {
      context.addIssue({
        code: "custom",
        message: "Integration capability IDs must be unique",
        path: ["capabilities"],
      });
    }
    if (entryIds.size !== snapshot.entries.length) {
      context.addIssue({
        code: "custom",
        message: "Integration entry IDs must be unique",
        path: ["entries"],
      });
    }

    snapshot.entries.forEach((entry, entryIndex) => {
      entry.capabilityIds.forEach((capabilityId, capabilityIndex) => {
        if (!capabilityIds.has(capabilityId)) {
          context.addIssue({
            code: "custom",
            message: "Integration entry references an unknown capability",
            path: ["entries", entryIndex, "capabilityIds", capabilityIndex],
          });
        }
      });
    });

    if (
      snapshot.catalogState === "ready" &&
      snapshot.capabilities.some(
        (capability) => capability.availability !== "ready",
      )
    ) {
      context.addIssue({
        code: "custom",
        message: "A ready catalog cannot contain degraded capabilities",
        path: ["catalogState"],
      });
    }

    const dynamicCapability = snapshot.capabilities.find(
      (capability) => capability.id === "dynamic-tool.lifecycle",
    );
    if (
      snapshot.dynamicTool.state === "ready" &&
      (dynamicCapability?.availability !== "ready" ||
        dynamicCapability.implementation !== "contract-only")
    ) {
      context.addIssue({
        code: "custom",
        message:
          "Dynamic-tool evidence must remain contract-only in Milestone 13A",
        path: ["dynamicTool"],
      });
    }
  });

export type IntegrationCatalogSnapshot = z.infer<
  typeof integrationCatalogSchema
>;

export const scaffoldIntegrationCatalog =
  integrationCatalogSchema.parse(integrationFixture);

export const integrationMutationOperationSchema = z.enum([
  "plugin-install",
  "plugin-remove",
  "marketplace-add",
  "marketplace-remove",
  "marketplace-upgrade",
]);

const integrationMutationWarningSchema = z.enum([
  "local-source",
  "repository-source",
  "package-registry-source",
  "network-access",
  "hook-execution",
  "mcp-servers",
  "connector-apps",
  "skill-content",
  "authentication-after-install",
  "mutable-remote-source",
  "removes-cached-plugin",
  "removes-marketplace-snapshot",
  "updates-marketplace-snapshot",
]);

const integrationMutationDiagnosticSchema = z.enum([
  "invalid-request",
  "cli-unavailable",
  "version-unsupported",
  "catalog-unavailable",
  "target-not-found",
  "operation-unavailable",
  "policy-blocked",
  "source-invalid",
  "source-unpinned",
  "source-unreviewable",
  "capacity-reached",
  "confirmation-expired",
  "stale-preview",
  "mutation-failed",
  "response-invalid",
  "postcondition-failed",
]);

const confirmationIdSchema = z
  .string()
  .regex(
    /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/u,
  );

const repositorySchema = z
  .string()
  .min(3)
  .max(160)
  .regex(/^[A-Za-z0-9][A-Za-z0-9._-]*\/[A-Za-z0-9][A-Za-z0-9._-]*$/u);

export const integrationMutationPreviewRequestSchema = z
  .object({
    operation: integrationMutationOperationSchema,
    targetEntryId: identifierSchema.nullable(),
    repository: repositorySchema.nullable(),
    reference: z.string().min(1).max(64).nullable(),
  })
  .strict()
  .superRefine((request, context) => {
    const addsMarketplace = request.operation === "marketplace-add";
    const addFieldsPresent =
      request.repository !== null && request.reference !== null;
    if (
      (addsMarketplace &&
        (request.targetEntryId !== null || !addFieldsPresent)) ||
      (!addsMarketplace &&
        (request.targetEntryId === null ||
          request.repository !== null ||
          request.reference !== null))
    ) {
      context.addIssue({
        code: "custom",
        message: "Integration mutation request fields are inconsistent",
      });
    }
  });

export const integrationMutationConfirmRequestSchema = z
  .object({ confirmationId: confirmationIdSchema })
  .strict();

export const integrationMutationPreviewSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["ready", "blocked", "unavailable"]),
    operation: integrationMutationOperationSchema,
    targetEntryId: identifierSchema.nullable(),
    targetDisplayName: displayTextSchema(128).nullable(),
    source: z.enum([
      "official",
      "marketplace",
      "local",
      "repository",
      "configuration",
      "unknown",
    ]),
    permissions: z.array(permissionSchema).max(64),
    warnings: z.array(integrationMutationWarningSchema).max(12),
    destructive: z.boolean(),
    confirmationId: confirmationIdSchema.nullable(),
    diagnosticCode: integrationMutationDiagnosticSchema.nullable(),
  })
  .strict()
  .superRefine((preview, context) => {
    const ready = preview.state === "ready";
    const removes =
      preview.operation === "plugin-remove" ||
      preview.operation === "marketplace-remove";
    const addsMarketplace = preview.operation === "marketplace-add";
    if (
      preview.destructive !== removes ||
      (addsMarketplace
        ? preview.targetEntryId !== null
        : preview.targetEntryId === null) ||
      (ready &&
        (preview.confirmationId === null ||
          preview.diagnosticCode !== null ||
          preview.targetDisplayName === null)) ||
      (!ready &&
        (preview.confirmationId !== null ||
          preview.diagnosticCode === null ||
          preview.targetDisplayName !== null ||
          preview.source !== "unknown" ||
          preview.permissions.length !== 0 ||
          preview.warnings.length !== 0)) ||
      new Set(preview.warnings).size !== preview.warnings.length
    ) {
      context.addIssue({
        code: "custom",
        message: "Integration mutation preview fields are inconsistent",
      });
    }
  });

export const integrationMutationResultSchema = z
  .object({
    schemaVersion: z.literal(1),
    state: z.enum(["applied", "unavailable"]),
    operation: integrationMutationOperationSchema.nullable(),
    targetEntryId: identifierSchema.nullable(),
    catalogRefreshRequired: z.boolean(),
    diagnosticCode: integrationMutationDiagnosticSchema.nullable(),
  })
  .strict()
  .superRefine((result, context) => {
    const applied = result.state === "applied";
    if (
      (applied &&
        (result.operation === null ||
          result.targetEntryId === null ||
          !result.catalogRefreshRequired ||
          result.diagnosticCode !== null)) ||
      (!applied &&
        (result.catalogRefreshRequired || result.diagnosticCode === null))
    ) {
      context.addIssue({
        code: "custom",
        message: "Integration mutation result fields are inconsistent",
      });
    }
  });

export type IntegrationMutationOperation = z.infer<
  typeof integrationMutationOperationSchema
>;
export type IntegrationMutationPreviewRequest = z.infer<
  typeof integrationMutationPreviewRequestSchema
>;
export type IntegrationMutationConfirmRequest = z.infer<
  typeof integrationMutationConfirmRequestSchema
>;
export type IntegrationMutationPreviewSnapshot = z.infer<
  typeof integrationMutationPreviewSchema
>;
export type IntegrationMutationResultSnapshot = z.infer<
  typeof integrationMutationResultSchema
>;

export const scaffoldIntegrationMutationPreview =
  integrationMutationPreviewSchema.parse(integrationMutationFixture.preview);
export const scaffoldIntegrationMutationResult =
  integrationMutationResultSchema.parse(integrationMutationFixture.result);
