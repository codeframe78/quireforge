import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { scaffoldIntegrationCatalog } from "./lib/integration";
import { ScheduledWorkspace } from "./ScheduledWorkspace";

describe("ScheduledWorkspace", () => {
  it("renders discovered templates as explicitly inert read-only metadata", () => {
    render(
      <ScheduledWorkspace
        availability="native"
        snapshot={scaffoldIntegrationCatalog}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "Review task templates without handing over control.",
      }),
    ).toBeInTheDocument();
    expect(screen.getByText("Weekly review")).toBeInTheDocument();
    expect(screen.getByText("Mon, Thu at 09:30")).toBeInTheDocument();
    expect(screen.getByText("Untrusted prompt preview")).toBeInTheDocument();
    expect(
      screen.getByText(/cannot create, edit, enable, run, pause, or delete/u),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("shows bounded degraded and empty states without management controls", () => {
    const snapshot = {
      ...scaffoldIntegrationCatalog,
      scheduledTasks: [],
      capabilities: scaffoldIntegrationCatalog.capabilities.map((capability) =>
        capability.id === "scheduled-task.catalog"
          ? {
              ...capability,
              availability: "degraded" as const,
              diagnosticCode: "scheduled-task-response-invalid",
            }
          : capability,
      ),
    };
    render(<ScheduledWorkspace availability="native" snapshot={snapshot} />);

    expect(screen.getByText("Catalog needs attention")).toBeInTheDocument();
    expect(
      screen.getByText("Diagnostic: scheduled-task-response-invalid"),
    ).toBeInTheDocument();
    expect(
      screen.getByText("No task templates discovered"),
    ).toBeInTheDocument();
  });

  it("formats every reviewed schedule variant", () => {
    const template = scaffoldIntegrationCatalog.scheduledTasks[0]!;
    render(
      <ScheduledWorkspace
        availability="native"
        snapshot={{
          ...scaffoldIntegrationCatalog,
          scheduledTasks: [
            {
              ...template,
              id: "scheduled-task:hourly",
              schedule: {
                type: "hourly",
                intervalHours: 2,
                days: ["MO", "FR"],
              },
            },
            {
              ...template,
              id: "scheduled-task:daily",
              schedule: { type: "daily", time: "08:00" },
            },
            {
              ...template,
              id: "scheduled-task:weekdays",
              schedule: { type: "weekdays", time: "17:30" },
            },
          ],
        }}
      />,
    );

    expect(screen.getByText("Every 2 hours on Mon, Fri")).toBeInTheDocument();
    expect(screen.getByText("Daily at 08:00")).toBeInTheDocument();
    expect(screen.getByText("Weekdays at 17:30")).toBeInTheDocument();
  });
});
