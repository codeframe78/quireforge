import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
} from "react";

import {
  integrationMutationPreviewRequestSchema,
  type IntegrationCatalogSnapshot,
  type IntegrationMutationOperation,
  type IntegrationMutationPreviewRequest,
  type IntegrationMutationPreviewSnapshot,
  type IntegrationMutationResultSnapshot,
} from "./lib/integration";

type IntegrationAvailability = "checking" | "native" | "preview";
type IntegrationEntry = IntegrationCatalogSnapshot["entries"][number];
type EntryKind = IntegrationEntry["kind"];
type HealthState = IntegrationEntry["health"]["state"];

interface IntegrationCenterProps {
  availability: IntegrationAvailability;
  snapshot: IntegrationCatalogSnapshot;
  preview: IntegrationMutationPreviewSnapshot | null;
  result: IntegrationMutationResultSnapshot | null;
  busy: boolean;
  actionError: boolean;
  onRefresh: () => Promise<void>;
  onPreview: (request: IntegrationMutationPreviewRequest) => Promise<void>;
  onConfirm: (confirmationId: string) => Promise<void>;
  onCancel: () => void;
}

const kindLabels: Record<EntryKind, string> = {
  connector: "Connectors",
  plugin: "Plugins",
  marketplace: "Marketplaces",
  skill: "Skills",
  "mcp-server": "MCP servers",
};

const healthLabels: Record<HealthState, string> = {
  ready: "Ready",
  degraded: "Needs attention",
  blocked: "Blocked",
  unavailable: "Unavailable",
  unknown: "Unknown",
};

const operationLabels: Record<IntegrationMutationOperation, string> = {
  "plugin-install": "Install plugin",
  "plugin-remove": "Remove plugin",
  "marketplace-add": "Add marketplace",
  "marketplace-remove": "Remove marketplace",
  "marketplace-upgrade": "Update marketplace snapshot",
};

const warningLabels: Record<
  IntegrationMutationPreviewSnapshot["warnings"][number],
  string
> = {
  "local-source": "This plugin comes from a local directory.",
  "repository-source": "This operation uses a repository source.",
  "package-registry-source": "This operation uses a package registry.",
  "network-access": "The operation requires network access.",
  "hook-execution":
    "The plugin declares hooks. Installation does not grant hook trust.",
  "mcp-servers": "The plugin includes MCP server declarations.",
  "connector-apps": "The plugin includes connector app declarations.",
  "skill-content": "The plugin includes reusable skill content.",
  "authentication-after-install":
    "Authentication, if needed, remains a separate action.",
  "mutable-remote-source":
    "The next marketplace snapshot is remote and cannot be pinned from current catalog evidence.",
  "removes-cached-plugin": "This removes the cached plugin installation.",
  "removes-marketplace-snapshot":
    "This removes the configured marketplace snapshot.",
  "updates-marketplace-snapshot":
    "This replaces the current marketplace snapshot.",
};

function sentenceCase(value: string): string {
  const words = value.replaceAll("-", " ");
  return `${words.charAt(0).toUpperCase()}${words.slice(1)}`;
}

function capabilityReady(
  snapshot: IntegrationCatalogSnapshot,
  capabilityId: string,
): boolean {
  const capability = snapshot.capabilities.find(
    (candidate) => candidate.id === capabilityId,
  );
  return (
    capability?.availability === "ready" &&
    capability.implementation === "ready"
  );
}

function operationsFor(
  entry: IntegrationEntry,
  snapshot: IntegrationCatalogSnapshot,
): IntegrationMutationOperation[] {
  if (entry.policy.state === "blocked") return [];
  if (entry.kind === "plugin") {
    if (
      entry.installation === "available" &&
      entry.capabilityIds.includes("plugin.install") &&
      capabilityReady(snapshot, "plugin.install")
    ) {
      return ["plugin-install"];
    }
    if (
      entry.installation === "installed" &&
      entry.capabilityIds.includes("plugin.remove") &&
      capabilityReady(snapshot, "plugin.remove")
    ) {
      return ["plugin-remove"];
    }
  }
  if (
    entry.kind === "marketplace" &&
    entry.capabilityIds.includes("marketplace.configure") &&
    capabilityReady(snapshot, "marketplace.configure")
  ) {
    return ["marketplace-upgrade", "marketplace-remove"];
  }
  return [];
}

function statusSummary(entry: IntegrationEntry): string {
  if (entry.health.state !== "ready") return healthLabels[entry.health.state];
  if (entry.authentication === "required") return "Authorization required";
  if (entry.enablement === "disabled") return "Disabled";
  return "Ready";
}

export function IntegrationCenter({
  availability,
  snapshot,
  preview,
  result,
  busy,
  actionError,
  onRefresh,
  onPreview,
  onConfirm,
  onCancel,
}: IntegrationCenterProps) {
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState<"all" | EntryKind>("all");
  const [health, setHealth] = useState<"all" | HealthState>("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [repository, setRepository] = useState("");
  const [reference, setReference] = useState("");
  const dialogRef = useRef<HTMLDialogElement>(null);
  const dialogFocusRef = useRef<HTMLButtonElement>(null);

  const visibleEntries = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    return snapshot.entries.filter(
      (entry) =>
        (kind === "all" || entry.kind === kind) &&
        (health === "all" || entry.health.state === health) &&
        (!normalizedQuery ||
          entry.displayName.toLocaleLowerCase().includes(normalizedQuery) ||
          entry.summary.toLocaleLowerCase().includes(normalizedQuery) ||
          entry.publisher?.toLocaleLowerCase().includes(normalizedQuery)),
    );
  }, [health, kind, query, snapshot.entries]);

  const selectedEntry =
    visibleEntries.find((entry) => entry.id === selectedId) ??
    visibleEntries[0] ??
    null;
  const selectedOperations = selectedEntry
    ? operationsFor(selectedEntry, snapshot)
    : [];
  const marketplaceAddRequest = {
    operation: "marketplace-add" as const,
    targetEntryId: null,
    repository: repository.trim() || null,
    reference: reference.trim() || null,
  };
  const marketplaceAddReady =
    availability === "native" &&
    snapshot.policy.installation !== "blocked" &&
    capabilityReady(snapshot, "marketplace.configure") &&
    integrationMutationPreviewRequestSchema.safeParse(marketplaceAddRequest)
      .success;

  useEffect(() => {
    if (!preview) return;
    const previousFocus =
      document.activeElement instanceof HTMLElement
        ? document.activeElement
        : null;
    dialogFocusRef.current?.focus();
    return () => previousFocus?.focus();
  }, [preview]);

  useEffect(() => {
    if (!preview) return;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || busy) return;
      event.preventDefault();
      onCancel();
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [busy, onCancel, preview]);

  async function runAction(action: () => Promise<void>) {
    try {
      await action();
    } catch {
      // The bounded action message is owned by App state.
    }
  }

  function trapDialogFocus(event: ReactKeyboardEvent<HTMLDialogElement>) {
    if (event.key !== "Tab") return;
    const controls = dialogRef.current?.querySelectorAll<HTMLButtonElement>(
      "button:not(:disabled)",
    );
    if (!controls?.length) return;
    const first = controls[0];
    const last = controls[controls.length - 1];
    if (!first || !last) return;
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  async function requestPreview(operation: IntegrationMutationOperation) {
    if (!selectedEntry) return;
    await runAction(() =>
      onPreview({
        operation,
        targetEntryId: selectedEntry.id,
        repository: null,
        reference: null,
      }),
    );
  }

  async function requestMarketplaceAdd() {
    const parsed = integrationMutationPreviewRequestSchema.safeParse(
      marketplaceAddRequest,
    );
    if (!parsed.success) return;
    await runAction(() => onPreview(parsed.data));
  }

  return (
    <section
      className="integration-center"
      id="integrations"
      aria-labelledby="integrations-title"
    >
      <div className="integration-center__intro">
        <div>
          <p className="eyebrow">Integration Center</p>
          <h2 id="integrations-title">Inspect trust before changing state.</h2>
        </div>
        <p>
          Codex remains authoritative. QuireForge presents a normalized catalog
          and requires a fresh native preview before every supported change.
        </p>
      </div>

      <div className="integration-toolbar">
        <label className="integration-search" htmlFor="integration-search">
          <span>Search integrations</span>
          <input
            id="integration-search"
            type="search"
            maxLength={128}
            value={query}
            placeholder="Name, summary, or publisher…"
            onChange={(event) => setQuery(event.target.value)}
          />
        </label>
        <label>
          <span>Category</span>
          <select
            value={kind}
            onChange={(event) => setKind(event.target.value as typeof kind)}
          >
            <option value="all">All categories</option>
            {Object.entries(kindLabels).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </label>
        <label>
          <span>Health</span>
          <select
            value={health}
            onChange={(event) => setHealth(event.target.value as typeof health)}
          >
            <option value="all">All health states</option>
            {Object.entries(healthLabels).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </label>
        <button
          className="integration-refresh"
          type="button"
          disabled={availability !== "native" || busy}
          onClick={() => void runAction(onRefresh)}
        >
          {busy ? "Refreshing…" : "Refresh catalog"}
        </button>
      </div>

      <div className="integration-status" aria-live="polite">
        <span
          className={`integration-state integration-state--${snapshot.catalogState}`}
        >
          {availability === "checking"
            ? "Checking native catalog"
            : availability === "preview"
              ? "Preview catalog"
              : `${sentenceCase(snapshot.catalogState)} catalog`}
        </span>
        <span>
          {visibleEntries.length} of {snapshot.entries.length} integrations
        </span>
        <span>Codex CLI {snapshot.cliVersion}</span>
        <span>
          Installation policy: {sentenceCase(snapshot.policy.installation)}
        </span>
      </div>

      {actionError && (
        <p className="integration-alert integration-alert--error" role="alert">
          The native integration action did not complete. No raw error or
          integration configuration entered the interface.
        </p>
      )}
      {result?.state === "applied" && (
        <p
          className="integration-alert integration-alert--success"
          role="status"
        >
          {result.operation ? operationLabels[result.operation] : "The change"}
          {" completed and the catalog was refreshed."}
        </p>
      )}
      {result?.state === "unavailable" && (
        <p className="integration-alert integration-alert--error" role="alert">
          The requested change was not applied ({result.diagnosticCode}).
        </p>
      )}

      <div className="integration-layout">
        <div className="integration-list" aria-label="Integration catalog">
          {visibleEntries.map((entry) => (
            <button
              className={`integration-row${selectedEntry?.id === entry.id ? " integration-row--selected" : ""}`}
              type="button"
              key={entry.id}
              aria-pressed={selectedEntry?.id === entry.id}
              onClick={() => setSelectedId(entry.id)}
            >
              <span className="integration-row__top">
                <span>{entry.displayName}</span>
                <span
                  className={`integration-health integration-health--${entry.health.state}`}
                >
                  {statusSummary(entry)}
                </span>
              </span>
              <span>{kindLabels[entry.kind]}</span>
              <small>{entry.summary}</small>
            </button>
          ))}
          {visibleEntries.length === 0 && (
            <div className="integration-empty">
              <strong>No integrations match these filters.</strong>
              <button
                type="button"
                onClick={() => {
                  setQuery("");
                  setKind("all");
                  setHealth("all");
                }}
              >
                Clear filters
              </button>
            </div>
          )}
        </div>

        <article className="integration-detail" aria-live="polite">
          {selectedEntry ? (
            <>
              <div className="integration-detail__heading">
                <div>
                  <p className="eyebrow">{kindLabels[selectedEntry.kind]}</p>
                  <h3>{selectedEntry.displayName}</h3>
                  <p>{selectedEntry.summary}</p>
                </div>
                <span
                  className={`integration-health integration-health--${selectedEntry.health.state}`}
                >
                  {healthLabels[selectedEntry.health.state]}
                </span>
              </div>

              <dl className="integration-facts">
                <div>
                  <dt>Source</dt>
                  <dd>{sentenceCase(selectedEntry.source)}</dd>
                </div>
                <div>
                  <dt>Scope</dt>
                  <dd>{sentenceCase(selectedEntry.scope)}</dd>
                </div>
                <div>
                  <dt>Installation</dt>
                  <dd>{sentenceCase(selectedEntry.installation)}</dd>
                </div>
                <div>
                  <dt>Enablement</dt>
                  <dd>{sentenceCase(selectedEntry.enablement)}</dd>
                </div>
                <div>
                  <dt>Authentication</dt>
                  <dd>{sentenceCase(selectedEntry.authentication)}</dd>
                </div>
                <div>
                  <dt>Policy</dt>
                  <dd>{sentenceCase(selectedEntry.policy.state)}</dd>
                </div>
                {selectedEntry.version && (
                  <div>
                    <dt>Version</dt>
                    <dd>{selectedEntry.version}</dd>
                  </div>
                )}
                {selectedEntry.publisher && (
                  <div>
                    <dt>Publisher</dt>
                    <dd>{selectedEntry.publisher}</dd>
                  </div>
                )}
              </dl>

              {selectedEntry.policy.reason && (
                <p className="integration-policy-note">
                  <strong>Policy review:</strong> {selectedEntry.policy.reason}
                </p>
              )}

              <div className="integration-review-grid">
                <section aria-labelledby="integration-permissions-title">
                  <h4 id="integration-permissions-title">Permissions</h4>
                  {selectedEntry.permissions.length ? (
                    <ul>
                      {selectedEntry.permissions.map((permission) => (
                        <li
                          key={`${permission.kind}:${permission.access}:${permission.target}`}
                        >
                          <strong>{sentenceCase(permission.access)}</strong>{" "}
                          {permission.target}
                          {permission.kind === "hook" && (
                            <span> — requires separate hook trust</span>
                          )}
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p>No normalized permissions were declared.</p>
                  )}
                </section>
                <section aria-labelledby="integration-requirements-title">
                  <h4 id="integration-requirements-title">Requirements</h4>
                  {selectedEntry.requirements.length ? (
                    <ul>
                      {selectedEntry.requirements.map((requirement) => (
                        <li key={`${requirement.kind}:${requirement.name}`}>
                          <strong>{requirement.name}</strong> —{" "}
                          {sentenceCase(requirement.state)}
                          {requirement.detail ? `: ${requirement.detail}` : ""}
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p>No normalized requirements were declared.</p>
                  )}
                </section>
              </div>

              {selectedEntry.health.diagnosticCodes.length > 0 && (
                <p className="integration-diagnostics">
                  Diagnostic: {selectedEntry.health.diagnosticCodes.join(", ")}
                </p>
              )}

              <div className="integration-actions">
                {selectedOperations.map((operation) => (
                  <button
                    type="button"
                    key={operation}
                    disabled={availability !== "native" || busy}
                    onClick={() => void requestPreview(operation)}
                  >
                    {operationLabels[operation]}
                  </button>
                ))}
                {selectedOperations.length === 0 && (
                  <p>
                    No supported management action is available. Authorization,
                    enablement, and skill configuration remain outside this
                    checkpoint.
                  </p>
                )}
              </div>
            </>
          ) : (
            <p>Select an integration to review its normalized details.</p>
          )}
        </article>
      </div>

      <form
        className="marketplace-add"
        onSubmit={(event) => {
          event.preventDefault();
          void requestMarketplaceAdd();
        }}
      >
        <div>
          <p className="eyebrow">Pinned marketplace source</p>
          <h3>Add a repository marketplace</h3>
          <p>
            Only an owner/repository identifier and an exact 40- or 64-character
            hexadecimal reference can enter the native preview boundary.
          </p>
        </div>
        <label htmlFor="marketplace-repository">
          Repository
          <input
            id="marketplace-repository"
            value={repository}
            maxLength={160}
            placeholder="owner/repository"
            autoComplete="off"
            disabled={availability !== "native" || busy}
            onChange={(event) => setRepository(event.target.value)}
          />
        </label>
        <label htmlFor="marketplace-reference">
          Pinned reference
          <input
            id="marketplace-reference"
            value={reference}
            maxLength={64}
            placeholder="40- or 64-character commit"
            autoComplete="off"
            spellCheck={false}
            disabled={availability !== "native" || busy}
            onChange={(event) => setReference(event.target.value)}
          />
        </label>
        <button type="submit" disabled={!marketplaceAddReady || busy}>
          Review marketplace
        </button>
      </form>

      {preview && (
        <div className="integration-dialog-backdrop">
          <dialog
            ref={dialogRef}
            open
            className="integration-dialog"
            aria-modal="true"
            aria-labelledby="integration-dialog-title"
            aria-describedby="integration-dialog-summary"
            onKeyDown={trapDialogFocus}
          >
            <p className="eyebrow">Fresh native preview</p>
            <h3 id="integration-dialog-title">
              {operationLabels[preview.operation]}
            </h3>
            <p id="integration-dialog-summary">
              {preview.state === "ready"
                ? `${preview.targetDisplayName} was revalidated against the current catalog and policy.`
                : `The operation is ${preview.state} (${preview.diagnosticCode}).`}
            </p>
            {preview.state === "ready" && (
              <>
                <dl className="integration-dialog__facts">
                  <div>
                    <dt>Source</dt>
                    <dd>{sentenceCase(preview.source)}</dd>
                  </div>
                  <div>
                    <dt>Destructive</dt>
                    <dd>{preview.destructive ? "Yes" : "No"}</dd>
                  </div>
                </dl>
                <section>
                  <h4>Permissions reviewed</h4>
                  {preview.permissions.length ? (
                    <ul>
                      {preview.permissions.map((permission) => (
                        <li
                          key={`${permission.kind}:${permission.access}:${permission.target}`}
                        >
                          {sentenceCase(permission.access)} {permission.target}
                          {permission.kind === "hook"
                            ? " — separate trust required"
                            : ""}
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p>No normalized permissions were declared.</p>
                  )}
                </section>
                <section>
                  <h4>Warnings</h4>
                  {preview.warnings.length ? (
                    <ul>
                      {preview.warnings.map((warning) => (
                        <li key={warning}>{warningLabels[warning]}</li>
                      ))}
                    </ul>
                  ) : (
                    <p>No normalized warnings were returned.</p>
                  )}
                </section>
              </>
            )}
            <div className="integration-dialog__actions">
              <button
                ref={dialogFocusRef}
                type="button"
                disabled={busy}
                onClick={onCancel}
              >
                {preview.state === "ready" ? "Cancel" : "Close"}
              </button>
              {preview.state === "ready" && preview.confirmationId && (
                <button
                  className={preview.destructive ? "button-danger" : ""}
                  type="button"
                  disabled={busy}
                  onClick={() =>
                    void runAction(() => onConfirm(preview.confirmationId!))
                  }
                >
                  {busy
                    ? "Applying…"
                    : preview.destructive
                      ? "Confirm destructive change"
                      : "Confirm change"}
                </button>
              )}
            </div>
          </dialog>
        </div>
      )}
    </section>
  );
}
