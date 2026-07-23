import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("./App", () => new Promise(() => undefined));

import { AppLoader } from "./AppLoader";

describe("AppLoader", () => {
  it("shows a bounded native loading state while the app shell loads", () => {
    render(<AppLoader />);

    expect(
      screen.getByRole("heading", { name: "Preparing QuireForge." }),
    ).toBeInTheDocument();
    expect(screen.getByRole("status")).toHaveTextContent(
      "Loading the local workspace interface.",
    );
  });
});
