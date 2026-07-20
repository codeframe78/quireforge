import fixture from "../../fixtures/conversation.json";
import { describe, expect, it } from "vitest";

import {
  conversationSnapshotSchema,
  conversationStartRequestSchema,
  scaffoldConversation,
} from "./conversation";

const conversationId = "018f0000-0000-7000-8000-000000000010";
const projectId = "018f0000-0000-7000-8000-000000000001";

describe("conversation contract", () => {
  it("parses the shared empty fixture", () => {
    expect(conversationSnapshotSchema.parse(fixture)).toEqual(
      scaffoldConversation,
    );
  });

  it("accepts only bounded explicit start controls", () => {
    expect(
      conversationStartRequestSchema.parse({
        projectId,
        prompt: "Inspect this project without changing files.",
        modelId: "gpt-5.6-sol",
        reasoningEffort: "high",
        sandboxMode: "read-only",
        approvalPolicy: "untrusted",
      }),
    ).toMatchObject({ projectId, sandboxMode: "read-only" });
  });

  it("rejects paths, protocol IDs, unknown fields, and an unsafe bypass combination", () => {
    const valid = {
      projectId,
      prompt: "Review the project.",
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      sandboxMode: "workspace-write" as const,
      approvalPolicy: "on-request" as const,
    };

    expect(() =>
      conversationStartRequestSchema.parse({
        ...valid,
        cwd: "/private/raw/path",
      }),
    ).toThrow();
    expect(() =>
      conversationStartRequestSchema.parse({ ...valid, threadId: projectId }),
    ).toThrow();
    expect(() =>
      conversationStartRequestSchema.parse({
        ...valid,
        sandboxMode: "danger-full-access",
        approvalPolicy: "never",
      }),
    ).toThrow();
  });

  it("accepts normalized ordered events without Codex protocol identity", () => {
    const snapshot = conversationSnapshotSchema.parse({
      schemaVersion: 1,
      state: "running",
      conversationId,
      projectId,
      modelId: "gpt-5.6-sol",
      reasoningEffort: "high",
      sandboxMode: "read-only",
      approvalPolicy: "untrusted",
      events: [
        { type: "lifecycle", sequence: 1, phase: "starting" },
        {
          type: "agent-message-delta",
          sequence: 2,
          delta: "Reviewing the project.",
        },
      ],
      diagnosticCode: null,
    });

    expect(snapshot.events).toHaveLength(2);
    expect(() =>
      conversationSnapshotSchema.parse({
        ...snapshot,
        threadId: "018f0000-0000-7000-8000-000000000020",
      }),
    ).toThrow();
    expect(() =>
      conversationSnapshotSchema.parse({
        ...snapshot,
        events: snapshot.events.toReversed(),
      }),
    ).toThrow();
  });
});
