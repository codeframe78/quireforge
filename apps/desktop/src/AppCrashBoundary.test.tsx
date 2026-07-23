import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { AppCrashBoundary } from "./AppCrashBoundary";

function BrokenView(): never {
  throw new Error("sensitive-render-diagnostic-should-not-render");
}

describe("AppCrashBoundary", () => {
  it("replaces a render failure with bounded recovery copy", () => {
    const reload = vi.fn();
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => undefined);

    render(
      <AppCrashBoundary onReload={reload}>
        <BrokenView />
      </AppCrashBoundary>,
    );

    expect(
      screen.getByRole("heading", { name: "QuireForge needs a fresh view." }),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/sensitive-render-diagnostic/u),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Reload workspace" }));
    expect(reload).toHaveBeenCalledOnce();
    consoleError.mockRestore();
  });
});
