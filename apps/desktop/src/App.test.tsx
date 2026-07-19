import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";
import { codexAuthSchema, scaffoldCodexAuth } from "./lib/auth";
import { scaffoldCodexRuntime } from "./lib/codex";
import { scaffoldBootstrap } from "./lib/contract";

describe("QuireForge desktop shell", () => {
  beforeEach(() => {
    window.localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
  });

  it("renders the honest scaffold state and verifies native data", async () => {
    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
      />,
    );

    expect(
      screen.getByRole("heading", {
        name: "A quiet place for ambitious work.",
      }),
    ).toBeInTheDocument();
    expect(await screen.findByText("Native IPC verified")).toBeInTheDocument();
    expect(await screen.findAllByText("Codex adapter ready")).toHaveLength(2);
    expect(screen.getByText("No project attached")).toBeInTheDocument();
    expect(
      await screen.findByRole("button", { name: "Continue in browser" }),
    ).toBeInTheDocument();
    expect(screen.getAllByText("planned")).toHaveLength(1);
    expect(
      screen.getByText(
        /not made, endorsed, supported, or distributed by OpenAI/u,
      ),
    ).toBeInTheDocument();
  });

  it("labels a browser-only render without simulating native success", async () => {
    render(
      <App
        loadBootstrap={() => Promise.reject(new Error("no IPC"))}
        loadRuntime={() => Promise.reject(new Error("no IPC"))}
        loadAuth={() => Promise.reject(new Error("no IPC"))}
      />,
    );

    expect(await screen.findByText("Browser preview")).toBeInTheDocument();
    expect(await screen.findAllByText("Native probe unavailable")).toHaveLength(
      2,
    );
    expect(screen.queryByText("Native IPC verified")).not.toBeInTheDocument();
    expect(
      await screen.findByText("Native authentication unavailable"),
    ).toBeInTheDocument();
  });

  it("persists the explicit theme choice", () => {
    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
      />,
    );

    const button = screen.getByRole("button", { name: /theme/u });
    fireEvent.click(button);

    expect(window.localStorage.getItem("quireforge-theme")).toBe(
      document.documentElement.dataset.theme,
    );
  });

  it("renders a device-code handoff and cancels through fixed actions", async () => {
    const pending = codexAuthSchema.parse({
      ...scaffoldCodexAuth,
      state: "login-pending",
      pendingMethod: "device-code",
      handoff: {
        verificationUrl: "https://auth.openai.com/device",
        userCode: "SAFE-CODE",
      },
    });
    const startAuth = vi.fn().mockResolvedValue(pending);
    const openAuthBrowser = vi.fn().mockResolvedValue(undefined);
    const cancelAuth = vi.fn().mockResolvedValue(scaffoldCodexAuth);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(scaffoldCodexAuth)}
        startAuth={startAuth}
        openAuthBrowser={openAuthBrowser}
        cancelAuth={cancelAuth}
      />,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Use a device code" }),
    );
    expect(await screen.findByText("SAFE-CODE")).toBeInTheDocument();
    expect(startAuth).toHaveBeenCalledWith("device-code");
    expect(openAuthBrowser).toHaveBeenCalledOnce();

    fireEvent.click(screen.getByRole("button", { name: "Cancel sign-in" }));
    expect(
      await screen.findByRole("button", { name: "Continue in browser" }),
    ).toBeInTheDocument();
    expect(cancelAuth).toHaveBeenCalledOnce();
  });

  it("requires a second explicit action before logout", async () => {
    const authenticated = codexAuthSchema.parse({
      ...scaffoldCodexAuth,
      state: "authenticated",
      accountKind: "chatgpt",
    });
    const logoutAuth = vi.fn().mockResolvedValue(scaffoldCodexAuth);

    render(
      <App
        loadBootstrap={() => Promise.resolve(scaffoldBootstrap)}
        loadRuntime={() => Promise.resolve(scaffoldCodexRuntime)}
        loadAuth={() => Promise.resolve(authenticated)}
        logoutAuth={logoutAuth}
      />,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: "Sign out of Codex" }),
    );
    expect(logoutAuth).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Confirm sign out" }));
    expect(logoutAuth).toHaveBeenCalledOnce();
  });
});
