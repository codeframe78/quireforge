import { describe, expect, it } from "vitest";

import {
  modelSelectionPolicySchema,
  modelSelectionSnapshotSchema,
  modelSelectionUpdateRequestSchema,
} from "./modelSelection";

const choice = {
  modelId: "gpt-5.6-sol",
  reasoningEffort: "xhigh",
};

describe("model selection contracts", () => {
  it("accepts visible effective, pending, provenance, and bounded policy state", () => {
    const snapshot = modelSelectionSnapshotSchema.parse({
      schemaVersion: 1,
      availability: "ready",
      effective: choice,
      pending: {
        choice: { modelId: "gpt-5.6-terra", reasoningEffort: "high" },
        provenance: "codex",
        application: "recommendation",
        rationale: "Use the bounded lower-latency option next.",
        requestedAtMs: 1,
      },
      policy: {
        ownership: "recommend",
        userLocked: false,
        allowedModelIds: [],
        reasoningCeiling: null,
      },
      diagnosticCode: null,
    });

    expect(snapshot.pending?.provenance).toBe("codex");
    expect(snapshot.pending?.application).toBe("recommendation");
  });

  it("requires an explicit automatic boundary and rejects unsafe provenance", () => {
    expect(
      modelSelectionPolicySchema.safeParse({
        ownership: "automatic",
        userLocked: false,
        allowedModelIds: [],
        reasoningCeiling: null,
      }).success,
    ).toBe(false);
    expect(
      modelSelectionSnapshotSchema.safeParse({
        schemaVersion: 1,
        availability: "ready",
        effective: choice,
        pending: {
          choice,
          provenance: "codex",
          application: "manual",
          rationale: "Hidden ownership change.",
          requestedAtMs: 1,
        },
        policy: {
          ownership: "manual",
          userLocked: false,
          allowedModelIds: [],
          reasoningCeiling: null,
        },
        diagnosticCode: null,
      }).success,
    ).toBe(false);
    expect(
      modelSelectionSnapshotSchema.safeParse({
        schemaVersion: 1,
        availability: "ready",
        effective: choice,
        pending: null,
        policy: {
          ownership: "automatic",
          userLocked: false,
          allowedModelIds: ["gpt-5.6-terra"],
          reasoningCeiling: "xhigh",
        },
        diagnosticCode: null,
      }).success,
    ).toBe(false);
  });

  it("accepts only a closed user update request", () => {
    const request = {
      conversationId: "018f0000-0000-7000-8000-000000000011",
      choice,
      policy: {
        ownership: "automatic" as const,
        userLocked: false,
        allowedModelIds: ["gpt-5.6-sol"],
        reasoningCeiling: "xhigh",
      },
      pendingAction: "keep" as const,
    };
    expect(modelSelectionUpdateRequestSchema.parse(request)).toEqual(request);
    expect(
      modelSelectionUpdateRequestSchema.safeParse({
        ...request,
        prompt: "do not forward",
      }).success,
    ).toBe(false);
  });
});
