import brandMark from "../../../assets/brand/quireforge-app-icon.svg";
import type { AuthLoginMethod, CodexAuthSnapshot } from "./lib/auth";

type AuthGateState = CodexAuthSnapshot["state"] | "checking" | "preview";
type Theme = "light" | "dark";

interface AuthGateProps {
  state: AuthGateState;
  snapshot: CodexAuthSnapshot;
  busy: boolean;
  actionError: boolean;
  cliVersion: string | null;
  theme: Theme;
  onThemeChange: () => void;
  onStart: (method: AuthLoginMethod) => void;
  onOpenBrowser: () => void;
  onCancel: () => void;
  onRefresh: () => void;
}

const stateCopy: Partial<Record<AuthGateState, string>> = {
  checking: "Checking for an existing Codex session…",
  preview:
    "Native Codex authentication is unavailable in this browser preview.",
  unavailable:
    "QuireForge could not verify Codex authentication through the local runtime.",
};

export function AuthGate({
  state,
  snapshot,
  busy,
  actionError,
  cliVersion,
  theme,
  onThemeChange,
  onStart,
  onOpenBrowser,
  onCancel,
  onRefresh,
}: AuthGateProps) {
  return (
    <main className="access-gate">
      <header className="access-gate__topbar">
        <div className="access-gate__brand">
          <img src={brandMark} alt="" />
          <strong>QuireForge</strong>
        </div>
        <button
          className="theme-toggle"
          type="button"
          aria-label={`Use ${theme === "dark" ? "light" : "dark"} theme`}
          onClick={onThemeChange}
        >
          <span className="theme-toggle__track" aria-hidden="true">
            <span className="theme-toggle__thumb" />
          </span>
        </button>
      </header>

      <section className="access-gate__content" aria-labelledby="access-title">
        <div className="access-gate__intro">
          <span className="access-gate__mark" aria-hidden="true">
            Q
          </span>
          <p className="eyebrow">Native Linux workspace</p>
          <h1 id="access-title">Welcome to QuireForge.</h1>
          <p>
            Sign in through Codex to start tasks, resume conversations, and use
            your available models. QuireForge never receives or stores your
            password, token, email address, or account identifier.
          </p>
        </div>

        <div className="access-gate__card" aria-live="polite">
          <div className="access-gate__status">
            <span data-state={state} aria-hidden="true" />
            <div>
              <strong>
                {state === "unauthenticated"
                  ? "Connect your Codex account"
                  : state === "login-pending"
                    ? "Complete sign-in"
                    : "Codex account check"}
              </strong>
              <small>
                Codex CLI {cliVersion ?? "not detected"} · credentials stay with
                Codex
              </small>
            </div>
          </div>

          {(state === "checking" ||
            state === "preview" ||
            state === "unavailable") && <p>{stateCopy[state]}</p>}

          {state === "unauthenticated" && (
            <>
              <p>
                Continue with ChatGPT in your browser, or use an official device
                code when the local callback is unavailable.
              </p>
              <div className="access-gate__actions">
                <button
                  className="auth-button auth-button--primary"
                  type="button"
                  disabled={busy}
                  onClick={() => onStart("browser")}
                >
                  Continue with ChatGPT
                </button>
                <button
                  className="auth-button"
                  type="button"
                  disabled={busy}
                  onClick={() => onStart("device-code")}
                >
                  Use a device code
                </button>
              </div>
            </>
          )}

          {state === "login-pending" && snapshot.handoff && (
            <>
              <p>
                Finish on the official Codex sign-in page. This handoff is
                short-lived and disappears after completion or cancellation.
              </p>
              {snapshot.handoff.userCode && (
                <div className="device-code">
                  <span>One-time device code</span>
                  <code>{snapshot.handoff.userCode}</code>
                </div>
              )}
              <div className="access-gate__actions">
                <button
                  className="auth-button auth-button--primary"
                  type="button"
                  disabled={busy}
                  onClick={onOpenBrowser}
                >
                  Open sign-in page
                </button>
                <button
                  className="auth-button"
                  type="button"
                  disabled={busy}
                  onClick={onCancel}
                >
                  Cancel sign-in
                </button>
              </div>
            </>
          )}

          {(state === "unavailable" || state === "preview") && (
            <button
              className="auth-button"
              type="button"
              disabled={busy}
              onClick={onRefresh}
            >
              Try again
            </button>
          )}

          {actionError && (
            <p className="auth-error" role="alert">
              The Codex authentication action did not complete. No credentials
              were changed by QuireForge.
            </p>
          )}
        </div>
      </section>

      <footer className="access-gate__footer">
        <span>Build boldly. Work locally.</span>
        <p>
          QuireForge is an unofficial community project and is not made,
          endorsed, supported, or distributed by OpenAI.
        </p>
      </footer>
    </main>
  );
}
