import type {
  IntegrationCatalogSnapshot,
  ScheduledTaskTemplate,
} from "./lib/integration";

type ScheduledAvailability = "checking" | "native" | "preview";

interface ScheduledWorkspaceProps {
  availability: ScheduledAvailability;
  snapshot: IntegrationCatalogSnapshot;
}

const weekdayLabels = {
  MO: "Mon",
  TU: "Tue",
  WE: "Wed",
  TH: "Thu",
  FR: "Fri",
  SA: "Sat",
  SU: "Sun",
} as const;

function dayList(days: readonly (keyof typeof weekdayLabels)[]): string {
  return days.map((day) => weekdayLabels[day]).join(", ");
}

function scheduledTaskScheduleLabel(
  schedule: ScheduledTaskTemplate["schedule"],
): string {
  switch (schedule.type) {
    case "hourly": {
      const interval =
        schedule.intervalHours === 1
          ? "Every hour"
          : `Every ${schedule.intervalHours} hours`;
      return schedule.days && schedule.days.length > 0
        ? `${interval} on ${dayList(schedule.days)}`
        : interval;
    }
    case "daily":
      return `Daily at ${schedule.time}`;
    case "weekdays":
      return `Weekdays at ${schedule.time}`;
    case "weekly":
      return `${dayList(schedule.days)} at ${schedule.time}`;
  }
}

export function ScheduledWorkspace({
  availability,
  snapshot,
}: ScheduledWorkspaceProps) {
  const capability = snapshot.capabilities.find(
    (candidate) => candidate.id === "scheduled-task.catalog",
  );
  const plugins = new Map(
    snapshot.entries
      .filter((entry) => entry.kind === "plugin")
      .map((entry) => [entry.id, entry.displayName]),
  );
  const status =
    availability === "checking"
      ? "Checking native task discovery"
      : availability === "preview"
        ? "Preview fixture"
        : capability?.availability === "ready"
          ? "Catalog ready"
          : "Catalog needs attention";

  return (
    <section
      className="scheduled-workspace"
      id="scheduled"
      aria-labelledby="scheduled-title"
    >
      <div className="scheduled-workspace__intro">
        <div>
          <p className="eyebrow">Scheduled task catalog</p>
          <h2 id="scheduled-title">
            Review task templates without handing over control.
          </h2>
        </div>
        <p>
          Installed plugins can declare task names, prompts, and schedules.
          QuireForge presents that metadata as inert text and never submits it
          as an instruction.
        </p>
      </div>

      <div className="scheduled-boundary" role="note">
        <strong>Read-only boundary</strong>
        <p>
          QuireForge cannot create, edit, enable, run, pause, or delete
          scheduled tasks. Task execution and management remain with supported
          official Codex surfaces.
        </p>
      </div>

      <div className="scheduled-status" role="status" aria-live="polite">
        <span
          className={`scheduled-state scheduled-state--${
            capability?.availability ?? "unknown"
          }`}
        >
          {status}
        </span>
        <span>
          {snapshot.scheduledTasks.length}{" "}
          {snapshot.scheduledTasks.length === 1 ? "template" : "templates"}
        </span>
        {capability?.diagnosticCode && (
          <span>Diagnostic: {capability.diagnosticCode}</span>
        )}
      </div>

      {snapshot.scheduledTasks.length === 0 ? (
        <div className="scheduled-empty">
          <strong>No task templates discovered</strong>
          <p>
            No installed, enabled plugin currently exposes a supported scheduled
            task template.
          </p>
        </div>
      ) : (
        <div className="scheduled-grid">
          {snapshot.scheduledTasks.map((task) => (
            <article className="scheduled-card" key={task.id}>
              <div className="scheduled-card__heading">
                <div>
                  <span>{plugins.get(task.sourcePluginId) ?? "Plugin"}</span>
                  <h3>{task.name}</h3>
                </div>
                <strong>{scheduledTaskScheduleLabel(task.schedule)}</strong>
              </div>
              <div className="scheduled-prompt">
                <span>Untrusted prompt preview</span>
                <p>{task.promptPreview}</p>
              </div>
              {task.promptTruncated && (
                <p className="scheduled-truncated">
                  Preview clipped at the native safety bound.
                </p>
              )}
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
