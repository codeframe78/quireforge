import integrationFixture from "../../fixtures/integration-catalog.json";
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
