import { describe, expect, it } from "vitest";

import type { ConversationEvent } from "./conversation";
import {
  buildConversationActivityViews,
  mergeConversationEvents,
} from "./conversationView";

describe("conversation event view", () => {
  it("deduplicates and orders batches by bounded sequence", () => {
    const current: ConversationEvent[] = [
      { type: "lifecycle", sequence: 1, phase: "starting" },
      { type: "lifecycle", sequence: 2, phase: "running" },
    ];
    const incoming: ConversationEvent[] = [
      { type: "lifecycle", sequence: 2, phase: "running" },
      { type: "agent-message-delta", sequence: 3, delta: "Ready." },
    ];

    expect(
      mergeConversationEvents(current, incoming).map(
        ({ sequence }) => sequence,
      ),
    ).toEqual([1, 2, 3]);
  });

  it("retains only the most recent 256 normalized events", () => {
    const incoming: ConversationEvent[] = Array.from(
      { length: 300 },
      (_, index) => ({
        type: "lifecycle" as const,
        sequence: index + 1,
        phase: "running" as const,
      }),
    );

    const result = mergeConversationEvents([], incoming);
    expect(result).toHaveLength(256);
    expect(result[0]?.sequence).toBe(45);
    expect(result.at(-1)?.sequence).toBe(300);
  });

  it("groups lifecycle updates and output under one stable activity", () => {
    const activityId = "018f0000-0000-7000-8000-000000000011";
    const events: ConversationEvent[] = [
      {
        type: "activity",
        sequence: 2,
        activityId,
        kind: "command-execution",
        status: "started",
        title: "Run checks",
        detail: "pnpm check",
        exitCode: null,
      },
      {
        type: "activity-output-delta",
        sequence: 3,
        activityId,
        delta: "Checking…\n",
      },
      {
        type: "activity",
        sequence: 4,
        activityId,
        kind: "command-execution",
        status: "completed",
        title: "Run checks",
        detail: null,
        exitCode: 0,
      },
    ];

    expect(buildConversationActivityViews(events)).toEqual([
      {
        activityId,
        kind: "command-execution",
        title: "Run checks",
        detail: "pnpm check",
        status: "completed",
        exitCode: 0,
        output: "Checking…\n",
        firstSequence: 2,
        lastSequence: 4,
      },
    ]);
  });

  it("keeps a bounded output tail when the activity start is no longer retained", () => {
    const activityId = "018f0000-0000-7000-8000-000000000012";
    const result = buildConversationActivityViews([
      {
        type: "activity-output-delta",
        sequence: 7,
        activityId,
        delta: "a".repeat(40 * 1024),
      },
    ]);

    expect(result[0]).toMatchObject({
      activityId,
      kind: "other",
      title: "Activity progress",
      status: "started",
      firstSequence: 7,
    });
    expect(result[0]?.output).toHaveLength(32 * 1024);
    expect(result[0]?.output).toMatch(/^… earlier output omitted …\n/u);
  });

  it("ignores unrelated events and retains at most 64 activity views", () => {
    const events: ConversationEvent[] = [
      { type: "lifecycle", sequence: 1, phase: "running" },
      ...Array.from({ length: 70 }, (_, index) => ({
        type: "activity" as const,
        sequence: index + 2,
        activityId: `018f0000-0000-7000-8000-${String(index).padStart(12, "0")}`,
        kind: "tool-call" as const,
        status: "started" as const,
        title: `Tool ${index}`,
        detail: null,
        exitCode: null,
      })),
    ];

    const result = buildConversationActivityViews(events);
    expect(result).toHaveLength(64);
    expect(result[0]?.title).toBe("Tool 6");
    expect(result.at(-1)?.title).toBe("Tool 69");
  });
});
