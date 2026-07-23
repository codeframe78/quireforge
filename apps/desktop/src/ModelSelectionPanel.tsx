import { useMemo, useState } from "react";

import type { CodexRuntimeSnapshot } from "./lib/codex";
import {
  modelSelectionUpdateRequestSchema,
  type ModelSelectionChoice,
  type ModelSelectionPolicy,
  type ModelSelectionSnapshot,
  type ModelSelectionUpdateRequest,
} from "./lib/modelSelection";

type Model = CodexRuntimeSnapshot["models"][number];

interface ModelSelectionPolicyFieldsProps {
  idPrefix: string;
  policy: ModelSelectionPolicy;
  effectiveChoice: ModelSelectionChoice;
  models: Model[];
  disabled: boolean;
  onChange: (policy: ModelSelectionPolicy) => void;
}

const ownershipDescriptions: Record<ModelSelectionPolicy["ownership"], string> =
  {
    manual: "Only you can choose the model and reasoning for the next turn.",
    recommend:
      "Codex may propose one next-turn choice; you must accept it explicitly.",
    automatic:
      "Codex may stage one next-turn choice inside the limits you set below.",
  };

const reasoningOrder = [
  "none",
  "minimal",
  "low",
  "medium",
  "high",
  "xhigh",
  "max",
  "ultra",
];

function ensureAutomaticBoundary(
  policy: ModelSelectionPolicy,
  effectiveChoice: ModelSelectionChoice,
): ModelSelectionPolicy {
  if (policy.ownership !== "automatic") return policy;
  if (policy.allowedModelIds.includes(effectiveChoice.modelId)) return policy;
  return {
    ...policy,
    allowedModelIds: [
      ...policy.allowedModelIds.filter(
        (modelId) => modelId !== effectiveChoice.modelId,
      ),
      effectiveChoice.modelId,
    ].slice(-32),
  };
}

export function ModelSelectionPolicyFields({
  idPrefix,
  policy,
  effectiveChoice,
  models,
  disabled,
  onChange,
}: ModelSelectionPolicyFieldsProps) {
  const reasoningCeilings = useMemo(
    () =>
      reasoningOrder.filter(
        (effort) =>
          effort === policy.reasoningCeiling ||
          models.some((model) =>
            model.supportedReasoningEfforts.includes(effort),
          ),
      ),
    [models, policy.reasoningCeiling],
  );

  return (
    <fieldset className="model-selection-policy">
      <legend>Next-turn ownership</legend>
      <label>
        <span>Who chooses</span>
        <select
          aria-describedby={`${idPrefix}-ownership-help`}
          value={policy.ownership}
          disabled={disabled}
          onChange={(event) => {
            const ownership = event.target
              .value as ModelSelectionPolicy["ownership"];
            onChange(
              ensureAutomaticBoundary(
                { ...policy, ownership },
                effectiveChoice,
              ),
            );
          }}
        >
          <option value="manual">Manual</option>
          <option value="recommend">Recommend</option>
          <option value="automatic">Automatic within limits</option>
        </select>
      </label>
      <p id={`${idPrefix}-ownership-help`}>
        {ownershipDescriptions[policy.ownership]}
      </p>
      <label className="model-selection-policy__lock">
        <input
          type="checkbox"
          checked={policy.userLocked}
          disabled={disabled}
          onChange={(event) =>
            onChange({ ...policy, userLocked: event.target.checked })
          }
        />
        <span>Lock Codex selection requests</span>
      </label>

      {policy.ownership === "automatic" && (
        <div className="model-selection-policy__limits">
          <strong>Automatic limits</strong>
          <p>
            The current choice stays allowed. Codex can request only one listed
            model per turn, at or below the optional reasoning ceiling.
          </p>
          <div
            className="model-selection-policy__models"
            aria-label="Automatically allowed models"
          >
            {models.slice(0, 32).map((model) => {
              const current = model.id === effectiveChoice.modelId;
              return (
                <label key={model.id}>
                  <input
                    type="checkbox"
                    checked={
                      current || policy.allowedModelIds.includes(model.id)
                    }
                    disabled={disabled || current}
                    onChange={(event) => {
                      const allowedModelIds = event.target.checked
                        ? [...policy.allowedModelIds, model.id]
                        : policy.allowedModelIds.filter(
                            (modelId) => modelId !== model.id,
                          );
                      onChange({
                        ...policy,
                        allowedModelIds: [...new Set(allowedModelIds)].slice(
                          0,
                          32,
                        ),
                      });
                    }}
                  />
                  <span>
                    {model.displayName}
                    {current ? " — current" : ""}
                  </span>
                </label>
              );
            })}
          </div>
          <label>
            <span>Reasoning ceiling</span>
            <select
              value={policy.reasoningCeiling ?? ""}
              disabled={disabled}
              onChange={(event) =>
                onChange({
                  ...policy,
                  reasoningCeiling: event.target.value || null,
                })
              }
            >
              <option value="">No additional ceiling</option>
              {reasoningCeilings.map((effort) => (
                <option value={effort} key={effort}>
                  {effort}
                </option>
              ))}
            </select>
          </label>
        </div>
      )}
    </fieldset>
  );
}

interface ModelSelectionPanelProps {
  conversationId: string;
  selection: ModelSelectionSnapshot;
  models: Model[];
  disabled: boolean;
  onUpdate: (
    request: ModelSelectionUpdateRequest,
  ) => Promise<ModelSelectionSnapshot>;
}

function modelLabel(models: Model[], modelId: string): string {
  return models.find((model) => model.id === modelId)?.displayName ?? modelId;
}

export function ModelSelectionPanel({
  conversationId,
  selection,
  models,
  disabled,
  onUpdate,
}: ModelSelectionPanelProps) {
  const [choice, setChoice] = useState(selection.effective);
  const [policy, setPolicy] = useState(selection.policy);
  const [saving, setSaving] = useState(false);
  const [failed, setFailed] = useState(false);

  const selectedModel =
    models.find((model) => model.id === choice.modelId) ??
    models.find((model) => model.id === selection.effective.modelId);
  const effectiveChoice = selectedModel
    ? {
        modelId: selectedModel.id,
        reasoningEffort: selectedModel.supportedReasoningEfforts.includes(
          choice.reasoningEffort,
        )
          ? choice.reasoningEffort
          : selectedModel.defaultReasoningEffort,
      }
    : choice;
  const reviewedPolicy = ensureAutomaticBoundary(policy, effectiveChoice);
  const update = (
    pendingAction: ModelSelectionUpdateRequest["pendingAction"],
  ) =>
    ({
      conversationId,
      choice:
        pendingAction === "accept" && selection.pending
          ? selection.pending.choice
          : effectiveChoice,
      policy: reviewedPolicy,
      pendingAction,
    }) satisfies ModelSelectionUpdateRequest;
  const canSave =
    selection.availability !== "unavailable" &&
    models.length > 0 &&
    modelSelectionUpdateRequestSchema.safeParse(update("keep")).success &&
    !disabled &&
    !saving;

  async function submit(
    pendingAction: ModelSelectionUpdateRequest["pendingAction"],
  ) {
    const request = update(pendingAction);
    if (
      disabled ||
      saving ||
      !modelSelectionUpdateRequestSchema.safeParse(request).success
    )
      return;
    setSaving(true);
    setFailed(false);
    try {
      const result = await onUpdate(request);
      setChoice(
        result.pending?.provenance === "user"
          ? result.pending.choice
          : result.effective,
      );
      setPolicy(result.policy);
    } catch {
      setFailed(true);
    } finally {
      setSaving(false);
    }
  }

  return (
    <section
      className="model-selection-panel"
      aria-labelledby={`model-selection-${conversationId}`}
    >
      <div className="model-selection-panel__heading">
        <div>
          <p className="eyebrow">Next-turn selector</p>
          <h3 id={`model-selection-${conversationId}`}>Model ownership</h3>
        </div>
        <span data-availability={selection.availability}>
          {selection.availability === "ready"
            ? "Control ready"
            : selection.availability === "recommendation-only"
              ? "Recommendations only"
              : "Unavailable"}
        </span>
      </div>

      <dl className="model-selection-panel__state">
        <div>
          <dt>Effective now</dt>
          <dd>
            {modelLabel(models, selection.effective.modelId)} ·{" "}
            {selection.effective.reasoningEffort}
          </dd>
        </div>
        <div>
          <dt>Pending next turn</dt>
          <dd>
            {selection.pending
              ? `${modelLabel(models, selection.pending.choice.modelId)} · ${selection.pending.choice.reasoningEffort}`
              : "No change"}
          </dd>
        </div>
      </dl>

      {selection.pending && (
        <div className="model-selection-panel__pending">
          <strong>
            {selection.pending.provenance === "codex"
              ? "Requested by Codex"
              : "Chosen by you"}
          </strong>
          <span>
            {selection.pending.application === "recommendation"
              ? "Recommendation — never automatic"
              : selection.pending.application === "automatic"
                ? "Automatic within your limits"
                : "Manual next-turn choice"}
          </span>
          <p>{selection.pending.rationale}</p>
          {selection.pending.provenance === "codex" && (
            <div>
              {selection.pending.application === "recommendation" && (
                <button
                  type="button"
                  disabled={disabled || saving}
                  onClick={() => void submit("accept")}
                >
                  Accept recommendation
                </button>
              )}
              <button
                type="button"
                disabled={disabled || saving}
                onClick={() => void submit("dismiss")}
              >
                Dismiss
              </button>
            </div>
          )}
        </div>
      )}

      {selection.availability === "recommendation-only" && (
        <p className="model-selection-panel__notice">
          This Codex version rejected the app-owned control registration. Manual
          next-turn choices remain available.
        </p>
      )}

      <div className="model-selection-panel__choice">
        <label>
          <span>Next model</span>
          <select
            value={effectiveChoice.modelId}
            disabled={disabled || saving || models.length === 0}
            onChange={(event) => {
              const model = models.find(
                (candidate) => candidate.id === event.target.value,
              );
              if (!model) return;
              const nextChoice = {
                modelId: model.id,
                reasoningEffort: model.defaultReasoningEffort,
              };
              setChoice(nextChoice);
              setPolicy((current) =>
                ensureAutomaticBoundary(current, nextChoice),
              );
            }}
          >
            {models.map((model) => (
              <option value={model.id} key={model.id}>
                {model.displayName}
              </option>
            ))}
          </select>
        </label>
        <label>
          <span>Next reasoning</span>
          <select
            value={effectiveChoice.reasoningEffort}
            disabled={disabled || saving || !selectedModel}
            onChange={(event) =>
              setChoice({
                ...effectiveChoice,
                reasoningEffort: event.target.value,
              })
            }
          >
            {selectedModel?.supportedReasoningEfforts.map((effort) => (
              <option value={effort} key={effort}>
                {effort}
              </option>
            ))}
          </select>
        </label>
      </div>

      <ModelSelectionPolicyFields
        idPrefix={`model-selection-${conversationId}`}
        policy={reviewedPolicy}
        effectiveChoice={effectiveChoice}
        models={models}
        disabled={disabled || saving}
        onChange={setPolicy}
      />

      <div className="model-selection-panel__actions">
        <button
          type="button"
          disabled={!canSave}
          onClick={() => void submit("keep")}
        >
          {saving ? "Saving…" : "Save next-turn settings"}
        </button>
        <small>
          The running turn never replaces itself. Every pending choice is
          revalidated against a fresh model catalog before the next turn.
        </small>
      </div>
      {failed && (
        <p className="model-selection-panel__error" role="alert">
          The selector update was rejected. The effective choice was not
          changed.
        </p>
      )}
    </section>
  );
}
