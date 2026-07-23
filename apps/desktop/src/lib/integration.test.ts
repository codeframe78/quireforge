import integrationFixture from "../../fixtures/integration-catalog.json";
import integrationMutationFixture from "../../fixtures/integration-mutation.json";
import { describe, expect, it } from "vitest";

import {
  integrationCatalogSchema,
  integrationMutationPreviewRequestSchema,
  integrationMutationPreviewSchema,
  integrationMutationResultSchema,
  scaffoldIntegrationCatalog,
  scaffoldIntegrationMutationPreview,
  scaffoldIntegrationMutationResult,
} from "./integration";

describe("integration catalog contract", () => {
  it("parses the deterministic normalized fixture", () => {
    expect(integrationCatalogSchema.parse(integrationFixture)).toEqual(
      scaffoldIntegrationCatalog,
    );
    expect(scaffoldIntegrationCatalog.dynamicTool).toMatchObject({
      registrationMethod: "thread/start",
      invocationMethod: "item/tool/call",
      currentTurnModelMutable: false,
    });
    expect(
      scaffoldIntegrationCatalog.capabilities.every(
        (capability) => capability.implementation === "contract-only",
      ),
    ).toBe(true);
    expect(scaffoldIntegrationCatalog.scheduledTasks).toEqual([
      expect.objectContaining({
        sourcePluginId: "plugin:fixture-review",
        promptTruncated: false,
        schedule: {
          type: "weekly",
          days: ["MO", "TH"],
          time: "09:30",
        },
      }),
    ]);
  });

  it("preserves blocked, degraded, and unknown states without raw payloads", () => {
    expect(
      scaffoldIntegrationCatalog.entries.map((entry) => entry.health.state),
    ).toEqual(expect.arrayContaining(["ready", "degraded", "blocked"]));
    expect(
      scaffoldIntegrationCatalog.entries.flatMap((entry) =>
        entry.requirements.map((requirement) => requirement.state),
      ),
    ).toContain("unknown");

    expect(() =>
      integrationCatalogSchema.parse({
        ...integrationFixture,
        accountId: "raw-account-identity",
      }),
    ).toThrow();
    expect(() =>
      integrationCatalogSchema.parse({
        ...integrationFixture,
        scheduledTasks: [
          {
            ...integrationFixture.scheduledTasks[0],
            sourcePluginId: "plugin:missing",
          },
        ],
      }),
    ).toThrow();
    expect(() =>
      integrationCatalogSchema.parse({
        ...integrationFixture,
        scheduledTasks: [
          {
            ...integrationFixture.scheduledTasks[0],
            schedule: { type: "daily", time: "24:00" },
          },
        ],
      }),
    ).toThrow();
    expect(() =>
      integrationCatalogSchema.parse({
        ...integrationFixture,
        entries: [
          {
            ...integrationFixture.entries[0],
            rawProtocolPayload: { authorizationUrl: "https://example.invalid" },
          },
        ],
      }),
    ).toThrow();
  });

  it("rejects dangling capabilities and unsafe mutation contracts", () => {
    expect(() =>
      integrationCatalogSchema.parse({
        ...integrationFixture,
        entries: [
          {
            ...integrationFixture.entries[0],
            capabilityIds: ["missing.capability"],
          },
        ],
      }),
    ).toThrow();

    expect(() =>
      integrationCatalogSchema.parse({
        ...integrationFixture,
        capabilities: integrationFixture.capabilities.map((capability) =>
          capability.id === "plugin.install"
            ? { ...capability, requiresConfirmation: false }
            : capability,
        ),
      }),
    ).toThrow();

    expect(() =>
      integrationCatalogSchema.parse({
        ...integrationFixture,
        dynamicTool: {
          ...integrationFixture.dynamicTool,
          currentTurnModelMutable: true,
        },
      }),
    ).toThrow();
  });
});

describe("integration mutation contract", () => {
  it("parses the deterministic preview and result fixture", () => {
    expect(
      integrationMutationPreviewSchema.parse(
        integrationMutationFixture.preview,
      ),
    ).toEqual(scaffoldIntegrationMutationPreview);
    expect(
      integrationMutationResultSchema.parse(integrationMutationFixture.result),
    ).toEqual(scaffoldIntegrationMutationResult);
  });

  it("accepts only operation-specific normalized requests", () => {
    expect(
      integrationMutationPreviewRequestSchema.parse({
        operation: "plugin-install",
        targetEntryId: "plugin:fixture-review",
        repository: null,
        reference: null,
      }),
    ).toBeDefined();
    expect(
      integrationMutationPreviewRequestSchema.parse({
        operation: "marketplace-add",
        targetEntryId: null,
        repository: "openai/example",
        reference: "a".repeat(40),
      }),
    ).toBeDefined();
    expect(() =>
      integrationMutationPreviewRequestSchema.parse({
        operation: "plugin-remove",
        targetEntryId: "plugin:fixture-review",
        repository: "openai/example",
        reference: null,
      }),
    ).toThrow();
  });

  it("rejects raw source evidence and inconsistent outcomes", () => {
    expect(() =>
      integrationMutationPreviewSchema.parse({
        ...integrationMutationFixture.preview,
        sourceUrl: "https://user:secret@example.invalid/plugin",
      }),
    ).toThrow();
    expect(() =>
      integrationMutationPreviewSchema.parse({
        ...integrationMutationFixture.preview,
        destructive: true,
      }),
    ).toThrow();
    expect(() =>
      integrationMutationResultSchema.parse({
        ...integrationMutationFixture.result,
        catalogRefreshRequired: false,
      }),
    ).toThrow();
  });
});
