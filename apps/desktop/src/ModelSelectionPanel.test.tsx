import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ModelSelectionPanel } from "./ModelSelectionPanel";
import { scaffoldCodexRuntime } from "./lib/codex";
import type { ModelSelectionSnapshot } from "./lib/modelSelection";

const conversationId = "018f0000-0000-7000-8000-000000000010";
const models = [
  ...scaffoldCodexRuntime.models,
  {
    id: "gpt-5.6-terra",
    displayName: "GPT-5.6 Terra",
    isDefault: false,
    defaultReasoningEffort: "medium",
    supportedReasoningEfforts: ["medium", "high"],
  },
];
const effective = {
  modelId: "gpt-5.6-sol",
  reasoningEffort: "high",
};

function selection(
  overrides: Partial<ModelSelectionSnapshot> = {},
): ModelSelectionSnapshot {
  return {
    schemaVersion: 1,
    availability: "ready",
    effective,
    pending: null,
    policy: {
      ownership: "manual",
      userLocked: false,
      allowedModelIds: [],
      reasoningCeiling: null,
    },
    diagnosticCode: null,
    ...overrides,
  };
}

describe("ModelSelectionPanel", () => {
  it("keeps effective and pending choices distinct with visible provenance", async () => {
    const snapshot = selection({
      policy: {
        ownership: "recommend",
        userLocked: false,
        allowedModelIds: [],
        reasoningCeiling: null,
      },
      pending: {
        choice: {
          modelId: "gpt-5.6-terra",
          reasoningEffort: "high",
        },
        provenance: "codex",
        application: "recommendation",
        rationale: "Use the lower-latency option for the next turn.",
        requestedAtMs: 1,
      },
    });
    const onUpdate = vi.fn().mockResolvedValue(snapshot);
    render(
      <ModelSelectionPanel
        conversationId={conversationId}
        selection={snapshot}
        models={models}
        disabled={false}
        onUpdate={onUpdate}
      />,
    );

    expect(screen.getByText("Effective now").nextSibling).toHaveTextContent(
      "GPT-5.6-Sol · high",
    );
    expect(screen.getByText("Pending next turn").nextSibling).toHaveTextContent(
      "GPT-5.6 Terra · high",
    );
    expect(screen.getByText("Requested by Codex")).toBeInTheDocument();
    expect(
      screen.getByText("Recommendation — never automatic"),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "Save next-turn settings" }),
    );
    await waitFor(() =>
      expect(onUpdate).toHaveBeenCalledWith({
        conversationId,
        choice: effective,
        policy: snapshot.policy,
        pendingAction: "keep",
      }),
    );
    onUpdate.mockClear();

    fireEvent.click(
      screen.getByRole("button", { name: "Accept recommendation" }),
    );
    await waitFor(() =>
      expect(onUpdate).toHaveBeenCalledWith({
        conversationId,
        choice: {
          modelId: "gpt-5.6-terra",
          reasoningEffort: "high",
        },
        policy: snapshot.policy,
        pendingAction: "accept",
      }),
    );
  });

  it("requires explicit automatic limits and preserves the user lock", async () => {
    const snapshot = selection();
    const onUpdate = vi.fn().mockResolvedValue(snapshot);
    render(
      <ModelSelectionPanel
        conversationId={conversationId}
        selection={snapshot}
        models={models}
        disabled={false}
        onUpdate={onUpdate}
      />,
    );

    fireEvent.change(screen.getByLabelText("Who chooses"), {
      target: { value: "automatic" },
    });
    expect(
      screen.getByText(/Codex may stage one next-turn choice/iu),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("checkbox", { name: /GPT-5.6-Sol — current/u }),
    ).toBeChecked();
    fireEvent.click(screen.getByRole("checkbox", { name: "GPT-5.6 Terra" }));
    fireEvent.change(screen.getByLabelText("Reasoning ceiling"), {
      target: { value: "high" },
    });
    fireEvent.click(
      screen.getByRole("checkbox", {
        name: "Lock Codex selection requests",
      }),
    );
    fireEvent.click(
      screen.getByRole("button", { name: "Save next-turn settings" }),
    );

    await waitFor(() =>
      expect(onUpdate).toHaveBeenCalledWith(
        expect.objectContaining({
          conversationId,
          pendingAction: "keep",
          policy: {
            ownership: "automatic",
            userLocked: true,
            allowedModelIds: ["gpt-5.6-sol", "gpt-5.6-terra"],
            reasoningCeiling: "high",
          },
        }),
      ),
    );
  });

  it("explains the supported recommendation-only degradation", () => {
    render(
      <ModelSelectionPanel
        conversationId={conversationId}
        selection={selection({ availability: "recommendation-only" })}
        models={models}
        disabled={false}
        onUpdate={vi.fn()}
      />,
    );

    expect(
      screen.getByText(/rejected the app-owned control registration/iu),
    ).toBeInTheDocument();
    expect(screen.getByText("Recommendations only")).toBeInTheDocument();
  });
});
