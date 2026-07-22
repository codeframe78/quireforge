import integrationFixture from "../../fixtures/integration-catalog.json";
import { describe, expect, it } from "vitest";

import {
  integrationCatalogSchema,
  scaffoldIntegrationCatalog,
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
