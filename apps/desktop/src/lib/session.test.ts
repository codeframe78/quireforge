import fixture from "../../fixtures/session-lifecycle.json";
import { describe, expect, it } from "vitest";

import {
  conversationContinueRequestSchema,
  scaffoldSessionLifecycle,
  sessionLifecycleSchema,
} from "./session";

const conversationId = "018f0000-0000-7000-8000-000000000010";
const projectId = "018f0000-0000-7000-8000-000000000001";

describe("session lifecycle contract", () => {
  it("parses the shared normalized empty fixture", () => {
    expect(sessionLifecycleSchema.parse(fixture)).toEqual(
      scaffoldSessionLifecycle,
    );
  });

  it("accepts only an opaque app ID and bounded prompt for resume or fork", () => {
    expect(
      conversationContinueRequestSchema.parse({
        conversationId,
        prompt: "Continue with the verified project.",
      }),
    ).toEqual({
      conversationId,
      prompt: "Continue with the verified project.",
    });
    expect(() =>
      conversationContinueRequestSchema.parse({
        conversationId,
        prompt: "Continue.",
        threadId: "018f0000-0000-7000-8000-000000000020",
      }),
    ).toThrow();
    expect(() =>
      conversationContinueRequestSchema.parse({
        conversationId,
        prompt: "Continue.",
        cwd: "/private/raw/path",
      }),
    ).toThrow();
  });

  it("accepts bounded app-owned lifecycle metadata", () => {
    const parsed = sessionLifecycleSchema.parse({
      schemaVersion: 1,
      state: "ready",
      sessions: [
        {
          conversationId,
          projectId,
          parentConversationId: null,
          modelId: "gpt-5.6-sol",
          reasoningEffort: "high",
          sandboxMode: "read-only",
          approvalPolicy: "untrusted",
          state: "interrupted",
          createdAtMs: 1,
          updatedAtMs: 2,
        },
      ],
      diagnosticCode: null,
    });

    expect(parsed.sessions[0]?.conversationId).toBe(conversationId);
  });

  it("rejects raw protocol, path, transcript, and inconsistent metadata", () => {
    const valid = {
      schemaVersion: 1 as const,
      state: "ready" as const,
      sessions: [
        {
          conversationId,
          projectId,
          parentConversationId: null,
          modelId: "gpt-5.6-sol",
          reasoningEffort: "high",
          sandboxMode: "read-only" as const,
          approvalPolicy: "untrusted" as const,
          state: "completed" as const,
          createdAtMs: 1,
          updatedAtMs: 2,
        },
      ],
      diagnosticCode: null,
    };

    expect(() =>
      sessionLifecycleSchema.parse({
        ...valid,
        sessions: [{ ...valid.sessions[0], cwd: "/private/raw/path" }],
      }),
    ).toThrow();
    expect(() =>
      sessionLifecycleSchema.parse({
        ...valid,
        sessions: [{ ...valid.sessions[0], threadId: conversationId }],
      }),
    ).toThrow();
    expect(() =>
      sessionLifecycleSchema.parse({
        ...valid,
        sessions: [{ ...valid.sessions[0], transcript: "private" }],
      }),
    ).toThrow();
    expect(() =>
      sessionLifecycleSchema.parse({ ...valid, state: "empty" }),
    ).toThrow();
  });
});
