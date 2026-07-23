import type { CodexUsageSnapshot, CodexUsageWindow } from "./lib/usage";

interface UsagePanelProps {
  snapshot: CodexUsageSnapshot;
  state: "checking" | "native" | "preview";
  busy: boolean;
  compact?: boolean;
  onRefresh: () => void;
}

function windowLabel(window: CodexUsageWindow): string {
  const minutes = window.windowDurationMinutes;
  if (minutes === 10_080) return "Weekly window";
  if (minutes && minutes % 1_440 === 0) return `${minutes / 1_440}-day window`;
  if (minutes && minutes % 60 === 0) return `${minutes / 60}-hour window`;
  if (minutes) return `${minutes}-minute window`;
  return window.kind === "primary" ? "Primary window" : "Secondary window";
}

function resetLabel(timestamp: number | null): string {
  if (timestamp === null) return "Reset time unavailable";
  const date = new Date(timestamp * 1000);
  if (Number.isNaN(date.getTime())) return "Reset time unavailable";
  return `Resets ${new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  }).format(date)}`;
}

export function UsagePanel({
  snapshot,
  state,
  busy,
  compact = false,
  onRefresh,
}: UsagePanelProps) {
  const windows = snapshot.meters.flatMap((meter) =>
    meter.windows.map((window) => ({ meter, window })),
  );
  const first = windows[0];

  if (compact) {
    return (
      <div className="usage-compact" aria-label="Codex usage">
        <div>
          <span>Usage available</span>
          <strong>
            {state === "native" && snapshot.state === "ready" && first
              ? `${first.window.remainingPercent}%`
              : "—"}
          </strong>
        </div>
        <small>
          {state === "checking"
            ? "Checking Codex usage"
            : first
              ? resetLabel(first.window.resetsAt)
              : "No metered window reported"}
        </small>
      </div>
    );
  }

  return (
    <section className="usage-panel" aria-labelledby="usage-title">
      <div className="usage-panel__heading">
        <div>
          <span>Account</span>
          <h2 id="usage-title">Remaining usage</h2>
        </div>
        <button
          type="button"
          disabled={busy || state !== "native"}
          onClick={onRefresh}
        >
          Refresh
        </button>
      </div>

      {state === "checking" && <p>Reading usage from the Codex runtime…</p>}
      {state === "preview" && (
        <p>Native usage information is unavailable in browser preview.</p>
      )}
      {state === "native" && snapshot.state === "not-metered" && (
        <p>This Codex account did not report a metered usage window.</p>
      )}
      {state === "native" && snapshot.state === "unavailable" && (
        <p>
          Codex usage is currently unavailable. QuireForge will not estimate the
          remaining amount.
        </p>
      )}

      {state === "native" && snapshot.state === "ready" && (
        <div className="usage-panel__meters">
          {windows.map(({ meter, window }) => (
            <article
              className="usage-meter"
              key={`${meter.limitId}-${window.kind}`}
            >
              <div className="usage-meter__copy">
                <span>
                  {meter.label} · {windowLabel(window)}
                </span>
                <strong>{window.remainingPercent}% remaining</strong>
                <small>{resetLabel(window.resetsAt)}</small>
              </div>
              <div
                className="usage-meter__bar"
                role="progressbar"
                aria-label={`${meter.label} ${windowLabel(window)} remaining`}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={window.remainingPercent}
              >
                <span style={{ width: `${window.remainingPercent}%` }} />
              </div>
              {meter.limited && (
                <em>Codex reports that this limit has been reached.</em>
              )}
            </article>
          ))}
        </div>
      )}

      <small className="usage-panel__note">
        Reported by Codex. QuireForge does not calculate or predict quota.
      </small>
    </section>
  );
}
